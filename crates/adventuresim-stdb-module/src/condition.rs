use adventuresim_core::prelude::*;
use adventuresim_core::{
    item_references::STANDARD_WATERSKIN_ID,
    physical_object::CarriedInventoryScope,
    provisioning::{STRATEGIC_TRAVEL_KCAL_PER_DAY, STRATEGIC_TRAVEL_WATER_ML_PER_DAY},
};
use spacetimedb::{ReducerContext, Table, reducer, table};
use std::collections::BTreeMap;

use crate::capability::StrategicEquipment;
use crate::character::character;
use crate::filth::character_filth;
use crate::investigation::case_site_authority;
use crate::item::item;
use crate::strategic::{
    PartyJourneyRoute, hostile_group_authority, party_authority, party_journey_authority,
    party_journey_route_authority, settlement,
};
use crate::{
    CharacterAttributes, CharacterLimbs, CharacterSkills, CharacterStats, character_attributes,
    character_limbs, character_skills, character_stats, character_time,
    character_training_schedule, inventory_item,
};
use adventuresim_core::physiology::{BodyMassKg, BodyRegion};
use adventuresim_world_schema::calendar::{MINUTES_PER_DAY, StrategicMinute};
mod error;
#[cfg(test)]
mod error_tests;
mod members;
use members::condition_projection_member_ids;
mod readiness;
pub(crate) mod readiness_error;
use adventuresim_core::identity::CharacterId;
use error::ConditionComponent;
pub(crate) use error::StrategicConditionError;
pub(crate) use readiness::{require_character_ready, require_characters_ready};
pub(crate) use readiness_error::CharacterReadinessError;
mod parts;
use parts::load_character_parts;
pub(crate) use parts::mental_check;
mod morale;
use morale::{base_morale, party_morale_support};
mod survival;
use survival::SurvivalProjection;
mod projection;
pub(crate) use projection::refresh_character_strategic_condition;
use projection::refresh_character_strategic_condition_projection;
mod fear;
mod initialization;
use fear::enemy_fear_multiplier;
pub(crate) use initialization::initialize_character_condition;
pub const RECENT_MORALE_DURATION_MINUTES: u64 = 7 * 24 * 60;
const LEISURE_MORALE_SOURCE_ID: &str = "settlement-leisure";
const MASTERY_MORALE_SOURCE_ID: &str = "mastery-enjoyment";
const INJURY_MORALE_PER_HEALTH_DEFICIT: f32 = 5.0;
#[derive(Clone, Debug)]
#[table(accessor = character_condition)]
pub struct CharacterCondition {
    #[primary_key]
    pub character_id: u64,
    pub body_weight_kg: f32,
    pub current_blood_ml: f32,
    pub maximum_blood_ml: f32,
    pub religion_id: Option<String>,
}

/// Durable strategic food and water state. Positive balances are short-term
/// physiological reserves; negative balances represent unsupported need.
#[derive(Clone, Debug)]
#[table(accessor = character_needs)]
pub struct CharacterNeeds {
    #[primary_key]
    pub character_id: u64,
    pub food_balance_kcal: f32,
    pub water_balance_ml: f32,
}

/// Durable strategic coating and temperature state. Wetness is intentionally
/// separate from filth: water changes exposure but carries no dirt/blood
/// provenance and washing never consumes it.
#[derive(Clone, Debug, PartialEq)]
#[table(accessor = character_exposure)]
pub struct CharacterExposure {
    #[primary_key]
    pub character_id: u64,
    pub wetness_bps: u16,
    /// Signed: negative is cold, positive is hot.
    pub thermal_strain: i32,
    pub frostbite_progress_minutes: u32,
}

/// A recent success or setback which decays linearly over strategic time.
#[derive(Clone, Debug)]
#[table(accessor = morale_event, public)]
pub struct MoraleEvent {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub character_id: u64,
    pub kind: MoraleEventKind,
    /// Positive values are successes; negative values are setbacks.
    pub magnitude: f32,
    pub occurred_at_minute: StrategicMinute,
    pub expires_at_minute: StrategicMinute,
    pub source_id: Option<String>,
}

/// Refreshable server-authoritative projection used by strategic clients.
/// Durable needs, wounds, illness, and morale stimuli own its inputs; condition
/// refresh and chronological settlement update this query projection together
/// with dependent capability. It does not store live tactical impairment.
#[derive(Clone, Debug, PartialEq)]
#[table(accessor = character_strategic_condition)]
pub struct CharacterStrategicCondition {
    #[primary_key]
    pub character_id: u64,
    pub morale: f32,
    /// This character's allocated share of the party's ally-restoration fraction.
    pub morale_bonus: f32,
    /// Maximum party restoration fraction at the current aggregate Command check.
    pub morale_bonus_cap: f32,
    /// Bounded strategic pressure toward inflexible religious behavior.
    pub fervor: f32,
    pub pain: f32,
    pub blood_loss: f32,
    pub fear: f32,
    pub fatigue: f32,
    pub hunger: f32,
    pub thirst: f32,
    pub thermal: f32,
    pub wetness_bps: u16,
    pub thermal_strain: i32,
    /// Positive physiological food reserve, expressed in travel days.
    pub food_days: f32,
    /// Positive physiological hydration reserve, expressed in travel days.
    pub water_days: f32,
    pub water_capacity_ml: u32,
    pub incapacitation: f32,
    pub check_multiplier: f32,
    pub status: IncapacitationStatus,
}

/// A signed contribution to the character's current projected morale.
#[derive(Clone, Debug)]
#[table(accessor = character_morale_source)]
pub struct CharacterMoraleSource {
    #[primary_key]
    pub id: String,
    #[index(btree)]
    pub character_id: u64,
    pub kind: MoraleSourceKind,
    pub label: String,
    pub magnitude: f32,
}

/// A strategic choice created when conviction begins demanding costly action.
#[derive(Clone, Debug)]
#[table(accessor = religious_demand, public)]
pub struct ReligiousDemand {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub character_id: u64,
    pub kind: String,
    pub title: String,
    pub description: String,
    pub fervor: f32,
    pub status: String,
    pub created_at_minute: StrategicMinute,
    pub resolved_at_minute: Option<StrategicMinute>,
    pub resolution: Option<String>,
}

enum ExposureLocation {
    Fixed(i32, i32, i16),
    Route {
        route: PartyJourneyRoute,
        completed_movement_minutes: u64,
    },
}

impl ExposureLocation {
    fn position(&self, movement_offset: u64) -> (i32, i32, i16) {
        match self {
            Self::Fixed(latitude, longitude, elevation) => (*latitude, *longitude, *elevation),
            Self::Route {
                route,
                completed_movement_minutes,
            } => crate::strategic::route_position_at_minute(
                route,
                completed_movement_minutes.saturating_add(movement_offset),
            )
            .and_then(|(longitude, latitude)| {
                adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(
                    longitude,
                    latitude,
                )
                .map(|coordinate| {
                    (coordinate.latitude().get(), coordinate.longitude().get(), 0)
                })
            })
            .unwrap_or((53_000_000, 10_000_000, 0)),
        }
    }
}

/// Load the durable location/route authority once. The potentially long
/// minute stepping below is then pure and performs no database queries.
fn exposure_location(ctx: &ReducerContext, character_id: CharacterId) -> ExposureLocation {
    let Some(character) = ctx.db.character().id().find(u64::from(character_id)) else {
        return ExposureLocation::Fixed(53_000_000, 10_000_000, 0);
    };
    if let Some(settlement_id) = character.current_settlement_id
        && let Some(place) = ctx.db.settlement().id().find(settlement_id)
        && let Some(coordinate) = adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(
            place.coord_x,
            place.coord_y,
        )
    {
        return ExposureLocation::Fixed(
            coordinate.latitude().get(),
            coordinate.longitude().get(),
            place.elevation.get(),
        );
    }
    if let Some(party_id) = character.party_id {
        if let Some(party) = ctx.db.party_authority().id().find(&party_id)
            && let Some(site_id) = party.current_case_site_id
            && let Some(site) = ctx
                .db
                .case_site_authority()
                .id_key()
                .find(site_id.to_string())
        {
            let coordinate = if site.coordinates_are_geographic {
                adventuresim_world_schema::coordinates::Wgs84CoordinateE7::new(
                    site.latitude_e7,
                    site.longitude_e7,
                )
                .map(adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees::from_e7)
                .map(|coordinate| (coordinate.latitude().get(), coordinate.longitude().get()))
            } else {
                use adventuresim_world_schema::coordinates::UnboundedCoordinateE7;
                Some((
                    UnboundedCoordinateE7::from_raw(site.latitude_e7)
                        .millionths_of_coordinate_unit(),
                    UnboundedCoordinateE7::from_raw(site.longitude_e7)
                        .millionths_of_coordinate_unit(),
                ))
            };
            if let Some((latitude, longitude)) = coordinate {
                return ExposureLocation::Fixed(latitude, longitude, 0);
            }
        }
        if let (Some(journey), Some(route)) = (
            ctx.db.party_journey_authority().party_id().find(&party_id),
            ctx.db
                .party_journey_route_authority()
                .party_id()
                .find(&party_id),
        ) {
            return ExposureLocation::Route {
                route,
                completed_movement_minutes: journey.completed_movement_minutes,
            };
        }
    }
    ExposureLocation::Fixed(53_000_000, 10_000_000, 0)
}

