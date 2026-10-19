//! Checked controller observations projected into retained locomotion state.

use super::*;

/// One authoritative fixed-tick locomotion observation. The tactical server
/// supplies this from its character controller; deterministic presentation
/// fixtures replay the same boundary without inventing gait phase directly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkeletonLocomotionInput {
    orientation: Quat,
    linear_velocity: Vec3,
    ground_contact: GroundContact,
    delta_seconds: AnimationElapsed,
    tick: AnimationSampleTick,
}

struct ProjectedObservation {
    local_velocity: Vec3,
    local_speed: NonnegativeSpeed,
    world_acceleration: Vec3,
    physical_speed: NonnegativeSpeed,
    landing_impact: Option<LandingImpactSpeed>,
}

impl SkeletonLocomotionInput {
    /// Admit the controller's native orientation and world velocity once. A
    /// finite quaternion need not be unit length at this existing boundary;
    /// transformed vectors and derived norms are checked during projection.
    pub fn from_controller(
        orientation: Quat,
        linear_velocity: Vec3,
        ground_contact: GroundContact,
        delta_seconds: AnimationElapsed,
        tick: AnimationSampleTick,
    ) -> AnimationValueResult<Self> {
        if !linear_velocity.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::WorldVelocity,
            });
        }
        if !orientation.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::BodyOrientation,
            });
        }
        Ok(Self {
            orientation,
            linear_velocity,
            ground_contact,
            delta_seconds,
            tick,
        })
    }
}

impl ProjectedObservation {
    fn admit(
        skeleton: &SkeletonState,
        input: SkeletonLocomotionInput,
        body_rotation: Quat,
    ) -> AnimationValueResult<Self> {
        if !skeleton.world_velocity.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::WorldVelocity,
            });
        }
        if !body_rotation.is_finite() {
            return Err(AnimationValueError::NonFinite {
                quantity: AnimationQuantity::BodyOrientation,
            });
        }
        // Native Bevy vectors are transformed into controller/body frames at this seam.
        let previous_world_velocity = skeleton.world_velocity;
        let controller_local = controller_yaw(input.orientation).inverse() * input.linear_velocity;
        let body_local = body_rotation.inverse() * input.linear_velocity;
        let local_velocity = if skeleton.weapon_guard() == WeaponGuardState::Raised {
            controller_local
        } else {
            body_local
        };
        let physical_speed = input.linear_velocity.xz().length();
        let local_speed = local_velocity.xz().length();
        let acceleration = if input.tick == skeleton.locomotion_sample_tick.wrapping_next() {
            (input.linear_velocity - previous_world_velocity) * locomotion_sample_rate().hertz()
        } else {
            Vec3::ZERO
        };
        if !local_velocity.is_finite()
            || !local_speed.is_finite()
            || !physical_speed.is_finite()
            || !acceleration.length_squared().is_finite()
        {
            return Err(AnimationValueError::Overflow {
                quantity: AnimationQuantity::WorldVelocity,
            });
        }
        let landing_impact = if !skeleton.body.is_surface_supported()
            && input.ground_contact == GroundContact::Grounded
        {
            // Upward finite velocity has zero downward impact; this is physical
            // projection after admission, not repair of an invalid speed.
            Some(LandingImpactSpeed::from_metres_per_second(
                (-previous_world_velocity.y).max(0.0),
            )?)
        } else {
            None
        };
        Ok(Self {
            local_velocity,
            local_speed: NonnegativeSpeed::from_metres_per_second(local_speed)?,
            physical_speed: NonnegativeSpeed::from_metres_per_second(physical_speed)?,
            world_acceleration: acceleration.clamp_length_max(80.0),
            landing_impact,
        })
    }
}

/// Projects controller motion into the compact replicated animation state.
/// Bone evaluation remains client-only; this is the shared server seam that
/// keeps deterministic captures on the same stride and posture rules.
pub fn project_skeleton_locomotion(
    skeleton: &mut SkeletonState,
    input: SkeletonLocomotionInput,
) -> AnimationValueResult<()> {
    project_skeleton_locomotion_with_intent(skeleton, input, None)
}

/// Server projection variant that preserves held movement intent while the
/// physical motor temporarily brakes to zero between guard contacts.
pub fn project_skeleton_locomotion_with_intent(
    skeleton: &mut SkeletonState,
    input: SkeletonLocomotionInput,
    requested_local_direction: Option<Vec2>,
) -> AnimationValueResult<()> {
    let body_rotation = controller_yaw(input.orientation);
    project_skeleton_locomotion_with_body_rotation(
        skeleton,
        input,
        body_rotation,
        requested_local_direction,
    )
}

