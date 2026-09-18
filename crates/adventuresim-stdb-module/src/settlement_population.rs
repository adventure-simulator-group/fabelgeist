//! Persistent strategic settlement residents and authoritative observable presences.
use crate::{
    character::{NpcLifeFacts, character, character__view, insert_persistent_npc_character},
    personality::{Presentation, Sex, character_personality, character_personality__view},
    relationship::{NpcPolicy, npc_policy},
    strategic::{settlement, strategic_gateway_authority__view},
};
use adventuresim_core::settlement_population::{
    self as population, AgeBand, GenerationInput, LocationContext, PresenceBridge, Profession,
    Schedule,
};
use adventuresim_core::strategic_place::{SettlementVenueKind, StrategicPlaceId};
use adventuresim_core::strategic_presence::{
    DailyPresenceWindow, PresenceFrontier, ScheduledStrategicPresence, StrategicPresence,
};
use adventuresim_core::strategic_time::MINUTES_PER_DAY;
use adventuresim_world_schema::settlement_buildings::BusinessId;
use serde::{Deserialize, Serialize};
use spacetimedb::{ReducerContext, SpacetimeType, Table, ViewContext, table, view};
use std::collections::BTreeSet;
use std::ops::Deref;

#[derive(Clone, Copy, Debug, SpacetimeType)]
pub enum NpcAgeBand {
    Child,
    Adolescent,
    Adult,
    Elder,
}

impl NpcAgeBand {
    pub(crate) const fn stable_id(self) -> &'static str {
        match self {
            Self::Child => "child",
            Self::Adolescent => "adolescent",
            Self::Adult => "adult",
            Self::Elder => "elder",
        }
    }

    pub(crate) const fn stable_variant_id(self) -> &'static str {
        match self {
            Self::Child => "Child",
            Self::Adolescent => "Adolescent",
            Self::Adult => "Adult",
            Self::Elder => "Elder",
        }
    }
}
#[derive(Clone, Copy, Debug, SpacetimeType)]
pub enum NpcPresentation {
    Man,
    Ambiguous,
    Woman,
}

#[derive(Clone, Debug)]
#[table(accessor = settlement_resident_profile)]
pub struct SettlementResidentProfile {
    #[primary_key]
    pub character_id: u64,
    /// Bounded traversal key for the fail-closed gateway view.
    #[index(btree)]
    pub projection_id: u64,
    #[index(btree)]
    pub home_settlement_id: String,
    pub height: String,
    pub build: String,
    pub hair: String,
    pub facial_hair: String,
    pub complexion: String,
    pub visible_features: String,
    pub clothing: String,
    pub profession: String,
    pub household: String,
    pub local_role: String,
    pub service_id: String,
    /// Explicit institution authority. Empty for NPCs not representing an organization.
    pub organization_id: String,
    pub conversation_id: String,
}

/// Reducer-local join of a resident's presentation metadata with the ordinary
/// Character and private personality components that own identity and
/// demographics. This is never persisted as a second person record.
#[derive(Clone, Debug)]
pub struct ResolvedSettlementResident {
    pub profile: SettlementResidentProfile,
    pub name: String,
    pub age_band: NpcAgeBand,
    pub sex: crate::personality::Sex,
    pub presentation: crate::personality::Presentation,
}

impl Deref for ResolvedSettlementResident {
    type Target = SettlementResidentProfile;

    fn deref(&self) -> &Self::Target {
        &self.profile
    }
}

pub fn resolve_settlement_resident(
    ctx: &ReducerContext,
    character_id: u64,
) -> Option<ResolvedSettlementResident> {
    let profile = ctx
        .db
        .settlement_resident_profile()
        .character_id()
        .find(character_id)?;
    let character = ctx.db.character().id().find(character_id)?;
    let personality = ctx
        .db
        .character_personality()
        .character_id()
        .find(character_id)?;
    Some(ResolvedSettlementResident {
        profile,
        name: character.name,
        age_band: match character.age_years {
            0..=12 => NpcAgeBand::Child,
            13..=17 => NpcAgeBand::Adolescent,
            18..=59 => NpcAgeBand::Adult,
            _ => NpcAgeBand::Elder,
        },
        sex: personality.sex,
        presentation: personality.presentation,
    })
}

