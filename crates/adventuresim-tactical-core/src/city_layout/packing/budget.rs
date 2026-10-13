//! Operational allocations and counted work, independent of physical city limits.
use serde::{Deserialize, Serialize};

/// A finite allocation of solver states. Zero explicitly denies further work.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SearchBudget(usize);

/// Actual attempted states, including row-order candidates and LP nodes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExploredSearchNodes(usize);

impl SearchBudget {
    pub const fn new(maximum: usize) -> Self {
        Self(maximum)
    }
    pub fn maximum(self) -> usize {
        self.0
    }
    pub fn remaining(self, explored: ExploredSearchNodes) -> Self {
        Self(self.0.saturating_sub(explored.0))
    }
    pub fn first_half(self) -> Self {
        Self(self.0 / 2)
    }
    pub fn capped_by(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }
    pub fn exhausted(self, explored: ExploredSearchNodes) -> bool {
        explored.0 >= self.0
    }
}

impl ExploredSearchNodes {
    pub const fn new(count: usize) -> Self {
        Self(count)
    }
    pub fn count(self) -> usize {
        self.0
    }
    pub(super) fn attempt(&mut self, allocation: SearchBudget) -> bool {
        if allocation.exhausted(*self) {
            return false;
        }
        // The allocation check leaves room for this one state, even at usize::MAX.
        self.0 += 1;
        true
    }
    pub(super) fn include(&mut self, other: Self) -> Result<(), super::CoupledPackingIssue> {
        self.0 = self
            .0
            .checked_add(other.0)
            .ok_or(super::CoupledPackingIssue::InvalidModel)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allocations_preserve_split_cap_and_actual_accounting() {
        let block = SearchBudget::new(100_000);
        let authored = ExploredSearchNodes::new(5_000);
        let coupled = block.remaining(authored);
        assert_eq!(coupled.first_half().maximum(), 47_500);
        assert_eq!(
            coupled.remaining(ExploredSearchNodes::new(32)).maximum(),
            94_968
        );
        assert_eq!(coupled.capped_by(SearchBudget::new(128)).maximum(), 128);
        let mut explored = ExploredSearchNodes::default();
        let two = SearchBudget::new(2);
        assert!(explored.attempt(two));
        assert!(explored.attempt(two));
        assert!(!explored.attempt(two));
        assert_eq!(explored.count(), 2);
        assert!(SearchBudget::new(0).exhausted(ExploredSearchNodes::default()));
    }
}
