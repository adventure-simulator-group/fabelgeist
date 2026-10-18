//! Quickstep actuator lifecycle and authoritative sample-clock alignment.

use super::*;

impl QuickstepPush {
    pub fn begin(&mut self, start_tick: u64, direction: Vec2, orientation: Quat, origin: Vec3) {
        self.start_tick = start_tick;
        self.direction = direction.normalize_or_zero();
        self.orientation = orientation;
        self.origin = origin;
        self.active = self.direction != Vec2::ZERO;
    }

    pub fn cancel(&mut self) {
        self.active = false;
    }
    /// Project the authoritative observation into this actuator's native action-clock delta.
    pub(super) fn elapsed_sample_ticks(&self, skeleton: Option<&SkeletonState>) -> u64 {
        skeleton
            .map_or(self.start_tick, |skeleton| {
                skeleton.locomotion_sample_tick.encoded()
            })
            .saturating_sub(self.start_tick)
    }
}
