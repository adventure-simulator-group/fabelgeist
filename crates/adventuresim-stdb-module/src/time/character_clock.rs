//! Read the one authoritative personal clock without inventing a fallback.

use super::*;
use adventuresim_core::identity::CharacterId;

pub(crate) fn canonical_now(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<StrategicMinute, CharacterClockError> {
    ctx.db
        .character_time()
        .character_id()
        .find(u64::from(character_id))
        .map(|time| time.minutes)
        .ok_or(CharacterClockError {
            character: character_id,
        })
}
