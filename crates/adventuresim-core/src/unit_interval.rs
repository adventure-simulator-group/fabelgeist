//! Checked finite fractions, independent of each producer's clamp policy.

use serde::{Deserialize, Serialize};
use std::fmt;

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
    pub fn from_fraction(value: f32) -> FractionResult<Self> {
        if !value.is_finite() {
            return Err(FractionError::NonFinite);
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(FractionError::OutsideUnitInterval);
        }
        Ok(Self(value))
    }
    pub fn fraction(self) -> f32 {
        self.0
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
