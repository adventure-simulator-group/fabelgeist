//! Preserve authored posture-facing ownership across animation projection.

use super::*;
use adventuresim_tactical_core::animation::{
    AnimationSampleTick, DownedFacingStep, SignedHalfTurns,
};

/// One authoritative facing observation, before locomotion projection.
/// This borrows the same immutable configuration snapshot for the whole solve.
pub(super) struct AuthoritativeFacing<'a> {
    pub(super) controller_orientation: Quat,
    pub(super) world_velocity: Vec3,
    pub(super) camera: CameraFacingIntent,
    pub(super) attack: Option<&'a AttackFacing>,
    pub(super) tick: AnimationSampleTick,
    pub(super) elapsed: AnimationElapsed,
    pub(super) config: &'a TacticalCombatConfig,
}

impl AuthoritativeFacing<'_> {
    pub(super) fn advance(
        self,
        skeleton: &mut SkeletonState,
        transform: &mut Transform,
    ) -> AnimationValueResult<()> {
        let posture_transitioning = posture_transition_locks_body_facing(skeleton);
        if posture_transitioning {
            // Authored transitions own their direction relative to a fixed
            // root until a roll or get-up has reached its endpoint.
            skeleton.set_downed_motion(DownedMotion::Resting);
        } else if matches!(skeleton.body(), BodyState::Prone | BodyState::Supine)
            && !skeleton.is_posture_transitioning()
        {
            let target = downed_camera_roll_target(transform.rotation, self.controller_orientation);
            if self.camera == CameraFacingIntent::DownedAlign {
                let next = advance_downed_body_facing_with_speed(
                    transform.rotation,
                    self.controller_orientation,
                    self.elapsed,
                    self.config.presentation.downed_turn_radians_per_second,
                )?;
                skeleton.set_downed_motion(if transform.rotation.angle_between(next) > 1.0e-5 {
                    DownedMotion::Turning
                } else {
                    DownedMotion::Resting
                });
                transform.rotation = next;
            } else {
                skeleton.set_downed_motion(DownedMotion::Resting);
                advance_downed_facing_for_camera(
                    skeleton,
                    self.camera,
                    target,
                    DownedFacingStep::for_frame(
                        self.elapsed,
                        self.config.movement.maneuvers.get_up_seconds,
                    ),
                )?;
            }
        } else if skeleton.body() != BodyState::Ragdolled {
            skeleton.set_downed_motion(DownedMotion::Resting);
            let next_rotation = if let Some(attack_facing) = self.attack {
                attack_facing.advance_rotation(transform, self.tick, self.elapsed)
            } else {
                advance_body_facing_with_turn(
                    transform.rotation,
                    self.controller_orientation,
                    self.world_velocity,
                    skeleton.action_kind(),
                    skeleton.weapon_guard(),
                    self.elapsed,
                    BodyFacingTurn::configured_half_turn(
                        self.config.presentation.body_turn_seconds_per_half_turn,
                    ),
                )
            };
            transform.rotation = next_rotation?;
        }
        Ok(())
    }
}

impl AttackFacing {
    pub(super) fn advance_rotation(
        &self,
        transform: &Transform,
        tick: AnimationSampleTick,
        elapsed: AnimationElapsed,
    ) -> AnimationValueResult<Quat> {
        let remaining =
            locomotion_sample_rate().elapsed_for_span(self.contact_tick.saturating_sub(tick))?;
        let desired_forward = self.target_position - transform.translation;
        let turn =
            BodyFacingTurn::for_deadline(transform.rotation, desired_forward, remaining, elapsed)?;
        turn.advance(transform.rotation, desired_forward, elapsed)
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
    target: SignedHalfTurns,
    maximum_step: DownedFacingStep,
) -> AnimationValueResult<bool> {
    if facing == CameraFacingIntent::Aim {
        skeleton.advance_downed_facing(target, DownedFacingAim::Held, maximum_step)
    } else if skeleton.downed_facing().is_some() {
        // Aim release resolves an already-active interpolation to its nearest
        // stable contact. A free camera cannot seed a new prone/supine roll.
        skeleton.advance_downed_facing(
            target,
            DownedFacingAim::Released,
            DownedFacingStep::SETTLE_CONTACT,
        )
    } else {
        Ok(false)
    }
}
