//! Authoritative landing observations retained by animation state.

use super::{AnimationQuantity, AnimationValueError, AnimationValueResult};
use derive_more::{Debug, Display, From};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, From, Display)]
#[serde(transparent)]
#[debug("{_0:?}")]
pub struct LandingSequence(u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
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
    /// Observed report continuity is distinct from checked event coalescing.
    pub fn wrapping_distance_from(self, previous: Self) -> u64 {
        self.0.wrapping_sub(previous.0)
    }
    /// Scalar event-stream encoding, separate from a rig slot or sample tick.
    pub fn encoded(self) -> u64 {
        self.0
    }
}

impl LandingImpactSpeed {
    pub const ZERO: Self = Self(0.0);
    pub fn from_metres_per_second(speed: f32) -> AnimationValueResult<Self> {
        if !speed.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::LandingImpactSpeed,
            });
        }
        if speed < 0.0 {
            return Err(AnimationValueError::Negative {
                quantity: AnimationQuantity::LandingImpactSpeed,
            });
        }
        Ok(Self(speed))
    }
    pub fn metres_per_second(self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for LandingImpactSpeed {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_metres_per_second(value)
    }
}
impl From<LandingImpactSpeed> for f32 {
    fn from(value: LandingImpactSpeed) -> Self {
        value.metres_per_second()
    }
}
