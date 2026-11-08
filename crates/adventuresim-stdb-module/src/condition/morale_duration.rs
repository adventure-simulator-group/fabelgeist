use super::*;

pub(super) fn stored_morale_event_duration(
    ctx: &ReducerContext,
    character_id: u64,
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

pub(super) fn leisure_morale_effect(magnitude: f32, age_minutes: f32, duration: u64) -> f32 {
    if duration == 0 {
        return 0.0;
    }
    (magnitude - LEISURE_MORALE_LIMIT * age_minutes.max(0.0) / duration as f32).max(0.0)
}
