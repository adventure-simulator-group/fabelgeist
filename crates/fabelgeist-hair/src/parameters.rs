//! Hair counts, policies and the fixed-width seed used by hair kernels.

use fabelgeist_determinism::Seed;
use serde::{Deserialize, Serialize};

/// Number of hair strands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StrandCount(usize);

/// Number of samples in each strand.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PointsPerStrand(usize);

/// Number of guide neighbours blended for a strand.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GuideNeighborCount(usize);

/// A hair shader's signed 32-bit seed, backed by the canonical seed owner.
/// The bit pattern, hash algorithm and numeric JSON representation are retained.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub struct HairSeed(Seed);

/// Whether coarse locks are gathered into ropes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "bool", into = "bool")]
pub enum LockFormation {
    #[default]
    Separate,
    Roped,
}

/// Whether the authored groom is draped under the requested acceleration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "bool", into = "bool")]
pub enum Draping {
    #[default]
    Disabled,
    Enabled,
}

impl StrandCount {
    pub const fn from_count(count: usize) -> Self {
        Self(count)
    }
    /// Collection indexing or native curve-kernel representation.
    pub const fn count(self) -> usize {
        self.0
    }
}

impl PointsPerStrand {
    pub const fn from_count(count: usize) -> Self {
        Self(count)
    }
    /// Collection indexing or native curve-kernel representation.
    pub const fn count(self) -> usize {
        self.0
    }
}

impl GuideNeighborCount {
    pub const fn from_count(count: usize) -> Self {
        Self(count)
    }
    /// Collection indexing or native curve-kernel representation.
    pub const fn count(self) -> usize {
        self.0
    }
}

impl HairSeed {
    pub const fn from_word(word: i32) -> Self {
        Self(Seed::from_u64(word as u32 as u64))
    }
    pub const fn word(self) -> i32 {
        self.0.to_u64() as u32 as i32
    }
}

impl From<i32> for HairSeed {
    fn from(word: i32) -> Self {
        Self::from_word(word)
    }
}

impl From<HairSeed> for i32 {
    fn from(seed: HairSeed) -> Self {
        seed.word()
    }
}

impl LockFormation {
    pub const fn from_enabled(enabled: bool) -> Self {
        if enabled { Self::Roped } else { Self::Separate }
    }
    pub const fn enabled(self) -> bool {
        matches!(self, Self::Roped)
    }
}

impl From<bool> for LockFormation {
    fn from(enabled: bool) -> Self {
        Self::from_enabled(enabled)
    }
}

impl From<LockFormation> for bool {
    fn from(formation: LockFormation) -> Self {
        formation.enabled()
    }
}

impl Draping {
    pub const fn from_enabled(enabled: bool) -> Self {
        if enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }
    pub const fn enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

impl From<bool> for Draping {
    fn from(enabled: bool) -> Self {
        Self::from_enabled(enabled)
    }
}

impl From<Draping> for bool {
    fn from(draping: Draping) -> Self {
        draping.enabled()
    }
}
