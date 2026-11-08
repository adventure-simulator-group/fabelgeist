use fabelgeist_geometry::SignedLength;
use adventuresim_core::linear_speed::{NonnegativeSpeed, SignedSpeed};
use adventuresim_core::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_ahoy::{
    CharacterController, CharacterControllerState, CharacterLook, input::AccumulatedInput,
};
use adventuresim_building_generator::spatial_geometry::{PlanDirection, Position, RigidRotation};
use crate::scene_coordinates::Scene;

use movement_speed::*;
pub use movement_speed::{
    BREATH_PER_METRE_PER_SECOND, TACTICAL_BREATH_RESPONSE_SCALE,
    TACTICAL_DIVE_HORIZONTAL_SPEED_METRES_PER_SECOND, TACTICAL_GUARD_SPEED_METRES_PER_SECOND,
    TACTICAL_PRONE_LATERAL_SPEED_SCALE, TACTICAL_PRONE_SPEED_METRES_PER_SECOND,
    TACTICAL_PRONE_WALK_SPEED_METRES_PER_SECOND, TACTICAL_QUICKSTEP_SPEED_METRES_PER_SECOND,
    TACTICAL_ROLL_SPEED_METRES_PER_SECOND, TACTICAL_RUN_SPEED_METRES_PER_SECOND,
    TACTICAL_WALK_SPEED_METRES_PER_SECOND, tactical_breath_recovery_per_second,
    tactical_exhaustion_change_per_second, tactical_jog_speed,
    tactical_movement_exhaustion_change_per_second, tactical_movement_speed,
    tactical_movement_speed_for_guard, tactical_movement_speed_for_pace, tactical_sprint_speed,
};

use crate::{
    animation::{
        AnimationElapsed, AnimationQuantity, AnimationSampleSpan, AnimationValueError,
        AnimationValueResult, DownedRollMotion, PositiveAnimationDuration, SkeletonState,
        WeaponGuardState, locomotion_sample_rate,
    },
    player::{CharacterDimensions, PlayerInputTick, TacticalPlayerViewer},
};

mod controller_config;
mod movement_speed;
mod plugin;
pub use quickstep::quickstep_action_contact_ticks;

mod quickstep;
pub use plugin::{AdventureSimulatorPhysicsPlugin, AdventureSimulatorPhysicsSet};

#[derive(
    Component,
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Reflect,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum MovementPace {
    #[default]
    Walk,
    Jog,
    Sprint,
}

type MovementControllerQuery<'world, 'state> = Query<
    'world,
    'state,
    (
        Entity,
        &'static AccumulatedInput,
        &'static mut CharacterController,
        &'static CharacterControllerState,
        &'static mut LinearVelocity,
        Option<&'static mut Mass>,
        Option<&'static CharacterLook>,
        Option<&'static SkeletonState>,
        Option<&'static MovementPace>,
        Option<&'static mut QuickstepPush>,
        Option<&'static MeleeLungeMovement>,
        &'static Transform,
        Has<QuickstepMismatchLogged>,
    ),
>;

const MINIMUM_ORDINARY_TURN_RADIUS_METRES: f32 = 0.25;

pub const TACTICAL_JUMP_HEIGHT_METRES: f32 = 0.30;
pub const TACTICAL_DIVE_JUMP_HEIGHT_METRES: f32 = 0.20;
pub const TACTICAL_GRAVITY_METRES_PER_SECOND_SQUARED: f32 = 9.81;
pub const TACTICAL_MAXIMUM_STEP_HEIGHT_METRES: f32 = 0.35;
pub const TACTICAL_MAXIMUM_WALKABLE_SLOPE_DEGREES: f32 = 40.0;

/// Builds the shared tactical controller instead of inheriting Ahoy's much
/// faster general-purpose default.
pub fn tactical_character_controller() -> CharacterController {
    CharacterController {
        speed: TACTICAL_RUN_SPEED_METRES_PER_SECOND.metres_per_second(),
        // Horizontal velocity is owned by the Fabelgeist motor below. Ahoy
        // remains the collision-constraint solver and must not accelerate or
        // brake the same state a second time.
        acceleration_hz: 0.0,
        air_acceleration_hz: 0.0,
        water_acceleration_hz: 0.0,
        friction_hz: 0.0,
        stop_speed: 0.0,
        gravity: TACTICAL_GRAVITY_METRES_PER_SECOND_SQUARED,
        step_size: TACTICAL_MAXIMUM_STEP_HEIGHT_METRES,
        min_walk_cos: TACTICAL_MAXIMUM_WALKABLE_SLOPE_DEGREES.to_radians().cos(),
        jump_height: TACTICAL_JUMP_HEIGHT_METRES,
        ..default()
    }
}

/// Compact rollback boundary produced after an authoritative controller step.
/// Clients can restore this state at `acknowledged_input_tick` and replay newer
/// locally buffered input without treating a render transform as simulation
/// authority. The acknowledgement is absent before the first accepted input;
/// an observed input tick of zero is not an initialization sentinel.
#[derive(
    Component, Debug, Clone, Copy, Default, PartialEq, Reflect, serde::Serialize, serde::Deserialize,
)]
#[reflect(Component)]
pub struct CharacterMotionSnapshot {
    pub acknowledged_input_tick: Option<PlayerInputTick>,
    pub translation: Vec3,
    pub rotation: Quat,
    pub linear_velocity: Vec3,
    pub grounded: bool,
    pub quickstep_push: QuickstepPush,
    pub melee_lunge: Option<MeleeLungeMovement>,
}

/// Rollback-safe state for a quickstep's finite, foot-supported push.
///
/// This is deliberately a force actuator, not a pending velocity change. The
/// direction and start tick are committed when the input edge is accepted;
/// the shared fixed-tick motor then integrates the same force curve on an
/// authoritative server or a predicting client.
#[derive(
    Component, Debug, Clone, Copy, PartialEq, Reflect, serde::Serialize, serde::Deserialize,
)]
#[reflect(Component)]
pub struct QuickstepPush {
    pub start_tick: crate::animation::AnimationSampleTick,
    pub direction: Vec2,
    pub orientation: RigidRotation,
    pub origin: Position<Scene>,
    pub active: bool,
}

/// A server-owned, collision-resolved forward movement committed when a melee
/// attack starts. It is intentionally independent of `SkeletonAction`, so the
/// attack animation remains authoritative while the lower body closes range.
#[derive(
    Component, Debug, Clone, Copy, PartialEq, Reflect, serde::Serialize, serde::Deserialize,
)]
#[reflect(Component)]
pub struct MeleeLungeMovement {
    pub origin: Position<Scene>,
    pub direction: PlanDirection<Scene>,
    pub distance_metres: SignedLength,
    pub quickstep: bool,
}

impl Default for QuickstepPush {
    fn default() -> Self {
        Self {
            start_tick: crate::animation::AnimationSampleTick::default(),
            direction: Vec2::ZERO,
            orientation: RigidRotation::IDENTITY,
            origin: Position::ORIGIN,
            active: false,
        }
    }
}

fn melee_lunge_has_arrived(travelled_metres: SignedLength, planned_distance_metres: SignedLength) -> bool {
    travelled_metres.metres() >= planned_distance_metres.metres()
}

#[derive(Component, Debug, Clone, Copy, Default)]
struct QuickstepMismatchLogged;

const QUICKSTEP_FORCE_CURVE_RAMP_FRACTION: f32 = 0.31;
/// Smooth rise, maximal mid-push force, and smooth release. Its normalized
/// area is 0.69, matching the force-time profile used to calibrate the
/// biomechanical values in `combat.yaml`.
pub fn quickstep_force_curve(normalized_time: f32) -> f32 {
    let t = normalized_time.clamp(0.0, 1.0);
    let edge = t.min(1.0 - t);
    if edge >= QUICKSTEP_FORCE_CURVE_RAMP_FRACTION {
        1.0
    } else {
        let x = edge / QUICKSTEP_FORCE_CURVE_RAMP_FRACTION;
        x * x * (3.0 - 2.0 * x)
    }
}

pub fn quickstep_push_duration(
    leg_agility: f32,
    motor: &crate::combat_config::CharacterMotorConfig,
) -> AnimationValueResult<PositiveAnimationDuration> {
    let seconds = (motor.reference_quickstep_push_seconds.seconds()
        - (leg_agility - motor.reference_leg_agility)
            * motor.quickstep_push_seconds_reduction_per_agility)
        .max(locomotion_sample_rate().interval().seconds());
    if !seconds.is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::Elapsed,
        });
    }
    PositiveAnimationDuration::try_from(seconds)
}

