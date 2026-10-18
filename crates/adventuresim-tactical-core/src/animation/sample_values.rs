//! Checked contribution weights and cyclic sample coordinates shared by evaluation.

use super::{AnimationQuantity, AnimationValueError, AnimationValueResult};
use serde::{Deserialize, Serialize};

/// Finite cycles, deliberately signed and unwrapped until a periodic operation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct CyclePhase(f32);

/// A finite nonnegative contribution. Independent contributions may exceed one.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct BlendWeight(f32);

/// Binary selection of the authored original or mirrored clip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "bool", into = "bool")]
pub enum ClipMirror {
    Original,
    Mirrored,
}

impl CyclePhase {
    pub fn from_cycles(cycles: f32) -> AnimationValueResult<Self> {
        if !cycles.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::CyclePhase,
            });
        }
        Ok(Self(cycles))
    }
    pub fn cycles(self) -> f32 {
        self.0
    }
    pub fn wrapped(self) -> Self {
        Self(self.0.rem_euclid(1.0))
    }
}

impl BlendWeight {
    pub const ZERO: Self = Self(0.0);
    pub const ONE: Self = Self(1.0);
    pub fn from_factor(factor: f32) -> AnimationValueResult<Self> {
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
    pub fn checked_add(self, other: Self) -> AnimationValueResult<Self> {
        let factor = self.0 + other.0;
        if !factor.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::BlendWeight,
            });
        }
        Ok(Self(factor))
    }
    /// Multiplies two admitted contributions without publishing an overflow.
    pub fn checked_scale(self, scale: Self) -> AnimationValueResult<Self> {
        let factor = self.0 * scale.0;
        if !factor.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::BlendWeight,
            });
        }
        Ok(Self(factor))
    }
    pub fn factor(self) -> f32 {
        self.0
    }
    /// There is no authored minimum contribution: every positive factor counts.
    pub fn is_absent(self) -> bool {
        self.0 <= 0.0
    }
}

impl TryFrom<f32> for CyclePhase {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_cycles(value)
    }
}
impl From<CyclePhase> for f32 {
    fn from(value: CyclePhase) -> Self {
        value.cycles()
    }
}
impl TryFrom<f32> for BlendWeight {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_factor(value)
    }
}
impl From<BlendWeight> for f32 {
    fn from(value: BlendWeight) -> Self {
        value.factor()
    }
}
impl From<bool> for ClipMirror {
    fn from(mirrored: bool) -> Self {
        if mirrored {
            Self::Mirrored
        } else {
            Self::Original
        }
    }
}
impl From<ClipMirror> for bool {
    fn from(mirror: ClipMirror) -> Self {
        mirror == ClipMirror::Mirrored
    }
}
