//! Pure generation jobs shared by browser workers and the renderer.
//! Products contain immutable scene assets, never tactical simulation state.
use adventuresim_building_generator::BuildingProgram;
use adventuresim_tactical_core::scene_input::{GeneratedTacticalScene, TacticalSceneInput};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

use adventuresim_tactical_core::geometry_transport::binary as packed;
pub(super) use building::PreparedFacade;
use error::{PreparationError, PreparationResult, ProductKind};
use residency::SceneReadiness;
pub(super) use venue::VenueGeometry;

mod building;
mod error;
pub(crate) mod landscape;
mod residency;
mod scene;
pub(super) mod venue;
#[cfg(target_family = "wasm")]
mod wasm;
#[cfg(target_family = "wasm")]
mod wasm_error;

const RETAINED_VENUE_RECIPES: usize = 64;
const RETAINED_SCENE_PRODUCTS: usize = 3;
static PRODUCTS: OnceLock<Mutex<PreparedProducts>> = OnceLock::new();

#[derive(Default)]
struct PreparedProducts {
    ground: Vec<landscape::GroundProduct>,
    grass: Vec<landscape::GrassProduct>,
    active_ground: Option<(String, String)>,
    scenes: Vec<GeneratedTacticalScene>,
    facades: Vec<PreparedFacade>,
    resident_facades: Vec<BuildingProgram>,
    sites: Vec<adventuresim_tactical_core::scene_input::ProgramFurnitureSite>,
    venues: Vec<venue::PreparedVenue>,
    placements: Vec<adventuresim_tactical_core::scene_input::TacticalBuildingPlacement>,
}
#[derive(Default, Serialize, Deserialize)]
struct Dependencies {
    grass: Option<landscape::GrassDependencies>,
    ground: Option<landscape::GroundDependencies>,
    sites: Vec<adventuresim_tactical_core::scene_input::ProgramFurnitureSite>,
    recipe: Option<adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe>,
    playable: Vec<adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe>,
}

#[derive(Serialize, Deserialize)]
enum GenerationJob {
    Scene(Box<TacticalSceneInput>),
    Building(Box<BuildingProgram>),
    Venue(Box<BuildingProgram>),
    Grass {
        input: Box<TacticalSceneInput>,
        graphics: String,
    },
    Ground {
        input: Box<TacticalSceneInput>,
        graphics: String,
    },
}

#[derive(Serialize, Deserialize)]
enum GenerationProduct {
    Scene(Box<scene::SceneProduct>),
    Building(Box<PreparedFacade>),
    Venue(Box<venue::PreparedVenue>),
    Grass(Box<landscape::GrassProduct>),
    Ground(Box<landscape::GroundProduct>),
}

pub(crate) fn take_scene(input: &TacticalSceneInput) -> PreparationResult<GeneratedTacticalScene> {
    let products = products()?;
    let mut scene = products.scene_for_installation(input)?;
    for placement in &products.placements {
        if !scene
            .buildings
            .iter()
            .any(|b| b.placement.id == placement.id)
        {
            let venue = products
                .venues
                .iter()
                .find(|v| v.recipe.program == placement.program)
                .ok_or(PreparationError::NotPrepared {
                    product: ProductKind::Venue,
                })?;
            let distant = input
                .distant_buildings
                .iter()
                .find(|b| b.id == placement.id)
                .ok_or(PreparationError::MissingDistantPlacement {
                    building: placement.id,
                })?;
            if *placement != (*distant).into() {
                return Err(PreparationError::PromotedPlacementMismatch {
                    building: placement.id,
                });
            }
            scene
                .buildings
                .push(adventuresim_tactical_core::scene_input::GeneratedBuilding {
                    placement: placement.clone(),
                    plan: venue.recipe.plan.clone(),
                    collision: venue.recipe.collision.clone(),
                });
        }
    }
    for building in &scene.buildings {
        let venue = products
            .venues
            .iter()
            .find(|v| v.recipe.program == building.placement.program)
            .ok_or(PreparationError::NotPrepared {
                product: ProductKind::Venue,
            })?;
        scene
            .furniture
            .install_interior(building, venue.interior.clone())?;
    }
    Ok(scene)
}

pub(super) fn take_venue_geometry(program: &BuildingProgram) -> PreparationResult<VenueGeometry> {
    products()?
        .venues
        .iter_mut()
        .find(|v| v.recipe.program == *program)
        .and_then(|v| v.geometry.take())
        .ok_or(PreparationError::NotPrepared {
            product: ProductKind::VenueGeometry,
        })
}

