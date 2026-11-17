//! Transactional acceptance of the real entry, closure and travel time writers.

use std::time::Duration;

use adventuresim_core::provisioning::LocalNeeds;

use super::*;
use crate::{character_limbs, character_stats};

#[reducer]
pub fn authority_test_local_visit_handoff(
    ctx: &ReducerContext,
    bootstrap_token: String,
    character_id: u64,
) -> VisitResult<()> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let mut visit = LocalSceneVisit::active(ctx, character_id)?.ok_or("Missing fixture visit")?;
    if visit.server != ctx.sender() {
        return Err("Fixture requires the owning scene identity".into());
    }
    let duplicate =
        LocalSceneVisit::begin(ctx, character_id, ctx.sender(), LocalSceneTimeOwner::Player)?;
    if duplicate.id != visit.id {
        return Err("Repeated admission changed the visit".into());
    }
    let elapsed = Duration::from_micros(1_500_001);
    visit.entered_at = ctx
        .timestamp
        .checked_sub_duration(elapsed)
        .ok_or("Fixture timestamp underflow")?;
    let expected_clock = visit
        .starting_clock
        .advance_real_time(elapsed)
        .map_err(|error| error.to_string())?;
    let expected_needs =
        LocalNeeds::from_balances_kcal_ml(visit.food_balance_kcal, visit.water_balance_ml)
            .and_then(|needs| needs.advance_between(visit.starting_clock, expected_clock))
            .map_err(|error| error.to_string())?;
    ctx.db.local_scene_visit().id().update(visit.clone());
    if crate::time::advance_character_time(ctx, character_id, 1).is_ok() {
        return Err("Travel advanced a live-owned clock".into());
    }
    let health_before = ctx
        .db
        .character_limbs()
        .character_id()
        .find(character_id)
        .ok_or("Missing fixture body")?;
    finish_local_scene_visit(
        ctx,
        visit.id,
        LocalSceneVisitExit::Projected { fatigue: 0.25 },
    )?;
    let clock = ctx
        .db
        .character_time()
        .character_id()
        .find(character_id)
        .ok_or("Missing fixture clock")?;
    if PersonalClock::new(clock.minutes, clock.fraction) != expected_clock {
        return Err("Visit did not commit its exact elapsed chronology".into());
    }
    let needs = ctx
        .db
        .character_needs()
        .character_id()
        .find(character_id)
        .ok_or("Missing fixture needs")?;
    if needs.food_balance_kcal != expected_needs.food_balance_kcal()
        || needs.water_balance_ml != expected_needs.water_balance_ml()
    {
        return Err("Visit needs differ from the shared live projection".into());
    }
    let health_after = ctx
        .db
        .character_limbs()
        .character_id()
        .find(character_id)
        .ok_or("Missing fixture body")?;
    if spacetimedb::sats::bsatn::to_vec(&health_before).map_err(|error| error.to_string())?
        != spacetimedb::sats::bsatn::to_vec(&health_after).map_err(|error| error.to_string())?
    {
        return Err("Basic clock handoff overwrote tactical health".into());
    }
    verify_replay_and_roundtrip(ctx, &visit, expected_clock)
}

fn verify_replay_and_roundtrip(
    ctx: &ReducerContext,
    visit: &LocalSceneVisit,
    departure: PersonalClock,
) -> VisitResult<()> {
    finish_local_scene_visit(
        ctx,
        visit.id,
        LocalSceneVisitExit::Projected { fatigue: 0.25 },
    )?;
    if finish_local_scene_visit(
        ctx,
        visit.id,
        LocalSceneVisitExit::Projected { fatigue: 0.5 },
    )
    .is_ok()
    {
        return Err("Conflicting departure receipt was accepted".into());
    }
    // Use the actual retained journey writer, preserving its calendar-minute
    // accounting while keeping the local fractional remainder untouched.
    let journey_minutes = 61;
    if !crate::time::advance_character_time(ctx, visit.character_id, journey_minutes)? {
        return Err("Fixture actor died on its short journey".into());
    }
    super::super::entry::enter_tactical_scene(ctx, visit.character_id, ctx.sender())?;
    let next =
        LocalSceneVisit::active(ctx, visit.character_id)?.ok_or("Reentry lost time ownership")?;
    let arrival = PersonalClock::new(
        departure
            .minute()
            .checked_add_minutes(journey_minutes)
            .ok_or("Fixture clock overflow")?,
        departure.fraction(),
    );
    if next.id == visit.id || next.starting_clock != arrival {
        return Err("Local to journey to local round trip changed chronology".into());
    }
    finish_local_scene_visit(
        ctx,
        visit.id,
        LocalSceneVisitExit::Projected { fatigue: 0.25 },
    )?;
    let still_open = LocalSceneVisit::active(ctx, visit.character_id)?
        .ok_or("Old receipt closed a later visit")?;
    if still_open.id != next.id {
        return Err("Delayed receipt changed the new owner".into());
    }
    // A zero-time loading departure preserves fatigue earned by earlier play.
    let before = ctx
        .db
        .character_stats()
        .character_id()
        .find(visit.character_id)
        .ok_or("Missing fixture stats")?;
    finish_local_scene_visit(ctx, next.id, LocalSceneVisitExit::Unprojected)?;
    let after = ctx
        .db
        .character_stats()
        .character_id()
        .find(visit.character_id)
        .ok_or("Missing fixture stats")?;
    if before.calories_used != after.calories_used {
        return Err("Loading departure fabricated fatigue recovery".into());
    }
    Ok(())
}
