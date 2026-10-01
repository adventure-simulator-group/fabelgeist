//! Executable database acceptance fixtures, excluded from production modules.
use super::properties::household_property_occupancy;
use super::*;
use crate::relationship::{Household, HouseholdMember, household};
use crate::settlement_population::settlement_resident_profile;

const ACTOR: u64 = 719001;
const GUEST: u64 = 719002;
const FAMILY: &str = "household:property-acceptance";

#[reducer]
pub fn authority_test_property_setup(
    ctx: &ReducerContext,
    bootstrap_token: String,
    catalog_json: String,
    economy_json: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    crate::strategic::require_strategic_gateway(ctx)?;
    let catalog =
        adventuresim_core::settlement_property::GeneratedHomeCatalog::parse(&catalog_json)
            .map_err(|error| error.to_string())?;
    let mut settlement = ctx
        .db
        .settlement()
        .iter()
        .next()
        .ok_or("Seeded world missing")?;
    settlement.id = catalog.settlement_id.clone();
    settlement.name = "Property acceptance town".into();
    settlement.population_estimate = catalog.population;
    settlement.economy = serde_json::from_str(&economy_json).map_err(|error| error.to_string())?;
    ctx.db.settlement().insert(settlement);
    properties::register_settlement_properties(ctx, catalog_json.clone())?;
    let characters = ctx.db.character().count();
    properties::register_settlement_properties(ctx, catalog_json)?;
    if ctx.db.character().count() != characters {
        return Err("Registration rematerialized residents".into());
    }
    let named = ctx
        .db
        .settlement_resident_profile()
        .home_settlement_id()
        .filter(&catalog.settlement_id)
        .count() as u64;
    let aggregate: u64 = ctx
        .db
        .household_property_occupancy()
        .iter()
        .filter(|row| {
            ctx.db
                .settlement_property()
                .id()
                .find(&row.property_id)
                .is_some_and(|property| property.settlement_id == catalog.settlement_id)
        })
        .map(|row| u64::from(row.unmaterialized_residents))
        .sum();
    if named + aggregate != u64::from(catalog.population) || named >= u64::from(catalog.population)
    {
        return Err(
            "Housing census must preserve population with a bounded character roster".into(),
        );
    }
    for actor in [ACTOR, GUEST] {
        crate::character::insert_new_character(
            ctx,
            format!("Property actor {actor}"),
            actor,
            false,
        )?;
        let mut row = ctx.db.character().id().find(actor).ok_or("Actor missing")?;
        row.current_settlement_id = Some(catalog.settlement_id.clone());
        ctx.db.character().id().update(row);
        crate::item::credit_personal_currency(ctx, actor, &catalog.settlement_id, 100_000)?;
    }
    ctx.db.household().insert(Household {
        id: FAMILY.into(),
        home_settlement_id: catalog.settlement_id,
        created_minute: StrategicMinute::ZERO,
    });
    for actor in [ACTOR, GUEST] {
        ctx.db.household_member().insert(HouseholdMember {
            id: format!("{FAMILY}:{actor}"),
            household_id: FAMILY.into(),
            character_id: actor,
            joined_minute: StrategicMinute::ZERO,
            role: HouseholdRole::Head,
        });
    }
    Ok(())
}

