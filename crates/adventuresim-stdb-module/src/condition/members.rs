//! Condition projections keep corpses separate from living party support.

use crate::character::StoredCharacterLifeState;
use adventuresim_core::identity::CharacterId;

pub(super) fn condition_projection_member_ids(
    character_id: CharacterId,
    life: StoredCharacterLifeState,
    living_party_members: Option<Vec<CharacterId>>,
) -> Vec<CharacterId> {
    if life == StoredCharacterLifeState::Dead {
        // A corpse still has a durable condition projection, but must not be
        // reintroduced into living party morale/capability aggregation.
        vec![character_id]
    } else {
        living_party_members.unwrap_or_else(|| vec![character_id])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corpse_projection_contains_the_requested_character_without_living_party_support() {
        assert_eq!(
            condition_projection_member_ids(
                CharacterId::from(7),
                StoredCharacterLifeState::Dead,
                Some(vec![CharacterId::from(8), CharacterId::from(9)])
            ),
            vec![CharacterId::from(7)]
        );
        assert_eq!(
            condition_projection_member_ids(
                CharacterId::from(7),
                StoredCharacterLifeState::Living,
                Some(vec![CharacterId::from(7), CharacterId::from(8)])
            ),
            vec![CharacterId::from(7), CharacterId::from(8)]
        );
    }

    #[test]
    fn living_projection_preserves_supplied_order_duplicates_and_empty_membership() {
        let character = CharacterId::from(7);
        let members = vec![CharacterId::from(9), character, CharacterId::from(9)];
        assert_eq!(
            condition_projection_member_ids(
                character,
                StoredCharacterLifeState::Living,
                Some(members.clone())
            ),
            members
        );
        assert_eq!(
            condition_projection_member_ids(
                character,
                StoredCharacterLifeState::Living,
                Some(Vec::new())
            ),
            Vec::new()
        );
        assert_eq!(
            condition_projection_member_ids(character, StoredCharacterLifeState::Living, None),
            vec![character]
        );
    }
}
