//! Authoritative lookup before admission of the latest stored life state.

use super::{
    Character, character,
    living::{LivingCharacterError, StoredCharacterLifeState},
};
use adventuresim_core::identity::CharacterId;
use spacetimedb::ReducerContext;

pub(crate) fn require_living_character(
    ctx: &ReducerContext,
    character_id: CharacterId,
) -> Result<Character, LivingCharacterError> {
    let character = ctx
        .db
        .character()
        .id()
        .find(u64::from(character_id))
        .ok_or(LivingCharacterError::Missing {
            character: character_id,
        })?;
    StoredCharacterLifeState::from(character.alive).require_living(character_id)?;
    Ok(character)
}
