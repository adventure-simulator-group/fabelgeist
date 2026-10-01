//! Pure generation jobs shared by browser workers and the renderer.
//! Products contain immutable scene assets, never tactical simulation state.
use adventuresim_building_generator::BuildingProgram;
use adventuresim_tactical_core::scene_input::{GeneratedTacticalScene, TacticalSceneInput};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

mod building;
mod packed;
pub(super) mod venue;
pub(super) use building::PreparedFacade;
pub(super) use venue::VenueGeometry;
#[cfg(test)]
mod tests;
#[cfg(target_family = "wasm")]
mod wasm;

#[derive(Serialize, Deserialize)]
enum GenerationJob {
    Scene(Box<TacticalSceneInput>),
    Building(Box<BuildingProgram>),
    Venue(Box<BuildingProgram>),
}

#[derive(Serialize, Deserialize)]
enum GenerationProduct {
    Scene(Box<GeneratedTacticalScene>),
    Building(Box<PreparedFacade>),
    Venue(Box<venue::PreparedVenue>),
}

#[derive(Default)]
struct PreparedProducts {
    scenes: Vec<GeneratedTacticalScene>,
    facades: Vec<PreparedFacade>,
    resident_facades: Vec<BuildingProgram>,
    sites: Vec<(
        BuildingProgram,
        adventuresim_tactical_core::scene_input::furniture::FurnitureSiteRecipe,
    )>,
    venues: Vec<venue::PreparedVenue>,
    placements: Vec<adventuresim_tactical_core::scene_input::TacticalBuildingPlacement>,
}
const RETAINED_VENUE_RECIPES: usize = 64;
static PRODUCTS: OnceLock<Mutex<PreparedProducts>> = OnceLock::new();

fn products() -> std::sync::MutexGuard<'static, PreparedProducts> {
    PRODUCTS
        .get_or_init(Default::default)
        .lock()
        .expect("generation product queue")
}

pub(crate) fn take_scene(input: &TacticalSceneInput) -> Result<GeneratedTacticalScene, String> {
    let digest = input.digest().map_err(|error| error.to_string())?;
    let mut products = products();
    let index = products
        .scenes
        .iter()
        .position(|scene| scene.digest == digest)
        .ok_or("scene generation was not completed before installation")?;
    let mut scene = products.scenes.swap_remove(index);
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
                .ok_or("venue generation was not completed before installation")?;
            let distant = input
                .distant_buildings
                .iter()
                .find(|b| b.id == placement.id)
                .ok_or("promoted venue has no distant placement")?;
            scene
                .buildings
                .push(adventuresim_tactical_core::scene_input::GeneratedBuilding {
                    placement: placement.clone(),
                    plan: venue.recipe.plan.clone(),
                    collision: venue.recipe.collision.clone(),
                    pad_elevation_metres: distant.base_elevation_metres,
                });
        }
    }
    for building in &scene.buildings {
        let venue = products
            .venues
            .iter()
            .find(|v| v.recipe.program == building.placement.program)
            .ok_or("interior generation was not completed before installation")?;
        scene
            .furniture
            .install_interior(building, venue.interior.clone());
    }
    Ok(scene)
}

pub(super) fn take_venue_geometry(program: &BuildingProgram) -> Result<VenueGeometry, String> {
    products()
        .venues
        .iter_mut()
        .find(|v| v.recipe.program == *program)
        .and_then(|v| v.geometry.take())
        .ok_or("venue meshes were not prepared before installation".into())
}

pub(super) fn take_facade(program: &BuildingProgram) -> Result<PreparedFacade, String> {
    let mut products = products();
    let index = products
        .facades
        .iter()
        .position(|facade| facade.program == *program)
        .ok_or("building generation was not completed before installation")?;
    Ok(products.facades.swap_remove(index))
}

pub(super) fn release_unused_facades() {
    products().facades.clear();
}

pub(super) fn retain_facade(program: &BuildingProgram) {
    let mut products = products();
    if !products.resident_facades.contains(program) {
        products.resident_facades.push(program.clone());
    }
}

pub(super) fn clear_residency() {
    *products() = Default::default();
}

