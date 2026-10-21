//! Authored angular coefficients retain their original scalar units.

use super::{
    AnimationQuantity, AnimationValueError, AnimationValueResult, measurement::checked_nonnegative,
};
use serde::{Deserialize, Serialize};

/// A finite, zero-permitting angle magnitude authored and serialized in degrees.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into,
)]
#[serde(try_from = "f32", into = "f32")]
pub struct AngularMagnitudeDegrees(f32);

/// A finite, zero-permitting angular speed authored in degrees per second.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
pub struct AngularSpeedDegrees(f32);

/// A finite nonnegative angular speed whose authored scalar is radians/second.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
pub struct AngularSpeedRadians(f32);

/// Finite signed authored degrees, without wrapping or magnitude normalization.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into,
)]
#[serde(try_from = "f32", into = "f32")]
pub struct SignedAngleDegrees(f32);

/// Finite signed authored degrees/second; a consumer owns any max-zero policy.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
pub struct SignedAngularSpeedDegrees(f32);

impl SignedAngleDegrees {
    /// Native minimum of admitted finite angles, preserving signed zero.
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    /// Native maximum of admitted finite angles, preserving signed zero.
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    pub const ZERO: Self = Self(0.0);
    pub fn from_degrees(value: f32) -> AnimationValueResult<Self> {
        if !value.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::BodyAngle,
            });
        }
        Ok(Self(value))
    }
    pub fn degrees(self) -> f32 {
        self.0
    }
    pub fn radians(self) -> f32 {
        self.0.to_radians()
    }
}
impl SignedAngularSpeedDegrees {
    pub fn from_degrees_per_second(value: f32) -> AnimationValueResult<Self> {
        if !value.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::BodyAngularVelocity,
            });
        }
        Ok(Self(value))
    }
    pub fn degrees_per_second(self) -> f32 {
        self.0
    }
}
impl TryFrom<f32> for SignedAngleDegrees {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_degrees(value)
    }
}
impl TryFrom<f32> for SignedAngularSpeedDegrees {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_degrees_per_second(value)
    }
}

impl AngularMagnitudeDegrees {
    /// Native minimum of admitted finite magnitudes, preserving signed zero.
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    /// Native maximum of admitted finite magnitudes, preserving signed zero.
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    pub const ZERO: Self = Self(0.0);
    pub fn degrees(self) -> f32 {
        self.0
    }
    pub fn from_degrees(value: f32) -> AnimationValueResult<Self> {
        checked_nonnegative(value, AnimationQuantity::BodyAngle).map(Self)
    }

    /// Native radian coefficient for an angle or quaternion calculation.
    pub fn radians(self) -> f32 {
        self.0.to_radians()
    }
}

impl AngularSpeedDegrees {
    pub fn from_degrees_per_second(value: f32) -> AnimationValueResult<Self> {
        checked_nonnegative(value, AnimationQuantity::BodyAngularVelocity).map(Self)
    }

    /// Native angular-speed cap for a radians/seconds numerical integrator.
    pub fn radians_per_second(self) -> f32 {
        self.0.to_radians()
    }
}

impl TryFrom<f32> for AngularMagnitudeDegrees {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_degrees(value)
    }
}

impl TryFrom<f32> for AngularSpeedDegrees {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_degrees_per_second(value)
    }
}

impl AngularSpeedRadians {
    pub const ZERO: Self = Self(0.0);
    pub fn from_radians_per_second(value: f32) -> AnimationValueResult<Self> {
        checked_nonnegative(value, AnimationQuantity::BodyAngularVelocity).map(Self)
    }
    /// Native radian-speed coefficient at the physics angular-velocity operation.
    pub fn radians_per_second(self) -> f32 {
        self.0
    }
}
impl TryFrom<f32> for AngularSpeedRadians {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_radians_per_second(value)
    }
}
