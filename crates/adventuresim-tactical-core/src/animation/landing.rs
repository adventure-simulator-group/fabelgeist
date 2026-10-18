//! Authoritative landing observations retained by animation state.

use derive_more::{Debug, Display, From};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, From, Display)]
#[serde(transparent)]
#[debug("{_0:?}")]
pub struct LandingSequence(u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
#[debug("{_0:?}")]
pub struct LandingImpactSpeed(f32);

impl LandingSequence {
    pub const ZERO: Self = Self(0);
    pub fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
    pub fn events_since(self, previous: Self) -> Option<u64> {
        self.0.checked_sub(previous.0)
    }
    /// Scalar event-stream encoding, separate from a rig slot or sample tick.
    pub fn encoded(self) -> u64 {
        self.0
    }
}

impl LandingImpactSpeed {
    pub const ZERO: Self = Self(0.0);
    pub fn from_metres_per_second(speed: f32) -> Self {
        Self(speed)
    }
    /// Native wire admission retains the skeleton decoder's finite, nonnegative rule.
    pub(super) fn from_wire_metres_per_second(speed: f32) -> Self {
        Self(if speed.is_finite() {
            speed.max(0.0)
        } else {
            0.0
        })
    }
    pub fn metres_per_second(self) -> f32 {
        self.0
    }
}
