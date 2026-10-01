//! Exact hostile identities captured when a mission binds.
use std::{collections::BTreeSet, fmt, num::NonZeroU32};

/// A nonempty unique enemy snapshot. Its cardinality is derived for launch
/// projections; party receipt limits do not constrain hostile enrollment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissionEnemyRoster {
    enemy_ids: Vec<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnemyRosterError {
    Empty,
    DuplicateEnemy,
    CountOverflow,
}

impl fmt::Display for EnemyRosterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "A bound mission requires at least one enemy",
            Self::DuplicateEnemy => "Mission enemy authority contains duplicate identities",
            Self::CountOverflow => "Mission enemy count exceeds its transport representation",
        })
    }
}
impl std::error::Error for EnemyRosterError {}

fn checked_count(length: usize) -> Result<NonZeroU32, EnemyRosterError> {
    let count = u32::try_from(length).map_err(|_| EnemyRosterError::CountOverflow)?;
    NonZeroU32::new(count).ok_or(EnemyRosterError::Empty)
}

impl TryFrom<Vec<u64>> for MissionEnemyRoster {
    type Error = EnemyRosterError;

    fn try_from(enemy_ids: Vec<u64>) -> Result<Self, Self::Error> {
        checked_count(enemy_ids.len())?;
        if enemy_ids.iter().collect::<BTreeSet<_>>().len() != enemy_ids.len() {
            return Err(EnemyRosterError::DuplicateEnemy);
        }
        Ok(Self { enemy_ids })
    }
}

impl MissionEnemyRoster {
    pub fn enemy_count(&self) -> NonZeroU32 {
        checked_count(self.enemy_ids.len()).expect("constructor validates cardinality")
    }

    pub fn enemy_ids(&self) -> &[u64] {
        &self.enemy_ids
    }

    pub fn into_enemy_ids(self) -> Vec<u64> {
        self.enemy_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_and_order_come_from_the_captured_enemies() {
        let ids: Vec<_> = (0..=super::super::MAX_TACTICAL_RECEIPT_PARTICIPANTS as u64)
            .rev()
            .collect();
        let roster = MissionEnemyRoster::try_from(ids.clone()).unwrap();
        assert_eq!(roster.enemy_count().get() as usize, ids.len());
        assert_eq!(roster.enemy_ids(), ids);
        assert_eq!(roster.into_enemy_ids(), ids);
    }

    #[test]
    fn empty_or_duplicate_snapshots_cannot_launch() {
        assert_eq!(
            MissionEnemyRoster::try_from(vec![]),
            Err(EnemyRosterError::Empty)
        );
        assert_eq!(
            MissionEnemyRoster::try_from(vec![7, 3, 7]),
            Err(EnemyRosterError::DuplicateEnemy)
        );
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn transport_count_cannot_overflow() {
        assert_eq!(checked_count(u32::MAX as usize).unwrap().get(), u32::MAX);
        assert_eq!(
            checked_count(u32::MAX as usize + 1),
            Err(EnemyRosterError::CountOverflow)
        );
    }
}
