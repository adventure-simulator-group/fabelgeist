//! Height treatment roles over shared finite nonnegative measurements.

use super::{
    AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult,
    NonnegativeLength, NonnegativeSpeed,
};
use adventuresim_building_generator::spatial_geometry::SignedLength;
use serde::{Deserialize, Serialize};

/// A zero-permitting height amplitude or compression, in metres.
#[derive(
    Clone,
    Copy,
    derive_more::Debug,
    Default,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    derive_more::Into,
)]
#[serde(try_from = "f32", into = "f32")]
#[debug("HeightDistance({:?})", self.0.metres())]
#[into(f32)]
pub struct HeightDistance(NonnegativeLength);

/// A nonnegative absorption or recovery rate, in metres per second.
#[derive(
    Clone,
    Copy,
    derive_more::Debug,
    Default,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    derive_more::Into,
)]
#[serde(try_from = "f32", into = "f32")]
#[debug("HeightSpeed({:?})", self.0.metres_per_second())]
#[into(f32)]
pub struct HeightSpeed(NonnegativeSpeed);

impl HeightDistance {
    pub const ZERO: Self = Self(NonnegativeLength::ZERO);
    pub fn from_metres(value: f32) -> AnimationValueResult<Self> {
        NonnegativeLength::from_metres(value)
            .map(Self)
            .map_err(|error| error.for_quantity(AnimationQuantity::HeightDistance))
    }
    pub fn metres(self) -> f32 {
        self.0.metres()
    }
    pub fn advance_towards(self, target: Self, maximum_delta: Self) -> AnimationValueResult<Self> {
        let value = self.metres()
            + (target.metres() - self.metres()).clamp(
                -maximum_delta.metres().max(0.0),
                maximum_delta.metres().max(0.0),
            );
        Self::from_arithmetic(value)
    }
    pub fn from_arithmetic(value: f32) -> AnimationValueResult<Self> {
        if !value.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::HeightDistance,
            });
        }
        Self::from_metres(value)
    }
    pub fn difference(self, other: Self) -> AnimationValueResult<SignedLength> {
        SignedLength::from_metres(self.metres() - other.metres()).map_err(|_| {
            AnimationValueError::Overflow {
                quantity: AnimationQuantity::HeightOffset,
            }
        })
    }
}
impl HeightSpeed {
    pub const ZERO: Self = Self(NonnegativeSpeed::ZERO);
    pub fn from_metres_per_second(value: f32) -> AnimationValueResult<Self> {
        NonnegativeSpeed::from_metres_per_second(value)
            .map(Self)
            .map_err(|error| error.for_quantity(AnimationQuantity::HeightSpeed))
    }
    pub fn metres_per_second(self) -> f32 {
        self.0.metres_per_second()
    }
    pub fn max(self, other: Self) -> Self {
        if self.0 >= other.0 { self } else { other }
    }
    pub fn recovery(
        compression: HeightDistance,
        elapsed: AnimationElapsed,
    ) -> AnimationValueResult<Self> {
        let rate = compression.metres() / elapsed.seconds();
        if !rate.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::HeightSpeed,
            });
        }
        Self::from_metres_per_second(rate)
    }
    pub fn over(self, elapsed: AnimationElapsed) -> AnimationValueResult<HeightDistance> {
        HeightDistance::from_arithmetic(self.0.metres_per_second() * elapsed.seconds())
    }
}

impl TryFrom<f32> for HeightDistance {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_metres(value)
    }
}
impl TryFrom<f32> for HeightSpeed {
    type Error = AnimationValueError;
    fn try_from(value: f32) -> AnimationValueResult<Self> {
        Self::from_metres_per_second(value)
    }
}
