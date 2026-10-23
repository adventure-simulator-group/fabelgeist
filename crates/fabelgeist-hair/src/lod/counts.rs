//! Host level populations and ordinals; native resource layouts check their
//! separate byte and draw limits at the GPU boundary.

use crate::parameters::StrandCount;

/// Populations available to the policy, from the host or GPU groom producer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LodCounts {
    pub strands: StrandCount,
    pub levels: StrandLevelCount,
    pub cards: CardLevelCount,
}

/// Number of strand levels, including levels with no strands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StrandLevelCount(usize);

/// Number of card meshes prepared at different detail levels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CardLevelCount(usize);

/// An ordinal in the strand-level collection; membership is checked at use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrandLevelIndex(usize);

/// An ordinal in the card-level collection; membership is checked at use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardLevelIndex(usize);

impl StrandLevelCount {
    pub const fn from_count(count: usize) -> Self {
        Self(count)
    }

    /// Collection or numerical selection boundary.
    pub const fn count(self) -> usize {
        self.0
    }

    pub fn select(self, requested: StrandLevelIndex) -> Option<StrandLevelIndex> {
        self.0
            .checked_sub(1)
            .map(|last| StrandLevelIndex(requested.0.min(last)))
    }
}

impl CardLevelCount {
    pub const fn from_count(count: usize) -> Self {
        Self(count)
    }

    /// Collection or numerical selection boundary.
    pub const fn count(self) -> usize {
        self.0
    }

    pub fn select(self, requested: CardLevelIndex) -> Option<CardLevelIndex> {
        self.0
            .checked_sub(1)
            .map(|last| CardLevelIndex(requested.0.min(last)))
    }
}

impl StrandLevelIndex {
    pub const fn from_index(index: usize) -> Self {
        Self(index)
    }

    /// Index a host collection or cross a native ordinal boundary.
    pub const fn index(self) -> usize {
        self.0
    }
}

impl CardLevelIndex {
    pub const fn from_index(index: usize) -> Self {
        Self(index)
    }

    /// Index a host collection or cross a native ordinal boundary.
    pub const fn index(self) -> usize {
        self.0
    }
}
