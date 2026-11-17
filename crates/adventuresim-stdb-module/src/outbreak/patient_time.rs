//! Background outbreak materialization cannot advance a live scene's clock.

use adventuresim_world_schema::calendar::StrategicMinute;
use spacetimedb::ReducerContext;

use crate::time::character_time;

pub(super) fn advance_unleased_patient_time(
    ctx: &ReducerContext,
    character_id: u64,
    minute: StrategicMinute,
) -> Result<(), String> {
    if crate::tactical::local_time::owns_character_time(ctx, character_id)? {
        return Ok(());
    }
    let mut time = ctx
        .db
        .character_time()
        .character_id()
        .find(character_id)
        .ok_or("Outbreak patient Character has no ordinary clock")?;
    if time.minutes >= minute {
        return Ok(());
    }
    time.minutes = minute;
    ctx.db.character_time().character_id().update(time);
    crate::time::settle_lifecycle_after_character_time_write(ctx, character_id, minute)
}
