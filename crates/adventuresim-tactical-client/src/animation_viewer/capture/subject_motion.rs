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
    let jump_charging = frame.scenario.is(ScenarioId::JumpChargeAnticipation)
        && (4..48).contains(&frame.scenario_frame);
    let tick = sequence.simulation_tick;
    let input = SkeletonLocomotionInput::from_controller(
        motion.orientation,
        motion.world_velocity,
        motion.contact,
        motion.elapsed,
        tick,
    )?;
    project_skeleton_locomotion_with_body_rotation(skeleton, input, transform.rotation, None)?;
    skeleton.set_jump_anticipation(JumpAnticipation::from(jump_charging));
    if sequence.warmup_frames == 0 && frame.scenario.transition().is_some() {
        let previous_transition = skeleton.posture_transition();
        skeleton.advance_posture_transition(tick);
        transform.rotation = (transform.rotation
            * dive_landing_facing_delta(previous_transition, skeleton.posture_transition())
            * supine_get_up_counter_yaw_delta(previous_transition, skeleton.posture_transition()))
        .normalize();
    }
    Ok(())
}

/// Admit the authored fixture action at its existing post-root-motion point.
pub(super) fn begin_capture_action(
    skeleton: &mut SkeletonState,
    frame: &PlannedFrame,
    tick: AnimationSampleTick,
    attack_start_frame: Option<usize>,
) {
    let quickstep = is_quickstep_scenario(&frame.scenario);
    let action_starts_now = match frame.action {
        SkeletonAction::Attack => attack_start_frame == Some(frame.scenario_frame),
        SkeletonAction::Dodge if quickstep => frame.scenario_frame == 0,
        SkeletonAction::Dodge | SkeletonAction::Block => skeleton.action_kind() != frame.action,
        SkeletonAction::None => false,
    };
    if action_starts_now {
        let start = tick;
        let contact = start.saturating_add(AnimationSampleSpan::from(
            if frame.action == SkeletonAction::Attack {
                19u64
            } else {
                64u64
            },
        ));
        match frame.action {
            SkeletonAction::Attack => {
                let attack = if frame.scenario.is(ScenarioId::AttackLiveStationarySwing) {
                    AttackSpec::new(AttackAnimation::Swing)
                } else {
                    AttackSpec::new(AttackAnimation::Thrust)
                };
                skeleton
                    .begin_attack(attack, start, contact)
                    .expect("viewer attack transition must be admitted");
            }
            SkeletonAction::Dodge => {
                let spec = if quickstep {
                    DodgeSpec::quickstep(frame.local_direction)
                        .expect("quickstep scenario direction must be non-zero")
                } else {
                    DodgeSpec::default()
                };
                skeleton
                    .begin_dodge(
                        spec,
                        start,
                        if quickstep {
                            start.saturating_add(quickstep_action_ticks().half())
                        } else {
                            contact
                        },
                    )
                    .expect("viewer dodge transition must be admitted");
            }
            SkeletonAction::Block => {
                skeleton
                    .begin_block(BlockSpec::default(), start, contact)
                    .expect("viewer block transition must be admitted");
            }
            SkeletonAction::None => {}
        }
    }
}