pub(super) fn take_facade(program: &BuildingProgram) -> PreparationResult<PreparedFacade> {
    let mut products = products()?;
    let index = products
        .facades
        .iter()
        .position(|facade| facade.program == *program)
        .ok_or(PreparationError::NotPrepared {
            product: ProductKind::Facade,
        })?;
    Ok(products.facades.swap_remove(index))
}

pub(super) fn release_unused_facades() -> PreparationResult<()> {
    products()?.facades.clear();
    Ok(())
}

pub(super) fn retain_facade(program: &BuildingProgram) -> PreparationResult<()> {
    let mut products = products()?;
    if !products.resident_facades.contains(program) {
        products.resident_facades.push(program.clone());
    }
    Ok(())
}

pub(super) fn clear_residency() -> PreparationResult<()> {
    *products()? = Default::default();
    Ok(())
}

fn products() -> PreparationResult<std::sync::MutexGuard<'static, PreparedProducts>> {
    PRODUCTS
        .get_or_init(Default::default)
        .lock()
        .map_err(|_| PreparationError::ResidencyPoisoned)
}

fn venue_jobs(input_json: &str, view_json: &str) -> PreparationResult<Vec<String>> {
    let input: TacticalSceneInput = serde_json::from_str(input_json)?;
    let request: venue::VenueRequest = serde_json::from_str(view_json)?;
    let mut products = products()?;
    products.placements = request.placements(&input);
    // Retain semantic products independently of meshes, which live in Bevy's
    // asset cache. Bound CPU recipe retention across an extended journey.
    while products.venues.len() > RETAINED_VENUE_RECIPES {
        products.venues.remove(0);
    }
    let mut programs = Vec::new();
    for placement in &products.placements {
        if !programs.contains(&placement.program)
            && !products
                .venues
                .iter()
                .any(|v| v.recipe.program == placement.program)
        {
            programs.push(placement.program.clone());
        }
    }
    programs
        .into_iter()
        .map(|p| {
            serde_json::to_string(&GenerationJob::Venue(Box::new(p)))
                .map_err(PreparationError::from)
        })
        .collect()
}

fn dependencies(job_json: &str) -> PreparationResult<Vec<u8>> {
    let job: GenerationJob = serde_json::from_str(job_json)?;
    let products = products()?;
    let mut data = Dependencies::default();
    match job {
        GenerationJob::Grass { input, .. } => {
            let digest = input.digest()?;
            let scene = products.scenes.iter().find(|s| s.digest == digest).ok_or(
                PreparationError::NotPrepared {
                    product: ProductKind::Scene,
                },
            )?;
            data.grass = Some(landscape::GrassDependencies {
                terrain: scene.terrain.clone(),
                ground: scene.ground.clone(),
            });
        }

        GenerationJob::Ground { input, .. } => {
            let digest = input.digest()?;
            let scene = products.scenes.iter().find(|s| s.digest == digest).ok_or(
                PreparationError::NotPrepared {
                    product: ProductKind::Scene,
                },
            )?;
            data.ground = Some(landscape::GroundDependencies {
                terrain: scene.terrain.clone(),
                groups: scene.furniture.groups.clone(),
            });
        }
        GenerationJob::Scene(input) => {
            data.sites = products.sites.clone();
            for building in input.buildings {
                if let Some(venue) = products
                    .venues
                    .iter()
                    .find(|v| v.recipe.program == building.program)
                {
                    data.playable.push(venue.recipe.clone());
                }
            }
        }
        GenerationJob::Venue(program) => {
            data.recipe = products
                .scenes
                .iter()
                .flat_map(|s| &s.buildings)
                .find(|b| b.placement.program == *program)
                .map(
                    |b| adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe {
                        program: b.placement.program.clone(),
                        plan: b.plan.clone(),
                        collision: b.collision.clone(),
                    },
                );
        }
        GenerationJob::Building(_) => {}
    }
    let mut bytes = Vec::new();
    ciborium::into_writer(&data, &mut bytes).map_err(PreparationError::DependenciesEncode)?;
    Ok(bytes)
}

