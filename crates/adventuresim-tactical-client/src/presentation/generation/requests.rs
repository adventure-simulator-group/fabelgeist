//! Dependency capture and job discovery within a preparation ticket.
use super::*;

pub(super) fn venue_jobs(
    ticket: PreparationTicket,
    input_json: &str,
    view_json: &str,
) -> PreparationResult<Vec<String>> {
    ticket.require_owner(PresentationOwner::Scene)?;
    let input: TacticalSceneInput = serde_json::from_str(input_json)?;
    let request: venue::VenueRequest = serde_json::from_str(view_json)?;
    let mut products = staged_products(ticket)?;
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

pub(super) fn dependencies(
    ticket: PreparationTicket,
    job_json: &str,
) -> PreparationResult<Vec<u8>> {
    let job: GenerationJob = serde_json::from_str(job_json)?;
    job.require_owner(ticket)?;
    let products = staged_products(ticket)?;
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
        GenerationJob::Building(_) | GenerationJob::RegionalCity { .. } => {}
    }
    let mut bytes = Vec::new();
    ciborium::into_writer(&data, &mut bytes).map_err(PreparationError::DependenciesEncode)?;
    Ok(bytes)
}

pub(super) fn jobs(ticket: PreparationTicket, input_json: &str) -> PreparationResult<Vec<String>> {
    ticket.require_owner(PresentationOwner::Scene)?;
    let input: TacticalSceneInput = serde_json::from_str(input_json)?;
    input.validate()?;
    let mut programs = Vec::new();
    let mut products = staged_products(ticket)?;
    let readiness = products.scene_readiness(&input)?;
    let resident = products.resident_facades().to_vec();
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
