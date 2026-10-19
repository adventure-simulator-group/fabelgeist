//! Checked phase of the pose buffer's fixed sample interval.

use adventuresim_tactical_core::animation::{
    AnimationElapsed, AnimationQuantity, AnimationValueError, AnimationValueResult, UnitInterval,
};

#[derive(Clone, Copy)]
pub(super) struct PoseInterpolation(UnitInterval);

impl PoseInterpolation {
    pub(super) const ZERO: Self = Self(UnitInterval::ZERO);
    pub(super) const ONE: Self = Self(UnitInterval::ONE);
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
        Self::from_fraction(
            fraction.clamp(UnitInterval::ZERO.fraction(), UnitInterval::ONE.fraction()),
        )
    }
    pub(super) fn fraction(self) -> f32 {
        self.0.fraction()
    }
    fn from_fraction(fraction: f32) -> AnimationValueResult<Self> {
        UnitInterval::from_fraction(fraction)
            .map(Self)
            .map_err(|error| error.for_quantity(AnimationQuantity::PoseInterpolation))
    }
}