#[reducer]
pub fn authority_test_property_contract(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    crate::strategic::require_strategic_gateway(ctx)?;
    let rental = primary_residence_holding(ctx, ACTOR).ok_or("Rental missing")?;
    if rental.tenure != ResidenceTenure::Renter || rental.holder_character_id != ACTOR {
        return Err("Rental must confer tenancy without ownership".into());
    }
    let owned_id = acquire_residence_internal(
        ctx,
        ACTOR,
        &ctx.db
            .settlement_property()
            .settlement_id()
            .filter(&rental.settlement_id)
            .find(|property| {
                property.tier == HousingTier::Fancy
                    && properties::available_property(
                        ctx,
                        &property.id,
                        residence_now(ctx, ACTOR).unwrap(),
                    )
                    .is_ok()
            })
            .ok_or("Owned property missing")?
            .id,
        ResidenceTenure::Owner,
    )?;
    let owned = ctx
        .db
        .residence_holding()
        .id()
        .find(&owned_id)
        .ok_or("Owner missing")?;
    if owned.property_id == rental.property_id
        || ctx
            .db
            .residence_holding()
            .id()
            .find(&rental.id)
            .is_none_or(|row| row.status != ResidenceHoldingStatus::Active)
    {
        return Err("Buying must preserve a separate rental holding".into());
    }
    admit_household_occupant(ctx, ACTOR, owned_id.clone(), GUEST)?;
    let guest = ctx
        .db
        .residence_occupant()
        .character_id()
        .find(GUEST)
        .ok_or("Guest missing")?;
    if guest.property_id != owned.property_id
        || guest.holding_id.as_deref() != Some(owned_id.as_str())
        || ctx
            .db
            .residence_holding()
            .holder_character_id()
            .filter(GUEST)
            .count()
            != 0
        || ctx
            .db
            .household_member()
            .character_id()
            .find(GUEST)
            .is_none_or(|member| member.household_id != FAMILY)
    {
        return Err("Occupancy changed legal title or membership".into());
    }
    remove_household_occupant(ctx, ACTOR, owned_id.clone(), GUEST)?;
    if ctx
        .db
        .residence_occupant()
        .character_id()
        .find(GUEST)
        .is_some()
        || ctx
            .db
            .household_member()
            .character_id()
            .find(GUEST)
            .is_none()
    {
        return Err("Removing occupancy must preserve household membership".into());
    }
    relinquish_residence(ctx, ACTOR, rental.id.clone())?;
    properties::available_property(ctx, &rental.property_id, residence_now(ctx, ACTOR)?)?;
    if properties::available_property(ctx, &owned.property_id, residence_now(ctx, ACTOR)?).is_ok() {
        return Err("Unoccupied legal holdings must reserve their exact property".into());
    }
    Ok(())
}

#[reducer]
pub fn authority_test_property_capacity(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    crate::strategic::require_strategic_gateway(ctx)?;
    let start = residence_now(ctx, ACTOR)?;
    let owned = primary_residence_holding(ctx, ACTOR).ok_or("Owned home missing")?;
    let cheap = ctx
        .db
        .settlement_property()
        .settlement_id()
        .filter(&owned.settlement_id)
        .find(|property| {
            property.tier == HousingTier::Cheap
                && properties::available_property(
                    ctx,
                    &property.id,
                    residence_now(ctx, ACTOR).unwrap(),
                )
                .is_ok()
        })
        .ok_or("Cheap home missing")?;
    let cheap_id = acquire_residence_internal(ctx, ACTOR, &cheap.id, ResidenceTenure::Owner)?;
    move_residence_occupant_at(ctx, &owned.id, GUEST, start.saturating_add_minutes(1000))?;
    for index in 0..cheap.resident_capacity - 1 {
        let actor = 719100 + u64::from(index);
        crate::character::insert_new_character(
            ctx,
            format!("Capacity actor {actor}"),
            actor,
            false,
        )?;
        move_residence_occupant_at(ctx, &cheap_id, actor, start.saturating_add_minutes(500))?;
    }
    if move_residence_occupant_at(ctx, &cheap_id, GUEST, start.saturating_add_minutes(100)).is_ok()
        || move_residence_occupant_at(ctx, &cheap_id, GUEST, start.saturating_add_minutes(600))
            .is_ok()
    {
        return Err("Capacity must reject both current and later interval overflow".into());
    }
    let current = ctx
        .db
        .residence_occupant()
        .character_id()
        .find(GUEST)
        .ok_or("Future home missing")?;
    if current.property_id != owned.property_id {
        return Err("Rejected admission changed future home".into());
    }
    Ok(())
}
