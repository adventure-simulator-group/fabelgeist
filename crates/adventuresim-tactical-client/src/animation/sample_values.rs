//! Checked values carried by resolved plans before numerical sampling.

use adventuresim_tactical_core::animation::{AnimationQuantity, AnimationValueError, CyclePhase};

pub(super) use adventuresim_tactical_core::animation::AnimationValueResult;

/// Existing coalescing resolution for samples from the same clip/layer.
const SAMPLE_COHERENCE_SECONDS: f32 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub(super) struct ClipDuration(f32);

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub(super) struct ClipTime(f32);

impl ClipDuration {
    pub(super) fn from_seconds(seconds: f32) -> AnimationValueResult<Self> {
        if !seconds.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::ClipDuration,
            });
        }
        if seconds < 0.0 {
            return Err(AnimationValueError::Negative {
                quantity: AnimationQuantity::ClipDuration,
            });
        }
        Ok(Self(seconds))
    }
    pub(super) fn seconds(self) -> f32 {
        self.0
    }
    pub(super) fn at_phase(self, phase: CyclePhase) -> AnimationValueResult<ClipTime> {
        let seconds = self.seconds() * phase.cycles();
        if !seconds.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::ClipTime,
            });
        }
        Ok(ClipTime(seconds))
    }
}

impl ClipTime {
    pub(super) const ZERO: Self = Self(0.0);
    pub(super) fn from_seconds(seconds: f32) -> AnimationValueResult<Self> {
        if !seconds.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::ClipTime,
            });
        }
        Ok(Self(seconds))
    }
    pub(super) fn seconds(self) -> f32 {
        self.0
    }
    pub(super) fn same_sample(self, other: Self) -> bool {
        (self.0 - other.0).abs() < SAMPLE_COHERENCE_SECONDS
    }
}

impl TryFrom<f32> for ClipDuration {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_seconds(value)
    }
}
impl From<ClipDuration> for f32 {
    fn from(value: ClipDuration) -> Self {
        value.seconds()
    }
}

impl TryFrom<f32> for ClipTime {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_seconds(value)
    }
}
impl From<ClipTime> for f32 {
    fn from(value: ClipTime) -> Self {
        value.seconds()
    }
}

/// Admission for the still-primitive noncyclic sampling channels. Their complete
/// quantity migration is separate; retain the existing finite-value rejection.
pub(super) fn finite_sample_value(value: f32) -> AnimationValueResult<f32> {
    if !value.is_finite() {
        return Err(AnimationValueError::NonFinite {
            quantity: AnimationQuantity::CyclePhase,
        });
    }
    Ok(value)
}
