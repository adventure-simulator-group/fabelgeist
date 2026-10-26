//! Private challenge records and their resolution-derived lifecycle.

use super::{ChallengePresenterCatalogId, NarrativeEncounterTrigger};
use adventuresim_core::strategic_place::CaseSiteId;
use adventuresim_world_schema::calendar::StrategicMinute;
use spacetimedb::table;

#[cfg(feature = "authority-tests")]
mod authority_tests;

/// A chat-native, non-puzzle interruption that becomes available only after
/// resting at its bound road camp. Choice presence owns resolution; openness
/// is derived. Ignoring it grants nothing but may carry a
/// bounded travel delay; choosing another response may affect the finale.
#[derive(Clone, Debug)]
#[table(accessor = road_challenge_authority)]
pub struct RoadChallengeAuthority {
    #[primary_key]
    pub id: String,
    #[index(btree)]
    pub gateway_bucket: u8,
    #[index(btree)]
    pub party_id: String,
    pub case_id: String,
    pub finale_case_site_id: Option<CaseSiteId>,
    pub finale_hostile_group_id: String,
    pub journey_departure_minute: StrategicMinute,
    pub camp_movement_minute: u64,
    pub available_at_elapsed_minute: u64,
    pub catalog_id: String,
    pub catalog_revision: u32,
    pub catalog_digest: String,
    pub absolute_minute: StrategicMinute,
    pub longitude_e7: i32,
    pub latitude_e7: i32,
    pub trigger: NarrativeEncounterTrigger,
    pub revision: u32,
    pub resolved_choice: Option<String>,
    pub resolved_deed: Option<String>,
    pub virtue_exemplified: Option<crate::personality::ChivalricVirtue>,
    pub result_transcript: Option<String>,
}

/// Private deterministic challenge authority. `puzzle_json` contains the seed
/// and canonical ordering and must never appear in a public table or view.
/// The solve timestamp owns completion; wrong attempts only advance revision.
#[derive(Clone, Debug)]
#[table(accessor = challenge_authority)]
pub struct ChallengeAuthority {
    #[primary_key]
    pub id: String,
    #[index(btree)]
    pub gateway_bucket: u8,
    #[index(btree)]
    pub case_id: String,
    #[index(btree)]
    pub party_id: String,
    pub finale_case_site_id: CaseSiteId,
    pub finale_hostile_group_id: String,
    pub journey_departure_minute: StrategicMinute,
    pub camp_movement_minute: u64,
    pub camp_elapsed_minute: u64,
    pub errantry_frame_json: String,
    pub puzzle_json: String,
    pub presenter_catalog_id: ChallengePresenterCatalogId,
    pub revision: u32,
    pub solved_at_minute: Option<StrategicMinute>,
}

impl ChallengeAuthority {
    pub(crate) fn is_open(&self) -> bool {
        self.solved_at_minute.is_none()
    }
}

impl RoadChallengeAuthority {
    pub(crate) fn is_open(&self) -> bool {
        self.resolved_choice.is_none()
    }
}
