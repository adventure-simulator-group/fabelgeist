//! Authored travel and inertial target policy in body-local coordinates.

use super::*;

/// Squared m/s comparison allowance for classifying measured deceleration.
const DECELERATION_SPEED_SQUARED_ALLOWANCE: f32 = 0.000_1;

pub(in crate::animation::procedural) fn body_response_target(
    velocity: BodyLocalVelocity,
    acceleration: BodyLocalAcceleration,
    braking_scale: BrakingLeanScale,
) -> AnimationValueResult<BodyResponseAngles> {
    velocity
        .planar_squared()
        .map_err(|_| AnimationValueError::Overflow {
            quantity: AnimationQuantity::BodyAngle,
        })?;
    let tuning = body_response_tuning();
    let travel = travel_lean(velocity, tuning)?;
    let inertial = inertial_lean(velocity, acceleration, braking_scale, tuning)?;
    // A cap-specific radian margin keeps degree reporting below the authored cap.
    let maximum = BodyAngularMagnitude::from_radians(
        tuning.maximum_response_degrees.to_radians() - BODY_RESPONSE_DEGREE_REPORT_MARGIN_RADIANS,
    )?;
    travel.checked_sum_capped(inertial, maximum)
}

pub(super) fn deceleration_lean_scale(
    previous_velocity: BodyLocalVelocity,
    velocity: BodyLocalVelocity,
    acceleration: BodyLocalAcceleration,
) -> AnimationValueResult<BrakingLeanScale> {
    let previous_squared = previous_velocity.planar_squared()?;
    let current_squared = velocity.planar_squared()?;
    let previous_velocity = previous_velocity.metres_per_second();
    let velocity = velocity.metres_per_second();
    let acceleration = acceleration.metres_per_second_squared();
    let previous_planar = previous_velocity.xz();
    let planar = velocity.xz();
    let planar_acceleration = acceleration.xz();
    if !previous_planar.dot(planar_acceleration).is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::BodyLocalVelocity,
        });
    }
    let is_decelerating = previous_planar.dot(planar_acceleration) < 0.0
        && current_squared <= previous_squared + DECELERATION_SPEED_SQUARED_ALLOWANCE;
    BrakingLeanScale::from_fraction(if is_decelerating {
        (planar.length() / run_locomotion_profile().reference_speed.metres_per_second())
            .clamp(0.0, 1.0)
    } else {
        1.0
    })
}

fn travel_lean(
    velocity: BodyLocalVelocity,
    tuning: BodyResponseConfig,
) -> AnimationValueResult<BodyResponseAngles> {
    let velocity = velocity.metres_per_second();

    // Tactical body forward is local +Z (the authored rig carries its own
    // facing correction). Authored locomotion keeps a straight back, so steady
    // travel supplies a pronounced base lean and acceleration adds a somewhat
    // stronger combined startup pose. Keeping most of the lean in travel avoids
    // a large initial snap followed by an almost upright steady run.
    let travel_pitch = BodyAngle::from_radians(
        velocity.z / run_locomotion_profile().reference_speed.metres_per_second()
            * tuning.steady_travel_lean_degrees.to_radians(),
    )?
    .radians()
    .clamp(
        -tuning.maximum_travel_lean_degrees.to_radians(),
        tuning.maximum_travel_lean_degrees.to_radians(),
    );
    let travel_roll = BodyAngle::from_radians(
        -velocity.x / run_locomotion_profile().reference_speed.metres_per_second()
            * tuning.steady_travel_lean_degrees.to_radians(),
    )?
    .radians()
    .clamp(
        -tuning.maximum_travel_lean_degrees.to_radians(),
        tuning.maximum_travel_lean_degrees.to_radians(),
    );
    BodyResponseAngles::from_radians(Vec2::new(travel_pitch, travel_roll))
}
fn inertial_lean(
    velocity: BodyLocalVelocity,
    acceleration: BodyLocalAcceleration,
    braking_scale: BrakingLeanScale,
    tuning: BodyResponseConfig,
) -> AnimationValueResult<BodyResponseAngles> {
    let velocity = velocity.metres_per_second();
    let acceleration = acceleration.metres_per_second_squared();
    // Acceleration is an early accent rather than a second full pose. Fade it
    // almost completely into the stable velocity posture, so reaching target
    // speed cannot release a late, visibly separate lean.
    let speed_ratio =
        velocity.xz().length() / run_locomotion_profile().reference_speed.metres_per_second();
    if !speed_ratio.is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::BodyAngle,
        });
    }
    let speed_fraction = speed_ratio.clamp(0.0, 1.0);
    let startup_inertial_scale = tuning
        .startup_inertial_lean_scale
        .lerp(tuning.sustained_inertial_lean_scale, speed_fraction);
    let inertial_pitch = if acceleration.z > 0.0 {
        BodyAngle::from_radians(
            acceleration.z / tuning.forward_reference_acceleration_metres_per_second_squared
                * tuning.forward_acceleration_lean_degrees.to_radians(),
        )?
        .radians()
        .clamp(
            0.0,
            tuning
                .maximum_forward_acceleration_lean_degrees
                .to_radians(),
        ) * startup_inertial_scale
    } else {
        BodyAngle::from_radians(
            acceleration.z / tuning.forward_reference_acceleration_metres_per_second_squared
                * tuning.braking_acceleration_lean_degrees.to_radians(),
        )?
        .radians()
        .clamp(-tuning.maximum_braking_lean_degrees.to_radians(), 0.0)
            * braking_scale.fraction()
    };
    // Turning should read clearly without the extreme motorcycle-like bank of
    // the first stronger-lean pass. Keep lateral travel posture unchanged and
    // scale only acceleration-driven turning response to 60% of that tuning.
    let inertial_roll = BodyAngle::from_radians(
        -acceleration.x / tuning.lateral_reference_acceleration_metres_per_second_squared
            * tuning.lateral_acceleration_lean_degrees.to_radians(),
    )?
    .radians()
    .clamp(
        -tuning
            .maximum_lateral_acceleration_lean_degrees
            .to_radians(),
        tuning
            .maximum_lateral_acceleration_lean_degrees
            .to_radians(),
    ) * startup_inertial_scale;
    BodyResponseAngles::from_radians(Vec2::new(inertial_pitch, inertial_roll))
}
