//! Checked scalar height treatment in metres, separate from root-local foot geometry.

use adventuresim_building_generator::spatial_geometry::SignedLength;
use adventuresim_tactical_core::animation::{
    AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult,
};

/// A zero-permitting height amplitude or compression, in metres.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub(crate) struct HeightDistance(f32);

/// A nonnegative absorption or recovery rate, in metres per second.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct HeightSpeed(f32);

impl HeightDistance {
    pub(super) const ZERO: Self = Self(0.0);
    pub(super) fn from_metres(value: f32) -> AnimationValueResult<Self> {
        checked_nonnegative(value, AnimationQuantity::HeightDistance).map(Self)
    }
    pub(crate) fn metres(self) -> f32 {
        self.0
    }
    pub(super) fn advance_towards(
        self,
        target: Self,
        maximum_delta: Self,
    ) -> AnimationValueResult<Self> {
        let value =
            self.0 + (target.0 - self.0).clamp(-maximum_delta.0.max(0.0), maximum_delta.0.max(0.0));
        checked_height_result(value).map(Self)
    }
    pub(super) fn difference(self, other: Self) -> AnimationValueResult<SignedLength> {
        offset_result(self.0 - other.0)
    }
}
impl HeightSpeed {
    pub(super) const ZERO: Self = Self(0.0);
    pub(super) fn from_metres_per_second(value: f32) -> AnimationValueResult<Self> {
        checked_nonnegative(value, AnimationQuantity::HeightSpeed).map(Self)
    }
    pub(super) fn recovery(
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
    pub(super) fn over(self, elapsed: AnimationElapsed) -> AnimationValueResult<HeightDistance> {
        checked_height_result(self.0 * elapsed.seconds()).map(HeightDistance)
    }
}

/// Advance a signed retained offset in its metre frame, rejecting intermediate overflow.
pub(super) fn advance_offset(
    current: SignedLength,
    maximum_delta: HeightDistance,
) -> AnimationValueResult<SignedLength> {
    let difference = -current.metres();
    offset_result(
        current.metres() + difference.clamp(-maximum_delta.0.max(0.0), maximum_delta.0.max(0.0)),
    )
}
pub(super) fn subtract_offsets(
    left: SignedLength,
    right: SignedLength,
) -> AnimationValueResult<SignedLength> {
    offset_result(left.metres() - right.metres())
}
pub(super) fn add_offsets(
    left: SignedLength,
    right: SignedLength,
) -> AnimationValueResult<SignedLength> {
    offset_result(left.metres() + right.metres())
}
fn offset_result(value: f32) -> AnimationValueResult<SignedLength> {
    SignedLength::from_metres(value).map_err(|_| AnimationValueError::Overflow {
        quantity: AnimationQuantity::HeightOffset,
    })
}
fn checked_height_result(value: f32) -> AnimationValueResult<f32> {
    if !value.is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::HeightDistance,
        });
    }
    checked_nonnegative(value, AnimationQuantity::HeightDistance)
}
fn checked_nonnegative(value: f32, quantity: AnimationQuantity) -> AnimationValueResult<f32> {
    if !value.is_finite() {
        return Err(AnimationValueError::NonFinite { quantity });
    }
    if value < 0.0 {
        return Err(AnimationValueError::Negative { quantity });
    }
    Ok(value)
}