/// Projection variant for the authoritative server, which owns both the
/// camera/controller frame and the independently rotating body root.
pub fn project_skeleton_locomotion_with_body_rotation(
    skeleton: &mut SkeletonState,
    input: SkeletonLocomotionInput,
    body_rotation: Quat,
    requested_local_direction: Option<Vec2>,
) -> AnimationValueResult<()> {
    let observation = ProjectedObservation::admit(skeleton, input, body_rotation)?;
    let ProjectedObservation {
        local_velocity,
        local_speed,
        world_acceleration,
        physical_speed,
        landing_impact,
        ..
    } = observation;
    skeleton.world_acceleration = world_acceleration;
    skeleton.local_velocity = local_velocity;
    skeleton.world_velocity = input.linear_velocity;
    skeleton.locomotion_sample_tick = input.tick;
    if skeleton.body == BodyState::Ragdolled {
        skeleton.set_raised_locomotion(RaisedLocomotionIntent::planted());
        skeleton.action = ActionState::default();
        return Ok(());
    }
    let landed = landing_impact.is_some();
    if let Some(impact) = landing_impact {
        skeleton.landing_sequence = skeleton.landing_sequence.next();
        skeleton.landing_impact_speed = impact;
    }
    skeleton.transition_body(if input.ground_contact == GroundContact::Grounded {
        match skeleton.body {
            BodyState::Prone | BodyState::Supine => skeleton.body,
            _ => BodyState::Grounded(GroundedPosture::Upright),
        }
    } else {
        BodyState::Airborne
    });
    if landed && skeleton.action_kind() == SkeletonAction::Dodge {
        // Contact ends presentation ownership immediately. Residual horizontal
        // velocity remains authoritative and can continue through ordinary
        // raised locomotion while the server applies its landing drag.
        skeleton.action = ActionState::default();
    }

    advance_projected_ground_cycle(
        skeleton,
        input,
        local_velocity,
        physical_speed,
        local_speed,
        requested_local_direction,
    )?;
    skeleton.advance_action(input.tick);
    Ok(())
}

fn advance_projected_ground_cycle(
    skeleton: &mut SkeletonState,
    input: SkeletonLocomotionInput,
    local_velocity: Vec3,
    physical_speed: NonnegativeSpeed,
    local_speed: NonnegativeSpeed,
    requested_local_direction: Option<Vec2>,
) -> AnimationValueResult<()> {
    let ground_speed = if skeleton.is_downed_turning() {
        // Turning in place has no physical velocity, but its crawl/scamper
        // cycle should run at twice the former synthetic cadence.
        physical_speed
            .max(crate::animation::locomotion::MINIMUM_DOWNED_ANIMATION_SPEED)
            .checked_scale(2.0)?
    } else if matches!(skeleton.body, BodyState::Prone | BodyState::Supine) {
        // Residual impulses and collision correction may translate a downed
        // body, but prone/supine presentation deliberately slides in its idle
        // pose. Velocity-driven locomotion belongs only to upright bodies.
        NonnegativeSpeed::ZERO
    } else {
        physical_speed
    };
    if skeleton.weapon_guard() == WeaponGuardState::Raised && skeleton.posture() == Posture::Upright
    {
        advance_raised_locomotion_intent(
            skeleton,
            local_velocity,
            local_speed,
            requested_local_direction,
        );
        advance_guard_footwork(
            skeleton,
            input
                .delta_seconds
                .min(AnimationElapsed::from_seconds(1.0)?),
            local_speed,
            input.tick,
        )?;
    } else {
        skeleton.set_raised_locomotion(RaisedLocomotionIntent::planted());
        if input.ground_contact == GroundContact::Grounded
            && ground_speed
                > crate::combat_config::runtime_animation_config()
                    .playback
                    .initial_locomotion_speed_metres_per_second
        {
            let profile = locomotion_profile(skeleton, skeleton.animation_speed()?)?;
            let phase = skeleton.gait_phase.wrapped();
            let next_phase = phase.checked_advance(gait_cycle_phase_delta(
                profile,
                ground_speed,
                input.delta_seconds.capped_by_seconds(1.0)?,
            )?)?;
            let handoffs = ((next_phase.cycles() * f32::from(LOCOMOTION_CONTACTS_PER_CYCLE))
                .floor()
                - (phase.cycles() * f32::from(LOCOMOTION_CONTACTS_PER_CYCLE)).floor())
            .max(0.0) as u32;
            skeleton.gait_phase = next_phase.wrapped();
            advance_contact_identity(skeleton, handoffs, None);
            if skeleton.weapon_guard() == WeaponGuardState::Lowered {
                skeleton.lead_foot =
                    if skeleton.gait_phase.cycles() < LOCOMOTION_SECOND_CONTACT_PHASE {
                        LeadFoot::Left
                    } else {
                        LeadFoot::Right
                    };
            }
        }
    }
    Ok(())
}
