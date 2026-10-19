//! Bounded locomotion body response applied after authored pose evaluation.

use super::*;
pub(in crate::animation::procedural) use target::body_response_target;
use target::deceleration_lean_scale;

mod target;

impl LocomotionBodyResponseState {
    fn advance_for_presentation(
        mut self,
        skeleton: &SkeletonState,
        owner_transform: &Transform,
        procedural_clock: &ProceduralAnimationClock,
        time: &Time,
    ) -> AnimationValueResult<Self> {
        let tick = skeleton.locomotion_sample_tick;
        let tick_delta = presentation_tick_delta(self.last_tick, tick);
        let delta_seconds = match procedural_clock.fixed_step() {
            Some(FixedAnimationStep {
                tick: fixed_tick, ..
            }) if self.last_tick == Some(fixed_tick) => AnimationElapsed::ZERO,
            Some(FixedAnimationStep { delta_seconds, .. }) => delta_seconds,
            None => AnimationElapsed::from(time.delta())
                .min(body_response_tuning().maximum_frame_seconds),
        };
        let inverse_body_rotation = owner_transform.rotation.inverse();
        let body_velocity = BodyLocalVelocity::from_metres_per_second(
            inverse_body_rotation * skeleton.world_velocity,
        )?;
        let body_acceleration = BodyLocalAcceleration::from_metres_per_second_squared(
            inverse_body_rotation * skeleton.world_acceleration,
        )?;
        let discontinuous = tick_delta.is_none();
        let source_changed = tick_delta.is_some_and(|delta| delta > AnimationSampleSpan::ZERO);
        if skeleton.is_posture_transitioning()
            || skeleton.action_kind() != SkeletonAction::None
            || !skeleton.is_grounded()
            || skeleton.weapon_guard() == WeaponGuardState::Raised
            || discontinuous
        {
            self.pitch_radians = BodyAngle::ZERO;
            self.roll_radians = BodyAngle::ZERO;
            self.angular_velocity_radians_per_second = BodyAngularVelocity::ZERO;
            self.target_pitch_radians = BodyAngle::ZERO;
            self.target_roll_radians = BodyAngle::ZERO;
            self.smoothed_body_acceleration = BodyLocalAcceleration::ZERO;
        } else {
            // Authoritative acceleration is a discrete finite difference and
            // remains unchanged between replicated samples. Filter that held
            // value in render time so packet delivery cannot create an
            // inertial impulse. Releasing the impulse more slowly also avoids
            // a second head snap when the motor reaches its target speed.
            self.smoothed_body_acceleration = smooth_acceleration(
                self.smoothed_body_acceleration,
                body_acceleration,
                delta_seconds,
            )?;
            let braking_scale = deceleration_lean_scale(
                self.last_body_velocity,
                body_velocity,
                self.smoothed_body_acceleration,
            )?;
            let combined = body_response_target(
                body_velocity,
                self.smoothed_body_acceleration,
                braking_scale,
            )?;
            self.target_pitch_radians = combined.pitch;
            self.target_roll_radians = combined.roll;
            let advanced = advance_body_response(
                BodyResponseAngles {
                    pitch: self.pitch_radians,
                    roll: self.roll_radians,
                },
                self.angular_velocity_radians_per_second,
                combined,
                delta_seconds,
            )?;
            self.pitch_radians = advanced.angles.pitch;
            self.roll_radians = advanced.angles.roll;
            self.angular_velocity_radians_per_second = advanced.angular_velocity;
        }
        if self.last_tick.is_none() || discontinuous || source_changed {
            self.last_body_velocity = body_velocity;
        }
        self.last_tick = Some(tick);
        self.last_posture = Some(skeleton.posture());
        self.last_action = Some(skeleton.action_kind());
        self.last_ground_contact = Some(GroundContact::from(skeleton));
        Ok(self)
    }
}

pub(in crate::animation::procedural) fn presentation_tick_delta(
    previous: Option<AnimationSampleTick>,
    current: AnimationSampleTick,
) -> Option<AnimationSampleSpan> {
    match previous {
        None => Some(AnimationSampleSpan::ZERO),
        Some(previous) => {
            let delta = current.wrapping_delta(previous);
            (delta <= body_response_tuning().maximum_presentation_sample_gap_ticks).then_some(delta)
        }
    }
}

