//! Validated durable effects at the end of a local visit, never per tick.

use adventuresim_core::{essential::fatigue_calories_after_handoff, provisioning::LocalNeeds};

use super::*;
use crate::{character_attributes, character_limbs, character_stats};

pub(super) fn apply(
    ctx: &ReducerContext,
    visit: &LocalSceneVisit,
    clock: PersonalClock,
    fatigue: DepartureFatigue,
) -> VisitResult<()> {
    let mut needs = ctx
        .db
        .character_needs()
        .character_id()
        .find(visit.character_id)
        .ok_or("Character needs are missing")?;
    if needs.food_balance_kcal != visit.food_balance_kcal
        || needs.water_balance_ml != visit.water_balance_ml
    {
        return Err("Character needs changed outside the live scene owner".into());
    }
    // Existing provision budgets are expressed per calendar day. This native
    // numerical adapter debits the interval once; it does not eat carried food,
    // refill water, grant paid board or apply another fatigue charge.
    let projected =
        LocalNeeds::from_balances_kcal_ml(visit.food_balance_kcal, visit.water_balance_ml)
            .and_then(|needs| needs.advance_between(visit.starting_clock, clock))
            .map_err(|error| error.to_string())?;
    needs.food_balance_kcal = projected.food_balance_kcal();
    needs.water_balance_ml = projected.water_balance_ml();
    ctx.db.character_needs().character_id().update(needs);
    if let DepartureFatigue::Projected(fatigue) = fatigue {
        let mut stats = ctx
            .db
            .character_stats()
            .character_id()
            .find(visit.character_id)
            .ok_or("Character stats are missing")?;
        let attributes = ctx
            .db
            .character_attributes()
            .character_id()
            .find(visit.character_id)
            .ok_or("Character attributes are missing")?;
        let body = ctx
            .db
            .character_limbs()
            .character_id()
            .find(visit.character_id)
            .ok_or("Character limbs are missing")?;
        stats.calories_used = fatigue_calories_after_handoff(&stats, &attributes, &body, fatigue)
            .map_err(|error| error.to_string())?;
        ctx.db.character_stats().character_id().update(stats);
    }
    crate::time::commit_local_chronology(ctx, visit.character_id, visit.starting_clock, clock)?;
    crate::condition::refresh_character_strategic_condition(ctx, visit.character_id)?;
    crate::capability::refresh_character_capability(ctx, visit.character_id)?;
    Ok(())
}