fn jobs(input_json: &str) -> PreparationResult<Vec<String>> {
    let input: TacticalSceneInput = serde_json::from_str(input_json)?;
    input.validate()?;
    let mut programs = Vec::new();
    let mut products = products()?;
    let readiness = products.scene_readiness(&input)?;
    let resident = products.resident_facades.clone();
    drop(products);
    for placement in &input.distant_buildings {
        let program = placement.occupied_program();
        if !programs.contains(&program) && !resident.contains(&program) {
            programs.push(program);
        }
    }
    (readiness == SceneReadiness::Missing)
        .then(|| GenerationJob::Scene(Box::new(input)))
        .into_iter()
        .chain(
            programs
                .into_iter()
                .map(|program| GenerationJob::Building(Box::new(program))),
        )
        .map(|job| serde_json::to_string(&job).map_err(PreparationError::from))
        .collect()
}

fn generate(job_json: &str, dependencies: &[u8]) -> PreparationResult<Vec<u8>> {
    let job: GenerationJob = serde_json::from_str(job_json)?;
    let dependencies: Dependencies =
        ciborium::from_reader(dependencies).map_err(PreparationError::DependenciesDecode)?;
    let product = match job {
        GenerationJob::Grass { input, graphics } => {
            GenerationProduct::Grass(Box::new(landscape::GrassProduct::generate(
                &input,
                graphics,
                dependencies
                    .grass
                    .ok_or(PreparationError::MissingDependencies {
                        product: ProductKind::Grass,
                    })?,
            )?))
        }

        GenerationJob::Ground { input, graphics } => {
            GenerationProduct::Ground(Box::new(landscape::GroundProduct::generate(
                &input,
                graphics,
                dependencies
                    .ground
                    .ok_or(PreparationError::MissingDependencies {
                        product: ProductKind::Ground,
                    })?,
            )?))
        }
        GenerationJob::Scene(input) => {
            let mut recipes =
                adventuresim_tactical_core::scene_input::GeneratedBuildingRecipes::default();
            recipes.sites = dependencies.sites;
            for recipe in dependencies.playable {
                recipes.insert(recipe);
            }
            GenerationProduct::Scene(Box::new(scene::SceneProduct::from_generated(
                input.generate_unfurnished(recipes)?,
            )))
        }
        GenerationJob::Building(program) => {
            GenerationProduct::Building(Box::new(PreparedFacade::generate(*program)?))
        }
        GenerationJob::Venue(program) => GenerationProduct::Venue(Box::new(
            venue::PreparedVenue::generate(*program, dependencies.recipe)?,
        )),
    };
    let mut bytes = Vec::new();
    ciborium::into_writer(&product, &mut bytes).map_err(PreparationError::ProductEncode)?;
    Ok(bytes)
}

fn receive(job_json: &str, bytes: &[u8]) -> PreparationResult<()> {
    let job: GenerationJob = serde_json::from_str(job_json)?;
    let product: GenerationProduct =
        ciborium::from_reader(bytes).map_err(PreparationError::ProductDecode)?;
    match (job, product) {
        (GenerationJob::Grass { input, graphics }, GenerationProduct::Grass(grass))
            if input.digest()? == grass.digest && graphics == grass.graphics =>
        {
            let mut products = products()?;
            products.grass.push(*grass);
            if products.grass.len() > RETAINED_SCENE_PRODUCTS {
                products.grass.remove(0);
            }
        }

        (GenerationJob::Ground { input, graphics }, GenerationProduct::Ground(ground))
            if input.digest()? == ground.digest && graphics == ground.graphics =>
        {
            landscape::retain(*ground)?;
        }
        (GenerationJob::Scene(input), GenerationProduct::Scene(scene))
            if input.digest()? == scene.digest() =>
        {
            let mut products = products()?;
            let scene = scene.restore(&input, &products)?;
            products.retain_scene(scene);
        }
        (GenerationJob::Building(program), GenerationProduct::Building(facade))
            if *program == facade.program =>
        {
            let mut products = products()?;
            if !products
                .sites
                .iter()
                .any(|site| site.program == facade.program)
            {
                products.sites.push(
                    adventuresim_tactical_core::scene_input::ProgramFurnitureSite {
                        program: facade.program.clone(),
                        recipe: facade.site.clone(),
                    },
                );
            }
            products.facades.push(*facade);
        }
        (GenerationJob::Venue(program), GenerationProduct::Venue(venue))
            if *program == venue.recipe.program =>
        {
            products()?.venues.push(*venue);
        }
        _ => return Err(PreparationError::ProductMismatch),
    }
    Ok(())
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod travel_fixtures;
