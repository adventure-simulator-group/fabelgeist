//! Checked animation measurements shared by configuration and presentation.

use super::{
    AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult, UnitInterval,
};
use adventuresim_building_generator::spatial_geometry::PositiveLength;
use bevy::math::FloatExt;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into,
)]
#[serde(try_from = "f32", into = "f32")]
pub struct NonnegativeLength(f32);

/// Positive elapsed seconds for action durations, divisors and decay lengths.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
#[into(f32)]
pub struct PositiveAnimationDuration(AnimationElapsed);

impl NonnegativeLength {
    pub const ZERO: Self = Self(0.0);
    pub const fn from_metres(value: f32) -> AnimationValueResult<Self> {
        match checked_nonnegative(value, AnimationQuantity::Length) {
            Ok(value) => Ok(Self(value)),
            Err(error) => Err(error),
        }
    }
    /// Admit a completed nonnegative distance computation before retaining it.
    pub fn from_arithmetic(value: f32) -> AnimationValueResult<Self> {
        if !value.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::Length,
            });
        }
        Self::from_metres(value)
    }
    pub fn metres(self) -> f32 {
        self.0
    }
    /// Metre scalars and the admitted fraction meet at Bevy's native interpolation kernel.
    pub fn checked_lerp(self, other: Self, fraction: UnitInterval) -> AnimationValueResult<Self> {
        Self::from_arithmetic(self.0.lerp(other.0, fraction.fraction()))
    }
    /// Halving remains finite/nonnegative; subnormal underflow may validly reach zero.
    pub fn half(self) -> Self {
        Self(self.0 * 0.5)
    }
}
impl PositiveAnimationDuration {
    pub fn from_elapsed(value: AnimationElapsed) -> AnimationValueResult<Self> {
        if value.is_zero() {
            return Err(AnimationValueError::NonPositive {
                quantity: AnimationQuantity::Elapsed,
            });
        }
        Ok(Self(value))
    }
    /// Duration covered by a native count of fixed sample intervals.
    pub fn sample_span(self, intervals: u32) -> AnimationValueResult<AnimationElapsed> {
        let seconds = self.seconds() * intervals as f32;
        if !seconds.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::Elapsed,
            });
        }
        Ok(AnimationElapsed::from_seconds(seconds)?)
    }
    pub fn elapsed(self) -> AnimationElapsed {
        self.0
    }
    pub fn seconds(self) -> f32 {
        self.0.seconds()
    }
}
impl From<PositiveLength> for NonnegativeLength {
    fn from(value: PositiveLength) -> Self {
        Self(value.metres())
    }
}
impl TryFrom<f32> for NonnegativeLength {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_metres(value)
    }
}

impl TryFrom<f32> for PositiveAnimationDuration {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_elapsed(AnimationElapsed::from_seconds(value)?)
    }
}

pub(super) const fn checked_nonnegative(
    value: f32,
    quantity: AnimationQuantity,
) -> AnimationValueResult<f32> {
    if !value.is_finite() {
        return Err(AnimationValueError::NonFinite { quantity });
    }
    if value < 0.0 {
        return Err(AnimationValueError::Negative { quantity });
    }
    Ok(value)
}
