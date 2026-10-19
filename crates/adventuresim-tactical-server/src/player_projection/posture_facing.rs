//! Preserve authored posture-facing ownership across animation projection.

use super::*;
use adventuresim_tactical_core::animation::AnimationSampleTick;

impl AttackFacing {
    pub(super) fn advance_rotation(
        &self,
        transform: &Transform,
        tick: AnimationSampleTick,
        elapsed: AnimationElapsed,
    ) -> Quat {
        let remaining_seconds = self.contact_tick.saturating_sub(tick).samples() as f32
            / locomotion_sample_rate().hertz();
        let desired_forward = self.target_position - transform.translation;
        let turn_speed = body_turn_speed_for_deadline(
            transform.rotation,
            desired_forward,
            remaining_seconds,
            elapsed.seconds(),
        );
        advance_body_facing_toward(
            transform.rotation,
            desired_forward,
            elapsed.seconds(),
            turn_speed,
        )
    }
}

pub(super) fn posture_transition_locks_body_facing(skeleton: &SkeletonState) -> bool {
    // Every authored posture transition encodes its own direction relative to
    // the current root. Turning the root toward residual/controller velocity
    // double-rotates directional dives and can reverse a get-up mid-pose.
    skeleton.is_posture_transitioning()
}

pub(super) fn advance_projected_posture(
    skeleton: &mut SkeletonState,
    transform: &mut Transform,
    physics_rotation: &mut Rotation,
    pace: MovementPace,
) {
    // Successful projection installed this same authoritative sample tick.
    let previous_transition = skeleton.posture_transition();
    skeleton.advance_posture_transition(skeleton.locomotion_sample_tick);
    advance_posture_transition_facing(
        transform,
        physics_rotation,
        previous_transition,
        skeleton.posture_transition(),
    );
    skeleton.set_guarded_locomotion_mode(GuardedLocomotionMode::from(pace));
}

pub(super) fn advance_posture_transition_facing(
    transform: &mut Transform,
    physics_rotation: &mut Rotation,
    previous_transition: Option<PostureTransitionState>,
    current_transition: Option<PostureTransitionState>,
) {
    // Directional dives transfer the downed contact pose's yaw to the root
    // during landing. A backward dive's half-turn is spread across contact
    // recovery; the supine get-up applies the inverse convention change.
    let rotation = (transform.rotation
        * dive_landing_facing_delta(previous_transition, current_transition)
        * supine_get_up_counter_yaw_delta(previous_transition, current_transition))
    .normalize();
    transform.rotation = rotation;
    physics_rotation.0 = rotation;
}

pub(super) fn advance_downed_facing_for_camera(
    skeleton: &mut SkeletonState,
    facing: CameraFacingIntent,
    target: f32,
    maximum_step: f32,
) {
    if facing == CameraFacingIntent::Aim {
        skeleton.advance_downed_facing(target, true, maximum_step);
    } else if skeleton.downed_facing().is_some() {
        // Aim release resolves an already-active interpolation to its nearest
        // stable contact. A free camera cannot seed a new prone/supine roll.
        skeleton.advance_downed_facing(target, false, 1.0);
    }
}
