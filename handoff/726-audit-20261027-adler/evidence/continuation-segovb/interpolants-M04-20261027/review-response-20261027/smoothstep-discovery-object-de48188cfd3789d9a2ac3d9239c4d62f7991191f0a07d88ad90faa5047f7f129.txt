//! Checked finite fractions, independent of each producer's clamp policy.

use serde::{Deserialize, Serialize};
use std::{fmt, ops::Mul};

pub type FractionResult<T> = Result<T, FractionError>;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct UnitInterval(f32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FractionError {
    NonFinite,
    OutsideUnitInterval,
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
    /// Evaluate the cubic on a half interval, then reflect its upper half.
    /// This keeps the result in the unit interval without an output clamp.
    pub fn smoothstep(self) -> Self {
        let reflected = self.0 > 0.5;
        let value = if reflected { self.complement() } else { self };
        let value = Self(value.0 * value.0 * (3.0 - 2.0 * value.0));
        if reflected { value.complement() } else { value }
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
impl fmt::Display for FractionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NonFinite => "unit fraction must be finite",
            Self::OutsideUnitInterval => "unit fraction must lie in [0, 1]",
        })
    }
}
impl std::error::Error for FractionError {}
