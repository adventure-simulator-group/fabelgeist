//! Checked finite fractions, independent of each producer's clamp policy.

use fabelgeist_math::units::Ratio;
use serde::{Deserialize, Serialize};
use std::ops::Mul;

pub type FractionResult<T> = Result<T, FractionError>;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct UnitInterval(f32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum FractionError {
    #[error("unit fraction must be finite")]
    NonFinite,
    #[error("unit fraction must lie in [0, 1]")]
    OutsideUnitInterval,
    #[error("unit fraction interpolation bounds must be strictly increasing")]
    NonIncreasingBounds,
}

impl UnitInterval {
    pub const ZERO: Self = Self(0.0);
    pub const ONE: Self = Self(1.0);
    pub const HALF: Self = Self(0.5);
    pub fn from_fraction(value: f32) -> FractionResult<Self> {
        if !value.is_finite() {
            return Err(FractionError::NonFinite);
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(FractionError::OutsideUnitInterval);
        }
        Ok(Self(value))
    }
    /// Selecting a confidence preserves admission, including native signed-zero semantics.
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
    pub fn complement(self) -> Self {
        Self(1.0 - self.0)
    }
    /// Subtract another fraction, saturating at zero when it is larger.
    pub fn saturating_sub(self, other: Self) -> Self {
        Self((self.0 - other.0).max(0.0))
    }
    /// Interpolate admitted endpoints and check the computed fraction.
    pub fn checked_lerp(self, other: Self, fraction: Self) -> FractionResult<Self> {
        Self::from_fraction(self.0 + (other.0 - self.0) * fraction.0)
    }
    /// Normalize within an increasing interval, saturating at its endpoints.
    pub fn clamped_inverse_lerp(self, lower: Self, upper: Self) -> FractionResult<Self> {
        if lower >= upper {
            return Err(FractionError::NonIncreasingBounds);
        }
        if self <= lower {
            return Ok(Self::ZERO);
        }
        if self >= upper {
            return Ok(Self::ONE);
        }
        Self::from_fraction((self.0 - lower.0) / (upper.0 - lower.0))
    }
    /// Evaluate cubic smoothstep and admit the resulting fraction.
    pub fn smoothstep(self) -> FractionResult<Self> {
        let fraction = self.0;
        Self::from_fraction(fraction * fraction * (3.0 - 2.0 * fraction))
    }
    pub fn fraction(self) -> f32 {
        self.0
    }
}
impl Mul for UnitInterval {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self(self.0 * other.0)
    }
}
impl From<UnitInterval> for f32 {
    fn from(value: UnitInterval) -> Self {
        value.0
    }
}
impl TryFrom<f32> for UnitInterval {
    type Error = FractionError;
    fn try_from(value: f32) -> FractionResult<Self> {
        Self::from_fraction(value)
    }
}
impl TryFrom<Ratio> for UnitInterval {
    type Error = FractionError;

    fn try_from(value: Ratio) -> FractionResult<Self> {
        Self::from_fraction(value.ratio())
    }
}
