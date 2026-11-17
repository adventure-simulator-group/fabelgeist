//! A local visit commits chronology without schedule or rest rewards.

use adventuresim_core::strategic_time::PersonalClock;

use super::*;

/// Admit a strategic writer only while no ordinary live visit owns the clock.
/// Initialization is safe only after this authority check has succeeded.
pub(super) fn require_unleased_character_time(
    ctx: &ReducerContext,
    character_id: u64,
) -> Result<CharacterTime, String> {
    crate::tactical::local_time::require_unleased_time(ctx, character_id)?;
    ensure_character_time(ctx, character_id)?;
    ctx.db
        .character_time()
        .character_id()
        .find(character_id)
        .ok_or_else(|| "Character time record not found".into())
}

pub(crate) fn commit_local_chronology(
    ctx: &ReducerContext,
    character_id: u64,
    starting: PersonalClock,
    ending: PersonalClock,
) -> Result<(), String> {
    let mut time = ctx
        .db
        .character_time()
        .character_id()
        .find(character_id)
        .ok_or("Character time is missing")?;
    if time.minutes != starting.minute() || time.fraction != starting.fraction() {
        return Err("Character chronology changed outside the live scene owner".into());
    }
    ending
        .minute()
        .checked_elapsed_since(starting.minute())
        .ok_or("Local chronology cannot move backwards")?;
    time.minutes = ending.minute();
    time.fraction = ending.fraction();
    ctx.db.character_time().character_id().update(time);
    // Unleased NPC relatives already follow this same wall interval through
    // their autonomous scheduler. Journey-style family propagation would count
    // the local interval a second time; live relatives own their own visits.
    settle_lifecycle_after_character_time_write(ctx, character_id, ending.minute())
}