/// Adds bounded travel and inertial body response from server-observed world
/// velocity and acceleration transformed through the current presentation body
/// frame.
/// Retained angles are client presentation only.
pub(in crate::animation) fn apply_locomotion_body_response(
    time: Res<Time>,
    procedural_clock: Res<ProceduralAnimationClock>,
    mut commands: Commands,
    mut owners: Query<
        (
            Entity,
            &PresentedSkeleton,
            &Transform,
            Option<&mut LocomotionBodyResponseState>,
        ),
        Without<HumanoidBone>,
    >,
    mut bones: Query<
        (&HumanoidBone, &AuthoredBindTransform, &mut Transform),
        Without<PresentedSkeleton>,
    >,
) {
    let _spike = crate::animation::diagnostics::SpikeGuard::new("apply_locomotion_body_response");
    let mut responses = BTreeMap::new();
    for (owner, skeleton, owner_transform, state) in &mut owners {
        let next = state
            .as_deref()
            .copied()
            .unwrap_or_default()
            .advance_for_presentation(skeleton, owner_transform, &procedural_clock, &time);
        let next = match next {
            Ok(next) => next,
            Err(error) => {
                warn!(?owner, %error, "Rejected locomotion body response");
                continue;
            }
        };
        responses.insert(owner, next);
        if let Some(mut state) = state {
            *state = next;
        } else {
            commands.entity(owner).insert(next);
        }
    }
    let mut leg_compensations = BTreeMap::new();
    for (bone, bind, mut transform) in &mut bones {
        let Some(response) = responses.get(&bone.owner) else {
            continue;
        };
        if bone.role != BoneRole::Pelvis {
            continue;
        }
        let response_rotation = Quat::from_euler(
            EulerRot::XYZ,
            response.pitch_radians.radians(),
            0.0,
            response.roll_radians.radians(),
        );
        let (pelvis_rotation, leg_compensation) =
            stable_pelvis_response(transform.rotation, bind.local.rotation, response_rotation);
        // Apply the response in the pelvis's stable authored reference frame.
        // Post-multiplying it into the live pelvis frame made ordinary gait
        // twist steer forward pitch alternately left and right.
        transform.rotation = pelvis_rotation;
        leg_compensations.insert(bone.owner, leg_compensation);
    }
    for (bone, _, mut transform) in &mut bones {
        if !matches!(bone.role, BoneRole::ThighLeft | BoneRole::ThighRight) {
            continue;
        }
        let Some(&compensation) = leg_compensations.get(&bone.owner) else {
            continue;
        };
        // The leg solver follows this pass. Exactly cancel the inherited
        // parent-space response at each hip so the authored leg pose and IK
        // targets are unchanged by torso lean.
        transform.rotation = compensation * transform.rotation;
    }
}

fn body_response_tuning() -> BodyResponseConfig {
    runtime_animation_config().procedural.body_response
}

fn smooth_acceleration(
    current: BodyLocalAcceleration,
    target: BodyLocalAcceleration,
    elapsed: AnimationElapsed,
) -> AnimationValueResult<BodyLocalAcceleration> {
    let target_squared = target.planar_squared()?;
    let current_squared = current.planar_squared()?;
    // Exponential filtering operates on native body-local metres/second squared.
    let current = current.metres_per_second_squared();
    let target = target.metres_per_second_squared();
    let rate = if target_squared > current_squared {
        body_response_tuning().acceleration_attack_response_per_second
    } else {
        body_response_tuning().acceleration_release_response_per_second
    };
    let exponent = -rate * elapsed.seconds();
    if !exponent.is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::BodyLocalAcceleration,
        });
    }
    let response = 1.0 - exponent.exp();
    BodyLocalAcceleration::from_metres_per_second_squared(current.lerp(target, response))
}

fn advance_body_response(
    current: BodyResponseAngles,
    velocity: BodyAngularVelocity,
    target: BodyResponseAngles,
    elapsed: AnimationElapsed,
) -> AnimationValueResult<BodyResponseStep> {
    if elapsed.is_zero() {
        return Ok(BodyResponseStep {
            angles: current,
            angular_velocity: velocity,
        });
    }
    // The implicit spring is a native two-axis radians/seconds numerical kernel.
    let current = current.radians();
    let target = target.radians();
    let velocity = velocity.radians_per_second();
    let delta_seconds = elapsed.seconds();
    let omega = 2.0 / body_response_tuning().smooth_time_seconds.seconds();
    let omega_squared = omega * omega;
    let denominator =
        1.0 + 2.0 * delta_seconds * omega + delta_seconds * delta_seconds * omega_squared;
    let numerator = velocity + delta_seconds * omega_squared * (target - current);
    if !denominator.is_finite() || !numerator.is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::BodyAngularVelocity,
        });
    }
    let next_velocity = numerator / denominator;
    if !next_velocity.length_squared().is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::BodyAngularVelocity,
        });
    }
    let maximum_speed = body_response_tuning().degrees_per_second.to_radians();
    let next_velocity = next_velocity.clamp_length_max(maximum_speed);
    let next = current + next_velocity * delta_seconds;
    Ok(BodyResponseStep {
        angles: BodyResponseAngles::from_radians(next)?,
        angular_velocity: BodyAngularVelocity::from_radians_per_second(next_velocity)?,
    })
}

