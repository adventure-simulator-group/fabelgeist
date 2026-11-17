//! Guarded schedule and authority transitions for resident admission acceptance.

use super::*;
use crate::strategic::settlement;
use adventuresim_world_schema::{SettlementService, calendar::MINUTES_PER_DAY};

#[derive(Clone, Copy, SpacetimeType)]
pub enum ResidentAdmissionFixture {
    Available,
    Away,
    Elsewhere,
    HealthUnavailable,
    TransferToPlayer,
}

/// Supply the small-settlement and trade-service preconditions. Scene request
/// preparation still generates the real buildings, operators and schedules.
#[reducer]
pub fn authority_test_merchant_area(
    ctx: &ReducerContext,
    bootstrap_token: String,
    settlement_id: String,
) -> ResidentResult<()> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let mut town = ctx
        .db
        .settlement()
        .id()
        .find(&settlement_id)
        .ok_or("Merchant fixture settlement is missing")?;
    town.population_level = 1;
    town.population_estimate = 300;
    if !town
        .economy
        .services
        .contains(&SettlementService::GeneralStore)
    {
        town.economy.services.push(SettlementService::GeneralStore);
        town.economy.services.sort();
    }
    town.economy.validate()?;
    ctx.db.settlement().id().update(town);
    Ok(())
}

#[reducer]
pub fn authority_test_resident_admission(
    ctx: &ReducerContext,
    bootstrap_token: String,
    character_id: u64,
    state: ResidentAdmissionFixture,
) -> ResidentResult<()> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    if matches!(state, ResidentAdmissionFixture::TransferToPlayer) {
        ctx.db.npc_policy().character_id().delete(character_id);
        return Ok(());
    }
    let mut presence = ctx
        .db
        .settlement_resident_presence()
        .character_id()
        .find(character_id)
        .ok_or("Resident admission fixture has no presence")?;
    presence.start_minute = 0;
    presence.end_minute = u16::try_from(MINUTES_PER_DAY).map_err(|_| "Fixture day overflow")?;
    presence.context_suppressed = false;
    presence.health_suppressed = matches!(state, ResidentAdmissionFixture::HealthUnavailable);
    if matches!(state, ResidentAdmissionFixture::Elsewhere) {
        presence.location_id = SettlementVenueKind::Keep.id().into();
    } else if let Some(assignment) = ctx
        .db
        .settlement_business_operator()
        .operator_character_id()
        .find(character_id)
    {
        presence.location_id = business_location(
            assignment.business_id.key.usage,
            assignment.business_id.key.ordinal,
        )
        .into();
    }
    if matches!(state, ResidentAdmissionFixture::Away) {
        let current = u64::from(crate::time::refresh_clock(ctx)?.minute_of_day());
        let start = (current + MINUTES_PER_DAY / 2) % MINUTES_PER_DAY;
        presence.start_minute = u16::try_from(start).map_err(|_| "Fixture start overflow")?;
        presence.end_minute =
            u16::try_from((start + 1) % MINUTES_PER_DAY).map_err(|_| "Fixture end overflow")?;
    }
    ctx.db
        .settlement_resident_presence()
        .character_id()
        .update(presence);
    Ok(())
}