/// The one strategic exposure seam. Call only after the authoritative clock
/// has committed its actually elapsed (possibly terminal-clipped) interval.
pub fn apply_weather_exposure(
    ctx: &ReducerContext,
    character_id: CharacterId,
    starting_minute: StrategicMinute,
    elapsed_minutes: u64,
    moving: bool,
    shelter: adventuresim_core::survival::ExposureShelter,
) -> Result<(), String> {
    initialize_character_condition(ctx, character_id.into());
    if elapsed_minutes == 0 {
        return Ok(());
    }
    let row = ctx
        .db
        .character_exposure()
        .character_id()
        .find(u64::from(character_id))
        .ok_or("Character exposure not found")?;
    let clothing = StrategicEquipment::load(ctx, (character_id).into()).survival_clothing();
    let location = exposure_location(ctx, (character_id).into());
    let wilderness_environment_start = ctx
        .db
        .character()
        .id()
        .find(u64::from(character_id))
        .and_then(|character| character.party_id)
        .and_then(|party_id| ctx.db.party_authority().id().find(&party_id))
        .and_then(|party| crate::strategic::party_wilderness_environment_minutes(&party))
        .map(|minutes| minutes.0);
    let weather = (0..elapsed_minutes).map(|offset| {
        let (latitude, longitude, elevation) = location.position(if moving { offset } else { 0 });
        let weather_minute = wilderness_environment_start.map_or_else(
            || starting_minute.saturating_add_minutes(offset),
            |environment_start| environment_start.wrapping_day_offset(offset),
        );
        adventuresim_core::weather::weather_at(
            adventuresim_core::weather::WORLD_WEATHER_SEED,
            weather_minute,
            latitude,
            longitude,
            elevation,
        )
    });
    let outcome = adventuresim_core::survival::advance_exposure(
        adventuresim_core::survival::SurvivalState {
            wetness_bps: row.wetness_bps,
            thermal_strain: row.thermal_strain,
            frostbite_progress_minutes: row.frostbite_progress_minutes,
        },
        weather,
        clothing,
        shelter,
    );
    ctx.db
        .character_exposure()
        .character_id()
        .update(CharacterExposure {
            character_id: u64::from(character_id),
            wetness_bps: outcome.state.wetness_bps,
            thermal_strain: outcome.state.thermal_strain,
            frostbite_progress_minutes: outcome.state.frostbite_progress_minutes,
        });
    // Replay each threshold at its canonical absolute minute. Frostbite is
    // durable non-bleeding tissue damage and is deliberately not healed by
    // ordinary injury settlement, so replay commutes with the already-clipped
    // interval while preserving identical limb projections across partitions.
    for event_offset in outcome.frostbite_event_offsets {
        let event_minute = starting_minute.saturating_add_minutes(event_offset);
        let peripheral = adventuresim_core::survival::frostbite_peripheral_index(
            clothing.peripheral_protection_bps,
            event_minute,
        );
        let limb = [
            BodyRegion::LeftArm,
            BodyRegion::RightArm,
            BodyRegion::LeftLeg,
            BodyRegion::RightLeg,
        ][peripheral];
        crate::surgery::commit_frostbite_injury(
            ctx,
            (character_id).into(),
            limb,
            adventuresim_core::survival::FROSTBITE_DAMAGE_PER_THRESHOLD,
        )?;
    }
    refresh_character_strategic_condition_projection(ctx, (character_id).into()).map(|_| ())
}

/// Reusable authoritative water-entry impulse for future ford/immersion
/// locations. Route terrain currently has no ford coordinates, so no caller
/// guesses immersion from wetlands.
pub fn apply_immersion_impulse(ctx: &ReducerContext, character_id: u64) -> Result<(), String> {
    initialize_character_condition(ctx, character_id.into());
    let row = ctx
        .db
        .character_exposure()
        .character_id()
        .find(character_id)
        .ok_or("Character exposure not found")?;
    let next =
        adventuresim_core::survival::apply_immersion(adventuresim_core::survival::SurvivalState {
            wetness_bps: row.wetness_bps,
            thermal_strain: row.thermal_strain,
            frostbite_progress_minutes: row.frostbite_progress_minutes,
        });
    ctx.db
        .character_exposure()
        .character_id()
        .update(CharacterExposure {
            character_id,
            wetness_bps: next.wetness_bps,
            thermal_strain: next.thermal_strain,
            frostbite_progress_minutes: next.frostbite_progress_minutes,
        });
    Ok(())
}

fn inventory_quantity(ctx: &ReducerContext, character_id: CharacterId, item_id: &str) -> u32 {
    ctx.db
        .inventory_item()
        .character_and_item_id()
        .filter((u64::from(character_id), item_id))
        .filter(|entry| {
            !crate::inventory_container::row_is_fireplace_rooted(
                ctx,
                CarriedInventoryScope::Personal,
                entry.id,
            )
        })
        .map(|entry| entry.quantity)
        .sum()
}

fn food_reserve_days(needs: &CharacterNeeds) -> f32 {
    needs.food_balance_kcal.max(0.0) / STRATEGIC_TRAVEL_KCAL_PER_DAY
}

fn water_reserve_days(needs: &CharacterNeeds) -> f32 {
    needs.water_balance_ml.max(0.0) / STRATEGIC_TRAVEL_WATER_ML_PER_DAY
}

pub(crate) fn water_capacity_ml(ctx: &ReducerContext, character_id: CharacterId) -> u32 {
    let capacity_per_container = ctx
        .db
        .item()
        .id()
        .find(STANDARD_WATERSKIN_ID.to_string())
        .map_or(0, |item| item.water_capacity_ml);
    inventory_quantity(ctx, character_id, STANDARD_WATERSKIN_ID)
        .saturating_mul(capacity_per_container)
}

pub fn prepare_party_waterskins(
    ctx: &ReducerContext,
    party_id: &str,
    from_settlement: bool,
) -> Result<(), String> {
    if from_settlement {
        let custody = adventuresim_core::physical_object::OperationalCustody::party(party_id)
            .map_err(|error| error.to_string())?;
        crate::inventory_container::fill_carried_waterskins(ctx, &custody)?;
    }
    Ok(())
}

pub fn prepare_character_waterskins(
    ctx: &ReducerContext,
    character_id: CharacterId,
    from_settlement: bool,
) -> Result<(), String> {
    initialize_character_condition(ctx, character_id.into());
    if from_settlement {
        let custody = adventuresim_core::physical_object::OperationalCustody::character(
            (character_id).into(),
        )
        .map_err(|error| error.to_string())?;
        crate::inventory_container::fill_carried_waterskins(ctx, &custody)?;
    }
    Ok(())
}

