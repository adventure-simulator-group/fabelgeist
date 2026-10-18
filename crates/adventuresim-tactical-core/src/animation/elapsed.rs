//! Finite elapsed animation time shared by authority and presentation.

use super::{AnimationQuantity, AnimationValueError, AnimationValueResult};
use bevy::prelude::Reflect;
use derive_more::Debug;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Elapsed seconds, including zero for an observation that does not advance.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Reflect, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
#[reflect(opaque)]
#[debug("{_0:?}")]
pub struct AnimationElapsed(f32);

impl AnimationElapsed {
    pub const ZERO: Self = Self(0.0);

    pub const fn from_seconds(seconds: f32) -> AnimationValueResult<Self> {
        if !seconds.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::Elapsed,
            });
        }
        if seconds < 0.0 {
            return Err(AnimationValueError::Negative {
                quantity: AnimationQuantity::Elapsed,
            });
        }
        Ok(Self(seconds))
    }

    pub fn seconds(self) -> f32 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0.0
    }

    pub fn min(self, other: Self) -> Self {
        if self.0 < other.0 { self } else { other }
    }

    /// Bound an elapsed observation by a configured or authored seconds limit.
    pub fn capped_by_seconds(self, maximum: f32) -> AnimationValueResult<Self> {
        Ok(self.min(Self::from_seconds(maximum)?))
    }

    pub fn checked_add(self, other: Self) -> AnimationValueResult<Self> {
        let seconds = self.0 + other.0;
        if !seconds.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::Elapsed,
            });
        }
        Ok(Self(seconds))
    }
}

impl From<Duration> for AnimationElapsed {
    /// A native Duration is nonnegative and at most u64 seconds, whose f32
    /// seconds representation is finite. This is Bevy's delta-seconds boundary.
    fn from(duration: Duration) -> Self {
        Self(duration.as_secs_f32())
    }
}

impl TryFrom<f32> for AnimationElapsed {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_seconds(value)
    }
}
impl From<AnimationElapsed> for f32 {
    fn from(value: AnimationElapsed) -> Self {
        value.seconds()
    }
}
