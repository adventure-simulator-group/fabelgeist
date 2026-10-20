use super::MeleeMovementAction;
use crate::elapsed::AnimationElapsed;
use crate::linear_speed::{NonnegativeSpeed, SignedSpeed, SpeedError, SpeedResult};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct AxisMotion {
    pub velocity_before_metres_per_second: SignedSpeed,
    pub velocity_after_metres_per_second: SignedSpeed,
    pub speed_limit_metres_per_second: NonnegativeSpeed,
    pub displacement_metres: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct OpposedMovement {
    pub distance_before_metres: f32,
    pub distance_after_metres: f32,
    pub first: AxisMotion,
    pub second: AxisMotion,
    pub elapsed_seconds: AnimationElapsed,
}

pub(super) fn ground_drive_acceleration(
    force_newtons: f32,
    leg_strength: f32,
    reference_leg_strength: f32,
    body_mass_kg: f32,
    equipment_mass_kg: f32,
    gravity_metres_per_second_squared: f32,
    traction_coefficient: f32,
) -> f32 {
    let mass = (body_mass_kg + equipment_mass_kg).max(1.0);
    let drive_force =
        force_newtons.max(0.0) * (leg_strength / reference_leg_strength.max(1.0)).max(0.0);
    (drive_force / mass)
        .min(gravity_metres_per_second_squared.max(0.0) * traction_coefficient.max(0.0))
}

#[expect(
    clippy::too_many_arguments,
    reason = "the pure integrator receives both actors' independent physical bounds"
)]
pub(super) fn integrate_opposed_movement(
    distance_metres: f32,
    first_velocity_metres_per_second: SignedSpeed,
    first_intent: MeleeMovementAction,
    first_maximum_speed_metres_per_second: NonnegativeSpeed,
    second_velocity_metres_per_second: SignedSpeed,
    second_intent: MeleeMovementAction,
    second_maximum_speed_metres_per_second: NonnegativeSpeed,
    first_maximum_acceleration_metres_per_second_squared: f32,
    second_maximum_acceleration_metres_per_second_squared: f32,
    elapsed_seconds: AnimationElapsed,
    minimum_distance_metres: f32,
    maximum_distance_metres: f32,
) -> SpeedResult<OpposedMovement> {
    let mut first = integrate_axis(
        first_velocity_metres_per_second,
        first_intent,
        first_maximum_speed_metres_per_second,
        first_maximum_acceleration_metres_per_second_squared,
        elapsed_seconds,
    )?;
    let mut second = integrate_axis(
        second_velocity_metres_per_second,
        second_intent,
        second_maximum_speed_metres_per_second,
        second_maximum_acceleration_metres_per_second_squared,
        elapsed_seconds,
    )?;
    let raw_after = distance_metres + first.displacement_metres + second.displacement_metres;
    let bounded_after = raw_after.clamp(minimum_distance_metres, maximum_distance_metres);
    let raw_delta = raw_after - distance_metres;
    let bounded_delta = bounded_after - distance_metres;
    if (raw_after - bounded_after).abs() > 0.0 && raw_delta.abs() > 0.0 {
        let travel_fraction = (bounded_delta / raw_delta).clamp(0.0, 1.0);
        first.displacement_metres *= travel_fraction;
        second.displacement_metres *= travel_fraction;
        if raw_after < minimum_distance_metres {
            first.velocity_after_metres_per_second = first
                .velocity_after_metres_per_second
                .max(SignedSpeed::ZERO);
            second.velocity_after_metres_per_second = second
                .velocity_after_metres_per_second
                .max(SignedSpeed::ZERO);
        } else {
            first.velocity_after_metres_per_second = first
                .velocity_after_metres_per_second
                .min(SignedSpeed::ZERO);
            second.velocity_after_metres_per_second = second
                .velocity_after_metres_per_second
                .min(SignedSpeed::ZERO);
        }
    }
    Ok(OpposedMovement {
        distance_before_metres: distance_metres,
        distance_after_metres: bounded_after,
        first,
        second,
        elapsed_seconds: elapsed_seconds.max(AnimationElapsed::ZERO),
    })
}

