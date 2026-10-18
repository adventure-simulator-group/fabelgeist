//! Capture controller admission and ordered post-projection state updates.

use super::*;
use adventuresim_tactical_core::animation::AnimationValueResult;

pub(super) struct CaptureMotion {
    pub(super) orientation: Quat,
    pub(super) world_velocity: Vec3,
    pub(super) contact: GroundContact,
    pub(super) elapsed: AnimationElapsed,
}

pub(super) fn project_capture_motion(
    skeleton: &mut SkeletonState,
    transform: &mut Transform,
    frame: &PlannedFrame,
    motion: CaptureMotion,
    sequence: &CaptureSequence,
) -> AnimationValueResult<()> {
    let jump_charging = frame.scenario.label() == "jump-charge-anticipation"
        && (4..48).contains(&frame.scenario_frame);
    let tick = AnimationSampleTick::from(sequence.simulation_tick);
    let input = SkeletonLocomotionInput::from_controller(
        motion.orientation,
        motion.world_velocity,
        motion.contact,
        motion.elapsed,
        tick,
    )?;
    project_skeleton_locomotion_with_body_rotation(skeleton, input, transform.rotation, None)?;
    skeleton.set_jump_anticipation(jump_charging);
    if sequence.warmup_frames == 0 && transition_for_scenario(frame.scenario.label()).is_some() {
        let previous_transition = skeleton.posture_transition();
        skeleton.advance_posture_transition(tick.encoded());
        transform.rotation = (transform.rotation
            * dive_landing_facing_delta(previous_transition, skeleton.posture_transition())
            * supine_get_up_counter_yaw_delta(previous_transition, skeleton.posture_transition()))
        .normalize();
    }
    Ok(())
}
