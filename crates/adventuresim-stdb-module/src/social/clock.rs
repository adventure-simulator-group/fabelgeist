use super::*;

pub(super) fn social_clock_or(
    ctx: &ReducerContext,
    character_id: u64,
    fallback: StrategicMinute,
) -> StrategicMinute {
    ctx.db
        .character_time()
        .character_id()
        .find(character_id)
        .map_or(fallback, |time| time.minutes)
}
