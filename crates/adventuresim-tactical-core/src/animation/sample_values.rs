//! Checked contribution weights and cyclic sample coordinates shared by evaluation.

use super::{
    AnimationQuantity, AnimationValueError, AnimationValueResult, LOCOMOTION_SECOND_CONTACT_PHASE,
    LeadFoot,
};
use serde::{Deserialize, Serialize};

/// Finite cycles, deliberately signed and unwrapped until a periodic operation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct CyclePhase(f32);

/// Finite signed cycle displacement; unlike a coordinate, it never wraps implicitly.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct CycleDelta(f32);

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
    pub const ZERO: Self = Self(0.0);
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
    pub fn checked_advance(self, delta: CycleDelta) -> AnimationValueResult<Self> {
        let cycles = self.0 + delta.0;
        if !cycles.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::CyclePhase,
            });
        }
        Ok(Self(cycles))
    }
    pub fn checked_difference(self, previous: Self) -> AnimationValueResult<CycleDelta> {
        CycleDelta::from_arithmetic(self.0 - previous.0)
    }
    /// Remaining cycles to the canonical bilateral contact, in the existing
    /// subtract-then-modulo order. Subtracting zero or half a cycle from a
    /// finite f32 cannot overflow, so this periodic kernel remains infallible.
    pub fn until_contact(self, foot: LeadFoot) -> CycleDelta {
        let contact = match foot {
            LeadFoot::Left => 0.0,
            LeadFoot::Right => LOCOMOTION_SECOND_CONTACT_PHASE,
        };
        CycleDelta((contact - self.0).rem_euclid(1.0))
    }
}

impl CycleDelta {
    pub const ZERO: Self = Self(0.0);
    pub fn from_cycles(cycles: f32) -> AnimationValueResult<Self> {
        if !cycles.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::CycleDelta,
            });
        }
        Ok(Self(cycles))
    }
    /// Admit the result of a native cycle arithmetic kernel before retention.
    pub fn from_arithmetic(cycles: f32) -> AnimationValueResult<Self> {
        if !cycles.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::CycleDelta,
            });
        }
        Ok(Self(cycles))
    }
    pub fn cycles(self) -> f32 {
        self.0
    }
    pub fn checked_add(self, other: Self) -> AnimationValueResult<Self> {
        Self::from_arithmetic(self.0 + other.0)
    }
    pub fn checked_sub(self, other: Self) -> AnimationValueResult<Self> {
        Self::from_arithmetic(self.0 - other.0)
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
impl TryFrom<f32> for CycleDelta {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_cycles(value)
    }
}
impl From<CycleDelta> for f32 {
    fn from(value: CycleDelta) -> Self {
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