pub fn resolve_settlement_resident_view(
    ctx: &ViewContext,
    character_id: u64,
) -> Option<ResolvedSettlementResident> {
    let profile = ctx
        .db
        .settlement_resident_profile()
        .character_id()
        .find(character_id)?;
    let character = ctx.db.character().id().find(character_id)?;
    let personality = ctx
        .db
        .character_personality()
        .character_id()
        .find(character_id)?;
    Some(ResolvedSettlementResident {
        profile,
        name: character.name,
        age_band: match character.age_years {
            0..=12 => NpcAgeBand::Child,
            13..=17 => NpcAgeBand::Adolescent,
            18..=59 => NpcAgeBand::Adult,
            _ => NpcAgeBand::Elder,
        },
        sex: personality.sex,
        presentation: personality.presentation,
    })
}

/// Settlement NPC facts that the registered gateway may project to players.
///
/// Keep this row explicit: `SettlementResidentProfile` also contains private demographic
/// and traversal authority that must never become subscription data merely
/// because the authoritative table gains another field.
#[derive(Clone, Debug, SpacetimeType)]
pub struct BackendSettlementResident {
    pub character_id: u64,
    pub home_settlement_id: String,
    pub name: String,
    pub age_band: NpcAgeBand,
    pub presentation: NpcPresentation,
    pub height: String,
    pub build: String,
    pub hair: String,
    pub facial_hair: String,
    pub complexion: String,
    pub visible_features: String,
    pub clothing: String,
    pub profession: String,
    pub household: String,
    pub local_role: String,
    pub service_id: String,
    pub organization_id: String,
    pub conversation_id: String,
}

fn project_backend_settlement_resident(
    ctx: &ViewContext,
    profile: SettlementResidentProfile,
) -> Option<BackendSettlementResident> {
    let character = ctx.db.character().id().find(profile.character_id)?;
    let personality = ctx
        .db
        .character_personality()
        .character_id()
        .find(profile.character_id)?;
    let age_band = match character.age_years {
        0..=12 => NpcAgeBand::Child,
        13..=17 => NpcAgeBand::Adolescent,
        18..=59 => NpcAgeBand::Adult,
        _ => NpcAgeBand::Elder,
    };
    let presentation = match personality.presentation {
        Presentation::Man => NpcPresentation::Man,
        Presentation::Ambiguous => NpcPresentation::Ambiguous,
        Presentation::Woman => NpcPresentation::Woman,
    };
    Some(BackendSettlementResident {
        character_id: profile.character_id,
        home_settlement_id: profile.home_settlement_id,
        name: character.name,
        age_band,
        presentation,
        height: profile.height,
        build: profile.build,
        hair: profile.hair,
        facial_hair: profile.facial_hair,
        complexion: profile.complexion,
        visible_features: profile.visible_features,
        clothing: profile.clothing,
        profession: profile.profession,
        household: profile.household,
        local_role: profile.local_role,
        service_id: profile.service_id,
        organization_id: profile.organization_id,
        conversation_id: profile.conversation_id,
    })
}

pub(crate) fn resident_is_dialogue_capable(profile: &SettlementResidentProfile) -> bool {
    adventuresim_dialogue::find_conversation(&profile.conversation_id).is_some_and(|conversation| {
        conversation
            .roles
            .values()
            .any(|role| role.kind == adventuresim_dialogue::ParticipantKind::Player)
            && conversation
                .roles
                .values()
                .any(|role| role.kind == adventuresim_dialogue::ParticipantKind::Npc)
    })
}

