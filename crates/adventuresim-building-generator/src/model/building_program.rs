use super::*;
mod fortified;
mod institutional;
mod residential;

/// High-level input recipe for procedural building generation.
///
/// The recipe is intentionally allowed to describe combinations that cannot be
/// built. The public [`crate::generate`] boundary is the validator: every
/// successful result has passed the complete structural audit, while an
/// unbuildable recipe returns [`crate::GenerationError`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BuildingProgram {
    pub archetype: BuildingArchetype,
    pub usage: Option<adventuresim_world_schema::settlement_buildings::BuildingUse>,
    pub seed: u64,
    pub footprint: Footprint,
    pub storey_height_metres: f32,
    pub storeys: Vec<StoreyProgram>,
    pub vertical_connections: Vec<VerticalConnectionRequirement>,
    pub wall_style: WallStyle,
    pub timber_frame_style: Option<TimberFrameStyle>,
    pub upper_storey_projection_metres: f32,
    pub roof_pitch_degrees: f32,
    /// Optional explicit kernel demonstrator used by deterministic proof plans.
    /// Curated archetypes leave this unset.
    #[serde(default)]
    pub roof_demonstrator: Option<RoofKind>,
    /// Present only when a church-specific structural program, rather than
    /// the generic room allocator, is authoritative.
    #[serde(default)]
    pub church_program: Option<ChurchProgram>,
}

impl BuildingProgram {
    pub fn fixture(archetype: BuildingArchetype, seed: u64) -> Self {
        match archetype {
            BuildingArchetype::ParishChurch => Self::parish_church(seed),
            BuildingArchetype::TownHouse => Self::town_house(seed),
            BuildingArchetype::HallHouse => Self::hall_house(seed),
            BuildingArchetype::FachwerkCottage => Self::fachwerk_cottage(seed),
            BuildingArchetype::FachwerkMerchantHouse => Self::fachwerk_merchant_house(seed),
            BuildingArchetype::RenaissanceTownHall => Self::renaissance_town_hall(seed),
            BuildingArchetype::Cathedral => Self::cathedral(seed),
            BuildingArchetype::CastleGatehouse => Self::castle_gatehouse(seed),
            BuildingArchetype::CourtyardCastle => Self::courtyard_castle(seed),
            BuildingArchetype::WalledKeep => Self::walled_keep(seed),
            BuildingArchetype::ArtilleryRondelCastle => Self::artillery_rondel_castle(seed),
        }
    }
}

pub const BUILDING_DOCUMENT_SCHEMA_VERSION: u32 = 4;
