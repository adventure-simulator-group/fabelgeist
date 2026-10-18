//! Identity of a sampled authoritative animation observation.

use derive_more::{Debug, Display, From};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, From, Display)]
#[serde(transparent)]
#[debug("{_0:?}")]
pub struct AnimationSampleTick(u64);

impl AnimationSampleTick {
    pub const ZERO: Self = Self(0);
    pub fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
    pub fn wrapping_delta(self, previous: Self) -> u64 {
        self.0.wrapping_sub(previous.0)
    }
    pub fn checked_delta(self, previous: Self) -> Option<u64> {
        self.0.checked_sub(previous.0)
    }
    /// Native simulation clock, event/report or deterministic-seed encoding.
    pub fn encoded(self) -> u64 {
        self.0
    }
}