#[view(accessor = backend_settlement_residents, public)]
pub fn backend_settlement_residents(ctx: &ViewContext) -> Vec<BackendSettlementResident> {
    let trusted = ctx
        .db
        .strategic_gateway_authority()
        .id()
        .find(0)
        .is_some_and(|authority| authority.identity == ctx.sender());
    if !trusted {
        return Vec::new();
    }
    ctx.db
        .settlement_resident_profile()
        .projection_id()
        .filter(0u64..)
        .filter(resident_is_dialogue_capable)
        .filter_map(|profile| project_backend_settlement_resident(ctx, profile))
        .collect()
}

/// Public presence contains only directly observable scheduling and location facts.
#[derive(Clone, Debug)]
#[table(accessor = settlement_resident_presence, public)]
pub struct SettlementResidentPresence {
    #[primary_key]
    pub character_id: u64,
    #[index(btree)]
    pub settlement_id: String,
    #[index(btree)]
    pub location_id: String,
    pub start_minute: u16,
    pub end_minute: u16,
    pub is_default: bool,
    /// Shared schedule/service suppression while another active context owns
    /// this Character's physical presence. The authored schedule is retained.
    pub context_suppressed: bool,
    /// Ordinary health availability, independent of quest/context lifecycle.
    pub health_suppressed: bool,
}

#[derive(Clone, Debug)]
#[table(accessor = settlement_resident_seed_explanation)]
pub struct SettlementResidentSeedExplanation {
    #[primary_key]
    pub character_id: u64,
    pub seed: String,
    pub relations_json: String,
}

/// Authoritative one-to-one assignment of a generated business to its resident operator.
#[derive(Clone, Debug)]
#[table(accessor = settlement_business_operator)]
pub struct SettlementBusinessOperator {
    #[primary_key]
    pub id: String,
    #[index(btree)]
    pub settlement_id: String,
    pub business_id: BusinessId,
    #[unique]
    pub operator_character_id: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedGenerationExplanation {
    input: GenerationInput,
    profile: population::GeneratedPopulationProfile,
}

mod resident_drafts;
const RESIDENT_BRIDGES: [PresenceBridge; 3] = [
    PresenceBridge::NearbyHome,
    PresenceBridge::HouseholdErrand,
    PresenceBridge::RetainerErrand,
];
use resident_drafts::{ResidentDraft, settlement_resident_drafts};

fn business_row_id(business_id: &BusinessId) -> Result<String, String> {
    serde_json::to_string(business_id)
        .map(|key| format!("business-operator:{key}"))
        .map_err(|error| format!("Could not serialize business identity: {error}"))
}
use adventuresim_world_schema::person_names::{FEMALE_NAMES, MALE_NAMES, SURNAMES};

#[derive(Clone, Copy)]
enum PopulationLocation<'a> {
    Overview,
    Market,
    Forge,
    Armoury,
    Tailor,
    Herbalist,
    Bookstore,
    Inn,
    Church,
    Residences,
    Keep,
    Organization(&'a str),
}

impl<'a> PopulationLocation<'a> {
    fn parse(value: &'a str) -> Result<Self, String> {
        match value {
            "overview" => Ok(Self::Overview),
            "market" => Ok(Self::Market),
            "forge" => Ok(Self::Forge),
            "armoury" => Ok(Self::Armoury),
            "tailor" => Ok(Self::Tailor),
            "herbalist" => Ok(Self::Herbalist),
            "bookstore" => Ok(Self::Bookstore),
            "inn" => Ok(Self::Inn),
            "church" => Ok(Self::Church),
            "residences" => Ok(Self::Residences),
            "keep" => Ok(Self::Keep),
            _ => value
                .strip_prefix("organization-")
                .filter(|organization_id| !organization_id.is_empty())
                .map(Self::Organization)
                .ok_or_else(|| format!("Unknown population location {value}")),
        }
    }

