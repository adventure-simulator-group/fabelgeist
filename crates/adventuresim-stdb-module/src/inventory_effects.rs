//! Shared durable effects; entry points own host or non-live authorization.

pub(crate) use transfer::PersonalInventoryTransfer;

use spacetimedb::ReducerContext;

use crate::{Character, tactical::tactical_server_authority};

mod transfer;

/// Non-live entry points cannot bypass the registered scene's inventory owner.
pub(crate) fn require_non_live_actor(
    ctx: &ReducerContext,
    character_id: u64,
) -> Result<Character, String> {
    crate::strategic::require_strategic_character_authority(ctx, character_id)?;
    let character = crate::character::require_living_character(ctx, character_id)?;
    if ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(character.server)
        .is_some()
    {
        return Err("Character inventory belongs to a live tactical scene".into());
    }
    crate::strategic::require_character_no_unresolved_encounter(ctx, character_id)?;
    Ok(character)
}
