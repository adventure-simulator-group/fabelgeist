//! Checked phase of the pose buffer's fixed sample interval.

use adventuresim_tactical_core::animation::{
    AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult,
};

#[derive(Clone, Copy)]
pub(super) struct PoseInterpolation(f32);

impl PoseInterpolation {
    pub(super) const ZERO: Self = Self(0.0);
    pub(super) const ONE: Self = Self(1.0);
    pub(super) fn from_sample_clock(
        elapsed: AnimationElapsed,
        interval: AnimationElapsed,
    ) -> AnimationValueResult<Self> {
        // Native seconds cancel only in this fixed-clock ratio kernel.
        let fraction = elapsed.seconds() / interval.seconds();
        if !fraction.is_finite() {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::PoseInterpolation,
            });
        }
        Self::from_fraction(fraction.clamp(0.0, 1.0))
    }
    pub(super) fn fraction(self) -> f32 {
        self.0
    }
    fn from_fraction(fraction: f32) -> AnimationValueResult<Self> {
        if !fraction.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::PoseInterpolation,
            });
        }
        if !(0.0..=1.0).contains(&fraction) {
            return Err(AnimationValueError::OutsideUnitInterval {
                quantity: AnimationQuantity::PoseInterpolation,
            });
        }
        Ok(Self(fraction))
    }
}