fn venue_jobs(input_json: &str, view_json: &str) -> Result<Vec<String>, String> {
    let input: TacticalSceneInput = serde_json::from_str(input_json).map_err(|e| e.to_string())?;
    let request: venue::VenueRequest =
        serde_json::from_str(view_json).map_err(|e| e.to_string())?;
    let mut products = products();
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
            serde_json::to_string(&GenerationJob::Venue(Box::new(p))).map_err(|e| e.to_string())
        })
        .collect()
}

#[derive(Default, Serialize, Deserialize)]
struct Dependencies {
    sites: Vec<(
        BuildingProgram,
        adventuresim_tactical_core::scene_input::furniture::FurnitureSiteRecipe,
    )>,
    recipe: Option<adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe>,
    playable: Vec<adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe>,
}

fn dependencies(job_json: &str) -> Result<Vec<u8>, String> {
    let job: GenerationJob = serde_json::from_str(job_json).map_err(|e| e.to_string())?;
    let products = products();
    let mut data = Dependencies::default();
    match job {
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
    ciborium::into_writer(&data, &mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}

fn jobs(input_json: &str) -> Result<Vec<String>, String> {
    let input: TacticalSceneInput = serde_json::from_str(input_json).map_err(|e| e.to_string())?;
    input.validate().map_err(|e| e.to_string())?;
    let mut programs = Vec::new();
    let resident = products().resident_facades.clone();
    for placement in &input.distant_buildings {
        let program = placement.exterior_program();
        if !programs.contains(&program) && !resident.contains(&program) {
            programs.push(program);
        }
    }
    std::iter::once(GenerationJob::Scene(Box::new(input)))
        .chain(
            programs
                .into_iter()
                .map(|program| GenerationJob::Building(Box::new(program))),
        )
        .map(|job| serde_json::to_string(&job).map_err(|e| e.to_string()))
        .collect()
}

fn generate(job_json: &str, dependencies: &[u8]) -> Result<Vec<u8>, String> {
    let job: GenerationJob = serde_json::from_str(job_json).map_err(|e| e.to_string())?;
    let dependencies: Dependencies =
        ciborium::from_reader(dependencies).map_err(|e| e.to_string())?;
    let product = match job {
        GenerationJob::Scene(input) => {
            let mut recipes =
                adventuresim_tactical_core::scene_input::GeneratedBuildingRecipes::default();
            recipes.sites = dependencies.sites;
            for recipe in dependencies.playable {
                recipes.insert(recipe);
            }
            GenerationProduct::Scene(Box::new(
                input
                    .generate_unfurnished(recipes)
                    .map_err(|e| e.to_string())?,
            ))
        }
        GenerationJob::Building(program) => {
            GenerationProduct::Building(Box::new(PreparedFacade::generate(*program)?))
        }
        GenerationJob::Venue(program) => GenerationProduct::Venue(Box::new(
            venue::PreparedVenue::generate(*program, dependencies.recipe)?,
        )),
    };
    let mut bytes = Vec::new();
    ciborium::into_writer(&product, &mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}

fn receive(job_json: &str, bytes: &[u8]) -> Result<(), String> {
    let job: GenerationJob = serde_json::from_str(job_json).map_err(|e| e.to_string())?;
    let product: GenerationProduct = ciborium::from_reader(bytes).map_err(|e| e.to_string())?;
    match (job, product) {
        (GenerationJob::Scene(input), GenerationProduct::Scene(scene))
            if input.digest().map_err(|e| e.to_string())? == scene.digest =>
        {
            products().scenes.push(*scene);
        }
        (GenerationJob::Building(program), GenerationProduct::Building(facade))
            if *program == facade.program =>
        {
            let mut products = products();
            if !products.sites.iter().any(|(p, _)| p == &facade.program) {
                products
                    .sites
                    .push((facade.program.clone(), facade.site.clone()));
            }
            products.facades.push(*facade);
        }
        (GenerationJob::Venue(program), GenerationProduct::Venue(venue))
            if *program == venue.recipe.program =>
        {
            products().venues.push(*venue);
        }
        _ => return Err("generation product does not match its requested input".into()),
    }
    Ok(())
}
