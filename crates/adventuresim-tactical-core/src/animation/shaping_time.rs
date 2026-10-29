//! Finite signed response-shaping seconds, independent of elapsed time.

use super::{
    AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult, SettleProgress,
    UnitInterval,
};
use bevy::math::FloatExt;
use serde::{Deserialize, Serialize};

/// A finite signed shaping coefficient in seconds. Each response kernel owns
/// its floor, division, completion cap or clamp; admission does not normalize it.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, derive_more::Debug, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
#[debug("{_0:?}")]
pub struct ResponseShapingTime(f32);

impl ResponseShapingTime {
    pub const fn from_seconds(value: f32) -> AnimationValueResult<Self> {
        if !value.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::ResponseShapingTime,
            });
        }
        Ok(Self(value))
    }
    pub fn seconds(self) -> f32 {
        self.0
    }
    /// Preserve addition after division and completion capping. A transient
    /// infinite quotient may become a finite completed response.
    pub fn advance_settle(
        self,
        progress: SettleProgress,
        elapsed: AnimationElapsed,
    ) -> AnimationValueResult<SettleProgress> {
        SettleProgress::from_arithmetic((progress.fraction() + elapsed.seconds() / self.0).min(1.0))
    }
    pub fn advance_guard(
        self,
        progress: super::UnitInterval,
        elapsed: AnimationElapsed,
    ) -> AnimationValueResult<super::UnitInterval> {
        Ok(super::UnitInterval::from_fraction(
            (progress.fraction() + elapsed.seconds() / self.0).clamp(0.0, 1.0),
        )?)
    }
}

impl ResponseShapingTime {
    /// Preserve the pinned scalar FloatExt difference form before a damping floor.
    pub fn checked_lerp(self, other: Self, blend: UnitInterval) -> AnimationValueResult<Self> {
        let seconds = self.0.lerp(other.0, blend.fraction());
        Self::from_seconds(seconds).map_err(|_| AnimationValueError::Overflow {
            quantity: AnimationQuantity::ResponseShapingTime,
        })
    }
}
impl TryFrom<f32> for ResponseShapingTime {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_seconds(value)
    }
}