    const fn context(self) -> LocationContext {
        match self {
            Self::Overview => LocationContext::Overview,
            Self::Market | Self::Bookstore => LocationContext::Market,
            Self::Forge => LocationContext::Forge,
            Self::Armoury => LocationContext::Armoury,
            Self::Tailor => LocationContext::Tailor,
            Self::Herbalist => LocationContext::Herbalist,
            Self::Inn => LocationContext::Inn,
            Self::Church => LocationContext::Church,
            Self::Residences => LocationContext::Residences,
            Self::Keep => LocationContext::Keep,
            Self::Organization(_organization_id) => LocationContext::Organization,
        }
    }
}

fn location_context(location: &str) -> Result<LocationContext, String> {
    Ok(PopulationLocation::parse(location)?.context())
}
fn age(value: AgeBand) -> NpcAgeBand {
    match value {
        AgeBand::Child => NpcAgeBand::Child,
        AgeBand::Adolescent => NpcAgeBand::Adolescent,
        AgeBand::Adult => NpcAgeBand::Adult,
        AgeBand::Elder => NpcAgeBand::Elder,
    }
}
fn profession(value: Profession) -> &'static str {
    match value {
        Profession::Artisan => "artisan",
        Profession::Householder => "householder",
        Profession::Laborer => "laborer",
        Profession::Retainer => "retainer",
        Profession::ServiceProvider => "service provider",
    }
}

#[derive(Clone, Copy)]
enum ResidentEntropyStream {
    Identity,
    Sex,
    GivenName,
    Surname,
    FacialHair,
    HouseholdName,
    Complexion,
    VisibleFeature,
}

impl ResidentEntropyStream {
    const fn stream(self) -> fabelgeist_determinism::StreamId {
        fabelgeist_determinism::StreamId::new(match self {
            Self::Identity => "resident.identity",
            Self::Sex => "resident.sex",
            Self::GivenName => "resident.given-name",
            Self::Surname => "resident.surname",
            Self::HouseholdName => "resident.household-name",
            Self::Complexion => "resident.complexion",
            Self::VisibleFeature => "resident.visible-feature",
            Self::FacialHair => "resident.facial-hair",
        })
    }
}

fn resident_random(
    seed: &str,
    stream: ResidentEntropyStream,
) -> fabelgeist_determinism::DeterministicRng {
    fabelgeist_determinism::Seed::derive(seed.as_bytes(), stream.stream(), &[]).rng()
}

fn resident_seed(settlement_id: &str, location: &str, ordinal: usize) -> String {
    format!("resident:{settlement_id}:{location}:{ordinal}")
}

fn organization_representative_seed(settlement_id: &str, organization_id: &str) -> String {
    format!("resident:organization-representative:{settlement_id}:{organization_id}")
}

fn resident_name(seed: &str, female: bool) -> String {
    let names = if female { &FEMALE_NAMES } else { &MALE_NAMES };
    let given = names[resident_random(seed, ResidentEntropyStream::GivenName).index(names.len())];
    let surname =
        SURNAMES[resident_random(seed, ResidentEntropyStream::Surname).index(SURNAMES.len())];
    format!("{given} {surname}")
}

fn resident_character_id(seed: &str) -> u64 {
    // Keep generated residents in the upper half of the identity space. The
    // stable source coordinate is the identity; there is no parallel string ID.
    resident_random(seed, ResidentEntropyStream::Identity).next_u64() | (1u64 << 63)
}

fn ensure_business_operator_row(
    ctx: &ReducerContext,
    business_id: Option<&BusinessId>,
    operator_character_id: u64,
) -> Result<(), String> {
    let Some(business_id) = business_id else {
        return Ok(());
    };
    let id = business_row_id(business_id)?;
    if let Some(existing) = ctx.db.settlement_business_operator().id().find(&id) {
        return (existing.business_id == *business_id
            && existing.operator_character_id == operator_character_id)
            .then_some(())
            .ok_or_else(|| "Business operator identity conflicts with existing authority".into());
    }
    if ctx
        .db
        .settlement_business_operator()
        .operator_character_id()
        .find(operator_character_id)
        .is_some()
    {
        return Err("Settlement resident is already assigned to another business".into());
    }
    ctx.db
        .settlement_business_operator()
        .insert(SettlementBusinessOperator {
            id,
            settlement_id: business_id.settlement_id.clone(),
            business_id: business_id.clone(),
            operator_character_id,
        });
    Ok(())
}

