//! Pure worker execution and ticket-checked product admission.
use super::*;

pub(super) fn generate(job_json: &str, dependencies: &[u8]) -> PreparationResult<Vec<u8>> {
    let job: GenerationJob = serde_json::from_str(job_json)?;
    let dependencies: Dependencies =
        ciborium::from_reader(dependencies).map_err(PreparationError::DependenciesDecode)?;
    let product = match job {
        GenerationJob::RegionalCity { input, graphics } => GenerationProduct::RegionalCity(
            Box::new(city::PreparedCityProduct::generate(*input, graphics)?),
        ),
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
    job.require_owner(ticket)?;
    let product: GenerationProduct =
        ciborium::from_reader(bytes).map_err(PreparationError::ProductDecode)?;
    match (job, product) {
        (
            GenerationJob::RegionalCity { input, graphics },
            GenerationProduct::RegionalCity(city),
        ) if city.matches(&input, &graphics) => {
            ticket.require_owner(PresentationOwner::RegionalMap)?;
            products.regional_city = Some(Arc::new(*city));
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
