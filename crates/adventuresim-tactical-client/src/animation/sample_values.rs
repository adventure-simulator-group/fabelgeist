//! Checked values carried by resolved plans before numerical sampling.

use adventuresim_tactical_core::animation::{AnimationQuantity, AnimationValueError};

pub(super) use adventuresim_tactical_core::animation::AnimationValueResult;

/// Existing coalescing resolution for samples from the same clip/layer.
const SAMPLE_COHERENCE_SECONDS: f32 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub(super) struct ClipDuration(f32);

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub(super) struct ClipTime(f32);

/// Cycles, deliberately unwrapped until the consuming periodic operation.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub(super) struct CyclePhase(f32);

/// A finite nonnegative contribution. Independent contributions may sum above one.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub(super) struct BlendWeight(f32);

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

impl CyclePhase {
    pub(super) fn from_cycles(cycles: f32) -> AnimationValueResult<Self> {
        if !cycles.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::CyclePhase,
            });
        }
        Ok(Self(cycles))
    }
    pub(super) fn cycles(self) -> f32 {
        self.0
    }
    pub(super) fn wrapped(self) -> Self {
        Self(self.0.rem_euclid(1.0))
    }
}

impl BlendWeight {
    pub(super) const ZERO: Self = Self(0.0);
    pub(super) const ONE: Self = Self(1.0);
    pub(super) fn from_factor(factor: f32) -> AnimationValueResult<Self> {
        if !factor.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::BlendWeight,
            });
        }
        if factor < 0.0 {
            return Err(AnimationValueError::Negative {
                quantity: AnimationQuantity::BlendWeight,
            });
        }
        Ok(Self(factor))
    }
    pub(super) fn checked_add(self, other: Self) -> AnimationValueResult<Self> {
        let factor = self.0 + other.0;
        if !factor.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::BlendWeight,
            });
        }
        Ok(Self(factor))
    }
    pub(super) fn factor(self) -> f32 {
        self.0
    }
    /// There is no authored minimum contribution: every positive factor counts.
    pub(super) fn is_absent(self) -> bool {
        self.0 <= 0.0
    }
}

impl<'de> serde::Deserialize<'de> for ClipDuration {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_seconds(<f32 as serde::Deserialize>::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

impl<'de> serde::Deserialize<'de> for ClipTime {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_seconds(<f32 as serde::Deserialize>::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

impl<'de> serde::Deserialize<'de> for CyclePhase {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_cycles(<f32 as serde::Deserialize>::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

impl<'de> serde::Deserialize<'de> for BlendWeight {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_factor(<f32 as serde::Deserialize>::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}
