//! Deduplicate retained IK observations without changing fixed-tick advance semantics.

use super::*;

impl LegIkMemory {
    pub(super) fn sample_elapsed(
        &mut self,
        clock: &ProceduralAnimationClock,
        time: &Time,
    ) -> Option<AnimationElapsed> {
        match clock.fixed_step() {
            Some(step) if self.evaluation_tick == Some(step.tick) => None,
            Some(step) => {
                self.evaluation_tick = Some(step.tick);
                Some(step.delta_seconds)
            }
            None => {
                let elapsed = AnimationElapsed::from(time.delta());
                (!elapsed.is_zero()).then_some(elapsed)
            }
        }
    }
}
