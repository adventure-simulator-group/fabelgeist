//! Checked scalar linear speeds shared by tactical and strategic mechanics.

use crate::unit_interval::{FractionError, UnitInterval};
use serde::{Deserialize, Serialize};

pub type SpeedResult<T> = Result<T, SpeedError>;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct NonnegativeSpeed(f32);

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct SignedSpeed(f32);

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct PositiveSpeed(NonnegativeSpeed);

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SpeedError {
    #[error("linear speed must be finite")]
    NonFinite,
    #[error("linear speed must be nonnegative")]
    Negative,
    #[error("linear speed must be positive")]
    NonPositive,
    #[error("linear speed arithmetic overflowed")]
    Overflow,
    #[error("linear speed interpolation bounds must be strictly increasing")]
    NonIncreasingBounds,
    #[error("invalid speed interpolation fraction: {0}")]
    Fraction(#[from] FractionError),
}

impl NonnegativeSpeed {
    pub const ZERO: Self = Self(0.0);

    pub const fn from_metres_per_second(value: f32) -> SpeedResult<Self> {
        if !value.is_finite() {
            return Err(SpeedError::NonFinite);
        }
        if value < 0.0 {
            return Err(SpeedError::Negative);
        }
        Ok(Self(value))
    }

    pub const fn metres_per_second(self) -> f32 {
        self.0
    }
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    pub fn checked_scale(self, factor: f32) -> SpeedResult<Self> {
        Self::from_metres_per_second(self.0 * factor).map_err(|error| {
            if error == SpeedError::NonFinite {
                SpeedError::Overflow
            } else {
                error
            }
        })
    }

    pub fn checked_lerp(self, other: Self, factor: UnitInterval) -> SpeedResult<Self> {
        let factor = factor.fraction();
        Self::from_metres_per_second(self.0 + (other.0 - self.0) * factor).map_err(|error| {
            if error == SpeedError::NonFinite {
                SpeedError::Overflow
            } else {
                error
            }
        })
    }

    /// Normalize speed within an increasing interval, saturating at its endpoints.
    pub fn clamped_inverse_lerp(self, lower: Self, upper: Self) -> SpeedResult<UnitInterval> {
        if lower >= upper {
            return Err(SpeedError::NonIncreasingBounds);
        }
        if self <= lower {
            return Ok(UnitInterval::ZERO);
        }
        if self >= upper {
            return Ok(UnitInterval::ONE);
        }
        // Metres per second cancel inside this scalar normalization kernel.
        Ok(UnitInterval::from_fraction(
            (self.0 - lower.0) / (upper.0 - lower.0),
        )?)
    }
}

impl SignedSpeed {
    pub const ZERO: Self = Self(0.0);

    pub const fn from_metres_per_second(value: f32) -> SpeedResult<Self> {
        if !value.is_finite() {
            return Err(SpeedError::NonFinite);
        }
        Ok(Self(value))
    }

    pub const fn metres_per_second(self) -> f32 {
        self.0
    }

    pub fn negated(self) -> Self {
        Self(-self.0)
    }
    pub fn magnitude(self) -> NonnegativeSpeed {
        NonnegativeSpeed(self.0.abs())
    }
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    /// Normalize an admitted signed control rate under an existing forward-only policy.
    pub fn nonnegative_part(self) -> NonnegativeSpeed {
        NonnegativeSpeed(self.0.max(0.0))
    }
}

impl PositiveSpeed {
    pub const fn from_metres_per_second(value: f32) -> SpeedResult<Self> {
        match NonnegativeSpeed::from_metres_per_second(value) {
            Ok(speed) if value != 0.0 => Ok(Self(speed)),
            Ok(_) => Err(SpeedError::NonPositive),
            Err(error) => Err(error),
        }
    }

    pub const fn nonnegative(self) -> NonnegativeSpeed {
        self.0
    }
    pub const fn metres_per_second(self) -> f32 {
        self.0.metres_per_second()
    }
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }
}

impl From<NonnegativeSpeed> for SignedSpeed {
    fn from(speed: NonnegativeSpeed) -> Self {
        Self(speed.0)
    }
}
impl From<NonnegativeSpeed> for f32 {
    fn from(speed: NonnegativeSpeed) -> Self {
        speed.0
    }
}
impl From<SignedSpeed> for f32 {
    fn from(speed: SignedSpeed) -> Self {
        speed.0
    }
}
impl From<PositiveSpeed> for f32 {
    fn from(speed: PositiveSpeed) -> Self {
        speed.metres_per_second()
    }
}
impl TryFrom<f32> for NonnegativeSpeed {
    type Error = SpeedError;
    fn try_from(value: f32) -> SpeedResult<Self> {
        Self::from_metres_per_second(value)
    }
}
impl TryFrom<f32> for SignedSpeed {
    type Error = SpeedError;
    fn try_from(value: f32) -> SpeedResult<Self> {
        Self::from_metres_per_second(value)
    }
}
impl TryFrom<f32> for PositiveSpeed {
    type Error = SpeedError;
    fn try_from(value: f32) -> SpeedResult<Self> {
        Self::from_metres_per_second(value)
    }
}
