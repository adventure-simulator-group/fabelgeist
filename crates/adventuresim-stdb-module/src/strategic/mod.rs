//! Persistent strategic authority, organized by gameplay domain.

//! Implementation is partitioned by behavior domain below. The fragments
//! intentionally share this module scope because SpacetimeDB macro discovery
//! and generated accessor names are scope-sensitive. Non-macro services use
//! ordinary child modules elsewhere; these ordered files preserve the exact
//! reducer/view/table ABI while keeping each gameplay domain navigable.

use adventuresim_core::encounter::EncounterChoice;
use adventuresim_core::strategic_state::vocabulary::{
    CourtshipStatus, HostileResolutionKind, MissionAttemptStatus,
};
use adventuresim_core::weather::Precipitation;
use adventuresim_dialogue::{PromptMode, ResolutionPolicy};
use std::str::FromStr;
mod authority;
pub(crate) use authority::{
    GatewayAdmissionError, StrategicCharacterAuthorityError, require_strategic_character_authority,
    require_strategic_gateway,
};
mod challenge_state;
mod departure;
mod encounter_admission;
use encounter_admission::unresolved_encounter;
pub(crate) use encounter_admission::{
    PendingEncounterError, require_character_no_unresolved_encounter,
    require_no_unresolved_encounter,
};
mod dialogue_policy;
use departure::{DepartureLocation, DepartureReadinessRule, revalidate_party_after_departure_sync};
#[path = "travel/redirect.rs"]
mod camp_redirect;
use camp_redirect::redirect_camped_party_to_settlement;
#[path = "travel/route_error.rs"]
mod route_error;
#[path = "travel/validation.rs"]
mod route_validation;
#[path = "travel/weather.rs"]
mod route_weather;
#[cfg(test)]
use route_validation::validate_journey_route_payload;
use route_validation::{
    validate_journey_route, validate_return_journey_route,
    validate_route_departure_weather_interval,
};
mod travel_error;
use travel_error::TravelError;
mod mission_roster;
mod party_members;
pub(crate) use party_members::living_party_member_ids;
mod party_readiness_error;
pub(crate) use party_readiness_error::PartyReadinessError;
#[cfg(feature = "authority-tests")]
mod outcome_fact_authority_tests;
#[cfg(test)]
use adventuresim_world_schema::calendar::MINUTES_PER_DAY;
use adventuresim_world_schema::calendar::StrategicMinute;
pub use challenge_state::*;

#[cfg(test)]
pub(crate) const STRATEGIC_SOURCE: &str = concat!(
    include_str!("autoresolve.rs"),
    include_str!("party_members.rs"),
    include_str!("party_members/selection.rs"),
    include_str!("party_readiness.rs"),
    include_str!("world_import.rs"),
    include_str!("authority_model.rs"),
    include_str!("authority.rs"),
    include_str!("authority/policy.rs"),
    include_str!("dialogue_schema.rs"),
    include_str!("dialogue_sessions.rs"),
    include_str!("dialogue_provenance.rs"),
    include_str!("dialogue_bindings.rs"),
    include_str!("dialogue_prompts.rs"),
    include_str!("dialogue_effects.rs"),
    include_str!("governance.rs"),
    include_str!("temporary_character_party.rs"),
    include_str!("inventory_trade.rs"),
    include_str!("hostile_negotiation.rs"),
    include_str!("hostile_surrender.rs"),
    include_str!("contracts.rs"),
    include_str!("travel_planning.rs"),
    include_str!("incidents.rs"),
    include_str!("journey_camp.rs"),
    include_str!("encounter_admission.rs"),
    include_str!("encounter_admission/policy.rs"),
    include_str!("encounters.rs"),
    include_str!("departure.rs"),
    include_str!("departure/error.rs"),
    include_str!("departure/policy.rs"),
    include_str!("travel_error.rs"),
    include_str!("travel/redirect.rs"),
    include_str!("travel/validation.rs"),
    include_str!("travel/route_error.rs"),
    include_str!("travel/weather.rs"),
    include_str!("travel_reducers.rs"),
    include_str!("custody_objectives.rs"),
    include_str!("development_scenarios.rs"),
    include_str!("tactical_enemy_fixture.rs"),
    include_str!("mission_bootstrap.rs"),
    include_str!("challenges.rs"),
    include_str!("challenge_state.rs"),
);

include!("autoresolve.rs");
include!("party_readiness.rs");
include!("world_import.rs");
include!("authority_model.rs");
include!("dialogue_schema.rs");
include!("dialogue_sessions.rs");
include!("dialogue_provenance.rs");
include!("dialogue_bindings.rs");
include!("dialogue_prompts.rs");
include!("dialogue_effects.rs");
include!("governance.rs");
include!("temporary_character_party.rs");
include!("inventory_trade.rs");
include!("hostile_negotiation.rs");
include!("hostile_surrender.rs");
include!("contracts.rs");
include!("travel_planning.rs");
include!("incidents.rs");
include!("journey_camp.rs");
include!("encounters.rs");
include!("travel_tests.rs");
include!("travel_reducers.rs");
include!("custody_objectives.rs");
include!("development_scenarios.rs");
include!("tactical_enemy_fixture.rs");
include!("mission_bootstrap.rs");
include!("challenges.rs");
include!("tests.rs");
#[path = "inventory_trade/streams.rs"]
mod inventory_trade_streams;