mod resident_persistence;
use resident_persistence::insert_resident_draft;

pub fn ensure_settlement_population(
    ctx: &ReducerContext,
    settlement_id: &str,
) -> Result<(), String> {
    crate::social_roles::ensure_settlement_social_organizations(ctx, settlement_id)?;
    let (drafts, household_groups) = settlement_resident_drafts(ctx, settlement_id)?;
    for draft in drafts {
        insert_resident_draft(ctx, settlement_id, draft)?;
    }
    for organization in adventuresim_core::organization::organizations_for_chapter(settlement_id) {
        let representative_character_id =
            adventuresim_core::organization::organization_representative_id(
                settlement_id,
                &organization.id,
            );
        let mut representative = ctx
            .db
            .settlement_resident_profile()
            .character_id()
            .find(representative_character_id)
            .ok_or("Organization representative was not seeded")?;
        representative.service_id.clear();
        representative.organization_id = organization.id.clone();
        representative.conversation_id = "organization-representative".into();
        representative.clothing =
            "well-kept clothing bearing the institution's public insignia".into();
        ctx.db
            .settlement_resident_profile()
            .character_id()
            .update(representative);
    }
    crate::relationship::ensure_seeded_family_households(ctx, settlement_id, &household_groups)?;
    Ok(())
}
pub fn npc_is_present(
    ctx: &ReducerContext,
    presence: &SettlementResidentPresence,
    minute: u64,
) -> bool {
    npc_presence_remaining_minutes_at(ctx, presence, minute).is_some()
}

/// Canonical exact place behind an authoritative settlement NPC location.
/// The `overview` route is the presentation alias for the public square.
pub fn canonical_npc_place(settlement_id: &str, location_id: &str) -> Option<StrategicPlaceId> {
    let venue = if matches!(location_id, "overview" | "public-square") {
        Some(SettlementVenueKind::PublicSquare)
    } else {
        SettlementVenueKind::from_id(location_id)
    };
    if let Some(kind) = venue {
        return StrategicPlaceId::settlement_venue(settlement_id, kind).ok();
    }
    let (organization, chapter) =
        adventuresim_core::organization::organization_chapter_at(settlement_id, location_id)?;
    StrategicPlaceId::chapter_venue(settlement_id, &organization.id, &chapter.location_id).ok()
}

/// Typed scheduled presence at the actor-relative personal minute. Historical
/// outbreak state is reconstructed without mutating or reading future state.
pub fn npc_strategic_presence_at(
    ctx: &ReducerContext,
    presence: &SettlementResidentPresence,
    observer_character_id: u64,
    minute: u64,
) -> Option<ScheduledStrategicPresence> {
    let suppression =
        crate::outbreak::patient_presence_suppression_at(ctx, presence.character_id, minute)?;
    let alive = crate::relationship::character_alive_at(ctx, presence.character_id, minute);
    StrategicPresence::scheduled_resident(
        presence.character_id,
        canonical_npc_place(&presence.settlement_id, &presence.location_id)?,
        PresenceFrontier {
            observer_character_id,
            personal_minute: minute,
        },
        DailyPresenceWindow {
            start_minute: presence.start_minute,
            end_minute: presence.end_minute,
        },
        alive,
        suppression.context_suppressed,
        suppression.health_suppressed,
    )
    .ok()
}

/// Remaining contiguous availability at an authoritative historical minute,
/// including outbreak suppression. This duration query does not need an
/// observer-scoped presence value.
pub fn npc_presence_remaining_minutes_at(
    ctx: &ReducerContext,
    presence: &SettlementResidentPresence,
    minute: u64,
) -> Option<u64> {
    let suppression =
        crate::outbreak::patient_presence_suppression_at(ctx, presence.character_id, minute)?;
    DailyPresenceWindow {
        start_minute: presence.start_minute,
        end_minute: presence.end_minute,
    }
    .remaining_minutes(
        minute,
        suppression.context_suppressed,
        suppression.health_suppressed,
    )
    .ok()
}

