//! Quickstep actuator lifecycle and authoritative sample-clock alignment.

use super::*;
use crate::animation::AnimationSampleSpan;

impl QuickstepPush {
    pub fn begin(
        &mut self,
        start_tick: crate::animation::AnimationSampleTick,
        direction: Vec2,
        orientation: Quat,
        origin: Vec3,
    ) {
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
            .map_or(self.start_tick, |skeleton| skeleton.locomotion_sample_tick)
            .saturating_sub(self.start_tick)
            .samples()
    }
}

/// Converts the configured complete quickstep duration into the midpoint tick
/// expected by the shared symmetric dodge timeline. Keeping this conversion
/// in one place prevents a contact duration from accidentally being doubled.
pub fn quickstep_action_contact_ticks(duration_seconds: f32) -> AnimationSampleSpan {
    let total_ticks = (duration_seconds.max(2.0 / locomotion_sample_hz()) * locomotion_sample_hz())
        .round()
        .max(2.0) as u64;
    AnimationSampleSpan::from((total_ticks / 2).max(1))
}
