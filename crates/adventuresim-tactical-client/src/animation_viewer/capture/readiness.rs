//! Capture warmup and authored-motion readiness, without advancing the frame early.

use super::*;

impl CaptureSequence {
    pub(super) fn ready_for_frame(
        &mut self,
        frame: &PlannedFrame,
        animation_runtime: &AnimationRuntime,
        playback: Option<&AnimationPlayback>,
    ) -> bool {
        if self.warmup_frames > 0 {
            self.warmup_frames -= 1;
            return false;
        }
        if frame.scenario_frame == 0
            && let Some(motion) = required_motion_for_scenario(frame.scenario.label())
        {
            if !animation_runtime.motion_is_processed(&crate::animation::MotionId::from(motion))
                || !playback.is_some_and(AnimationPlayback::presentation_is_settled)
            {
                self.motion_ready_frames = 0;
                return false;
            }
            self.motion_ready_frames += 1;
            if self.motion_ready_frames < 2 {
                return false;
            }
        }
        true
    }
}
