//! Latest stored membership participating in strategic party activity.

mod selection;

use super::party_member;
use crate::character::{StoredCharacterLifeState, character};
use adventuresim_core::identity::CharacterId;
use spacetimedb::ReducerContext;

/// Durable dead memberships remain stored, but never participate in activity.
/// Missing character rows are omitted, and surviving IDs retain numeric order.
pub(crate) fn living_party_member_ids(ctx: &ReducerContext, party_id: &str) -> Vec<CharacterId> {
    selection::living_member_ids(
        ctx.db
            .party_member()
            .party_id()
            .filter(party_id)
            .filter_map(|membership| {
                ctx.db
                    .character()
                    .id()
                    .find(membership.character_id)
                    .map(|character| {
                        (
                            CharacterId::from(character.id),
                            StoredCharacterLifeState::from(character.alive),
                        )
                    })
            }),
    )
}