/// Integrates one actor's signed contribution to separation over real elapsed
/// time. Closing velocity is negative and retreat velocity is positive. The
/// trapezoidal result is exact while the authored acceleration limit is active
/// and while velocity is capped at guarded movement speed.
fn integrate_axis(
    velocity_metres_per_second: SignedSpeed,
    intent: MeleeMovementAction,
    maximum_speed_metres_per_second: NonnegativeSpeed,
    maximum_acceleration_metres_per_second_squared: f32,
    elapsed_seconds: AnimationElapsed,
) -> SpeedResult<AxisMotion> {
    let elapsed_seconds = elapsed_seconds.seconds().max(0.0);
    let maximum_speed = maximum_speed_metres_per_second.max(NonnegativeSpeed::ZERO);
    let maximum_speed_scalar = maximum_speed.metres_per_second();
    let acceleration = maximum_acceleration_metres_per_second_squared.max(0.0);
    let before = velocity_metres_per_second
        .metres_per_second()
        .clamp(-maximum_speed_scalar, maximum_speed_scalar);
    let target = intent.target_velocity(maximum_speed).metres_per_second();
    let delta = target - before;
    let acceleration_seconds = if acceleration > 0.0 {
        (delta.abs() / acceleration).min(elapsed_seconds)
    } else {
        elapsed_seconds
    };
    let acceleration_direction = delta.signum();
    let after_acceleration = before + acceleration_direction * acceleration * acceleration_seconds;
    if !after_acceleration.is_finite() {
        return Err(SpeedError::Overflow);
    }
    let after = if acceleration_seconds < elapsed_seconds {
        target
    } else {
        after_acceleration.clamp(before.min(target), before.max(target))
    };
    let accelerating_displacement = (before + after_acceleration) * 0.5 * acceleration_seconds;
    let constant_displacement = target * (elapsed_seconds - acceleration_seconds);
    Ok(AxisMotion {
        velocity_before_metres_per_second: SignedSpeed::from_metres_per_second(before)?,
        velocity_after_metres_per_second: SignedSpeed::from_metres_per_second(after)?,
        speed_limit_metres_per_second: maximum_speed,
        displacement_metres: accelerating_displacement + constant_displacement,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_acceleration_coasts_and_zero_elapsed_time_does_not_advance() {
        let coast = integrate_axis(
            crate::linear_speed::SignedSpeed::from_metres_per_second(1.0).unwrap(),
            MeleeMovementAction::Close,
            crate::linear_speed::NonnegativeSpeed::from_metres_per_second(2.0).unwrap(),
            0.0,
            AnimationElapsed::from_seconds(0.5).unwrap(),
        )
        .unwrap();
        assert_eq!(
            coast.velocity_after_metres_per_second.metres_per_second(),
            1.0
        );
        assert_eq!(coast.displacement_metres, 0.5);
        let paused = integrate_axis(
            crate::linear_speed::SignedSpeed::from_metres_per_second(1.0).unwrap(),
            MeleeMovementAction::Close,
            crate::linear_speed::NonnegativeSpeed::from_metres_per_second(2.0).unwrap(),
            1.0,
            AnimationElapsed::ZERO,
        )
        .unwrap();
        assert_eq!(
            paused.velocity_after_metres_per_second.metres_per_second(),
            1.0
        );
        assert_eq!(paused.displacement_metres, 0.0);
    }

    const SPEED: NonnegativeSpeed = match NonnegativeSpeed::from_metres_per_second(2.0) {
        Ok(speed) => speed,
        Err(_) => panic!("invalid fixture speed"),
    };
    const ACCELERATION: f32 = 24.0;

    #[test]
    fn displacement_never_exceeds_speed_times_elapsed() {
        for intent in [
            MeleeMovementAction::Close,
            MeleeMovementAction::Hold,
            MeleeMovementAction::Retreat,
        ] {
            let motion = integrate_axis(
                crate::linear_speed::SignedSpeed::from_metres_per_second(0.0).unwrap(),
                intent,
                SPEED,
                ACCELERATION,
                AnimationElapsed::from_seconds(0.25).unwrap(),
            )
            .unwrap();
            assert!(motion.displacement_metres.abs() <= SPEED.metres_per_second() * 0.25 + 1.0e-6);
        }
    }

    #[test]
    fn acceleration_integration_is_substep_invariant() {
        let whole = integrate_axis(
            crate::linear_speed::SignedSpeed::from_metres_per_second(0.0).unwrap(),
            MeleeMovementAction::Close,
            SPEED,
            ACCELERATION,
            AnimationElapsed::from_seconds(0.2).unwrap(),
        )
        .unwrap();
        let first = integrate_axis(
            crate::linear_speed::SignedSpeed::from_metres_per_second(0.0).unwrap(),
            MeleeMovementAction::Close,
            SPEED,
            ACCELERATION,
            AnimationElapsed::from_seconds(0.1).unwrap(),
        )
        .unwrap();
        let second = integrate_axis(
            first.velocity_after_metres_per_second,
            MeleeMovementAction::Close,
            SPEED,
            ACCELERATION,
            AnimationElapsed::from_seconds(0.1).unwrap(),
        )
        .unwrap();
        assert!(
            (whole.displacement_metres - (first.displacement_metres + second.displacement_metres))
                .abs()
                < 1.0e-6
        );
        assert!(
            (whole.velocity_after_metres_per_second.metres_per_second()
                - second.velocity_after_metres_per_second.metres_per_second())
            .abs()
                < 1.0e-6
        );
    }

    #[test]
    fn added_body_or_equipment_mass_reduces_ground_acceleration() {
        let light = ground_drive_acceleration(1_000.0, 2.0, 3.0, 70.0, 5.0, 9.81, 0.9);
        let armored = ground_drive_acceleration(1_000.0, 2.0, 3.0, 80.0, 25.0, 9.81, 0.9);
        assert!(light > armored);
    }

    #[test]
    fn opposed_intent_cannot_reverse_velocity_instantaneously() {
        let motion = integrate_axis(
            crate::linear_speed::SignedSpeed::from_metres_per_second(2.0).unwrap(),
            MeleeMovementAction::Close,
            SPEED,
            12.0,
            AnimationElapsed::from_seconds(0.01).unwrap(),
        )
        .unwrap();
        assert!(motion.velocity_after_metres_per_second.metres_per_second() > 0.0);
        assert!(motion.displacement_metres > 0.0);
    }

    #[test]
    fn opposed_intents_use_signed_simultaneous_displacement() {
        let mutual_close = integrate_opposed_movement(
            2.0,
            crate::linear_speed::SignedSpeed::from_metres_per_second(0.0).unwrap(),
            MeleeMovementAction::Close,
            SPEED,
            crate::linear_speed::SignedSpeed::from_metres_per_second(0.0).unwrap(),
            MeleeMovementAction::Close,
            SPEED,
            ACCELERATION,
            ACCELERATION,
            crate::elapsed::AnimationElapsed::from_seconds(0.25).unwrap(),
            0.0,
            3.0,
        )
        .unwrap();
        let chase = integrate_opposed_movement(
            2.0,
            crate::linear_speed::SignedSpeed::from_metres_per_second(0.0).unwrap(),
            MeleeMovementAction::Close,
            SPEED,
            crate::linear_speed::SignedSpeed::from_metres_per_second(0.0).unwrap(),
            MeleeMovementAction::Retreat,
            SPEED,
            ACCELERATION,
            ACCELERATION,
            crate::elapsed::AnimationElapsed::from_seconds(0.25).unwrap(),
            0.0,
            3.0,
        )
        .unwrap();
        assert!(mutual_close.distance_after_metres < 2.0);
        assert!((chase.distance_after_metres - 2.0).abs() < 1.0e-6);
    }

    #[test]
    fn body_collision_prevents_center_overlap_and_stops_closing_velocity() {
        let movement = integrate_opposed_movement(
            0.81,
            SignedSpeed::from(SPEED).negated(),
            MeleeMovementAction::Close,
            SPEED,
            SignedSpeed::from(SPEED).negated(),
            MeleeMovementAction::Close,
            SPEED,
            ACCELERATION,
            ACCELERATION,
            crate::elapsed::AnimationElapsed::from_seconds(0.1).unwrap(),
            0.8,
            3.0,
        )
        .unwrap();
        assert!((movement.distance_after_metres - 0.8).abs() < 1.0e-6);
        assert!(
            movement
                .first
                .velocity_after_metres_per_second
                .metres_per_second()
                >= 0.0
        );
        assert!(
            movement
                .second
                .velocity_after_metres_per_second
                .metres_per_second()
                >= 0.0
        );
        assert!(movement.first.displacement_metres.abs() <= SPEED.metres_per_second() * 0.1);
        assert!(movement.second.displacement_metres.abs() <= SPEED.metres_per_second() * 0.1);
    }

    #[test]
    fn body_collision_floor_is_substep_invariant_and_cannot_tunnel() {
        let whole = integrate_opposed_movement(
            1.0,
            SignedSpeed::from(SPEED).negated(),
            MeleeMovementAction::Close,
            SPEED,
            SignedSpeed::from(SPEED).negated(),
            MeleeMovementAction::Close,
            SPEED,
            ACCELERATION,
            ACCELERATION,
            crate::elapsed::AnimationElapsed::from_seconds(0.2).unwrap(),
            0.8,
            3.0,
        )
        .unwrap();
        let first = integrate_opposed_movement(
            1.0,
            SignedSpeed::from(SPEED).negated(),
            MeleeMovementAction::Close,
            SPEED,
            SignedSpeed::from(SPEED).negated(),
            MeleeMovementAction::Close,
            SPEED,
            ACCELERATION,
            ACCELERATION,
            crate::elapsed::AnimationElapsed::from_seconds(0.1).unwrap(),
            0.8,
            3.0,
        )
        .unwrap();
        let second = integrate_opposed_movement(
            first.distance_after_metres,
            first.first.velocity_after_metres_per_second,
            MeleeMovementAction::Close,
            SPEED,
            first.second.velocity_after_metres_per_second,
            MeleeMovementAction::Close,
            SPEED,
            ACCELERATION,
            ACCELERATION,
            crate::elapsed::AnimationElapsed::from_seconds(0.1).unwrap(),
            0.8,
            3.0,
        )
        .unwrap();
        assert!((whole.distance_after_metres - 0.8).abs() < 1.0e-6);
        assert!((second.distance_after_metres - 0.8).abs() < 1.0e-6);
    }
}
