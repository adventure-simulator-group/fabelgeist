//! Checked animation measurements shared by configuration and presentation.

use super::{AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult};
use adventuresim_building_generator::spatial_geometry::PositiveLength;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into,
)]
#[serde(try_from = "f32", into = "f32")]
pub struct NonnegativeLength(f32);

#[derive(
    Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into,
)]
#[serde(try_from = "f32", into = "f32")]
pub struct NonnegativeSpeed(f32);

/// A finite fraction invariant, independent of its producer's clamp/reject policy.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into,
)]
#[serde(try_from = "f32", into = "f32")]
pub struct UnitInterval(f32);

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
impl NonnegativeSpeed {
    pub const ZERO: Self = Self(0.0);
    pub fn from_metres_per_second(value: f32) -> AnimationValueResult<Self> {
        checked_nonnegative(value, AnimationQuantity::Speed).map(Self)
    }
    pub fn metres_per_second(self) -> f32 {
        self.0
    }
}
impl UnitInterval {
    pub const ZERO: Self = Self(0.0);
    pub const ONE: Self = Self(1.0);
    pub fn from_fraction(value: f32) -> AnimationValueResult<Self> {
        if !value.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::UnitInterval,
            });
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(AnimationValueError::OutsideUnitInterval {
                quantity: AnimationQuantity::UnitInterval,
            });
        }
        Ok(Self(value))
    }
    pub fn fraction(self) -> f32 {
        self.0
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
impl TryFrom<f32> for NonnegativeSpeed {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_metres_per_second(value)
    }
}
impl TryFrom<f32> for UnitInterval {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_fraction(value)
    }
}
impl TryFrom<f32> for PositiveAnimationDuration {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_elapsed(AnimationElapsed::from_seconds(value)?)
    }
}

pub(super) fn checked_nonnegative(
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
