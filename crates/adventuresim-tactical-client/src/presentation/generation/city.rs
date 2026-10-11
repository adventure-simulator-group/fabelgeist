//! Focused-city exteriors retain complete canonical placement and graded support.
use super::*;
use crate::presentation::vista::streets::prepared::PreparedCityGround;
use adventuresim_tactical_core::{prelude::SceneTerrain, regional_city::RegionalCityInput};

/// Static presentation products omit interiors, furniture and tactical actors.
/// The exact admitted document binds source, settlement and physical placement.
#[derive(Serialize, Deserialize)]
pub(in crate::presentation) struct PreparedCityProduct {
    pub document: RegionalCityInput,
    pub terrain: SceneTerrain,
    pub landform: PreparedTerrainLandform,
    pub surface_ground: adventuresim_tactical_core::prelude::SceneGround,
    pub ground: Arc<PreparedCityGround>,
    graphics: String,
}

impl PreparedCityProduct {
    pub(super) fn generate(
        document: RegionalCityInput,
        graphics: String,
    ) -> PreparationResult<Self> {
        let config = crate::presentation::config::TacticalGraphicsConfig::parse(&graphics)
            .map_err(|message| PreparationError::GraphicsConfiguration { message })?;
        let supported = document
            .input()
            .prepare_supported_terrain(&mut Default::default())?;
        let terrain_patch = document
            .input()
            .landform
            .map(|recipe| {
                adventuresim_tactical_core::prelude::terrain_landform_patch(
                    &supported.terrain,
                    recipe,
                )
            })
            .transpose()
            .map_err(|cause| {
                adventuresim_tactical_core::scene_input::SceneInputError::from(
                    adventuresim_tactical_core::scene_input::SceneValidationError::Terrain(cause),
                )
            })?;
        let landform = PreparedTerrainLandform::from(terrain_patch);
        let ground = PreparedCityGround::from_scene(
            document.input(),
            &supported.terrain,
            &landform,
            &[],
            config.rendering.vista.maximum_lods,
        )?;
        Ok(Self {
            document,
            terrain: supported.terrain,
            landform,
            surface_ground: supported.ground,
            ground: Arc::new(ground),
            graphics,
        })
    }

    pub(super) fn matches(&self, document: &RegionalCityInput, graphics: &str) -> bool {
        self.document == *document && self.graphics == graphics
    }
}

pub(super) fn jobs(
    ticket: PreparationTicket,
    document_json: &str,
    graphics: &str,
) -> PreparationResult<Vec<String>> {
    ticket.require_owner(PresentationOwner::RegionalMap)?;
    let products = staged_products(ticket)?;
    let document: RegionalCityInput = serde_json::from_str(document_json)?;
    crate::presentation::config::TacticalGraphicsConfig::parse(graphics)
        .map_err(|message| PreparationError::GraphicsConfiguration { message })?;
    let mut jobs = Vec::new();
    if !products
        .regional_city
        .as_ref()
        .is_some_and(|city| city.matches(&document, graphics))
    {
        jobs.push(GenerationJob::RegionalCity {
            input: Box::new(document.clone()),
            graphics: graphics.into(),
        });
    }
    let mut programs = Vec::new();
    for placement in document.input().physical_placements() {
        if !products.resident_facades().contains(&placement.program)
            && !programs.contains(&placement.program)
        {
            programs.push(placement.program);
        }
    }
    jobs.extend(
        programs
            .into_iter()
            .map(|program| GenerationJob::Building(Box::new(program))),
    );
    jobs.into_iter()
        .map(|job| serde_json::to_string(&job).map_err(PreparationError::from))
        .collect()
}