fn stable_pelvis_response(
    authored_rotation: Quat,
    reference_rotation: Quat,
    response_rotation: Quat,
) -> (Quat, Quat) {
    let parent_space_response =
        (reference_rotation * response_rotation * reference_rotation.inverse()).normalize();
    let leg_compensation =
        (authored_rotation.inverse() * parent_space_response.inverse() * authored_rotation)
            .normalize();
    (
        (parent_space_response * authored_rotation).normalize(),
        leg_compensation,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pelvis_response_uses_a_stable_reference_and_exactly_compensates_legs() {
        let reference = Quat::from_euler(EulerRot::XYZ, 0.12, -0.08, 0.04);
        let response = Quat::from_euler(EulerRot::XYZ, 0.21, 0.0, -0.09);
        let expected_parent_response = (reference * response * reference.inverse()).normalize();

        for authored in [
            reference,
            Quat::from_euler(EulerRot::XYZ, 0.35, 0.22, -0.18),
        ] {
            let (pelvis, leg_compensation) = stable_pelvis_response(authored, reference, response);
            assert!(
                (pelvis * authored.inverse()).angle_between(expected_parent_response) < 0.000_1
            );
            let compensation_error = (pelvis * leg_compensation).angle_between(authored);
            assert!(compensation_error < 0.001, "{compensation_error}");
        }
    }

    #[test]
    fn deceleration_lean_follows_current_planar_speed_only_while_braking() {
        let walking = Vec3::Z
            * walk_locomotion_profile()
                .reference_speed
                .metres_per_second();
        let walking_scale = deceleration_lean_scale(
            BodyLocalVelocity::from_metres_per_second(walking).unwrap(),
            BodyLocalVelocity::from_metres_per_second(walking).unwrap(),
            BodyLocalAcceleration::from_metres_per_second_squared(Vec3::NEG_Z * 12.0).unwrap(),
        )
        .unwrap()
        .fraction();
        assert!(
            (walking_scale
                - walk_locomotion_profile()
                    .reference_speed
                    .metres_per_second()
                    / run_locomotion_profile().reference_speed.metres_per_second())
            .abs()
                <= 0.0
        );
        assert_eq!(
            deceleration_lean_scale(
                BodyLocalVelocity::from_metres_per_second(Vec3::Z * 0.5).unwrap(),
                BodyLocalVelocity::from_metres_per_second(Vec3::ZERO).unwrap(),
                BodyLocalAcceleration::from_metres_per_second_squared(Vec3::NEG_Z * 12.0).unwrap()
            )
            .unwrap()
            .fraction(),
            0.0
        );
        assert_eq!(
            deceleration_lean_scale(
                BodyLocalVelocity::from_metres_per_second(Vec3::ZERO).unwrap(),
                BodyLocalVelocity::from_metres_per_second(Vec3::NEG_Z).unwrap(),
                BodyLocalAcceleration::from_metres_per_second_squared(Vec3::NEG_Z * 12.0).unwrap()
            )
            .unwrap()
            .fraction(),
            1.0
        );
        assert_eq!(
            deceleration_lean_scale(
                BodyLocalVelocity::from_metres_per_second(Vec3::Z * 5.5).unwrap(),
                BodyLocalVelocity::from_metres_per_second(Vec3::Z * 5.5).unwrap(),
                BodyLocalAcceleration::from_metres_per_second_squared(Vec3::NEG_Z * 12.0).unwrap()
            )
            .unwrap()
            .fraction(),
            1.0
        );
    }

    #[test]
    fn acceleration_response_has_smooth_attack_and_release() {
        let frame_seconds = 1.0 / 60.0;
        let raw = Vec3::Z * 12.0;
        let attacked = smooth_acceleration(
            BodyLocalAcceleration::from_metres_per_second_squared(Vec3::ZERO).unwrap(),
            BodyLocalAcceleration::from_metres_per_second_squared(raw).unwrap(),
            AnimationElapsed::from_seconds(frame_seconds).unwrap(),
        )
        .unwrap()
        .metres_per_second_squared();
        assert!(attacked.z > 0.0 && attacked.z < raw.z);

        let released = smooth_acceleration(
            BodyLocalAcceleration::from_metres_per_second_squared(attacked).unwrap(),
            BodyLocalAcceleration::from_metres_per_second_squared(Vec3::ZERO).unwrap(),
            AnimationElapsed::from_seconds(frame_seconds).unwrap(),
        )
        .unwrap()
        .metres_per_second_squared();
        assert!(released.z > 0.0 && released.z < attacked.z);
    }

    #[test]
    fn body_response_step_is_bounded_by_render_time() {
        let frame_seconds = 1.0 / 60.0;
        let current = Vec2::ZERO;
        let target = Vec2::splat(30.0_f32.to_radians());
        let (advanced, velocity) = advance_body_response(
            BodyResponseAngles::from_radians(current).unwrap(),
            BodyAngularVelocity::from_radians_per_second(Vec2::ZERO).unwrap(),
            BodyResponseAngles::from_radians(target).unwrap(),
            AnimationElapsed::from_seconds(frame_seconds).unwrap(),
        )
        .map(|step| {
            (
                step.angles.radians(),
                step.angular_velocity.radians_per_second(),
            )
        })
        .unwrap();
        let maximum_step = body_response_tuning().degrees_per_second.to_radians() * frame_seconds;
        assert!(advanced.length() <= maximum_step + 0.000_001);
        assert!(
            velocity.length() <= body_response_tuning().degrees_per_second.to_radians() + 0.000_001
        );
    }

    #[test]
    fn body_response_preserves_motion_through_a_target_step() {
        let frame_seconds = 1.0 / 60.0;
        let velocity = Vec2::new(28.0_f32.to_radians(), 0.0);
        let current = Vec2::new(14.0_f32.to_radians(), 0.0);
        let target = Vec2::new(16.0_f32.to_radians(), 0.0);
        let (advanced, next_velocity) = advance_body_response(
            BodyResponseAngles::from_radians(current).unwrap(),
            BodyAngularVelocity::from_radians_per_second(velocity).unwrap(),
            BodyResponseAngles::from_radians(target).unwrap(),
            AnimationElapsed::from_seconds(frame_seconds).unwrap(),
        )
        .map(|step| {
            (
                step.angles.radians(),
                step.angular_velocity.radians_per_second(),
            )
        })
        .unwrap();

        assert!(advanced.x > current.x);
        assert!(next_velocity.x > 0.0);
        assert!((advanced.x - current.x).to_degrees() < 0.75);
    }

    #[test]
    fn sprint_ramp_is_continuous_and_settles_below_its_startup_peak() {
        let frame_seconds = 1.0 / 64.0;
        let mut acceleration = Vec3::ZERO;
        let mut response = Vec2::ZERO;
        let mut velocity = Vec2::ZERO;
        let mut maximum_pitch = 0.0_f32;
        let mut maximum_step = 0.0_f32;

        for frame in 0..160 {
            let speed = (frame as f32 / 32.0).clamp(0.0, 1.0)
                * run_locomotion_profile().reference_speed.metres_per_second();
            let raw_acceleration = if frame <= 32 {
                Vec3::Z * 11.0
            } else {
                Vec3::ZERO
            };
            acceleration = smooth_acceleration(
                BodyLocalAcceleration::from_metres_per_second_squared(acceleration).unwrap(),
                BodyLocalAcceleration::from_metres_per_second_squared(raw_acceleration).unwrap(),
                AnimationElapsed::from_seconds(frame_seconds).unwrap(),
            )
            .unwrap()
            .metres_per_second_squared();
            let target = body_response_target(
                BodyLocalVelocity::from_metres_per_second(Vec3::Z * speed).unwrap(),
                BodyLocalAcceleration::from_metres_per_second_squared(acceleration).unwrap(),
                BrakingLeanScale::from_fraction(1.0).unwrap(),
            )
            .unwrap()
            .radians();
            let previous = response;
            (response, velocity) = advance_body_response(
                BodyResponseAngles::from_radians(response).unwrap(),
                BodyAngularVelocity::from_radians_per_second(velocity).unwrap(),
                BodyResponseAngles::from_radians(target).unwrap(),
                AnimationElapsed::from_seconds(frame_seconds).unwrap(),
            )
            .map(|step| {
                (
                    step.angles.radians(),
                    step.angular_velocity.radians_per_second(),
                )
            })
            .unwrap();
            maximum_pitch = maximum_pitch.max(response.x);
            maximum_step = maximum_step.max((response.x - previous.x).abs());
        }

        let stable_pitch = response.x.to_degrees();
        let startup_peak = maximum_pitch.to_degrees();
        assert!((15.9..=16.1).contains(&stable_pitch), "{stable_pitch}");
        assert!((16.25..=18.0).contains(&startup_peak), "{startup_peak}");
        assert!(maximum_step.to_degrees() <= 0.6);
    }
}
