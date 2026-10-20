//! Finite elapsed seconds shared by simulation, authority and presentation.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

pub type ElapsedResult<T> = Result<T, ElapsedError>;

/// Elapsed seconds, including zero for an observation that does not advance.
#[derive(Clone, Copy, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct AnimationElapsed(f32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElapsedError {
    NonFinite,
    Negative,
    Overflow,
    NativeRange,
    NonFiniteContactOffset,
    ContactOffsetOverflow,
}

impl AnimationElapsed {
    pub const ZERO: Self = Self(0.0);
    pub const SECOND: Self = Self(1.0);

    pub const fn from_seconds(seconds: f32) -> ElapsedResult<Self> {
        if !seconds.is_finite() {
            return Err(ElapsedError::NonFinite);
        }
        if seconds < 0.0 {
            return Err(ElapsedError::Negative);
        }
        Ok(Self(seconds))
    }

    pub fn seconds(self) -> f32 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0.0
    }

    pub fn min(self, other: Self) -> Self {
        if self.0 < other.0 { self } else { other }
    }

    pub fn max(self, other: Self) -> Self {
        if self.0 > other.0 { self } else { other }
    }

    /// Bound an elapsed observation by a configured or authored seconds limit.
    pub fn capped_by_seconds(self, maximum: f32) -> ElapsedResult<Self> {
        Ok(self.min(Self::from_seconds(maximum)?))
    }

    /// Remaining duration, floored at zero after the same seconds subtraction.
    pub fn saturating_sub(self, other: Self) -> Self {
        Self((self.0 - other.0).max(0.0))
    }

    /// Separation of two finite, nonnegative elapsed observations. Their
    /// difference cannot exceed the larger operand, so it remains finite.
    pub fn abs_diff(self, other: Self) -> Self {
        Self((self.0 - other.0).abs())
    }

    pub fn checked_scale(self, factor: f32) -> ElapsedResult<Self> {
        let seconds = self.0 * factor;
        if !seconds.is_finite() {
            return Err(ElapsedError::Overflow);
        }
        Self::from_seconds(seconds)
    }

    /// Refine seconds at the native timer/authority boundary without a cap.
    pub fn native_duration(self) -> ElapsedResult<Duration> {
        Duration::try_from_secs_f32(self.0).map_err(|_| ElapsedError::NativeRange)
    }

    pub fn checked_add(self, other: Self) -> ElapsedResult<Self> {
        let seconds = self.0 + other.0;
        if !seconds.is_finite() {
            return Err(ElapsedError::Overflow);
        }
        Ok(Self(seconds))
    }
}

impl From<Duration> for AnimationElapsed {
    /// A native Duration is nonnegative and at most u64 seconds, whose f32
    /// seconds representation is finite. This is Bevy's delta-seconds boundary.
    fn from(duration: Duration) -> Self {
        Self(duration.as_secs_f32())
    }
}

impl TryFrom<f32> for AnimationElapsed {
    type Error = ElapsedError;
    fn try_from(value: f32) -> ElapsedResult<Self> {
        Self::from_seconds(value)
    }
}

impl From<AnimationElapsed> for f32 {
    fn from(elapsed: AnimationElapsed) -> Self {
        elapsed.0
    }
}

impl fmt::Debug for AnimationElapsed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl fmt::Display for ElapsedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NonFinite => "elapsed animation seconds must be finite",
            Self::Negative => "elapsed animation seconds must be nonnegative",
            Self::Overflow => "elapsed animation seconds arithmetic overflowed",
            Self::NonFiniteContactOffset => "contact timing offset seconds must be finite",
            Self::ContactOffsetOverflow => "contact timing offset seconds arithmetic overflowed",
            Self::NativeRange => "elapsed seconds cannot be represented by the native clock",
        })
    }
}
impl std::error::Error for ElapsedError {}
