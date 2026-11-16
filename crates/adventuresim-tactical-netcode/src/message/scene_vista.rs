//! Captured town presentation delivered once to each enrolled connection.

use super::*;

/// One bounded, ordered scene asset delivered only to an enrolled client.
/// Vista samples intentionally bypass ordinary ECS component replication.
#[derive(Debug, Clone, Event, Serialize, Deserialize)]
pub struct SceneVistaBundle {
    pub properties: Option<adventuresim_core::settlement_property::GeneratedHomeCatalog>,
    pub scene_digest: String,
    /// Half-width and half-depth of the authoritative playable heightfield.
    /// Presentation-only vista rings clip exactly to this rectangle.
    pub playable_half_extent_metres: Vec2,
    /// Presentation-only shell instances surrounding the tactical area.
    pub distant_buildings: Vec<DistantBuildingPlacement>,
    /// Operator-backed identity for playable and distant businesses.
    pub establishments: Vec<SceneEstablishment>,
    /// The same connected surface network used to classify playable ground.
    pub streets: Vec<CityStreetPatch>,
    /// Developed block interiors that replace blanket meadow ground cover.
    pub yards: Vec<CityYardPatch>,
    pub parishes: Vec<adventuresim_tactical_core::city_layout::CityParish>,
    pub compounds: Vec<CityCompound>,
    pub gardens: Vec<CityGarden>,
    /// Accepted activity areas drive local ground wear around outdoor furniture.
    pub furniture_groups: Vec<FurnitureGroup>,
    /// Accepted visual furniture outside the tactical world bounds.
    pub distant_furniture: Vec<GeneratedFurniture>,
    pub lods: Vec<VistaLod>,
}

impl TryFrom<&TacticalSceneInput> for SceneVistaBundle {
    type Error = adventuresim_tactical_core::scene_input::SceneInputError;

    fn try_from(input: &TacticalSceneInput) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            properties: input.properties.clone(),
            scene_digest: input.digest()?,
            playable_half_extent_metres: Vec2::new(
                f32::from(input.playable.width.saturating_sub(1))
                    * input.playable.spacing_metres
                    * 0.5,
                f32::from(input.playable.depth.saturating_sub(1))
                    * input.playable.spacing_metres
                    * 0.5,
            ),
            distant_buildings: input.distant_buildings.clone(),
            establishments: input.establishments.clone(),
            streets: input.streets.clone(),
            yards: input.yards.clone(),
            parishes: input.parishes.clone(),
            compounds: input.compounds.clone(),
            gardens: input.gardens.clone(),
            furniture_groups: Vec::new(),
            distant_furniture: Vec::new(),
            lods: input.vista.lods.clone(),
        })
    }
}
