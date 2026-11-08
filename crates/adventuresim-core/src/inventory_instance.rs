//! Bounded changes and removal order for individually tracked inventory.

use crate::physical_object::PersonalInventoryRowId;

/// Each change can create at most this many separate persisted objects.
pub const MAX_INSTANCE_QUANTITY_CHANGE: u32 = 64;

/// The number of individual objects admitted in one change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstanceCount(u32);

impl InstanceCount {
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstanceQuantityChange {
    Add(InstanceCount),
    Remove(InstanceCount),
}

impl TryFrom<i32> for InstanceQuantityChange {
    type Error = &'static str;

    fn try_from(change: i32) -> Result<Self, Self::Error> {
        let amount = change.unsigned_abs();
        if amount > MAX_INSTANCE_QUANTITY_CHANGE {
            return Err("instance quantity change exceeds the per-call limit");
        }
        Ok(if change >= 0 {
            Self::Add(InstanceCount(amount))
        } else {
            Self::Remove(InstanceCount(amount))
        })
    }
}

/// An inventory row selected at the database boundary and its equipment state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstanceRemovalCandidate {
    pub row_id: PersonalInventoryRowId,
    pub equipped: bool,
}

/// Prefer unequipped instances, then the lowest persisted row identifier.
pub fn select_removals(
    mut candidates: Vec<InstanceRemovalCandidate>,
    count: InstanceCount,
) -> Vec<InstanceRemovalCandidate> {
    candidates.sort_by_key(|candidate| (candidate.equipped, candidate.row_id));
    candidates.truncate(count.get() as usize);
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_removal_prefers_spares_before_equipped_instances() {
        let candidate = |row_id, equipped| InstanceRemovalCandidate {
            row_id: PersonalInventoryRowId::from_raw(row_id),
            equipped,
        };
        assert_eq!(
            select_removals(
                vec![
                    candidate(10, true),
                    candidate(12, false),
                    candidate(11, false)
                ],
                InstanceCount(2)
            ),
            vec![candidate(11, false), candidate(12, false)]
        );
        assert_eq!(
            select_removals(
                vec![candidate(10, true), candidate(11, false)],
                InstanceCount(2)
            ),
            vec![candidate(11, false), candidate(10, true)]
        );
    }

    #[test]
    fn instance_quantity_changes_are_bounded_and_exact() {
        assert_eq!(
            InstanceQuantityChange::try_from(3),
            Ok(InstanceQuantityChange::Add(InstanceCount(3)))
        );
        assert_eq!(
            InstanceQuantityChange::try_from(-2),
            Ok(InstanceQuantityChange::Remove(InstanceCount(2)))
        );
        assert!(InstanceQuantityChange::try_from(65).is_err());
        assert!(InstanceQuantityChange::try_from(i32::MIN).is_err());
    }
}
