//! Checked coordinates carried from queued attacks to the continuation spline.

use super::{AnimationQuantity, AnimationValueError, AnimationValueResult, AttackPoseCoordinate};
use serde::{Deserialize, Deserializer, Serialize};

/// Measured follow-ready location on the authored preparation interval.
const FOLLOW_READY_PREPARATION_FRACTION: f32 = 7.0 / 12.0;

/// Finite nonnegative attack-coordinate change over the readiness interval.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
pub struct IncomingAttackTangent(f32);

/// Readiness precedes contact at phase 0.5 and follows the start at zero.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, derive_more::Into)]
#[serde(try_from = "f32", into = "f32")]
pub struct ContinuationReadiness(f32);

/// One observation of the continuation variant of the authoritative attack.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttackContinuation {
    pub start_coordinate: AttackPoseCoordinate,
    pub incoming_tangent: IncomingAttackTangent,
}

impl IncomingAttackTangent {
    pub const ZERO: Self = Self(0.0);

    pub fn from_coordinate_delta(delta: f32) -> AnimationValueResult<Self> {
        let quantity = AnimationQuantity::ContinuationTangent;
        if !delta.is_finite() {
            return Err(AnimationValueError::NonFinite { quantity });
        }
        if delta < 0.0 {
            return Err(AnimationValueError::Negative { quantity });
        }
        // The former span admission normalized negative zero with max(0).
        Ok(Self(delta.max(0.0)))
    }

    pub fn from_arithmetic(delta: f32) -> AnimationValueResult<Self> {
        if !delta.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::ContinuationTangent,
            });
        }
        Self::from_coordinate_delta(delta)
    }

    pub fn coordinate_delta(self) -> f32 {
        self.0
    }
}

impl ContinuationReadiness {
    /// Preserve the measured follow-ready crossover within preparation.
    pub const FOLLOW_READY: Self = Self(0.5 * FOLLOW_READY_PREPARATION_FRACTION);

    pub fn from_phase(phase: f32) -> AnimationValueResult<Self> {
        if !phase.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::ContinuationReadiness,
            });
        }
        if phase <= 0.0 || phase >= 0.5 {
            return Err(AnimationValueError::InvalidContinuationReadiness);
        }
        Ok(Self(phase))
    }

    pub fn phase(self) -> f32 {
        self.0
    }

    /// The attack's preparation occupies half of the complete action phase.
    pub fn preparation_fraction(self) -> f32 {
        self.0 * 2.0
    }
}

impl TryFrom<f32> for IncomingAttackTangent {
    type Error = AnimationValueError;

    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_coordinate_delta(value)
    }
}

impl TryFrom<f32> for ContinuationReadiness {
    type Error = AnimationValueError;

    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_phase(value)
    }
}

/// Preserve finite negative-tangent normalization at the existing decode
/// boundary, while rejecting nonfinite input before it can enter state.
pub(super) fn deserialize_tangent<'de, D>(
    deserializer: D,
) -> Result<IncomingAttackTangent, D::Error>
where
    D: Deserializer<'de>,
{
    let value = f32::deserialize(deserializer)?;
    if !value.is_finite() {
        return Err(serde::de::Error::custom(AnimationValueError::NonFinite {
            quantity: AnimationQuantity::ContinuationTangent,
        }));
    }
    IncomingAttackTangent::from_coordinate_delta(value.max(0.0)).map_err(serde::de::Error::custom)
}