pub fn quickstep_peak_horizontal_force_newtons(
    biological_mass_kg: f32,
    leg_strength: f32,
    motor: &crate::combat_config::CharacterMotorConfig,
) -> f32 {
    let bodyweights = (motor.reference_quickstep_peak_horizontal_force_bodyweights
        + (leg_strength - motor.reference_leg_strength)
            * motor.quickstep_peak_horizontal_force_bodyweights_per_strength)
        .max(0.0);
    biological_mass_kg.max(0.0) * motor.gravity_metres_per_second_squared * bodyweights
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuickstepMotionTarget {
    pub displacement_metres: SignedLength,
    pub velocity_metres_per_second: SignedSpeed,
    pub acceleration_metres_per_second_squared: f32,
}

/// Scales the authored reference travel by the character's hip-to-ankle chain.
pub fn quickstep_target_displacement_metres(
    leg_length_metres: SignedLength,
    motor: &crate::combat_config::CharacterMotorConfig,
) -> AnimationValueResult<SignedLength> {
    let reference_leg_length = motor.reference_quickstep_leg_length_metres.metres();
    let leg_length = if leg_length_metres.metres() > 0.0 {
        leg_length_metres.metres()
    } else {
        reference_leg_length
    };
    // Preserve the original product-then-division order and nonpositive-leg
    // reference policy at this complete biomechanical scaling kernel.
    SignedLength::from_metres(motor.reference_quickstep_target_displacement_metres.metres() * leg_length / reference_leg_length)
        .map_err(|_| AnimationValueError::Overflow { quantity: AnimationQuantity::Length })
}

/// Samples a monotone cubic through the authored 0/3/6/9/12 root positions.
/// End tangents are zero so the physical body can enter and leave combat idle
/// without an instantaneous velocity change.
pub fn quickstep_motion_target(
    normalized_time: f32,
    target_displacement_metres: SignedLength,
    duration: AnimationElapsed,
    profile: [f32; 5],
) -> AnimationValueResult<QuickstepMotionTarget> {
    let phase = normalized_time.clamp(0.0, 1.0);
    if phase >= 1.0 || duration.is_zero() {
        return Ok(QuickstepMotionTarget {
            displacement_metres: target_displacement_metres,
            velocity_metres_per_second: SignedSpeed::ZERO,
            acceleration_metres_per_second_squared: 0.0,
        });
    }
    // The monotone interpolation kernel consumes metres only after the
    // terminal observation above has retained the checked target unchanged.
    let target_displacement_metres = target_displacement_metres.metres();
    let segment = ((phase * 4.0).floor() as usize).min(3);
    let u = phase * 4.0 - segment as f32;
    let secants =
        std::array::from_fn::<_, 4, _>(|index| (profile[index + 1] - profile[index]) * 4.0);
    let mut tangents = [0.0; 5];
    for index in 1..4 {
        let before = secants[index - 1];
        let after = secants[index];
        if before > 0.0 && after > 0.0 {
            tangents[index] = 2.0 * before * after / (before + after);
        }
    }
    let p0 = profile[segment];
    let p1 = profile[segment + 1];
    let m0 = tangents[segment] * 0.25;
    let m1 = tangents[segment + 1] * 0.25;
    let u2 = u * u;
    let u3 = u2 * u;
    let position = (2.0 * u3 - 3.0 * u2 + 1.0) * p0
        + (u3 - 2.0 * u2 + u) * m0
        + (-2.0 * u3 + 3.0 * u2) * p1
        + (u3 - u2) * m1;
    let derivative_u = (6.0 * u2 - 6.0 * u) * p0
        + (3.0 * u2 - 4.0 * u + 1.0) * m0
        + (-6.0 * u2 + 6.0 * u) * p1
        + (3.0 * u2 - 2.0 * u) * m1;
    let second_derivative_u = (12.0 * u - 6.0) * p0
        + (6.0 * u - 4.0) * m0
        + (-12.0 * u + 6.0) * p1
        + (6.0 * u - 2.0) * m1;
    let duration = duration.seconds();
    Ok(QuickstepMotionTarget {
        displacement_metres: SignedLength::from_metres(position * target_displacement_metres).map_err(|_| AnimationValueError::Overflow { quantity: AnimationQuantity::Length })?,
        velocity_metres_per_second: SignedSpeed::from_metres_per_second(
            derivative_u * 4.0 * target_displacement_metres / duration,
        )?,
        acceleration_metres_per_second_squared: second_derivative_u
            * 16.0
            * target_displacement_metres
            / (duration * duration),
    })
}

/// Returns the bounded longitudinal ground-reaction force needed to follow the
/// authored trajectory. Position and velocity feedback correct collision and
/// integration error; neither term directly changes transform or velocity.
pub fn quickstep_tracking_force_newtons(
    displacement_metres: SignedLength,
    velocity_metres_per_second: SignedSpeed,
    target: QuickstepMotionTarget,
    mass_kg: f32,
    maximum_force_newtons: f32,
    elapsed: AnimationElapsed,
) -> f32 {
    let delta_seconds = elapsed.seconds();
    let velocity_metres_per_second = velocity_metres_per_second.metres_per_second();
    if delta_seconds <= 0.0 {
        return 0.0;
    }
    const TRACKING_FREQUENCY_RADIANS_PER_SECOND: f32 = 12.0;
    // Ground projection can leave a few ten-thousandths of a metre per second
    // behind on an otherwise stationary character. Treat that contact noise as
    // rest: applying the reversal guard to it limits a launch to the tiny force
    // needed to reach exactly zero velocity, suppressing the entire quickstep.
    const VELOCITY_REVERSAL_GUARD_METRES_PER_SECOND: f32 = 0.02;
    let position_error = target.displacement_metres.metres() - displacement_metres.metres();
    let velocity_error =
        target.velocity_metres_per_second.metres_per_second() - velocity_metres_per_second;
    let acceleration = target.acceleration_metres_per_second_squared
        + TRACKING_FREQUENCY_RADIANS_PER_SECOND.powi(2) * position_error
        + 2.0 * TRACKING_FREQUENCY_RADIANS_PER_SECOND * velocity_error;
    let mass = mass_kg.max(1.0);
    let force = (mass * acceleration).clamp(
        -maximum_force_newtons.max(0.0),
        maximum_force_newtons.max(0.0),
    );
    let stopping_force = -velocity_metres_per_second * mass / delta_seconds;
    if velocity_metres_per_second > VELOCITY_REVERSAL_GUARD_METRES_PER_SECOND {
        force.max(stopping_force)
    } else if velocity_metres_per_second < -VELOCITY_REVERSAL_GUARD_METRES_PER_SECOND {
        force.min(stopping_force)
    } else {
        force
    }
}

fn apply_character_motor(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut controllers: MovementControllerQuery<'_, '_>,
    viewer: TacticalPlayerViewer,
    dimensions: Query<&CharacterDimensions>,
    combat_config: Res<crate::combat_config::TacticalCombatConfig>,
) {
    let movement_config = &combat_config.movement;
    let speeds = &movement_config.speeds_metres_per_second;
    for (
        entity,
        input,
        mut controller,
        controller_state,
        mut velocity,
        mass_component,
        look,
        skeleton,
        pace,
        quickstep_push,
        melee_lunge,
        transform,
        mismatch_logged,
    ) in &mut controllers
    {
        let attack_quickstep_ended = melee_lunge.is_some_and(|lunge| lunge.quickstep)
            && !skeleton.is_some_and(|skeleton| {
                skeleton.action_kind() == crate::animation::SkeletonAction::Attack
            });
        let melee_lunge = melee_lunge.filter(|_| {
            skeleton.is_some_and(|skeleton| {
                skeleton.action_kind() == crate::animation::SkeletonAction::Attack
            })
        });
        if melee_lunge.is_none() {
            commands.entity(entity).remove::<MeleeLungeMovement>();
        }
        // Ahoy's crouch flag supplies the short collider used by downed and
        // posture-transition states. Their configured speeds are already the
        // final physical targets, so the collider must not scale speed again.
        controller.crouch_speed_scale = 1.0;
        let maneuver_jump_height = skeleton.and_then(|skeleton| {
            match skeleton
                .posture_transition()
                .map(|transition| transition.kind())
            {
                Some(crate::animation::PostureTransitionKind::DiveToDowned {
                    trajectory: crate::animation::DiveTrajectory::Airborne,
                    ..
                }) => Some(movement_config.jump_heights_metres.dive),
                Some(crate::animation::PostureTransitionKind::DiveToDowned {
                    trajectory: crate::animation::DiveTrajectory::GroundedSlide,
                    ..
                }) => Some(0.0),
                _ => None,
            }
        });
        let motor = &movement_config.motor;
        controller_config::apply_motor_config(&mut controller, motor);
        let guard = skeleton.map_or(WeaponGuardState::Lowered, SkeletonState::weapon_guard);
        let roll_motion =
            skeleton.map_or(DownedRollMotion::REST, SkeletonState::downed_lateral_motion);
        if roll_motion.is_moving() {
            controller.speed = speeds.roll.metres_per_second() * roll_motion.fraction().abs();
        } else if skeleton.is_some_and(SkeletonState::is_posture_transitioning) {
            controller.speed = 0.0;
        } else if skeleton.is_some_and(|skeleton| {
            skeleton.action_kind() == crate::animation::SkeletonAction::Dodge
                && skeleton.action_direction() != Vec2::ZERO
        }) {
            controller.speed = speeds.quickstep.metres_per_second();
        } else if let Some(lunge) = melee_lunge {
            controller.speed = if lunge.quickstep {
                speeds.quickstep.metres_per_second()
            } else {
                speeds.run.metres_per_second()
            };
        } else {
            controller.speed = match movement_speed::ordinary_controller_speed(
                input.last_movement,
                pace.copied(),
                guard,
                skeleton.map(SkeletonState::body),
                viewer.get(entity).ok(),
                movement_config,
            ) {
                Ok(speed) => speed.metres_per_second(),
                Err(error) => {
                    warn!(%error, ?entity, "Rejected character movement speed");
                    continue;
                }
            };
        }

        let (mass_kg, biological_mass_kg, leg_strength, leg_agility) = viewer.get(entity).map_or(
            (
                motor.fallback_character_mass_kg,
                motor.fallback_character_mass_kg,
                motor.reference_leg_strength,
                motor.reference_leg_agility,
            ),
            |player| {
                let burden = player.body_weight() + player.inventory_weight();
                let left_health = player.body_part_health(BodyPart::LeftLeg).max(0.0);
                let right_health = player.body_part_health(BodyPart::RightLeg).max(0.0);
                (
                    burden.max(1.0),
                    player.body_weight().max(1.0),
                    (player.raw_limb_attr(LimbAttribute::Strength, BodyPart::LeftLeg)
                        * left_health
                        + player.raw_limb_attr(LimbAttribute::Strength, BodyPart::RightLeg)
                            * right_health)
                        * 0.5,
                    (player.raw_limb_attr(LimbAttribute::Agility, BodyPart::LeftLeg) * left_health
                        + player.raw_limb_attr(LimbAttribute::Agility, BodyPart::RightLeg)
                            * right_health)
                        * 0.5,
                )
            },
        );
        if let Some(mut mass) = mass_component {
            mass.0 = mass_kg;
        } else {
            commands.entity(entity).insert(Mass(mass_kg));
        }
        controller.jump_height = maneuver_jump_height.unwrap_or_else(|| {
            ordinary_jump_height(
                movement_config.jump_heights_metres.ordinary,
                mass_kg,
                leg_strength,
                leg_agility,
                motor,
            )
        });

        let orientation = look.map_or(controller_state.orientation, CharacterLook::to_quat);

        let quickstep_action = skeleton.is_some_and(|skeleton| {
            skeleton.action_kind() == crate::animation::SkeletonAction::Dodge
                && skeleton.action_direction() != Vec2::ZERO
        });
        let attack_quickstep = melee_lunge.is_some_and(|lunge| lunge.quickstep);
        let quickstep_push_present = quickstep_push.is_some();
        let quickstep_push_active = quickstep_push.as_ref().is_some_and(|push| push.active);
        if let Some(mut push) = quickstep_push {
            if attack_quickstep_ended {
                push.cancel();
            }
            let world_direction = (push.orientation.quaternion()
                * Vec3::new(push.direction.x, 0.0, -push.direction.y))
            .xz()
            .normalize_or_zero();
            let action_duration = movement_config.maneuvers.quickstep_duration_seconds;
            let clock = push.sample_clock(skeleton, action_duration);
            let elapsed_ticks = clock.elapsed;
            let forward_velocity = velocity.xz().dot(world_direction);
            let bounded_recovery = clock.bounded_recovery(quickstep_action, forward_velocity);
            if push.active && (quickstep_action || attack_quickstep || bounded_recovery) {
                let support_duration = match quickstep_push_duration(leg_agility, motor) {
                    Ok(duration) => duration,
                    Err(error) => {
                        warn!(%error, ?entity, "Rejected quickstep support duration");
                        continue;
                    }
                };
                let velocity_before = velocity.0;
                if mismatch_logged {
                    commands.entity(entity).remove::<QuickstepMismatchLogged>();
                }
                let displacement = match SignedLength::from_metres((transform.translation - push.origin.metres())
                    .xz()
                    .dot(world_direction)
                    .max(0.0)) {
                    Ok(distance) => distance,
                    Err(error) => { warn!(%error, ?entity, "Rejected quickstep observed travel"); continue; }
                };
                if let Some(lunge) = melee_lunge
                    && lunge.quickstep
                    && melee_lunge_has_arrived(displacement, lunge.distance_metres)
                {
                    push.cancel();
                    commands.entity(entity).remove::<MeleeLungeMovement>();
                    continue;
                }
                // Target the state produced by this integration step. Sampling
                // the current tick leaves one full force step of momentum for
                // ordinary locomotion to clean up after the dodge has ended.
                let phase = clock.target_phase();
                let leg_length = dimensions
                    .get(entity)
                    .map_or(motor.reference_quickstep_leg_length_metres, |dimensions| {
                        dimensions.leg_length_metres
                    });
                let target_displacement = match quickstep_target_displacement_metres(leg_length, motor) {
                    Ok(distance) => melee_lunge.map_or(distance, |lunge| distance.min(lunge.distance_metres)),
                    Err(error) => { warn!(%error, ?entity, "Rejected quickstep target displacement"); continue; }
                };
                let target = match quickstep_motion_target(
                    phase,
                    target_displacement,
                    action_duration.elapsed(),
                    motor.quickstep_authored_displacement_profile,
                ) {
                    Ok(target) => target,
                    Err(error) => {
                        warn!(%error, ?entity, "Rejected quickstep target speed");
                        continue;
                    }
                };
                let forward_speed = match SignedSpeed::from_metres_per_second(forward_velocity) {
                    Ok(speed) => speed,
                    Err(error) => {
                        warn!(%error, ?entity, "Rejected quickstep observed speed");
                        continue;
                    }
                };
                let maximum_force = quickstep_peak_horizontal_force_newtons(
                    biological_mass_kg,
                    leg_strength,
                    motor,
                );
                let tracking_force = quickstep_tracking_force_newtons(
                    displacement,
                    forward_speed,
                    target,
                    mass_kg,
                    maximum_force,
                    AnimationElapsed::from(time.delta()),
                );
                let planar_delta =
                    world_direction * tracking_force / mass_kg.max(1.0) * time.delta_secs();
                velocity.x += planar_delta.x;
                velocity.z += planar_delta.y;
                if !quickstep_action
                    && !attack_quickstep
                    && velocity.xz().dot(world_direction).abs() <= 0.02
                {
                    push.cancel();
                }

                let support_ticks = AnimationSampleSpan::from(
                    (support_duration.seconds() * locomotion_sample_rate().hertz())
                        .ceil()
                        .max(1.0) as u64,
                );
                let length_scale =
                    target_displacement.metres() / motor.reference_quickstep_target_displacement_metres.metres();
                if elapsed_ticks < support_ticks
                    && displacement.metres()
                        < motor.quickstep_maximum_supported_root_displacement_metres * length_scale
                {
                    // While the virtual legs remain in contact, their baseline
                    // normal force supports body weight. Ahoy therefore must
                    // not apply airborne gravity yet; only the excess vertical
                    // ground-reaction force changes vertical velocity.
                    controller.gravity = 0.0;
                    let support_phase =
                        (elapsed_ticks.samples() as f32 + 0.5) / support_ticks.samples() as f32;
                    let force_scale = quickstep_force_curve(support_phase);
                    let horizontal_force = maximum_force * force_scale;
                    let vertical_net_force =
                        horizontal_force * motor.quickstep_takeoff_angle_degrees.to_radians().tan();
                    velocity.y += vertical_net_force / mass_kg * time.delta_secs();
                }
                info!(
                    target: "quickstep_trace",
                    ?entity,
                    skeleton_tick = ?skeleton.map(|skeleton| skeleton.locomotion_sample_tick),
                    push_start_tick = push.start_tick.encoded(),
                    elapsed_ticks = elapsed_ticks.samples(),
                    ?world_direction,
                    phase,
                    displacement = displacement.metres(),
                    forward_velocity,
                    target_displacement = target.displacement_metres.metres(),
                    target_velocity = target.velocity_metres_per_second.metres_per_second(),
                    target_acceleration = target.acceleration_metres_per_second_squared,
                    tracking_force,
                    maximum_force,
                    ?velocity_before,
                    velocity_after = ?velocity.0,
                    grounded = ?controller_state.grounded,
                    "[quickstep][motor-pre-collision] integrated force"
                );
                continue;
            }
            if push.active && !quickstep_action && !attack_quickstep {
                push.cancel();
            }
        }
        if quickstep_action {
            if !mismatch_logged {
                warn!(
                    target: "quickstep_trace",
                    ?entity,
                    skeleton_tick = ?skeleton.map(|skeleton| skeleton.locomotion_sample_tick),
                    action_direction = ?skeleton.map_or(Vec2::ZERO, SkeletonState::action_direction),
                    push_present = quickstep_push_present,
                    push_active = quickstep_push_active,
                    translation = ?transform.translation,
                    linear_velocity = ?velocity.0,
                    "[quickstep][motor] dodge animation has no active force actuator"
                );
                commands.entity(entity).insert(QuickstepMismatchLogged);
            }
            // A missing actuator can occur only during partial state restore.
            // Stop through the same bounded leg force instead of snapping the
            // velocity so rollback recovery remains physically continuous.
            let maximum_force =
                quickstep_peak_horizontal_force_newtons(biological_mass_kg, leg_strength, motor);
            let requested_force = -velocity.xz() * mass_kg.max(1.0) / time.delta_secs();
            let force = requested_force.clamp_length_max(maximum_force);
            velocity.x += force.x / mass_kg.max(1.0) * time.delta_secs();
            velocity.z += force.y / mass_kg.max(1.0) * time.delta_secs();
            continue;
        }
        if mismatch_logged {
            commands.entity(entity).remove::<QuickstepMismatchLogged>();
        }
        if let Some(transition) = skeleton.and_then(SkeletonState::posture_transition)
            && matches!(
                transition.kind(),
                crate::animation::PostureTransitionKind::DiveToDowned {
                    trajectory: crate::animation::DiveTrajectory::GroundedSlide,
                    ..
                }
            )
        {
            // The authored first half lowers the body while retaining exact
            // sprint momentum. After body contact, integrate bounded kinetic
            // drag; do not hand the slide to the ordinary target-speed motor.
            if transition.phase().fraction() >= 0.5 && controller_state.grounded.is_some() {
                let next = slide_drag_velocity(velocity.xz(), motor, time.delta_secs());
                velocity.x = next.x;
                velocity.z = next.y;
            }
            continue;
        }
        let movement = input.last_movement.unwrap_or_default();
        let forward = (orientation * Vec3::NEG_Z).with_y(0.0).normalize_or_zero();
        let right = (orientation * Vec3::X).with_y(0.0).normalize_or_zero();
        let desired_direction = if let Some(lunge) = melee_lunge {
            let travelled = match SignedLength::from_metres((transform.translation - lunge.origin.metres())
                .xz()
                .dot(lunge.direction.vector())
                .max(0.0)) {
                Ok(distance) => distance,
                Err(error) => { warn!(%error, ?entity, "Rejected melee observed travel"); continue; }
            };
            if melee_lunge_has_arrived(travelled, lunge.distance_metres) {
                commands.entity(entity).remove::<MeleeLungeMovement>();
                Vec3::ZERO
            } else {
                Vec3::new(lunge.direction.vector().x, 0.0, lunge.direction.vector().y)
            }
        } else {
            (movement.y * forward + movement.x * right).normalize_or_zero()
        };
        let grounded = controller_state.grounded.is_some();
        let strength_scale = (leg_strength.max(0.0) / motor.reference_leg_strength).max(0.0);
        let agility_scale = (leg_agility.max(0.0) / motor.reference_leg_agility).max(0.0);
        let (drive_force, braking_force) =
            character_motor_force_limits(motor, mass_kg, strength_scale);
        let horizontal = velocity.xz();
        let target_speed = controller.speed;
        let target = desired_direction.xz() * target_speed;
        let next = if grounded {
            let traction_acceleration =
                motor.gravity_metres_per_second_squared * motor.traction_coefficient;
            let sustained_lateral_acceleration = (motor.gravity_metres_per_second_squared
                * motor.reference_lateral_acceleration_gravities
                * agility_scale)
                .min(traction_acceleration);
            let lateral_acceleration = cut_lateral_acceleration(
                horizontal,
                desired_direction.xz(),
                sustained_lateral_acceleration,
                traction_acceleration,
                leg_agility,
            );
            let reference_turn_radius = agility_sprint_turn_radius(leg_agility, motor);
            let observed_speed = match NonnegativeSpeed::from_metres_per_second(horizontal.length())
            {
                Ok(speed) => speed,
                Err(error) => {
                    warn!(%error, ?entity, "Rejected ordinary turn speed");
                    continue;
                }
            };
            let turn_radius = ordinary_turn_radius(observed_speed, reference_turn_radius);
            approach_ground_velocity(
                horizontal,
                target,
                GroundVelocityLimits {
                    drive_acceleration: drive_force / mass_kg,
                    braking_acceleration: braking_force / mass_kg,
                    lateral_acceleration,
                    traction_acceleration,
                    turn_radius,
                },
                time.delta_secs(),
            )
        } else {
            approach_velocity(
                horizontal,
                target,
                drive_force / mass_kg * motor.air_control_force_scale,
                time.delta_secs(),
            )
        };
        velocity.x = next.x;
        velocity.z = next.y;
    }
}

fn slide_drag_velocity(
    velocity: Vec2,
    motor: &crate::combat_config::CharacterMotorConfig,
    delta_seconds: f32,
) -> Vec2 {
    approach_velocity(
        velocity,
        Vec2::ZERO,
        motor.gravity_metres_per_second_squared * motor.slide_drag_coefficient,
        delta_seconds,
    )
}

fn character_motor_force_limits(
    motor: &crate::combat_config::CharacterMotorConfig,
    mass_kg: f32,
    strength_scale: f32,
) -> (f32, f32) {
    let traction_force =
        mass_kg.max(1.0) * motor.gravity_metres_per_second_squared * motor.traction_coefficient;
    (
        (motor.reference_ground_drive_force_newtons * strength_scale.max(0.0)).min(traction_force),
        (motor.reference_ground_braking_force_newtons * strength_scale.max(0.0))
            .min(traction_force),
    )
}

fn ordinary_jump_height(
    reference_height: f32,
    mass_kg: f32,
    leg_strength: f32,
    leg_agility: f32,
    motor: &crate::combat_config::CharacterMotorConfig,
) -> f32 {
    let athleticism = ((leg_strength / motor.reference_leg_strength
        + leg_agility / motor.reference_leg_agility)
        * 0.5)
        .max(0.0);
    // Attribute 3 is the reference healthy adult. Attribute 4 produces about
    // a 0.39 m unburdened rise and attribute 5 about 0.49 m; burden reduces
    // takeoff energy per unit mass without changing Earth-normal gravity.
    let takeoff_velocity_scale = 0.58 + 0.42 * athleticism;
    let burden_scale = (REFERENCE_BURDEN_KG / mass_kg.max(1.0)).clamp(0.25, 1.25);
    reference_height * takeoff_velocity_scale * takeoff_velocity_scale * burden_scale
}

/// Converts the configured full-sprint arc into a speed-dependent ordinary
/// turn. Below the reference sprint, constant centripetal acceleration gives
/// `r = v^2 / a`, so running and jogging turn progressively more tightly while
/// retaining their speed. The small floor represents the space required to
/// redirect a standing body instead of collapsing toward a zero-radius pivot.
fn ordinary_turn_radius(speed: NonnegativeSpeed, reference_sprint_radius: f32) -> f32 {
    let speed_fraction = (speed.metres_per_second().max(0.0)
        / REFERENCE_SPRINT_SPEED_METRES_PER_SECOND.metres_per_second())
    .clamp(0.0, 1.0);
    (reference_sprint_radius * speed_fraction * speed_fraction)
        .max(MINIMUM_ORDINARY_TURN_RADIUS_METRES)
}

/// Smooth bounded response through the declarative Agility 1, 3, and 5 sprint
/// radii. In normalized attribute space `x = (agility - 3) / 2`, the unique
/// quadratic through the three anchors is `reference + linear*x + curve*x^2`.
/// Validation guarantees its derivative remains negative over `-1..=1`.
fn agility_sprint_turn_radius(
    leg_agility: f32,
    motor: &crate::combat_config::CharacterMotorConfig,
) -> f32 {
    let x = (leg_agility.clamp(1.0, 5.0) - 3.0) * 0.5;
    let low = motor.agility_one_sprint_turn_radius_metres;
    let reference = motor.reference_sprint_turn_radius_metres;
    let high = motor.agility_five_sprint_turn_radius_metres;
    let linear = (high - low) * 0.5;
    let curve = (high + low) * 0.5 - reference;
    reference + linear * x + curve * x * x
}

fn cut_lateral_acceleration(
    current: Vec2,
    desired_direction: Vec2,
    sustained_acceleration: f32,
    traction_acceleration: f32,
    leg_agility: f32,
) -> f32 {
    if current.length_squared() <= 1.0e-6 || desired_direction == Vec2::ZERO {
        return sustained_acceleration;
    }
    let cosine = current.normalize().dot(desired_direction).clamp(-1.0, 1.0);
    let lateral_demand = (1.0 - cosine * cosine).sqrt();
    // A hard plant can briefly approach the surface traction ceiling. Average
    // agility exploits part of that reserve; John (Agility 4) can use it all.
    let plant_skill = (leg_agility / 4.0).clamp(0.0, 1.0);
    sustained_acceleration.lerp(traction_acceleration, lateral_demand * plant_skill)
}

#[derive(Debug, Clone, Copy)]
struct GroundVelocityLimits {
    drive_acceleration: f32,
    braking_acceleration: f32,
    lateral_acceleration: f32,
    traction_acceleration: f32,
    turn_radius: f32,
}

fn approach_ground_velocity(
    current: Vec2,
    target: Vec2,
    limits: GroundVelocityLimits,
    delta_seconds: f32,
) -> Vec2 {
    let GroundVelocityLimits {
        drive_acceleration,
        braking_acceleration,
        lateral_acceleration,
        traction_acceleration,
        turn_radius,
    } = limits;
    if current == target || delta_seconds <= 0.0 {
        return current;
    }
    let current_speed = current.length();
    let target_speed = target.length();
    if target_speed <= 1.0e-6 {
        return approach_velocity(
            current,
            Vec2::ZERO,
            braking_acceleration.min(traction_acceleration),
            delta_seconds,
        );
    }
    if current_speed <= 1.0e-6 {
        return approach_velocity(
            Vec2::ZERO,
            target,
            drive_acceleration.min(traction_acceleration),
            delta_seconds,
        );
    }

    let current_direction = current / current_speed;
    let target_direction = target / target_speed;
    let cosine = current_direction.dot(target_direction).clamp(-1.0, 1.0);
    // An exact reversal has no physically meaningful turn side. Brake to zero
    // before accelerating backward instead of choosing an arbitrary U-turn.
    if cosine < -0.999 {
        return approach_velocity(
            current,
            Vec2::ZERO,
            braking_acceleration.min(traction_acceleration),
            delta_seconds,
        );
    }

    let cross = current_direction.perp_dot(target_direction);
    let angle = cross.atan2(cosine);
    let radius_acceleration = current_speed * current_speed / turn_radius.max(0.01);
    let steering_acceleration = lateral_acceleration.max(radius_acceleration);
    let maximum_turn = steering_acceleration * delta_seconds / current_speed;
    let turn = angle.clamp(-maximum_turn, maximum_turn);
    let (sin, cos) = turn.sin_cos();
    let turned_direction = Vec2::new(
        current_direction.x * cos - current_direction.y * sin,
        current_direction.x * sin + current_direction.y * cos,
    );
    let speed_acceleration = if target_speed >= current_speed {
        drive_acceleration
    } else {
        braking_acceleration
    }
    .min(traction_acceleration)
    .max(0.0);
    let next_speed = current_speed
        + (target_speed - current_speed).clamp(
            -speed_acceleration * delta_seconds,
            speed_acceleration * delta_seconds,
        );
    turned_direction * next_speed
}

fn approach_velocity(current: Vec2, target: Vec2, acceleration: f32, delta_seconds: f32) -> Vec2 {
    let maximum_delta = (acceleration.max(0.0) * delta_seconds.max(0.0)).max(0.0);
    current + (target - current).clamp_length_max(maximum_delta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn melee_lunge_does_not_stop_one_centimetre_before_strike_ready_distance() {
        assert!(!melee_lunge_has_arrived(SignedLength::from_metres(0.990).unwrap(), SignedLength::from_metres(1.0).unwrap()));
        assert!(!melee_lunge_has_arrived(SignedLength::from_metres(0.999_9).unwrap(), SignedLength::from_metres(1.0).unwrap()));
        assert!(melee_lunge_has_arrived(SignedLength::from_metres(1.0).unwrap(), SignedLength::from_metres(1.0).unwrap()));
        assert!(melee_lunge_has_arrived(SignedLength::from_metres(1.01).unwrap(), SignedLength::from_metres(1.0).unwrap()));
    }

    #[test]
    fn conservative_forward_lunge_arrival_covers_fixed_tick_motor_path() {
        let config = crate::combat_config::TacticalCombatConfig::default();
        let motor = &config.movement.motor;
        let mass = motor.fallback_character_mass_kg;
        let (drive_force, braking_force) = character_motor_force_limits(motor, mass, 1.0);
        let drive_acceleration = drive_force / mass;
        let braking_acceleration = braking_force / mass;
        let traction_acceleration =
            motor.gravity_metres_per_second_squared * motor.traction_coefficient;
        let predicted_acceleration =
            crate::combat::conservative_forward_lunge_acceleration(motor).unwrap();
        let run_speed = config.movement.speeds_metres_per_second.run;
        let dt = locomotion_sample_rate().interval().seconds();

        for distance in [0.101, 0.404, 0.441, 0.499] {
            let arrival = crate::combat::melee_lunge_arrival(
                crate::combat::MeleeLunge::Forward {
                    distance_metres: distance,
                },
                run_speed,
                predicted_acceleration,
                config.movement.maneuvers.quickstep_duration_seconds,
            )
            .unwrap();
            let predicted_seconds = match arrival {
                crate::combat::MeleeLungeArrival::ArrivesIn(elapsed) => elapsed.seconds(),
                other => panic!("expected lunge arrival: {other:?}"),
            };
            let ticks = (predicted_seconds * locomotion_sample_rate().hertz()).ceil() as usize;
            let mut velocity = Vec2::ZERO;
            let mut displacement = 0.0;
            for _ in 0..ticks {
                velocity = approach_ground_velocity(
                    velocity,
                    Vec2::X * run_speed.metres_per_second(),
                    GroundVelocityLimits {
                        drive_acceleration,
                        braking_acceleration,
                        lateral_acceleration: traction_acceleration,
                        traction_acceleration,
                        turn_radius: motor.reference_sprint_turn_radius_metres,
                    },
                    dt,
                );
                displacement += velocity.x * dt;
            }
            assert!(
                displacement >= distance,
                "distance {distance:.3}: predicted {predicted_seconds:.3}s reached only {displacement:.3}m"
            );
        }
    }

    #[test]
    fn slide_drag_is_bounded_and_preserves_heading() {
        let config = crate::combat_config::TacticalCombatConfig::default();
        let motor = &config.movement.motor;
        let incoming = Vec2::new(3.0, 4.0);
        let next = slide_drag_velocity(incoming, motor, 0.25);
        let expected_speed =
            5.0 - motor.gravity_metres_per_second_squared * motor.slide_drag_coefficient * 0.25;

        assert!((next.length() - expected_speed).abs() < 1.0e-5);
        assert!(next.normalize().abs_diff_eq(incoming.normalize(), 1.0e-6));
        assert_eq!(slide_drag_velocity(incoming, motor, 0.0), incoming);
        assert_eq!(slide_drag_velocity(Vec2::ZERO, motor, 1.0), Vec2::ZERO);
    }

    #[test]
    fn controller_speed_system_uses_guard_state_and_lowered_fallback() {
        let mut world = World::new();
        let raised = world
            .spawn((
                AccumulatedInput {
                    last_movement: Some(Vec2::new(0.3, 0.4)),
                    ..default()
                },
                CharacterController::default(),
                SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised),
            ))
            .id();
        let generic = world
            .spawn((
                AccumulatedInput {
                    last_movement: Some(Vec2::ONE),
                    ..default()
                },
                CharacterController::default(),
            ))
            .id();
        world.insert_resource(crate::combat_config::TacticalCombatConfig::default());
        let mut fixed_time = Time::<Fixed>::from_hz(64.0);
        fixed_time.advance_by(std::time::Duration::from_secs_f64(1.0 / 64.0));
        world.insert_resource(fixed_time);
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_character_motor);
        schedule.run(&mut world);

        assert_eq!(
            world.get::<CharacterController>(raised).unwrap().speed,
            TACTICAL_GUARD_SPEED_METRES_PER_SECOND.metres_per_second() * 0.5
        );
        assert_eq!(
            world
                .get::<CharacterController>(raised)
                .unwrap()
                .acceleration_hz,
            0.0
        );
        assert_eq!(
            world.get::<CharacterController>(generic).unwrap().speed,
            TACTICAL_RUN_SPEED_METRES_PER_SECOND.metres_per_second()
        );
        let controller = world.get::<CharacterController>(generic).unwrap();
        assert_eq!(controller.acceleration_hz, 0.0);
        assert_eq!(controller.friction_hz, 0.0);
        assert_eq!(controller.gravity, 9.81);
        assert_eq!(controller.step_size, 0.35);
        assert_eq!(world.get::<Mass>(generic), Some(&Mass(80.0)));
    }

    #[test]
    fn quickstep_support_uses_force_without_an_ahoy_jump_event() {
        let mut skeleton = SkeletonState::default().with_weapon_guard(WeaponGuardState::Raised);
        skeleton
            .begin_dodge(
                crate::animation::DodgeSpec::quickstep(Vec2::Y).unwrap(),
                crate::animation::AnimationSampleTick::from(0u64),
                crate::animation::AnimationSampleTick::from(20u64),
            )
            .unwrap();
        let mut push = QuickstepPush::default();
        push.begin(
            crate::animation::AnimationSampleTick::from(0u64),
            Vec2::Y,
            RigidRotation::IDENTITY,
            Position::ORIGIN,
        );
        let mut world = World::new();
        let entity = world
            .spawn((
                AccumulatedInput {
                    last_movement: Some(Vec2::Y),
                    ..default()
                },
                CharacterController::default(),
                skeleton,
                push,
            ))
            .id();
        world.insert_resource(crate::combat_config::TacticalCombatConfig::default());
        let mut fixed_time = Time::<Fixed>::from_hz(64.0);
        fixed_time.advance_by(std::time::Duration::from_secs_f64(1.0 / 64.0));
        world.insert_resource(fixed_time);
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_character_motor);
        schedule.run(&mut world);

        let controller = world.get::<CharacterController>(entity).unwrap();
        assert_eq!(controller.gravity, 0.0);
        assert_eq!(
            controller.speed,
            TACTICAL_QUICKSTEP_SPEED_METRES_PER_SECOND.metres_per_second()
        );
        assert!(
            world
                .get::<AccumulatedInput>(entity)
                .unwrap()
                .jumped
                .is_none()
        );
        let velocity = world.get::<LinearVelocity>(entity).unwrap();
        assert!(velocity.z < 0.0);
        assert!(velocity.y > 0.0);
    }

    #[test]
    fn motor_approaches_target_without_overshoot() {
        assert_eq!(
            approach_velocity(Vec2::ZERO, Vec2::X * 5.0, 8.0, 0.25),
            Vec2::X * 2.0
        );
        assert_eq!(
            approach_velocity(Vec2::X * 4.0, Vec2::ZERO, 8.0, 0.25),
            Vec2::X * 2.0
        );
        assert_eq!(approach_velocity(Vec2::ZERO, Vec2::X, 8.0, 0.25), Vec2::X);
    }

    fn integrated_right_angle_turn(speed: f32, agility: f32) -> (f32, f32, f32) {
        let grounded = avian3d::character_controller::move_and_slide::MoveHitData {
            entity: Entity::PLACEHOLDER,
            distance: 0.0,
            point1: Vec3::ZERO,
            point2: Vec3::ZERO,
            normal1: Vec3::Y,
            normal2: Vec3::NEG_Y,
            collision_distance: 0.0,
        };
        let defaults = crate::combat_config::TacticalCombatConfig::default();
        let movement = crate::combat_config::TacticalMovementConfig {
            speeds_metres_per_second: crate::combat_config::MovementSpeedsConfig {
                run: NonnegativeSpeed::from_metres_per_second(speed).unwrap(),
                ..defaults.movement.speeds_metres_per_second.clone()
            },
            ..defaults.movement.clone()
        };
        let config = crate::combat_config::TacticalCombatConfig {
            movement,
            ..defaults
        };
        let motor = &config.movement.motor;
        let turn_radius = ordinary_turn_radius(
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(speed)
                .unwrap(),
            agility_sprint_turn_radius(agility, motor),
        );
        let attributes = crate::player::TacticalAttributes(PlayerAttributeValues {
            left_leg_agility: agility,
            right_leg_agility: agility,
            ..default()
        });
        let mut world = World::new();
        let entity = world
            .spawn((
                AccumulatedInput {
                    // Identity controller orientation maps backward local input
                    // to world +Z, ninety degrees left of the initial +X run.
                    last_movement: Some(-Vec2::Y),
                    ..default()
                },
                CharacterController::default(),
                CharacterControllerState {
                    grounded: Some(grounded),
                    ..default()
                },
                LinearVelocity(Vec3::X * speed),
                Mass(80.0),
                Transform::default(),
                crate::player::Limbs::default(),
                crate::player::Skills::default(),
                crate::player::Stats::default(),
                attributes,
            ))
            .id();
        let delta_seconds = locomotion_sample_rate().interval().seconds();
        world.insert_resource(config);
        let mut fixed_time = Time::<Fixed>::from_hz(locomotion_sample_rate().hertz() as f64);
        fixed_time.advance_by(std::time::Duration::from_secs_f32(delta_seconds));
        world.insert_resource(fixed_time);
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_character_motor);

        let centre = Vec2::Y * turn_radius;
        let mut maximum_radius_error = 0.0_f32;
        let mut minimum_speed = speed;

        for _ in 0..128 {
            schedule.run(&mut world);
            let velocity = world.get::<LinearVelocity>(entity).unwrap().0;
            world.get_mut::<Transform>(entity).unwrap().translation += velocity * delta_seconds;
            let position = world.get::<Transform>(entity).unwrap().translation.xz();
            let horizontal_velocity = velocity.xz();
            minimum_speed = minimum_speed.min(horizontal_velocity.length());
            maximum_radius_error =
                maximum_radius_error.max(((position - centre).length() - turn_radius).abs());
            if horizontal_velocity.normalize().dot(Vec2::Y) > 0.99999 {
                break;
            }
        }

        let position = world.get::<Transform>(entity).unwrap().translation.xz();
        (
            position.length() / std::f32::consts::SQRT_2,
            minimum_speed,
            maximum_radius_error,
        )
    }

    #[test]
    fn ordinary_turns_combine_bounded_agility_with_nonlinear_speed_scaling() {
        let config = crate::combat_config::TacticalCombatConfig::default();
        let motor = &config.movement.motor;
        assert_eq!(agility_sprint_turn_radius(1.0, motor), 4.5);
        assert_eq!(agility_sprint_turn_radius(3.0, motor), 3.0);
        assert!((agility_sprint_turn_radius(5.0, motor) - 2.2).abs() < 1.0e-6);
        for agility_tenths in 10..50 {
            let agility = agility_tenths as f32 * 0.1;
            assert!(
                agility_sprint_turn_radius(agility + 0.1, motor)
                    < agility_sprint_turn_radius(agility, motor)
            );
        }

        let speeds = [
            REFERENCE_SPRINT_SPEED_METRES_PER_SECOND.metres_per_second(),
            TACTICAL_RUN_SPEED_METRES_PER_SECOND.metres_per_second(),
            3.75,
        ];
        let agilities = [1.0, 3.0, 5.0];
        let mut measured_radii = [[0.0_f32; 3]; 3];
        for (agility_index, agility) in agilities.into_iter().enumerate() {
            let sprint_radius = agility_sprint_turn_radius(agility, motor);
            for (speed_index, speed) in speeds.into_iter().enumerate() {
                let expected_radius = ordinary_turn_radius(
                    adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(
                        speed,
                    )
                    .unwrap(),
                    sprint_radius,
                );
                let (measured_radius, minimum_speed, maximum_radius_error) =
                    integrated_right_angle_turn(speed, agility);
                assert!(
                    (minimum_speed - speed).abs() < 1.0e-4,
                    "Agility {agility:.0} at {speed:.2} m/s fell to {minimum_speed:.3} m/s"
                );
                assert!(
                    (measured_radius - expected_radius).abs() < 0.06,
                    "Agility {agility:.0} at {speed:.2} m/s measured {measured_radius:.3} m instead of {expected_radius:.3} m"
                );
                assert!(
                    maximum_radius_error < speed / locomotion_sample_rate().hertz() + 0.01,
                    "Agility {agility:.0} at {speed:.2} m/s radius error was {maximum_radius_error:.3} m"
                );
                measured_radii[agility_index][speed_index] = measured_radius;
            }
        }

        for radii_at_agility in &measured_radii {
            assert!(radii_at_agility[0] > radii_at_agility[1]);
            assert!(radii_at_agility[1] > radii_at_agility[2]);
        }
        for ((average, john), olympian) in measured_radii[0]
            .iter()
            .zip(&measured_radii[1])
            .zip(&measured_radii[2])
        {
            assert!(average > john);
            assert!(john > olympian);
        }
    }

    #[test]
    fn stop_and_exact_reversal_still_brake_before_accelerating_backward() {
        let speed = REFERENCE_SPRINT_SPEED_METRES_PER_SECOND.metres_per_second();
        let delta_seconds = locomotion_sample_rate().interval().seconds();
        let braking_acceleration = 9.0;
        let limits = GroundVelocityLimits {
            drive_acceleration: 9.0,
            braking_acceleration,
            lateral_acceleration: 6.0,
            traction_acceleration: 9.0,
            turn_radius: 3.0,
        };
        let mut stopped = Vec2::X * speed;
        for _ in 0..128 {
            stopped = approach_ground_velocity(stopped, Vec2::ZERO, limits, delta_seconds);
        }
        assert_eq!(stopped, Vec2::ZERO);

        let first_reversal_tick =
            approach_ground_velocity(Vec2::X * speed, Vec2::Y, limits, delta_seconds);
        assert!(
            first_reversal_tick.y > 0.0,
            "a right-angle input should steer"
        );

        let exact_reversal_tick =
            approach_ground_velocity(Vec2::X * speed, Vec2::NEG_X * speed, limits, delta_seconds);
        assert!(exact_reversal_tick.x > 0.0 && exact_reversal_tick.x < speed);
        assert_eq!(exact_reversal_tick.y, 0.0);
    }

    #[test]
    fn ordinary_jump_height_maps_attributes_and_burden_to_ballistic_rise() {
        let config = crate::combat_config::TacticalCombatConfig::default();
        let motor = &config.movement.motor;
        let reference = config.movement.jump_heights_metres.ordinary;
        let average = ordinary_jump_height(reference, 70.0, 3.0, 3.0, motor);
        let john_unburdened = ordinary_jump_height(reference, 70.0, 4.0, 4.0, motor);
        let olympian_unburdened = ordinary_jump_height(reference, 70.0, 5.0, 5.0, motor);
        let john_burdened = ordinary_jump_height(reference, 90.0, 4.0, 4.0, motor);

        assert!((average - 0.30).abs() < 1.0e-6);
        assert!((john_unburdened - 0.39).abs() < 0.005);
        assert!((olympian_unburdened - 0.49).abs() < 0.005);
        assert!(john_burdened < john_unburdened);
    }

    #[test]
    fn quickstep_uses_a_finite_strength_and_mass_aware_force_curve() {
        let config = crate::combat_config::TacticalCombatConfig::default();
        let motor = &config.movement.motor;
        assert_eq!(quickstep_force_curve(0.0), 0.0);
        assert_eq!(quickstep_force_curve(0.5), 1.0);
        assert_eq!(quickstep_force_curve(1.0), 0.0);

        let reference_force = quickstep_peak_horizontal_force_newtons(70.0, 3.0, motor);
        let john_force = quickstep_peak_horizontal_force_newtons(70.0, 4.0, motor);
        let olympian_force = quickstep_peak_horizontal_force_newtons(70.0, 5.0, motor);
        assert!((reference_force / (70.0 * 9.81) - 4.5).abs() < 1.0e-5);
        assert!((john_force / (70.0 * 9.81) - 5.1).abs() < 1.0e-5);
        assert!((olympian_force / (70.0 * 9.81) - 5.7).abs() < 1.0e-5);
        assert!((quickstep_push_duration(3.0, motor).unwrap().seconds() - 0.40).abs() < 1.0e-6);
        assert!((quickstep_push_duration(4.0, motor).unwrap().seconds() - 0.38).abs() < 1.0e-6);
        assert!((quickstep_push_duration(5.0, motor).unwrap().seconds() - 0.36).abs() < 1.0e-6);

        let duration = quickstep_push_duration(4.0, motor).unwrap();
        let ticks = (duration.seconds() * locomotion_sample_rate().hertz()).ceil() as u64;
        let force_time = (0..ticks)
            .map(|tick| {
                quickstep_force_curve((tick as f32 + 0.5) / ticks as f32)
                    / locomotion_sample_rate().hertz()
            })
            .sum::<f32>();
        let unburdened_speed = john_force * force_time / 70.0;
        let burdened_speed = john_force * force_time / 93.9;
        assert!((unburdened_speed - 13.48).abs() < 0.05);
        assert!((burdened_speed - 10.05).abs() < 0.05);
        assert!(burdened_speed < unburdened_speed);
    }

    #[test]
    fn quickstep_force_follows_the_authored_profile_and_stops_at_scaled_distance() {
        let config = crate::combat_config::TacticalCombatConfig::default();
        let motor = &config.movement.motor;
        let target_distance = quickstep_target_displacement_metres(
            crate::player::CharacterDimensions::default().leg_length_metres,
            motor,
        ).unwrap();
        let duration = config.movement.maneuvers.quickstep_duration_seconds;
        let ticks = (duration.seconds() * locomotion_sample_rate().hertz()).round() as usize;
        let delta_seconds = locomotion_sample_rate().interval().seconds();
        let maximum_force = quickstep_peak_horizontal_force_newtons(70.0, 3.0, motor);
        let mut displacement = 0.0;
        let mut velocity = 0.0;
        let mut keyed_displacements = Vec::new();
        for tick in 0..ticks {
            if tick % (ticks / 4) == 0 {
                keyed_displacements.push(displacement);
            }
            let target = quickstep_motion_target(
                (tick + 1) as f32 / ticks as f32,
                target_distance,
                duration.elapsed(),
                motor.quickstep_authored_displacement_profile,
            )
            .unwrap();
            let force = quickstep_tracking_force_newtons(
                SignedLength::from_metres(displacement).unwrap(),
                adventuresim_core::linear_speed::SignedSpeed::from_metres_per_second(velocity)
                    .unwrap(),
                target,
                70.0,
                maximum_force,
                AnimationElapsed::from_seconds(delta_seconds).unwrap(),
            );
            velocity += force / 70.0 * delta_seconds;
            displacement += velocity * delta_seconds;
        }
        keyed_displacements.push(displacement);

        for (actual, normalized) in keyed_displacements
            .iter()
            .zip(motor.quickstep_authored_displacement_profile)
        {
            let expected = normalized * target_distance.metres();
            assert!(
                (actual - expected).abs() <= 0.05,
                "actual={actual} expected={expected}"
            );
        }
        assert!(velocity.abs() <= 0.20, "terminal velocity={velocity}");
    }

    #[test]
    fn attack_quickstep_uses_force_tracking_but_caps_travel_at_the_melee_gap() {
        let config = crate::combat_config::TacticalCombatConfig::default();
        let motor = &config.movement.motor;
        let duration = config.movement.maneuvers.quickstep_duration_seconds;
        let ticks = (duration.seconds() * locomotion_sample_rate().hertz()).round() as usize;
        let delta_seconds = locomotion_sample_rate().interval().seconds();
        let maximum_force = quickstep_peak_horizontal_force_newtons(70.0, 3.0, motor);
        let maximum_gap = quickstep_target_displacement_metres(
            motor.reference_quickstep_leg_length_metres,
            motor,
        ).unwrap().metres();
        for gap in [0.501, 0.70, maximum_gap - 0.01] {
            let mut displacement = 0.0;
            let mut velocity = 0.0;
            for tick in 0..ticks {
                let target = quickstep_motion_target(
                    (tick + 1) as f32 / ticks as f32,
                    SignedLength::from_metres(gap).unwrap(),
                    duration.elapsed(),
                    motor.quickstep_authored_displacement_profile,
                )
                .unwrap();
                let force = quickstep_tracking_force_newtons(
                    SignedLength::from_metres(displacement).unwrap(),
                    adventuresim_core::linear_speed::SignedSpeed::from_metres_per_second(velocity)
                        .unwrap(),
                    target,
                    70.0,
                    maximum_force,
                    AnimationElapsed::from_seconds(delta_seconds).unwrap(),
                );
                velocity += force / 70.0 * delta_seconds;
                displacement += velocity * delta_seconds;
            }

            assert!(
                (displacement - gap).abs() <= 0.05,
                "gap={gap} travel={displacement}"
            );
            assert!(displacement <= maximum_gap + 0.01);
        }
    }

    #[test]
    fn quickstep_launch_overcomes_adverse_ground_contact_drift() {
        let config = crate::combat_config::TacticalCombatConfig::default();
        let motor = &config.movement.motor;
        let duration = config.movement.maneuvers.quickstep_duration_seconds;
        let ticks = (duration.seconds() * locomotion_sample_rate().hertz()).round() as usize;
        let delta_seconds = locomotion_sample_rate().interval().seconds();
        let mass_kg = 70.0;
        let maximum_force = quickstep_peak_horizontal_force_newtons(mass_kg, 3.0, motor);
        let target = quickstep_motion_target(
            1.0 / ticks as f32,
            quickstep_target_displacement_metres(
                crate::player::CharacterDimensions::default().leg_length_metres,
                motor,
            ).unwrap(),
            duration.elapsed(),
            motor.quickstep_authored_displacement_profile,
        )
        .unwrap();
        // This is larger than the adverse residuals observed after Ahoy ground
        // projection, while still being physically indistinguishable from rest.
        let adverse_velocity = -0.0001;
        let force = quickstep_tracking_force_newtons(
            SignedLength::ZERO,
            adventuresim_core::linear_speed::SignedSpeed::from_metres_per_second(adverse_velocity)
                .unwrap(),
            target,
            mass_kg,
            maximum_force,
            AnimationElapsed::from_seconds(delta_seconds).unwrap(),
        );
        let velocity_after_force = adverse_velocity + force / mass_kg * delta_seconds;

        assert!(
            force >= maximum_force * 0.99,
            "launch force {force} was suppressed below maximum {maximum_force}"
        );
        assert!(
            velocity_after_force > 0.5,
            "launch did not overcome adverse drift: {velocity_after_force}"
        );
    }

    #[test]
    fn quickstep_distance_scales_with_leg_length() {
        let config = crate::combat_config::TacticalCombatConfig::default();
        let motor = &config.movement.motor;
        let reference = motor.reference_quickstep_leg_length_metres;
        assert!((quickstep_target_displacement_metres(reference, motor).unwrap().metres() - 1.0).abs() < 1.0e-6);
        assert!(
            (quickstep_target_displacement_metres(reference.scaled_by(1.2).unwrap(), motor).unwrap().metres() - 1.2).abs() < 1.0e-6
        );
        assert!(
            (quickstep_target_displacement_metres(reference.scaled_by(0.8).unwrap(), motor).unwrap().metres() - 0.8).abs() < 1.0e-6
        );
    }

    #[test]
    fn motor_force_respects_mass_injury_and_traction() {
        let config = crate::combat_config::TacticalCombatConfig::default();
        let motor = &config.movement.motor;
        let (reference_drive, reference_brake) = character_motor_force_limits(motor, 80.0, 1.0);
        let traction = 80.0 * motor.gravity_metres_per_second_squared * motor.traction_coefficient;
        assert_eq!(reference_drive, traction);
        assert_eq!(reference_brake, traction);

        let (burdened_drive, _) = character_motor_force_limits(motor, 160.0, 1.0);
        assert_eq!(burdened_drive, motor.reference_ground_drive_force_newtons);
        assert!(burdened_drive / 160.0 < reference_drive / 80.0);

        let (injured_drive, _) = character_motor_force_limits(motor, 80.0, 0.5);
        assert_eq!(
            injured_drive,
            motor.reference_ground_drive_force_newtons * 0.5
        );
        assert!(injured_drive < reference_drive);
    }

    #[test]
    fn grounded_motor_accelerates_and_brakes_the_canonical_velocity() {
        let mut world = World::new();
        let grounded = avian3d::character_controller::move_and_slide::MoveHitData {
            entity: Entity::PLACEHOLDER,
            distance: 0.0,
            point1: Vec3::ZERO,
            point2: Vec3::ZERO,
            normal1: Vec3::Y,
            normal2: Vec3::NEG_Y,
            collision_distance: 0.0,
        };
        let entity = world
            .spawn((
                AccumulatedInput {
                    last_movement: Some(Vec2::Y),
                    ..default()
                },
                CharacterController::default(),
                CharacterControllerState {
                    grounded: Some(grounded),
                    ..default()
                },
                LinearVelocity::default(),
                Mass(80.0),
            ))
            .id();
        world.insert_resource(crate::combat_config::TacticalCombatConfig::default());
        let mut fixed_time = Time::<Fixed>::from_hz(64.0);
        fixed_time.advance_by(std::time::Duration::from_secs_f64(1.0 / 64.0));
        world.insert_resource(fixed_time);
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_character_motor);

        schedule.run(&mut world);
        let first_speed = world.get::<LinearVelocity>(entity).unwrap().xz().length();
        assert!((0.13..0.15).contains(&first_speed));
        for _ in 0..63 {
            schedule.run(&mut world);
        }
        assert!(
            world.get::<LinearVelocity>(entity).unwrap().xz().length()
                <= TACTICAL_RUN_SPEED_METRES_PER_SECOND.metres_per_second()
        );

        world
            .get_mut::<AccumulatedInput>(entity)
            .unwrap()
            .last_movement = None;
        for _ in 0..64 {
            schedule.run(&mut world);
        }
        assert_eq!(
            world.get::<LinearVelocity>(entity).unwrap().xz(),
            Vec2::ZERO
        );
    }

    #[test]
    fn prone_and_supine_share_selected_pace_after_directional_scaling() {
        let mut world = World::new();
        let prone = world
            .spawn((
                AccumulatedInput {
                    last_movement: Some(Vec2::Y),
                    ..default()
                },
                CharacterController::default(),
                SkeletonState::default().with_body_state(crate::animation::BodyState::Prone),
                MovementPace::Sprint,
            ))
            .id();
        let supine = world
            .spawn((
                AccumulatedInput {
                    last_movement: Some(Vec2::Y),
                    ..default()
                },
                CharacterController::default(),
                SkeletonState::default().with_body_state(crate::animation::BodyState::Supine),
                MovementPace::Sprint,
            ))
            .id();
        let prone_walk = world
            .spawn((
                AccumulatedInput {
                    last_movement: Some(Vec2::Y),
                    ..default()
                },
                CharacterController::default(),
                SkeletonState::default().with_body_state(crate::animation::BodyState::Prone),
                MovementPace::Walk,
            ))
            .id();
        let prone_half_input = world
            .spawn((
                AccumulatedInput {
                    last_movement: Some(Vec2::Y * 0.5),
                    ..default()
                },
                CharacterController::default(),
                SkeletonState::default().with_body_state(crate::animation::BodyState::Prone),
                MovementPace::Walk,
            ))
            .id();
        let prone_jog = world
            .spawn((
                AccumulatedInput {
                    last_movement: Some(Vec2::Y),
                    ..default()
                },
                CharacterController::default(),
                SkeletonState::default().with_body_state(crate::animation::BodyState::Prone),
                MovementPace::Jog,
            ))
            .id();
        let mut transitioning = SkeletonState::default();
        assert!(transitioning.begin_posture_transition(
            crate::animation::PostureTransitionKind::UprightToProne,
            crate::animation::AnimationSampleTick::from(0u64),
            crate::animation::AnimationSampleSpan::from(10u64),
        ));
        let transition = world
            .spawn((
                AccumulatedInput {
                    last_movement: Some(Vec2::Y),
                    ..default()
                },
                CharacterController::default(),
                transitioning,
                MovementPace::Sprint,
            ))
            .id();
        let mut rolling =
            SkeletonState::default().with_body_state(crate::animation::BodyState::Prone);
        assert!(rolling.begin_posture_transition(
            crate::animation::PostureTransitionKind::ProneToSupine {
                direction: crate::animation::RollDirection::Left,
            },
            crate::animation::AnimationSampleTick::from(0u64),
            crate::animation::AnimationSampleSpan::from(10u64),
        ));
        let roll = world
            .spawn((
                AccumulatedInput {
                    last_movement: Some(-Vec2::X),
                    ..default()
                },
                CharacterController::default(),
                rolling,
                MovementPace::Sprint,
            ))
            .id();
        world.insert_resource(crate::combat_config::TacticalCombatConfig::default());
        world.insert_resource(Time::<Fixed>::from_hz(64.0));
        let mut schedule = Schedule::default();
        schedule.add_systems(apply_character_motor);
        schedule.run(&mut world);

        assert_eq!(
            world.get::<CharacterController>(prone).unwrap().speed,
            TACTICAL_PRONE_SPEED_METRES_PER_SECOND.metres_per_second()
        );
        assert_eq!(
            world
                .get::<CharacterController>(prone)
                .unwrap()
                .crouch_speed_scale,
            1.0,
            "downed collision crouching must not reduce the final crawl target"
        );
        assert_eq!(
            world.get::<CharacterController>(supine).unwrap().speed,
            TACTICAL_PRONE_SPEED_METRES_PER_SECOND.metres_per_second()
        );
        assert_eq!(
            world.get::<CharacterController>(prone_walk).unwrap().speed,
            TACTICAL_PRONE_WALK_SPEED_METRES_PER_SECOND.metres_per_second()
        );
        assert_eq!(
            world
                .get::<CharacterController>(prone_half_input)
                .unwrap()
                .speed,
            TACTICAL_PRONE_WALK_SPEED_METRES_PER_SECOND.metres_per_second() * 0.5
        );
        assert_eq!(
            world.get::<CharacterController>(prone_jog).unwrap().speed,
            3.75 / TACTICAL_PRONE_EFFORT_SCALE
        );
        assert_eq!(
            world.get::<CharacterController>(transition).unwrap().speed,
            0.0
        );
        assert_eq!(
            world.get::<CharacterController>(roll).unwrap().speed,
            TACTICAL_ROLL_SPEED_METRES_PER_SECOND.metres_per_second()
        );
    }
}
