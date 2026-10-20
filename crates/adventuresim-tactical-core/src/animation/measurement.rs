//! Checked animation measurements shared by configuration and presentation.

use super::{AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult};
use adventuresim_building_generator::spatial_geometry::PositiveLength;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into,
)]
#[serde(try_from = "f32", into = "f32")]
pub struct NonnegativeLength(f32);

/// Positive elapsed seconds for settings used as divisors or decay lengths.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
#[into(f32)]
pub struct PositiveAnimationDuration(AnimationElapsed);

impl NonnegativeLength {
    pub const ZERO: Self = Self(0.0);
    pub fn from_metres(value: f32) -> AnimationValueResult<Self> {
        checked_nonnegative(value, AnimationQuantity::Length).map(Self)
    }
    pub fn metres(self) -> f32 {
        self.0
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
