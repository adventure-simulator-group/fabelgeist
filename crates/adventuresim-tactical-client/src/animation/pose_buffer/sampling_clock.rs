//! Fixed sample-clock advancement between the pose buffer's solved targets.

use super::*;
use adventuresim_tactical_core::animation::PositiveAnimationDuration;

impl PoseBufferRig {
    pub(super) fn advance_buffered_sample(
        &mut self,
        target: RigJointTable<LocalPose>,
        interval: PositiveAnimationDuration,
    ) -> AnimationValueResult<()> {
        let due = samples_due(self.sample_accumulator, interval);
        let elapsed = interval.sample_span(due)?;
        let remainder = self.sample_accumulator.saturating_sub(elapsed);
        let fraction = PoseInterpolation::from_sample_clock(remainder, interval)?;
        if due > 0 {
            self.previous = self.next.clone();
            self.next = target;
            for joint in self.next.ordinals() {
                let (linear, angular) =
                    local_pose_velocity(self.previous[joint], self.next[joint], elapsed);
                self.target_linear_velocities[joint] = linear;
                self.target_angular_velocities[joint] = angular;
            }
            self.sample_accumulator = remainder;
        }
        self.interpolation_alpha = fraction;
        Ok(())
    }
}

pub(super) fn samples_due(
    accumulator: AnimationElapsed,
    interval: PositiveAnimationDuration,
) -> u32 {
    (accumulator.seconds().max(0.0) / interval.seconds()).floor() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn large_frame_gaps_are_bounded_but_consume_sampler_debt() {
        assert_eq!(
            samples_due(AnimationElapsed::ZERO, pose_sample_interval()),
            0
        );
        assert_eq!(
            samples_due(
                AnimationElapsed::from_seconds(pose_sample_interval().seconds() * 3.2).unwrap(),
                pose_sample_interval()
            ),
            3
        );
        assert_eq!(
            samples_due(
                AnimationElapsed::from_seconds(0.1).unwrap(),
                pose_sample_interval()
            ),
            3
        );
    }
}
