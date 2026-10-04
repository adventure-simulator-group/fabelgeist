//! Select living participants without normalizing duplicate memberships.

use crate::character::StoredCharacterLifeState;
use adventuresim_core::identity::CharacterId;

pub(super) fn living_member_ids(
    members: impl IntoIterator<Item = (CharacterId, StoredCharacterLifeState)>,
) -> Vec<CharacterId> {
    let mut ids: Vec<_> = members
        .into_iter()
        .filter_map(|(character, life)| {
            (life == StoredCharacterLifeState::Living).then_some(character)
        })
        .collect();
    ids.sort_unstable();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn living_members_are_sorted_and_duplicate_memberships_are_retained() {
        let first = CharacterId::from(7);
        let last = CharacterId::from(17);
        let dead = CharacterId::from(3);
        assert_eq!(
            living_member_ids([
                (last, StoredCharacterLifeState::Living),
                (dead, StoredCharacterLifeState::Dead),
                (first, StoredCharacterLifeState::Living),
                (last, StoredCharacterLifeState::Living),
            ]),
            vec![first, last, last]
        );
        assert_eq!(
            living_member_ids([(dead, StoredCharacterLifeState::Dead)]),
            Vec::new()
        );
        assert_eq!(living_member_ids([]), Vec::new());
    }

    #[test]
    fn living_identity_order_preserves_zero_and_full_storage_width() {
        let first = CharacterId::from(0);
        let last = CharacterId::from(u64::MAX);
        assert_eq!(
            living_member_ids([
                (last, StoredCharacterLifeState::Living),
                (first, StoredCharacterLifeState::Living),
                (CharacterId::from(1), StoredCharacterLifeState::Dead),
            ]),
            vec![first, last]
        );
    }
}
