//! Initialize personal time and its schedule from the official clock once.

use super::*;
use adventuresim_core::identity::CharacterId;

impl CharacterTrainingSchedule {
    fn default_for_character(character: CharacterId) -> Self {
        Self {
            character_id: u64::from(character),
            downtime: ScheduleAllocation::default(),
        }
    }
}

pub(crate) fn initialize_character_time(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<(), WorldClockError> {
    let official_minutes = refresh_clock(ctx)?;
    if ctx
        .db
        .character_time()
        .character_id()
        .find(u64::from(character_id))
        .is_none()
    {
        ctx.db.character_time().insert(CharacterTime {
            character_id: u64::from(character_id),
            scan_id: u64::from(character_id),
            minutes: official_minutes,
        });
    }
    if ctx
        .db
        .character_training_schedule()
        .character_id()
        .find(u64::from(character_id))
        .is_none()
    {
        ctx.db.character_training_schedule().insert(
            CharacterTrainingSchedule::default_for_character(character_id),
        );
    }
    Ok(())
}
