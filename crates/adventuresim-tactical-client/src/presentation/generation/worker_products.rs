//! Pure worker execution and ticket-checked product admission.
use super::*;

pub(super) fn generate(job_json: &str, dependencies: &[u8]) -> PreparationResult<Vec<u8>> {
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

pub(super) fn receive(
    ticket: PreparationTicket,
    job_json: &str,
    bytes: &[u8],
) -> PreparationResult<()> {
    let mut products = staged_products(ticket)?;
    let job: GenerationJob = serde_json::from_str(job_json)?;
    let product: GenerationProduct =
        ciborium::from_reader(bytes).map_err(PreparationError::ProductDecode)?;
    match (job, product) {
        (GenerationJob::Grass { input, graphics }, GenerationProduct::Grass(grass))
            if input.digest()? == grass.digest && graphics == grass.graphics =>
        {
            products.grass.push(*grass);
            if products.grass.len() > RETAINED_SCENE_PRODUCTS {
                products.grass.remove(0);
            }
        }

        (GenerationJob::Ground { input, graphics }, GenerationProduct::Ground(ground))
            if input.digest()? == ground.digest && graphics == ground.graphics =>
        {
            products.retain_ground(*ground);
        }
        (GenerationJob::Scene(input), GenerationProduct::Scene(scene))
            if input.digest()? == scene.digest() =>
        {
            let scene = scene.restore(&input, &products)?;
            products.retain_scene(scene);
        }
        (GenerationJob::Building(program), GenerationProduct::Building(facade))
            if *program == facade.program =>
        {
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
            products.facades.push(Arc::new(*facade));
        }
        (GenerationJob::Venue(program), GenerationProduct::Venue(venue))
            if *program == venue.recipe.program =>
        {
            products.venues.push(*venue);
        }
        _ => return Err(PreparationError::ProductMismatch),
    }
    Ok(())
}
