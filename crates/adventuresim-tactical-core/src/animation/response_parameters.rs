//! Scalar tuning contracts for body response, compression and look distribution.

use super::{
    AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult, HeightDistance,
    LandingImpactSpeed, UnitInterval, measurement::checked_nonnegative,
};
use serde::{Deserialize, Serialize};

/// A positive reference acceleration magnitude in metres per second squared.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
pub struct ReferenceAcceleration(f32);

/// A zero-permitting response coefficient in inverse seconds.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
pub struct ResponseRate(f32);

/// A finite nonnegative body-lean gain; values above one remain valid.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
pub struct InertialLeanGain(f32);

/// Compression metres per impact metre/second, independent of elapsed time.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
pub struct ImpactCompressionCompliance(f32);

/// A positive, possibly fractional divisor distributing camera look over joints.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
pub struct LookDistribution(f32);

/// Unbounded nonnegative muscle-envelope gain, separate from body-lean gain.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
pub struct MuscleTone(f32);

/// Angular acceleration per radian of pose error, in inverse seconds squared.
/// The PD controller integrates this into an angular-velocity kick, not torque.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
pub struct AngularStiffness(f32);

impl ReferenceAcceleration {
    pub fn from_metres_per_second_squared(value: f32) -> AnimationValueResult<Self> {
        let value = checked_nonnegative(value, AnimationQuantity::ReferenceAcceleration)?;
        if value == 0.0 {
            return Err(AnimationValueError::NonPositive {
                quantity: AnimationQuantity::ReferenceAcceleration,
            });
        }
        Ok(Self(value))
    }
    /// Native component/reference ratio inside the body-local acceleration kernel.
    pub fn component_ratio(self, acceleration: f32) -> f32 {
        acceleration / self.0
    }
}

impl ResponseRate {
    pub fn from_per_second(value: f32) -> AnimationValueResult<Self> {
        checked_nonnegative(value, AnimationQuantity::ResponseRate).map(Self)
    }
    pub fn exponential_fraction(
        self,
        elapsed: AnimationElapsed,
    ) -> AnimationValueResult<UnitInterval> {
        let exponent = -self.0 * elapsed.seconds();
        if !exponent.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::BodyLocalAcceleration,
            });
        }
        Ok(UnitInterval::from_fraction(1.0 - exponent.exp())?)
    }
    /// Linear per-step drain, capped at full response by the existing damping law.
    pub fn step_fraction(self, elapsed: AnimationElapsed) -> AnimationValueResult<UnitInterval> {
        Ok(UnitInterval::from_fraction(
            (self.0 * elapsed.seconds()).min(1.0),
        )?)
    }
    /// Native coefficient multiplying angular velocity in the PD muscle kernel.
    pub fn per_second(self) -> f32 {
        self.0
    }
}

impl InertialLeanGain {
    pub fn from_gain(value: f32) -> AnimationValueResult<Self> {
        checked_nonnegative(value, AnimationQuantity::InertialLeanGain).map(Self)
    }
    pub fn interpolated(self, other: Self, fraction: UnitInterval) -> AnimationValueResult<Self> {
        // Same scalar order as glam's FloatExt::lerp, including subnormal results.
        Self::from_gain(self.0 + (other.0 - self.0) * fraction.fraction())
            .map_err(|error| error.for_quantity(AnimationQuantity::BodyAngle))
    }
    /// Native multiplier of a signed body angle, independent of additive pose weight.
    pub fn gain(self) -> f32 {
        self.0
    }
}

impl ImpactCompressionCompliance {
    pub fn from_metres_per_metres_per_second(value: f32) -> AnimationValueResult<Self> {
        checked_nonnegative(value, AnimationQuantity::ImpactCompressionCompliance).map(Self)
    }
    pub fn compression(
        self,
        impact: LandingImpactSpeed,
        minimum: HeightDistance,
        maximum: HeightDistance,
    ) -> AnimationValueResult<HeightDistance> {
        let compression = impact.metres_per_second() * self.0;
        if !compression.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::HeightDistance,
            });
        }
        if minimum > maximum {
            return Err(AnimationValueError::InvertedBounds {
                quantity: AnimationQuantity::HeightDistance,
            });
        }
        HeightDistance::from_arithmetic(compression.clamp(minimum.metres(), maximum.metres()))
    }
}

impl LookDistribution {
    pub fn from_divisor(value: f32) -> AnimationValueResult<Self> {
        let value = checked_nonnegative(value, AnimationQuantity::LookDistribution)?;
        if value == 0.0 {
            return Err(AnimationValueError::NonPositive {
                quantity: AnimationQuantity::LookDistribution,
            });
        }
        Ok(Self(value))
    }
    /// Native signed yaw/pitch distribution immediately before joint clamping.
    pub fn distribute_radians(self, radians: f32) -> f32 {
        radians / self.0
    }
}

impl TryFrom<f32> for ReferenceAcceleration {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_metres_per_second_squared(value)
    }
}

impl TryFrom<f32> for ResponseRate {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_per_second(value)
    }
}

impl TryFrom<f32> for InertialLeanGain {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_gain(value)
    }
}

impl TryFrom<f32> for ImpactCompressionCompliance {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_metres_per_metres_per_second(value)
    }
}

impl TryFrom<f32> for LookDistribution {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_divisor(value)
    }
}

impl MuscleTone {
    pub fn from_gain(value: f32) -> AnimationValueResult<Self> {
        checked_nonnegative(value, AnimationQuantity::MuscleTone).map(Self)
    }
    pub fn from_envelope(value: f32) -> AnimationValueResult<Self> {
        Self::from_gain(value).map_err(|error| {
            if matches!(error, AnimationValueError::NonFinite { .. }) {
                AnimationValueError::Overflow {
                    quantity: AnimationQuantity::MuscleTone,
                }
            } else {
                error
            }
        })
    }
    /// Native multiplier in the complete envelope and PD velocity-kick kernels.
    pub fn gain(self) -> f32 {
        self.0
    }
}
impl AngularStiffness {
    pub fn from_per_second_squared(value: f32) -> AnimationValueResult<Self> {
        checked_nonnegative(value, AnimationQuantity::AngularStiffness).map(Self)
    }
    /// Native coefficient multiplying a radian pose-error vector into rad/s².
    pub fn per_second_squared(self) -> f32 {
        self.0
    }
}
impl TryFrom<f32> for MuscleTone {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_gain(value)
    }
}
impl TryFrom<f32> for AngularStiffness {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_per_second_squared(value)
    }
}
