//! Preserve authored posture-facing ownership across animation projection.

use super::*;

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
    skeleton.advance_posture_transition(skeleton.locomotion_sample_tick.encoded());
    advance_posture_transition_facing(
        transform,
        physics_rotation,
        previous_transition,
        skeleton.posture_transition(),
    );
    skeleton.set_guarded_sprint_locomotion(pace == MovementPace::Sprint);
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