pub fn replenish_needs_at_settlement(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<(), String> {
    initialize_character_condition(ctx, character_id.into());
    let mut needs = ctx
        .db
        .character_needs()
        .character_id()
        .find(u64::from(character_id))
        .ok_or("Character needs not found")?;
    // Arrival grants no free provisions and clears any travel surplus so the
    // character can immediately eat a deliberate dinner.
    needs.food_balance_kcal = needs.food_balance_kcal.min(0.0);
    needs.water_balance_ml = STRATEGIC_TRAVEL_WATER_ML_PER_DAY;
    ctx.db.character_needs().character_id().update(needs);
    let custody =
        adventuresim_core::physical_object::OperationalCustody::character((character_id).into())
            .map_err(|error| error.to_string())?;
    crate::inventory_container::fill_carried_waterskins(ctx, &custody)?;
    Ok(())
}

pub fn apply_elapsed_needs(
    ctx: &ReducerContext,
    character_id: CharacterId,
    elapsed_minutes: u64,
) -> Result<(), String> {
    apply_elapsed_needs_with_provision(
        ctx,
        (character_id).into(),
        elapsed_minutes,
        ElapsedNeedsProvision::PersonalSupplies,
    )
}

/// Applies settlement-rest needs exactly once. Every settlement provision
/// supplies water; inn board and a residence additionally supply food.
pub(crate) fn apply_settlement_rest_elapsed_needs(
    ctx: &ReducerContext,
    character_id: CharacterId,
    elapsed_minutes: u64,
    provision: crate::time::SettlementRestProvision,
) -> Result<(), String> {
    apply_elapsed_needs_with_provision(
        ctx,
        (character_id).into(),
        elapsed_minutes,
        settlement_rest_elapsed_needs_provision(provision),
    )
}

fn settlement_rest_elapsed_needs_provision(
    provision: crate::time::SettlementRestProvision,
) -> ElapsedNeedsProvision {
    match provision {
        crate::time::SettlementRestProvision::PublicService(
            adventuresim_world_schema::SettlementActionService::Inn,
        )
        | crate::time::SettlementRestProvision::Residence => ElapsedNeedsProvision::FullBoard,
        crate::time::SettlementRestProvision::PublicService(
            adventuresim_world_schema::SettlementActionService::Temple,
        )
        | crate::time::SettlementRestProvision::PrivateDowntime => {
            ElapsedNeedsProvision::SettlementWater
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ElapsedNeedsProvision {
    PersonalSupplies,
    SettlementWater,
    FullBoard,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ElapsedNeedsPlan {
    food_balance_kcal: f32,
    consume_stored_food: bool,
    water_balance_ml: f32,
    consume_stored_water: bool,
}

fn elapsed_needs_plan(
    starting_food_balance_kcal: f32,
    starting_water_balance_ml: f32,
    elapsed_minutes: u64,
    provision: ElapsedNeedsProvision,
) -> ElapsedNeedsPlan {
    match provision {
        ElapsedNeedsProvision::PersonalSupplies => {
            let elapsed_days = elapsed_minutes as f32 / (24.0 * 60.0);
            ElapsedNeedsPlan {
                food_balance_kcal: starting_food_balance_kcal
                    - elapsed_days * STRATEGIC_TRAVEL_KCAL_PER_DAY,
                consume_stored_food: true,
                water_balance_ml: starting_water_balance_ml
                    - elapsed_days * STRATEGIC_TRAVEL_WATER_ML_PER_DAY,
                consume_stored_water: true,
            }
        }
        // Public wells and ordinary settlement hospitality cover drinking
        // water during downtime. Food still comes from personal or party
        // stores unless the character pays for full board at an inn.
        ElapsedNeedsProvision::SettlementWater => {
            let elapsed_days = elapsed_minutes as f32 / (24.0 * 60.0);
            ElapsedNeedsPlan {
                food_balance_kcal: starting_food_balance_kcal
                    - elapsed_days * STRATEGIC_TRAVEL_KCAL_PER_DAY,
                consume_stored_food: true,
                water_balance_ml: starting_water_balance_ml.max(0.0),
                consume_stored_water: false,
            }
        }
        // Full board covers the elapsed interval and brings an underfed guest
        // back to neutral, without creating surplus fullness or consuming
        // provisions carried by the guest or party.
        ElapsedNeedsProvision::FullBoard => ElapsedNeedsPlan {
            food_balance_kcal: starting_food_balance_kcal.max(0.0),
            consume_stored_food: false,
            water_balance_ml: starting_water_balance_ml.max(0.0),
            consume_stored_water: false,
        },
    }
}

fn apply_elapsed_needs_with_provision(
    ctx: &ReducerContext,
    character_id: CharacterId,
    elapsed_minutes: u64,
    provision: ElapsedNeedsProvision,
) -> Result<(), String> {
    initialize_character_condition(ctx, character_id.into());
    let mut needs = ctx
        .db
        .character_needs()
        .character_id()
        .find(u64::from(character_id))
        .ok_or("Character needs not found")?;

    let needs_plan = elapsed_needs_plan(
        needs.food_balance_kcal,
        needs.water_balance_ml,
        elapsed_minutes,
        provision,
    );
    needs.food_balance_kcal = needs_plan.food_balance_kcal;
    ctx.db
        .character_needs()
        .character_id()
        .update(needs.clone());
    if needs_plan.consume_stored_food {
        crate::food::consume_travel_food_to_zero(ctx, (character_id).into())?;
    }
    needs = ctx
        .db
        .character_needs()
        .character_id()
        .find(u64::from(character_id))
        .ok_or("Character needs not found")?;

    needs.water_balance_ml = needs_plan.water_balance_ml;
    if needs_plan.consume_stored_water && needs.water_balance_ml < 0.0 {
        if let Some(party_id) = ctx
            .db
            .character()
            .id()
            .find(u64::from(character_id))
            .and_then(|row| row.party_id)
        {
            let party_custody =
                adventuresim_core::physical_object::OperationalCustody::party(party_id.clone())
                    .map_err(|error| error.to_string())?;
            let contained = crate::inventory_container::consume_contained_water(
                ctx,
                (character_id).into(),
                &party_custody,
                (-needs.water_balance_ml).max(0.0).ceil() as u64,
            )?;
            needs.water_balance_ml += contained as f32;
        }
        let personal_custody = adventuresim_core::physical_object::OperationalCustody::character(
            (character_id).into(),
        )
        .map_err(|error| error.to_string())?;
        let contained = crate::inventory_container::consume_contained_water(
            ctx,
            (character_id).into(),
            &personal_custody,
            (-needs.water_balance_ml).max(0.0).ceil() as u64,
        )?;
        needs.water_balance_ml += contained as f32;
    }
    ctx.db.character_needs().character_id().update(needs);
    Ok(())
}

fn total_damage(limbs: &CharacterLimbs) -> f32 {
    [
        limbs.left_arm_health,
        limbs.right_arm_health,
        limbs.left_leg_health,
        limbs.right_leg_health,
        limbs.head_health,
        limbs.chest_health,
        limbs.stomach_health,
    ]
    .into_iter()
    .map(|health| (1.0 - health).max(0.0))
    .sum()
}

#[derive(Clone, Debug)]
struct ProjectedMoraleSource {
    key: String,
    kind: MoraleSourceKind,
    label: String,
    magnitude: f32,
}

fn rank_morale_sources(raw_sources: &mut [ProjectedMoraleSource], will: f32) {
    let mut positive: Vec<_> = raw_sources
        .iter_mut()
        .filter(|source| source.magnitude > 0.0)
        .collect();
    positive.sort_by(|left, right| right.magnitude.total_cmp(&left.magnitude));
    for (index, source) in positive.into_iter().enumerate() {
        source.magnitude /= (index + 1) as f32;
    }
    let mut negative: Vec<_> = raw_sources
        .iter_mut()
        .filter(|source| source.magnitude < 0.0)
        .collect();
    negative.sort_by(|left, right| left.magnitude.total_cmp(&right.magnitude));
    for (index, source) in negative.into_iter().enumerate() {
        source.magnitude /= (index + 1) as f32 * will;
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct PartyReligionContext {
    own_cohort: f32,
    foreign_pressure: f32,
    party_command: f32,
    knowledge: f32,
}

fn religion_label(religion_id: &str) -> String {
    religion_id
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn party_character_ids(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<Vec<CharacterId>, StrategicConditionError> {
    let character = ctx
        .db
        .character()
        .id()
        .find(u64::from(character_id))
        .ok_or(StrategicConditionError::Missing {
            character: character_id,
            component: ConditionComponent::Character,
        })?;
    Ok(condition_projection_member_ids(
        character_id,
        crate::character::StoredCharacterLifeState::from(character.alive),
        character
            .party_id
            .as_ref()
            .map(|party_id| crate::strategic::living_party_member_ids(ctx, party_id)),
    ))
}

fn religion_knowledge_check(
    ctx: &ReducerContext,
    character_id: CharacterId,
    religion: adventuresim_world_schema::OfficialReligion,
) -> Result<f32, StrategicConditionError> {
    let (attributes, limbs, stats, skills) = load_character_parts(ctx, character_id)?;
    Ok(adventuresim_core::capability::religion_knowledge_check(
        skills.religion_hours.effective(religion),
        attributes.instinct,
        attributes.intelligence,
        stats.focus,
        limbs.head_health,
    ))
}

fn party_religion_context(
    ctx: &ReducerContext,
    character_id: CharacterId,
    party_members: &[CharacterId],
) -> Result<Option<(String, PartyReligionContext)>, StrategicConditionError> {
    let mut cohorts: BTreeMap<String, Vec<f32>> = BTreeMap::new();
    let mut commands = Vec::with_capacity(party_members.len());
    for member_id in party_members.iter().copied() {
        initialize_character_condition(ctx, member_id);
        commands.push(adventuresim_world_schema::language_scaled_effect(
            mental_check(ctx, member_id, Skill::Command)?,
            crate::character::shared_language_coefficient(ctx, member_id, character_id),
        ));
        if let Some(religion_id) = ctx
            .db
            .character_condition()
            .character_id()
            .find(u64::from(member_id))
            .and_then(|condition| condition.religion_id)
        {
            cohorts.entry(religion_id).or_default().push(
                crate::personality::personality_or_neutral(ctx, member_id)
                    .conviction
                    .strength(),
            );
        }
    }
    let own_religion = ctx
        .db
        .character_condition()
        .character_id()
        .find(u64::from(character_id))
        .and_then(|condition| condition.religion_id);
    let Some(own_religion) = own_religion else {
        return Ok(None);
    };
    let religion =
        adventuresim_world_schema::OfficialReligion::from_id(&own_religion).ok_or_else(|| {
            StrategicConditionError::UnknownReligion {
                character: character_id,
                stored_key: own_religion.clone(),
            }
        })?;
    let (own_cohort, foreign_pressure) = religion_cohort_pressure(cohorts, &own_religion);
    let knowledge_checks = party_members
        .iter()
        .copied()
        .map(|member_id| religion_knowledge_check(ctx, member_id, religion))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some((
        own_religion,
        PartyReligionContext {
            own_cohort,
            foreign_pressure,
            party_command: aggregate_party_command(commands),
            knowledge: aggregate_party_check(knowledge_checks).clamp(0.0, 5.0),
        },
    )))
}

fn religion_cohort_pressure(cohorts: BTreeMap<String, Vec<f32>>, own_religion: &str) -> (f32, f32) {
    let cohort_checks: BTreeMap<_, _> = cohorts
        .into_iter()
        .map(|(religion, checks)| (religion, aggregate_party_check(checks).clamp(0.0, 5.0)))
        .collect();
    let own_cohort = cohort_checks.get(own_religion).copied().unwrap_or(0.0);
    let foreign_pressure = aggregate_party_check(
        cohort_checks
            .iter()
            .filter_map(|(religion, check)| (religion != own_religion).then_some(*check)),
    )
    .clamp(0.0, 5.0);
    (own_cohort, foreign_pressure)
}

/// Feed all rejected effective skill training into one shared, durable morale
/// source. Callers aggregate every award in one logical clock interval before
/// recording it, so skill choice and award order cannot multiply enjoyment.
pub fn record_mastery_training_morale(
    ctx: &ReducerContext,
    character_id: CharacterId,
    elapsed_minutes: u64,
    excess_effective_hours: f32,
) {
    if excess_effective_hours <= 0.0 || !excess_effective_hours.is_finite() {
        return;
    }
    let interval_end = character_minute(ctx, (character_id).into());
    let interval_start = interval_end.saturating_sub_minutes(elapsed_minutes);
    let existing = ctx
        .db
        .morale_event()
        .character_id()
        .filter(u64::from(character_id))
        .find(|event| event.source_id.as_deref() == Some(MASTERY_MORALE_SOURCE_ID));
    let at_interval_start = existing.as_ref().map_or(0.0, |event| {
        let occurred_at = event.occurred_at_minute;
        let duration = event.expires_at_minute.elapsed_since(occurred_at);
        event.magnitude
            * adventuresim_core::morale::mastery_enjoyment_decay(
                interval_start.elapsed_since(occurred_at),
                duration,
            )
    });
    // Endpoint semantics: the helper decays this interval-start magnitude
    // through `elapsed_minutes` before applying the one aggregated award.
    let magnitude = adventuresim_core::morale::mastery_enjoyment_after_interval(
        at_interval_start,
        excess_effective_hours,
        elapsed_minutes,
        RECENT_MORALE_DURATION_MINUTES,
    );
    let event = MoraleEvent {
        id: existing.as_ref().map_or(0, |event| event.id),
        character_id: u64::from(character_id),
        kind: MoraleEventKind::MasteryEnjoyment,
        magnitude,
        occurred_at_minute: interval_end,
        expires_at_minute: interval_end.saturating_add_minutes(RECENT_MORALE_DURATION_MINUTES),
        source_id: Some(MASTERY_MORALE_SOURCE_ID.into()),
    };
    if existing.is_some() {
        ctx.db.morale_event().id().update(event);
    } else {
        ctx.db.morale_event().insert(event);
    }
}

fn character_minute(ctx: &ReducerContext, character_id: CharacterId) -> StrategicMinute {
    ctx.db
        .character_time()
        .character_id()
        .find(u64::from(character_id))
        .map_or(StrategicMinute::ZERO, |time| time.minutes)
}

fn ensure_holy_day_demand(
    ctx: &ReducerContext,
    condition: &CharacterStrategicCondition,
) -> Result<(), StrategicConditionError> {
    if condition.fervor <= 0.0 {
        return Ok(());
    }
    let professes_religion = ctx
        .db
        .character_condition()
        .character_id()
        .find(condition.character_id)
        .is_some_and(|condition| condition.religion_id.is_some());
    if !professes_religion {
        return Ok(());
    }
    let current_minute = crate::time::refresh_clock(ctx)?;
    let current_day = current_minute.day_index();
    if current_day.weekday() != adventuresim_world_schema::calendar::StrategicWeekday::Sunday {
        return Ok(());
    }
    let demands: Vec<_> = ctx
        .db
        .religious_demand()
        .character_id()
        .filter(condition.character_id)
        .collect();
    if demands.iter().any(|demand| demand.status == "pending") {
        return Ok(());
    }
    if demands.iter().any(|demand| {
        demand.kind == "holy_day" && demand.created_at_minute.day_index() == current_day
    }) {
        return Ok(());
    }
    let at_settlement = ctx
        .db
        .character()
        .id()
        .find(condition.character_id)
        .is_some_and(|character| character.current_settlement_id.is_some());
    if !at_settlement {
        return Ok(());
    }
    ctx.db.religious_demand().insert(ReligiousDemand {
        id: 0,
        character_id: condition.character_id,
        kind: "holy_day".into(),
        title: "Keep the holy day".into(),
        description: "Sunday is a day of worship and rest. Conviction demands a full day away from the road and worldly business; daily prayer is managed through the activity schedule.".into(),
        fervor: condition.fervor,
        status: "pending".into(),
        created_at_minute: current_minute,
        resolved_at_minute: None,
        resolution: None,
    });
    Ok(())
}

fn holy_day_demand_has_expired(
    created_day: adventuresim_world_schema::calendar::StrategicDayIndex,
    current_day: adventuresim_world_schema::calendar::StrategicDayIndex,
    departing: bool,
) -> bool {
    created_day < current_day || (departing && created_day == current_day)
}

fn refuse_expired_holy_day_demands(
    ctx: &ReducerContext,
    character_id: CharacterId,
    departing: bool,
) -> Result<bool, StrategicConditionError> {
    let current_minute = crate::time::refresh_clock(ctx)?;
    let current_day = current_minute.day_index();
    let pending: Vec<_> = ctx
        .db
        .religious_demand()
        .character_id()
        .filter(u64::from(character_id))
        .filter(|demand| {
            demand.kind == "holy_day"
                && demand.status == "pending"
                && holy_day_demand_has_expired(
                    demand.created_at_minute.day_index(),
                    current_day,
                    departing,
                )
        })
        .collect();
    if pending.is_empty() {
        return Ok(false);
    }

    let command = party_command(ctx, character_id)?;
    for mut demand in pending {
        demand.status = "resolved".into();
        demand.resolved_at_minute = Some(current_minute);
        demand.resolution = Some("refuse".into());
        let penalty = religious_neglect_morale(demand.fervor, command);
        let source_id = format!("religious-demand:{}", demand.id);
        ctx.db.religious_demand().id().update(demand);
        if penalty > 0.0 && !has_morale_source(ctx, character_id, &source_id) {
            insert_morale_event_without_refresh(
                ctx,
                character_id,
                MoraleEventKind::ReligiousObservanceNeglected,
                -penalty,
                source_id,
            );
        }
    }
    Ok(true)
}

#[reducer]
pub fn resolve_religious_demand(
    ctx: &ReducerContext,
    demand_id: u64,
    choice: String,
) -> Result<(), String> {
    let mut demand = ctx
        .db
        .religious_demand()
        .id()
        .find(demand_id)
        .ok_or("Religious demand not found")?;
    crate::character::require_living_character(ctx, (demand.character_id).into())
        .map_err(|error: crate::character::LivingCharacterError| error.to_string())?;
    if demand.status != "pending" {
        return Err("Religious demand has already been resolved".into());
    }
    let character = ctx
        .db
        .character()
        .id()
        .find(demand.character_id)
        .ok_or("Character not found")?;
    if character.server != ctx.sender() {
        return Err("Only this character's player may answer the demand".into());
    }
    if !matches!(choice.as_str(), "observe" | "refuse") {
        return Err("Unknown religious-demand choice".into());
    }
    if choice == "observe" && demand.kind == "holy_day" {
        if character.current_settlement_id.is_none() {
            return Err("A holy day can only be observed at a settlement".into());
        }
        let current_day = crate::time::refresh_clock(ctx)
            .map_err(|error: crate::time::WorldClockError| error.to_string())?
            .day_index();
        if current_day != demand.created_at_minute.day_index() {
            return Err("This holy day has already passed".into());
        }
    }
    demand.status = "resolved".into();
    demand.resolved_at_minute = Some(
        crate::time::refresh_clock(ctx)
            .map_err(|error: crate::time::WorldClockError| error.to_string())?,
    );
    demand.resolution = Some(choice.clone());
    ctx.db.religious_demand().id().update(demand.clone());

    match choice.as_str() {
        "observe" if demand.kind == "holy_day" => {
            // Holy-day demand represents private observance and abstention
            // from work; it does not imply access to a Church service.
            crate::time::spend_private_settlement_downtime(
                ctx,
                (demand.character_id).into(),
                adventuresim_world_schema::calendar::MINUTES_PER_DAY,
                true,
            )?;
            record_morale_event(
                ctx,
                (demand.character_id).into(),
                MoraleEventKind::HolyDayObserved,
                2.0,
                Some(format!("religious-demand:{}", demand.id)),
            )?;
        }
        "refuse" => {
            let party_ids = party_character_ids(ctx, (demand.character_id).into())
                .map_err(|error: crate::condition::StrategicConditionError| error.to_string())?;
            let party_command = aggregate_party_command(
                party_ids
                    .into_iter()
                    .map(|id| mental_check(ctx, id, Skill::Command))
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error: crate::condition::StrategicConditionError| {
                        error.to_string()
                    })?,
            );
            let penalty = religious_neglect_morale(demand.fervor, party_command);
            if penalty > 0.0 {
                record_morale_event(
                    ctx,
                    (demand.character_id).into(),
                    MoraleEventKind::ReligiousObservanceNeglected,
                    -penalty,
                    Some(format!("religious-demand:{}", demand.id)),
                )?;
            }
        }
        _ => return Err("Religious demand kind cannot be observed".into()),
    }
    refresh_character_strategic_condition(ctx, (demand.character_id).into())
        .map(|_| ())
        .map_err(|error: StrategicConditionError| error.to_string())
}

pub fn record_morale_event(
    ctx: &ReducerContext,
    character_id: CharacterId,
    kind: MoraleEventKind,
    magnitude: f32,
    source_id: Option<String>,
) -> Result<(), String> {
    if magnitude == 0.0 || !magnitude.is_finite() {
        return Ok(());
    }
    let occurred_at_minute = ctx
        .db
        .character_time()
        .character_id()
        .find(u64::from(character_id))
        .map_or(StrategicMinute::ZERO, |time| time.minutes);
    let duration = stored_morale_event_duration(ctx, (character_id).into(), magnitude);
    ctx.db.morale_event().insert(MoraleEvent {
        id: 0,
        character_id: u64::from(character_id),
        kind,
        magnitude,
        occurred_at_minute,
        expires_at_minute: occurred_at_minute.saturating_add_minutes(duration),
        source_id,
    });
    refresh_character_strategic_condition(ctx, (character_id).into())?;
    Ok(())
}

/// Replace one durable, refreshable morale stimulus at an explicit strategic
/// minute. Callers performing a chronological batch should refresh the derived
/// condition once after the batch completes.
pub fn upsert_refreshable_morale_event_at_without_refresh(
    ctx: &ReducerContext,
    character_id: CharacterId,
    kind: MoraleEventKind,
    magnitude: f32,
    occurred_at_minute: StrategicMinute,
    source_id: &str,
) -> Result<(), String> {
    if magnitude == 0.0 || !magnitude.is_finite() {
        return Ok(());
    }
    let duration = stored_morale_event_duration(ctx, (character_id).into(), magnitude);
    let existing = ctx
        .db
        .morale_event()
        .character_id()
        .filter(u64::from(character_id))
        .find(|event| event.source_id.as_deref() == Some(source_id));
    let event = MoraleEvent {
        id: existing.as_ref().map_or(0, |event| event.id),
        character_id: u64::from(character_id),
        kind,
        magnitude,
        occurred_at_minute,
        expires_at_minute: occurred_at_minute.saturating_add_minutes(duration),
        source_id: Some(source_id.into()),
    };
    if existing.is_some() {
        ctx.db.morale_event().id().update(event);
    } else {
        ctx.db.morale_event().insert(event);
    }
    Ok(())
}

/// Replace a bounded morale source with rules-owned magnitude and duration.
/// Lifecycle systems use this when personality must not alter a contractual
/// effect's retention window.
pub(crate) fn upsert_fixed_morale_event_without_refresh(
    ctx: &ReducerContext,
    character_id: CharacterId,
    kind: MoraleEventKind,
    magnitude: f32,
    occurred_at_minute: StrategicMinute,
    expires_at_minute: StrategicMinute,
    source_id: &str,
) {
    let existing = ctx
        .db
        .morale_event()
        .character_id()
        .filter(u64::from(character_id))
        .find(|event| event.source_id.as_deref() == Some(source_id));
    let event = MoraleEvent {
        id: existing.as_ref().map_or(0, |event| event.id),
        character_id: u64::from(character_id),
        kind,
        magnitude,
        occurred_at_minute,
        expires_at_minute,
        source_id: Some(source_id.into()),
    };
    if existing.is_some() {
        ctx.db.morale_event().id().update(event);
    } else {
        ctx.db.morale_event().insert(event);
    }
}

fn insert_morale_event_without_refresh(
    ctx: &ReducerContext,
    character_id: CharacterId,
    kind: MoraleEventKind,
    magnitude: f32,
    source_id: String,
) {
    if magnitude == 0.0 || !magnitude.is_finite() {
        return;
    }
    let occurred_at_minute = character_minute(ctx, character_id);
    let duration = stored_morale_event_duration(ctx, character_id, magnitude);
    ctx.db.morale_event().insert(MoraleEvent {
        id: 0,
        character_id: u64::from(character_id),
        kind,
        magnitude,
        occurred_at_minute,
        expires_at_minute: occurred_at_minute.saturating_add_minutes(duration),
        source_id: Some(source_id),
    });
}

fn stored_morale_event_duration(
    ctx: &ReducerContext,
    character_id: CharacterId,
    magnitude: f32,
) -> u64 {
    if magnitude < 0.0 {
        crate::personality::negative_event_duration_for_character(
            ctx,
            character_id,
            RECENT_MORALE_DURATION_MINUTES,
        )
    } else {
        RECENT_MORALE_DURATION_MINUTES
    }
}

/// Record the one-off nonlinear morale result of an explicit prayer or
/// meditation interval. This deliberately does not inspect or alter the saved
/// daily activity schedule.
pub(crate) fn record_immediate_prayer_morale(
    ctx: &ReducerContext,
    character_id: u64,
    minutes: u16,
) -> Result<(), String> {
    let party_members = party_character_ids(ctx, (character_id).into())?;
    let (kind, magnitude) = if let Some((_religion_id, religion)) =
        party_religion_context(ctx, (character_id).into(), &party_members)?
    {
        (
            MoraleEventKind::Prayer,
            adventuresim_core::activity::led_prayer_morale(minutes, religion.knowledge),
        )
    } else {
        (
            MoraleEventKind::Meditation,
            adventuresim_core::activity::meditation_morale(minutes),
        )
    };
    record_morale_event(
        ctx,
        (character_id).into(),
        kind,
        magnitude,
        Some(format!("activity:{}", kind.as_str())),
    )
}

fn party_command(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<f32, StrategicConditionError> {
    Ok(aggregate_party_command(
        party_character_ids(ctx, character_id)?
            .into_iter()
            .map(|id| mental_check(ctx, id, Skill::Command))
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

fn has_morale_source(ctx: &ReducerContext, character_id: CharacterId, source_id: &str) -> bool {
    ctx.db
        .morale_event()
        .character_id()
        .filter(u64::from(character_id))
        .any(|event| event.source_id.as_deref() == Some(source_id))
}

/// Advance fatigue for strategic travel. The existing `calories_used` field is
/// treated as a recoverable fatigue reservoir until food/day-boundary state is
/// implemented.
pub fn apply_travel_condition(
    ctx: &ReducerContext,
    character_id: CharacterId,
    starting_minute: StrategicMinute,
    elapsed_minutes: u64,
    prayer_minutes: u16,
) -> Result<(), String> {
    if elapsed_minutes > adventuresim_core::alcohol::MAX_ALCOHOL_INTERVAL_MINUTES {
        return Err("Travel condition interval cannot exceed one year".into());
    }
    let interval_end = starting_minute
        .checked_add_minutes(elapsed_minutes)
        .ok_or("Travel condition interval overflow")?;
    for (segment_start, segment_end, history_minute) in
        adventuresim_core::alcohol::travel_evening_segments(starting_minute, interval_end)
            .map_err(|error| error.to_string())?
    {
        apply_elapsed_needs(
            ctx,
            (character_id).into(),
            segment_end.elapsed_since(segment_start),
        )?;
        // Movement alone may spend potable alcohol as emergency hydration.
        // Attribute each whole serving to the evening in which the deficit
        // arose; generic waits and camp downtime never invoke this fallback.
        if let Some(mut needs) = ctx
            .db
            .character_needs()
            .character_id()
            .find(u64::from(character_id))
            && needs.water_balance_ml < 0.0
        {
            let supplied = crate::alcohol::consume_emergency_hydration(
                ctx,
                (character_id).into(),
                -needs.water_balance_ml,
                history_minute,
            );
            needs.water_balance_ml += supplied as f32;
            ctx.db.character_needs().character_id().update(needs);
        }
    }
    let mut stats = ctx
        .db
        .character_stats()
        .character_id()
        .find(u64::from(character_id))
        .ok_or("Character stats not found")?;
    stats.calories_used += elapsed_minutes as f32 / (24.0 * 60.0) * STRATEGIC_TRAVEL_KCAL_PER_DAY;
    ctx.db.character_stats().character_id().update(stats);

    refuse_expired_holy_day_demands(ctx, (character_id).into(), true)?;
    let condition = refresh_character_strategic_condition_projection(ctx, (character_id).into())?;
    let professes_religion = ctx
        .db
        .character_condition()
        .character_id()
        .find(u64::from(character_id))
        .is_some_and(|row| row.religion_id.is_some());
    if professes_religion && condition.fervor > 0.0 {
        let command = party_command(ctx, (character_id).into())?;
        let daily_penalty = religious_neglect_morale(condition.fervor, command);
        let missed_prayer = 1.0 - prayer_observance(condition.fervor, prayer_minutes);
        let elapsed_days = elapsed_minutes as f32 / MINUTES_PER_DAY as f32;
        let prayer_penalty = daily_penalty * missed_prayer * elapsed_days;
        if prayer_penalty > 0.0 {
            insert_morale_event_without_refresh(
                ctx,
                (character_id).into(),
                MoraleEventKind::TravelPrayerNeglected,
                -prayer_penalty,
                format!(
                    "travel-prayer:{starting_minute}:{}",
                    starting_minute.saturating_add_minutes(elapsed_minutes)
                ),
            );
        }
    }
    refresh_character_strategic_condition_projection(ctx, (character_id).into()).map(|_| ())
}

/// Resolve canonical Sundays that passed while a character was outside a
/// settlement. Subjective wilderness days never create additional holy days.
pub(crate) fn apply_canonical_wilderness_observance(
    ctx: &ReducerContext,
    character_id: CharacterId,
    canonical_start: StrategicMinute,
    canonical_end: StrategicMinute,
) -> Result<(), String> {
    let elapsed = canonical_end.elapsed_since(canonical_start);
    if elapsed == 0 {
        return Ok(());
    }
    let condition = refresh_character_strategic_condition_projection(ctx, (character_id).into())?;
    let professes_religion = ctx
        .db
        .character_condition()
        .character_id()
        .find(u64::from(character_id))
        .is_some_and(|row| row.religion_id.is_some());
    if !professes_religion || condition.fervor <= 0.0 {
        return Ok(());
    }
    let penalty =
        religious_neglect_morale(condition.fervor, party_command(ctx, (character_id).into())?);
    for sunday in sundays_overlapping(canonical_start, elapsed) {
        let source_id = format!("missed-canonical-sunday:{sunday}");
        if penalty > 0.0 && !has_morale_source(ctx, (character_id).into(), &source_id) {
            insert_morale_event_without_refresh(
                ctx,
                (character_id).into(),
                MoraleEventKind::ReligiousObservanceNeglected,
                -penalty,
                source_id,
            );
        }
    }
    refresh_character_strategic_condition_projection(ctx, (character_id).into()).map(|_| ())
}

pub fn apply_rest_condition(
    ctx: &ReducerContext,
    character_id: CharacterId,
    elapsed_minutes: u64,
) -> Result<(), String> {
    initialize_character_condition(ctx, character_id.into());
    let _ = elapsed_minutes;
    refresh_character_strategic_condition(ctx, (character_id).into()).map(|_| ())
}

fn leisure_morale_effect(magnitude: f32, age_minutes: f32, duration: u64) -> f32 {
    if duration == 0 {
        return 0.0;
    }
    (magnitude - LEISURE_MORALE_LIMIT * age_minutes.max(0.0) / duration as f32).max(0.0)
}

fn accumulated_leisure_morale(
    existing: Option<(f32, StrategicMinute, StrategicMinute)>,
    earned: f32,
    morale_earning_minutes: f32,
    interval_end_minute: StrategicMinute,
) -> f32 {
    let retained = existing.map_or(0.0, |(magnitude, occurred_at, expires_at)| {
        let duration = expires_at.elapsed_since(occurred_at);
        let age_at_interval_end = interval_end_minute.elapsed_since(occurred_at) as f32;
        let age_before_earning = (age_at_interval_end - morale_earning_minutes.max(0.0)).max(0.0);
        leisure_morale_effect(magnitude, age_before_earning, duration)
    });
    (retained + earned).clamp(0.0, LEISURE_MORALE_LIMIT)
}

fn upsert_leisure_morale(
    ctx: &ReducerContext,
    character_id: CharacterId,
    earned: f32,
    morale_earning_minutes: f32,
    interval_end_minute: StrategicMinute,
) {
    if earned <= 0.0 || !earned.is_finite() {
        return;
    }
    let existing = ctx
        .db
        .morale_event()
        .character_id()
        .filter(u64::from(character_id))
        .find(|event| event.source_id.as_deref() == Some(LEISURE_MORALE_SOURCE_ID));
    let magnitude = accumulated_leisure_morale(
        existing.as_ref().map(|event| {
            (
                event.magnitude,
                event.occurred_at_minute,
                event.expires_at_minute,
            )
        }),
        earned,
        morale_earning_minutes,
        interval_end_minute,
    );
    if let Some(mut event) = existing {
        event.kind = MoraleEventKind::Leisure;
        event.magnitude = magnitude;
        event.occurred_at_minute = interval_end_minute;
        event.expires_at_minute =
            interval_end_minute.saturating_add_minutes(RECENT_MORALE_DURATION_MINUTES);
        ctx.db.morale_event().id().update(event);
    } else {
        ctx.db.morale_event().insert(MoraleEvent {
            id: 0,
            character_id: u64::from(character_id),
            kind: MoraleEventKind::Leisure,
            magnitude,
            occurred_at_minute: interval_end_minute,
            expires_at_minute: interval_end_minute
                .saturating_add_minutes(RECENT_MORALE_DURATION_MINUTES),
            source_id: Some(LEISURE_MORALE_SOURCE_ID.into()),
        });
    }
}

/// Apply the shared settlement Leisure outcome to durable fatigue and to one
/// stable morale source. Morale is earned from the interval's starting state;
/// it is never recomputed prospectively from the post-rest fatigue value.
/// This is separate from healing because only time that reaches the saved
/// downtime schedule receives its Leisure allocation.
pub fn apply_settlement_leisure_condition(
    ctx: &ReducerContext,
    character_id: CharacterId,
    schedule: DailySchedule,
    elapsed_minutes: u64,
    interval_end_minute: StrategicMinute,
) -> Result<(), String> {
    let mut stats = ctx
        .db
        .character_stats()
        .character_id()
        .find(u64::from(character_id))
        .ok_or("Character stats not found")?;
    let outcome = settlement_leisure_outcome(schedule, elapsed_minutes, stats.calories_used);
    stats.calories_used = (stats.calories_used + outcome.fatigue_delta).max(0.0);
    ctx.db.character_stats().character_id().update(stats);
    upsert_leisure_morale(
        ctx,
        (character_id).into(),
        outcome.morale,
        outcome.morale_earning_minutes,
        interval_end_minute,
    );
    crate::residence::apply_residence_leisure_morale(
        ctx,
        (character_id).into(),
        outcome.morale,
        interval_end_minute,
    )?;
    Ok(())
}

/// Rest performed away from a settlement. Camps relieve fatigue and permit
/// natural recovery but do not refill rations or water.
pub fn apply_camp_rest_condition(
    ctx: &ReducerContext,
    character_id: u64,
    elapsed_minutes: u64,
) -> Result<(), String> {
    initialize_character_condition(ctx, character_id.into());
    apply_elapsed_needs(ctx, (character_id).into(), elapsed_minutes)?;
    apply_camp_rest_recovery_condition(ctx, (character_id).into(), elapsed_minutes)
}

/// Apply only the recovery portion of camp rest. Callers that must process a
/// disease terminal boundary consume needs first, then skip this after death.
pub fn apply_camp_rest_recovery_condition(
    ctx: &ReducerContext,
    character_id: CharacterId,
    elapsed_minutes: u64,
) -> Result<(), String> {
    let days = elapsed_minutes as f32 / (24.0 * 60.0);
    let mut stats = ctx
        .db
        .character_stats()
        .character_id()
        .find(u64::from(character_id))
        .ok_or("Character stats not found")?;
    stats.calories_used = (stats.calories_used - STRATEGIC_TRAVEL_KCAL_PER_DAY * days).max(0.0);
    ctx.db.character_stats().character_id().update(stats);
    refresh_character_strategic_condition(ctx, (character_id).into()).map(|_| ())
}

pub fn apply_blood_loss(
    ctx: &ReducerContext,
    character_id: CharacterId,
    fraction_of_maximum: f32,
) -> Result<(), String> {
    initialize_character_condition(ctx, character_id.into());
    let mut condition = ctx
        .db
        .character_condition()
        .character_id()
        .find(u64::from(character_id))
        .ok_or("Character condition not found")?;
    condition.current_blood_ml = (condition.current_blood_ml
        - condition.maximum_blood_ml * fraction_of_maximum.max(0.0))
    .max(0.0);
    let circulatory_failure = condition.maximum_blood_ml > 0.0
        && condition.current_blood_ml / condition.maximum_blood_ml <= 0.10;
    ctx.db
        .character_condition()
        .character_id()
        .update(condition);
    if circulatory_failure {
        crate::transition_character_to_dead(
            ctx,
            (character_id).into(),
            crate::DeathCause::CirculatoryFailure,
            crate::DeathSource::Strategic,
            Some("critical-blood-loss".into()),
        )?;
        Ok(())
    } else {
        refresh_character_strategic_condition(ctx, (character_id).into()).map(|_| ())
    }
}

/// Set the authoritative blood fraction after a combined bleeding/recovery
/// interval and commit circulatory death at the caller's already-clipped clock.
pub fn set_blood_fraction(
    ctx: &ReducerContext,
    character_id: CharacterId,
    fraction: f32,
) -> Result<(), String> {
    initialize_character_condition(ctx, character_id.into());
    let mut condition = ctx
        .db
        .character_condition()
        .character_id()
        .find(u64::from(character_id))
        .ok_or("Character condition not found")?;
    condition.current_blood_ml = condition.maximum_blood_ml * fraction.clamp(0.0, 1.0);
    let terminal = condition.maximum_blood_ml > 0.0
        && condition.current_blood_ml / condition.maximum_blood_ml <= 0.10;
    ctx.db
        .character_condition()
        .character_id()
        .update(condition);
    if terminal {
        crate::transition_character_to_dead(
            ctx,
            (character_id).into(),
            crate::DeathCause::CirculatoryFailure,
            crate::DeathSource::Strategic,
            Some("critical-blood-loss".into()),
        )?;
        Ok(())
    } else {
        refresh_character_strategic_condition(ctx, (character_id).into()).map(|_| ())
    }
}

#[reducer]
pub fn refresh_strategic_condition(ctx: &ReducerContext, character_id: u64) -> Result<(), String> {
    refresh_character_strategic_condition(ctx, (character_id).into())
        .map(|_| ())
        .map_err(|error: StrategicConditionError| error.to_string())
}

#[reducer]
pub fn set_character_religion(
    ctx: &ReducerContext,
    character_id: u64,
    religion_id: String,
) -> Result<(), String> {
    crate::strategic::require_strategic_character_authority(ctx, (character_id).into())
        .map_err(|error: crate::strategic::StrategicCharacterAuthorityError| error.to_string())?;
    crate::character::require_living_character(ctx, (character_id).into())
        .map_err(|error: crate::character::LivingCharacterError| error.to_string())?;
    initialize_character_condition(ctx, character_id.into());
    let religion_id = religion_id.trim();
    if !religion_id.is_empty() {
        let character = ctx
            .db
            .character()
            .id()
            .find(character_id)
            .ok_or("Character not found")?;
        let settlement_id = character
            .current_settlement_id
            .ok_or("A religion can only be professed at a settlement")?;
        let settlement = ctx
            .db
            .settlement()
            .id()
            .find(&settlement_id)
            .ok_or("Character's settlement not found")?;
        require_profession_service(&settlement.economy)?;
        if !settlement
            .religious_status
            .represented_religions()
            .iter()
            .any(|religion| religion.religion_id() == religion_id)
        {
            return Err("This settlement's priest cannot receive that profession of faith".into());
        }
    }
    let mut condition = ctx
        .db
        .character_condition()
        .character_id()
        .find(character_id)
        .ok_or("Character condition not found")?;
    condition.religion_id = (!religion_id.is_empty()).then(|| religion_id.into());
    ctx.db
        .character_condition()
        .character_id()
        .update(condition);
    refresh_character_strategic_condition(ctx, (character_id).into())
        .map(|_| ())
        .map_err(|error: StrategicConditionError| error.to_string())
}

fn require_profession_service(
    profile: &adventuresim_world_schema::SettlementEconomyProfile,
) -> Result<(), String> {
    use adventuresim_core::settlement_economy::action_service_available;
    use adventuresim_world_schema::SettlementActionService;
    if action_service_available(profile, SettlementActionService::Temple) {
        Ok(())
    } else {
        Err("This settlement has no church to receive a profession of faith".into())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CharacterNeeds, ElapsedNeedsProvision, MoraleSourceKind, ProjectedMoraleSource,
        STRATEGIC_TRAVEL_KCAL_PER_DAY, STRATEGIC_TRAVEL_WATER_ML_PER_DAY,
        accumulated_leisure_morale, elapsed_needs_plan, food_reserve_days,
        holy_day_demand_has_expired, leisure_morale_effect, rank_morale_sources,
        religion_cohort_pressure, require_profession_service,
        settlement_rest_elapsed_needs_provision, water_reserve_days,
    };
    use adventuresim_world_schema::calendar::StrategicMinute;
    use std::collections::BTreeMap;

    #[test]
    fn settlement_rest_needs_preserve_public_and_private_provisions() {
        use adventuresim_world_schema::SettlementActionService;

        for provision in [
            crate::time::SettlementRestProvision::PublicService(SettlementActionService::Inn),
            crate::time::SettlementRestProvision::Residence,
        ] {
            assert_eq!(
                settlement_rest_elapsed_needs_provision(provision),
                ElapsedNeedsProvision::FullBoard
            );
        }
        for provision in [
            crate::time::SettlementRestProvision::PublicService(SettlementActionService::Temple),
            crate::time::SettlementRestProvision::PrivateDowntime,
        ] {
            assert_eq!(
                settlement_rest_elapsed_needs_provision(provision),
                ElapsedNeedsProvision::SettlementWater
            );
        }
    }

    #[test]
    fn profession_requires_an_available_temple_service() {
        let mut profile = adventuresim_world_schema::SettlementEconomyProfile::stage_placeholder();
        assert!(require_profession_service(&profile).is_err());
        profile
            .services
            .push(adventuresim_world_schema::SettlementService::Temple);
        assert!(require_profession_service(&profile).is_ok());
    }

    #[test]
    fn irreverent_only_cohorts_create_no_religious_pressure() {
        let cohorts = BTreeMap::from([
            ("roman_catholic".to_string(), vec![0.0]),
            ("lutheran".to_string(), vec![0.0]),
        ]);
        assert_eq!(
            religion_cohort_pressure(cohorts, "roman_catholic"),
            (0.0, 0.0)
        );
        assert_eq!(
            religion_cohort_pressure(BTreeMap::new(), "judaism"),
            (0.0, 0.0)
        );
    }

    use adventuresim_core::strategic_schedule::{
        DailySchedule, LEISURE_MORALE_LIMIT, settlement_leisure_outcome,
    };
    use adventuresim_world_schema::calendar::MINUTES_PER_DAY;

    #[test]
    fn condition_reserve_days_exclude_carried_provisions() {
        let needs = CharacterNeeds {
            character_id: 1,
            food_balance_kcal: 3_000.0,
            water_balance_ml: 1_000.0,
        };

        assert_eq!(food_reserve_days(&needs), 0.5);
        assert_eq!(water_reserve_days(&needs), 0.25);
    }

    #[test]
    fn inn_board_clears_food_and_water_deficits_without_consuming_provisions() {
        let plan = elapsed_needs_plan(
            -1_450.0,
            -900.0,
            MINUTES_PER_DAY,
            ElapsedNeedsProvision::FullBoard,
        );

        assert_eq!(plan.food_balance_kcal, 0.0);
        assert_eq!(plan.water_balance_ml, 0.0);
        assert!(!plan.consume_stored_food);
        assert!(!plan.consume_stored_water);
    }

    #[test]
    fn inn_board_preserves_existing_food_and_water_fullness_without_creating_more() {
        let plan = elapsed_needs_plan(
            250.0,
            375.0,
            MINUTES_PER_DAY * 3,
            ElapsedNeedsProvision::FullBoard,
        );

        assert_eq!(plan.food_balance_kcal, 250.0);
        assert_eq!(plan.water_balance_ml, 375.0);
        assert!(!plan.consume_stored_food);
        assert!(!plan.consume_stored_water);
    }

    #[test]
    fn non_inn_elapsed_needs_still_draw_from_carried_provisions() {
        let plan = elapsed_needs_plan(
            -500.0,
            250.0,
            MINUTES_PER_DAY,
            ElapsedNeedsProvision::PersonalSupplies,
        );

        assert_eq!(
            plan.food_balance_kcal,
            -500.0 - STRATEGIC_TRAVEL_KCAL_PER_DAY
        );
        assert_eq!(
            plan.water_balance_ml,
            250.0 - STRATEGIC_TRAVEL_WATER_ML_PER_DAY
        );
        assert!(plan.consume_stored_food);
        assert!(plan.consume_stored_water);
    }

    #[test]
    fn settlement_rest_supplies_water_but_not_food() {
        let plan = elapsed_needs_plan(
            -500.0,
            -900.0,
            MINUTES_PER_DAY,
            ElapsedNeedsProvision::SettlementWater,
        );

        assert_eq!(
            plan.food_balance_kcal,
            -500.0 - STRATEGIC_TRAVEL_KCAL_PER_DAY
        );
        assert_eq!(plan.water_balance_ml, 0.0);
        assert!(plan.consume_stored_food);
        assert!(!plan.consume_stored_water);
    }

    #[test]
    fn holy_day_demands_expire_after_their_day_or_on_departure() {
        assert!(!holy_day_demand_has_expired(6.into(), 6.into(), false));
        assert!(holy_day_demand_has_expired(6.into(), 6.into(), true));
        assert!(holy_day_demand_has_expired(6.into(), 7.into(), false));
        assert!(!holy_day_demand_has_expired(13.into(), 12.into(), true));
    }

    #[test]
    fn raw_personality_reaction_is_ranked_before_will() {
        let mut sources = vec![
            ProjectedMoraleSource {
                key: "trait-adjusted".into(),
                kind: MoraleSourceKind::Defeat,
                label: "Defeat (Proud)".into(),
                magnitude: -30.0,
            },
            ProjectedMoraleSource {
                key: "other".into(),
                kind: MoraleSourceKind::Defeat,
                label: "Other".into(),
                magnitude: -10.0,
            },
        ];
        rank_morale_sources(&mut sources, 2.0);
        assert_eq!(sources[0].magnitude, -15.0);
        assert_eq!(sources[1].magnitude, -2.5);
    }

    #[test]
    fn leisure_morale_upsert_is_capped_independent_of_sync_frequency() {
        let duration = super::RECENT_MORALE_DURATION_MINUTES;
        let mut magnitude = 0.0;
        let mut occurred_at = StrategicMinute::ZERO;
        for interval_end in 1..=1_000 {
            let interval_end = StrategicMinute::new(interval_end);
            magnitude = accumulated_leisure_morale(
                Some((
                    magnitude,
                    occurred_at,
                    occurred_at.saturating_add_minutes(duration),
                )),
                LEISURE_MORALE_LIMIT / 100.0,
                1.0,
                interval_end,
            );
            occurred_at = interval_end;
        }
        assert_eq!(magnitude, LEISURE_MORALE_LIMIT);
    }

    #[test]
    fn zero_earned_leisure_does_not_create_morale() {
        assert_eq!(
            accumulated_leisure_morale(None, 0.0, 0.0, StrategicMinute::new(1_440)),
            0.0
        );
    }

    #[test]
    fn carried_fatigue_prevents_morale_until_the_next_qualifying_interval() {
        let schedule = DailySchedule {
            combat_training_minutes: 16 * 60,
            ..Default::default()
        };
        let first = settlement_leisure_outcome(schedule, MINUTES_PER_DAY, 200.0);
        let after_first = accumulated_leisure_morale(
            None,
            first.morale,
            first.morale_earning_minutes,
            StrategicMinute::new(MINUTES_PER_DAY),
        );
        assert_eq!(first.fatigue_delta, -200.0);
        assert_eq!(after_first, 0.0);

        let second = settlement_leisure_outcome(schedule, MINUTES_PER_DAY, 0.0);
        let after_second = accumulated_leisure_morale(
            None,
            second.morale,
            second.morale_earning_minutes,
            StrategicMinute::new(MINUTES_PER_DAY.saturating_mul(2)),
        );
        assert!(after_second > 0.0);
    }

    fn apply_partitioned_leisure(
        schedule: DailySchedule,
        step_minutes: u64,
        total_minutes: u64,
        starting_minute: StrategicMinute,
        starting_fatigue: f32,
        starting_morale: f32,
    ) -> (f32, f32) {
        let duration = super::RECENT_MORALE_DURATION_MINUTES;
        let mut fatigue = starting_fatigue;
        let mut morale = starting_morale;
        let mut occurred_at = StrategicMinute::ZERO;
        let mut elapsed = 0;
        while elapsed < total_minutes {
            let interval = step_minutes.min(total_minutes - elapsed);
            let interval_end = starting_minute.saturating_add_minutes(elapsed + interval);
            let outcome = settlement_leisure_outcome(schedule, interval, fatigue);
            fatigue += outcome.fatigue_delta;
            if outcome.morale > 0.0 {
                morale = accumulated_leisure_morale(
                    Some((
                        morale,
                        occurred_at,
                        occurred_at.saturating_add_minutes(duration),
                    )),
                    outcome.morale,
                    outcome.morale_earning_minutes,
                    interval_end,
                );
                occurred_at = interval_end;
            }
            elapsed += interval;
        }
        let effect = leisure_morale_effect(
            morale,
            starting_minute
                .saturating_add_minutes(total_minutes)
                .elapsed_since(occurred_at) as f32,
            duration,
        );
        (fatigue, effect)
    }

    #[test]
    fn leisure_source_with_carried_fatigue_is_partition_independent() {
        let schedule = DailySchedule {
            combat_training_minutes: 16 * 60,
            ..Default::default()
        };
        let total = 4 * MINUTES_PER_DAY;
        let start = StrategicMinute::new(2 * MINUTES_PER_DAY);
        let aggregate = apply_partitioned_leisure(schedule, total, total, start, 350.0, 2.0);
        let daily = apply_partitioned_leisure(schedule, MINUTES_PER_DAY, total, start, 350.0, 2.0);
        let hourly = apply_partitioned_leisure(schedule, 60, total, start, 350.0, 2.0);

        assert!((aggregate.0 - daily.0).abs() < 0.001);
        assert!((aggregate.0 - hourly.0).abs() < 0.001);
        assert!((aggregate.1 - daily.1).abs() < 0.001);
        assert!((aggregate.1 - hourly.1).abs() < 0.001);
        assert_eq!(aggregate.1, LEISURE_MORALE_LIMIT);
    }

    #[test]
    fn leisure_source_decay_before_earning_is_partition_independent_below_cap() {
        let schedule = DailySchedule {
            combat_training_minutes: 17 * 60,
            ..Default::default()
        };
        let total = 2 * MINUTES_PER_DAY;
        let start = StrategicMinute::new(MINUTES_PER_DAY);
        let aggregate = apply_partitioned_leisure(schedule, total, total, start, 150.0, 2.0);
        let daily = apply_partitioned_leisure(schedule, MINUTES_PER_DAY, total, start, 150.0, 2.0);
        let hourly = apply_partitioned_leisure(schedule, 60, total, start, 150.0, 2.0);

        assert!((aggregate.1 - daily.1).abs() < 0.001);
        assert!((aggregate.1 - hourly.1).abs() < 0.001);
        assert!(aggregate.1 > 0.0 && aggregate.1 < LEISURE_MORALE_LIMIT);
    }
}
