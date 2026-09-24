//! Observer-safe site location and current ordinary activity eligibility.
use super::*;

#[derive(Clone, Debug, SpacetimeType)]
pub struct BackendCaseSitePin {
    pub owner_character_id: u64,
    pub case_id: String,
    pub case_site_id: CaseSiteId,
    pub origin_settlement_id: String,
    pub name: String,
    pub description: String,
    pub scene_key: String,
    pub longitude_e7: i32,
    pub latitude_e7: i32,
    pub coordinates_are_geographic: bool,
    pub distance_m: u64,
    /// Current ordinary activity eligibility, independent of public case aliases.
    pub raiding_allowed: bool,
    pub knowledge_stage: DestinationKnowledgeStage,
    pub tracked: bool,
    /// Observer-safe problem wording from a fully validated generated manifest,
    /// or the ordinary site name for a manual case.
    pub display_title: String,
    /// Generated presentation is deliberately independent of contract state.
    pub generated_case: bool,
    /// Observer-safe completion state for the generated case.
    pub case_resolved: bool,
    /// This reveals only that combat is currently a permitted onsite action.
    pub combat_available: bool,
    /// Present only with `combat_available`; aggregate observer-safe strength
    /// of the exact generated hostile group, never hostile identity.
    pub opposition_count: Option<u32>,
    pub opposition_combat_power: Option<u64>,
}

#[derive(Clone, Debug, SpacetimeType)]
pub struct BackendCharacterCaseSiteLocation {
    pub character_id: u64,
    pub case_site_id: CaseSiteId,
}