pub fn npc_presence_remaining_minutes_at_view(
    ctx: &ViewContext,
    presence: &SettlementResidentPresence,
    minute: u64,
) -> Option<u64> {
    let suppression =
        crate::outbreak::patient_presence_suppression_at_view(ctx, presence.character_id, minute)?;
    DailyPresenceWindow {
        start_minute: presence.start_minute,
        end_minute: presence.end_minute,
    }
    .remaining_minutes(
        minute,
        suppression.context_suppressed,
        suppression.health_suppressed,
    )
    .ok()
}

/// Remaining contiguous minutes in the NPC's current daily presence window.
/// Wrapped schedules (for example 20:00–02:00) remain one continuous window.
pub fn npc_presence_remaining_minutes(
    presence: &SettlementResidentPresence,
    minute: u64,
) -> Option<u64> {
    DailyPresenceWindow {
        start_minute: presence.start_minute,
        end_minute: presence.end_minute,
    }
    .remaining_minutes(
        minute,
        presence.context_suppressed,
        presence.health_suppressed,
    )
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_npc_places_use_physical_venue_identity() {
        let overview = canonical_npc_place("lubeck", "overview").unwrap();
        let square = canonical_npc_place("lubeck", "public-square").unwrap();
        let inn = canonical_npc_place("lubeck", "inn").unwrap();

        assert_eq!(overview, square);
        assert_ne!(overview, inn);
        assert!(canonical_npc_place("lubeck", "unknown-route-value").is_none());
    }

    fn presence(start_minute: u16, end_minute: u16) -> SettlementResidentPresence {
        SettlementResidentPresence {
            character_id: 42,
            settlement_id: "settlement".into(),
            location_id: "inn".into(),
            start_minute,
            end_minute,
            is_default: true,
            context_suppressed: false,
            health_suppressed: false,
        }
    }

    #[test]
    fn presence_remaining_handles_daytime_and_wrapped_schedules() {
        let daytime = presence(480, 1_020);
        assert_eq!(npc_presence_remaining_minutes(&daytime, 900), Some(120));
        assert_eq!(npc_presence_remaining_minutes(&daytime, 1_020), None);

        let overnight = presence(1_200, 120);
        assert_eq!(npc_presence_remaining_minutes(&overnight, 1_380), Some(180));
        assert_eq!(npc_presence_remaining_minutes(&overnight, 60), Some(60));
        assert_eq!(npc_presence_remaining_minutes(&overnight, 600), None);
    }

    #[test]
    fn contextual_membership_suppresses_without_rewriting_schedule() {
        let mut row = presence(480, 1_020);
        row.context_suppressed = true;
        assert_eq!(npc_presence_remaining_minutes(&row, 900), None);
        assert_eq!((row.start_minute, row.end_minute), (480, 1_020));

        row.context_suppressed = false;
        assert_eq!(npc_presence_remaining_minutes(&row, 900), Some(120));

        row.health_suppressed = true;
        assert_eq!(npc_presence_remaining_minutes(&row, 900), None);
        assert_eq!((row.start_minute, row.end_minute), (480, 1_020));
    }

    #[test]
    fn persisted_explanation_is_valid_complete_and_deterministic() {
        let input = GenerationInput {
            seed: "npc:test".into(),
            location: LocationContext::Overview,
            is_service_provider: false,
            service_id: None,
            profession_override: None,
            local_role: "resident".into(),
            age: None,
            available_bridges: BTreeSet::from([
                PresenceBridge::NearbyHome,
                PresenceBridge::HouseholdErrand,
                PresenceBridge::RetainerErrand,
            ]),
        };
        let profile = population::generate(&input).unwrap();
        let value = PersistedGenerationExplanation { input, profile };
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(value, serde_json::from_str(&json).unwrap());
        assert_eq!(json, serde_json::to_string(&value).unwrap());
        assert_eq!(value.profile.decisions.len(), 7);
    }

    #[test]
    fn backend_settlement_resident_view_is_an_explicit_fail_closed_projection() {
        let source = crate::production_source(include_str!("settlement_population.rs"));
        let row = source
            .split("pub struct BackendSettlementResident {")
            .nth(1)
            .and_then(|tail| tail.split_once('}').map(|(body, _)| body))
            .expect("backend settlement NPC row");
        for field in [
            "character_id",
            "home_settlement_id",
            "name",
            "age_band",
            "presentation",
            "height",
            "build",
            "hair",
            "facial_hair",
            "complexion",
            "visible_features",
            "clothing",
            "profession",
            "household",
            "local_role",
            "service_id",
            "organization_id",
            "conversation_id",
        ] {
            assert!(
                row.contains(&format!("pub {field}:")),
                "missing player-visible field {field}"
            );
        }
        assert!(!row.contains("sex:"));
        assert!(!row.contains("projection_id:"));

        let view = source
            .split("pub fn backend_settlement_residents")
            .nth(1)
            .and_then(|tail| tail.split("/// Public presence contains").next())
            .expect("backend settlement NPC view");
        assert!(view.contains("-> Vec<BackendSettlementResident>"));
        assert!(view.contains(".filter(resident_is_dialogue_capable)"));
        assert!(
            view.contains(
                ".filter_map(|profile| project_backend_settlement_resident(ctx, profile))"
            )
        );
        assert!(!view.contains("-> Vec<SettlementResidentProfile>"));
    }

    #[test]
    fn every_authored_chapter_seeds_one_bound_persistent_representative() {
        let source = crate::production_source(include_str!("settlement_population.rs"));
        let drafts =
            crate::production_source(include_str!("settlement_population/resident_drafts.rs"));
        assert!(drafts.contains("organizations_for_chapter(settlement_id)"));
        assert!(drafts.contains("chapter_effective_location_id"));
        assert!(drafts.contains("organization_representative_seed"));
        assert!(drafts.contains("organization_representative_id"));
        assert!(source.contains("fn resident_seed("));
        assert!(source.contains("fn organization_representative_seed("));
        assert!(drafts.contains("location == chapter.location_id"));

        let ensure = source
            .split("pub fn ensure_settlement_population")
            .nth(1)
            .and_then(|tail| tail.split("pub fn npc_is_present").next())
            .expect("population seeding body");
        assert!(ensure.contains("insert_resident_draft"));
        assert!(ensure.contains("representative.organization_id = organization.id.clone()"));
        assert!(ensure.contains("\"organization-representative\""));
    }

    #[test]
    fn authoritative_presence_reconstructs_historical_outbreak_state_without_mutation() {
        let source = crate::production_source(include_str!("settlement_population.rs"));
        let typed_projection = source
            .split("pub fn npc_strategic_presence_at")
            .nth(1)
            .and_then(|tail| {
                tail.split("pub fn npc_presence_remaining_minutes_at")
                    .next()
            })
            .expect("typed historical presence projection");
        assert!(typed_projection.contains("patient_presence_suppression_at"));
        assert!(typed_projection.contains("character_alive_at"));
        assert!(!typed_projection.contains("world_clock"));
        assert!(!typed_projection.contains("refresh_patient_context_after_time_write"));
        assert!(!typed_projection.contains("character.alive"));

        let projection = source
            .split("pub fn npc_presence_remaining_minutes_at")
            .nth(1)
            .and_then(|tail| tail.split("pub fn npc_presence_remaining_minutes").next())
            .expect("authoritative presence projection");
        assert!(projection.contains("patient_presence_suppression_at"));
        assert!(!projection.contains("PresenceFrontier"));
        assert!(!projection.contains("refresh_patient_context_after_time_write"));
    }
}
