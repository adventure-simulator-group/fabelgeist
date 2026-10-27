//! Checked scalar linear speeds shared by tactical and strategic mechanics.

use crate::unit_interval::{FractionError, UnitInterval};
use fabelgeist_math::units::Ratio;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

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
    #[error("linear speed divisor must be nonzero")]
    ZeroDivisor,
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
        Self::from_arithmetic(self.0 * factor)
    }

    pub fn checked_lerp(self, other: Self, fraction: UnitInterval) -> SpeedResult<Self> {
        let difference = other - self;
        let scaled_difference = difference * fraction;
        let interpolated_speed = (self + scaled_difference)?;
        Ok(Self(interpolated_speed.0))
    }

    /// Normalize speed within an increasing interval, saturating at its endpoints.
    pub fn clamped_inverse_lerp(
        self,
        lower: impl Into<Self>,
        upper: impl Into<Self>,
    ) -> SpeedResult<UnitInterval> {
        let lower = lower.into();
        let upper = upper.into();
        if lower >= upper {
            return Err(SpeedError::NonIncreasingBounds);
        }
        if self <= lower {
            return Ok(UnitInterval::ZERO);
        }
        if self >= upper {
            return Ok(UnitInterval::ONE);
        }
        let offset = self - lower;
        let span = upper - lower;
        let quotient = (offset / span)?;
        Ok(UnitInterval::try_from(quotient)?)
    }

    fn from_arithmetic(value: f32) -> SpeedResult<Self> {
        Self::from_metres_per_second(value).map_err(|error| {
            if error == SpeedError::NonFinite {
                SpeedError::Overflow
            } else {
                error
            }
        })
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

    fn from_arithmetic(value: f32) -> SpeedResult<Self> {
        if !value.is_finite() {
            return Err(SpeedError::Overflow);
        }
        Ok(Self(value))
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

/// Adding a signed offset permits negative results and rejects overflow.
impl Add<SignedSpeed> for NonnegativeSpeed {
    type Output = SpeedResult<SignedSpeed>;

    fn add(self, offset: SignedSpeed) -> Self::Output {
        SignedSpeed::from_arithmetic(self.0 + offset.0)
    }
}

/// Two finite nonnegative speeds differ by at most the largest finite magnitude.
impl Sub for NonnegativeSpeed {
    type Output = SignedSpeed;

    fn sub(self, other: Self) -> Self::Output {
        SignedSpeed(self.0 - other.0)
    }
}

impl Mul<UnitInterval> for SignedSpeed {
    type Output = Self;

    fn mul(self, fraction: UnitInterval) -> Self {
        Self(self.0 * fraction.fraction())
    }
}

/// Speed units cancel; signed ratios and ratios above one remain valid.
impl Div for SignedSpeed {
    type Output = SpeedResult<Ratio>;

    fn div(self, divisor: Self) -> Self::Output {
        if divisor.0 == 0.0 {
            return Err(SpeedError::ZeroDivisor);
        }
        let quotient = self.0 / divisor.0;
        if !quotient.is_finite() {
            return Err(SpeedError::Overflow);
        }
        Ok(Ratio::from_ratio(quotient))
    }
}

impl From<PositiveSpeed> for NonnegativeSpeed {
    fn from(speed: PositiveSpeed) -> Self {
        speed.0
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
