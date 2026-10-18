//! Authored travel and inertial target policy in body-local coordinates.

use super::*;

pub(in crate::animation::procedural) fn body_response_target(
    velocity: BodyLocalVelocity,
    acceleration: BodyLocalAcceleration,
    braking_scale: f32,
) -> AnimationValueResult<BodyResponseAngles> {
    let velocity = velocity.metres_per_second();
    let acceleration = acceleration.metres_per_second_squared();
    if !velocity.xz().length_squared().is_finite() || !braking_scale.is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::BodyAngle,
        });
    }

    // Tactical body forward is local +Z (the authored rig carries its own
    // facing correction). Authored locomotion keeps a straight back, so steady
    // travel supplies a pronounced base lean and acceleration adds a somewhat
    // stronger combined startup pose. Keeping most of the lean in travel avoids
    // a large initial snap followed by an almost upright steady run.
    let travel_pitch = BodyAngle::from_radians(
        velocity.z / run_locomotion_profile().reference_speed
            * body_response_tuning()
                .steady_travel_lean_degrees
                .to_radians(),
    )?
    .radians()
    .clamp(-18.0_f32.to_radians(), 18.0_f32.to_radians());
    let travel_roll = BodyAngle::from_radians(
        -velocity.x / run_locomotion_profile().reference_speed
            * body_response_tuning()
                .steady_travel_lean_degrees
                .to_radians(),
    )?
    .radians()
    .clamp(-18.0_f32.to_radians(), 18.0_f32.to_radians());
    // Acceleration is an early accent rather than a second full pose. Fade it
    // almost completely into the stable velocity posture, so reaching target
    // speed cannot release a late, visibly separate lean.
    let speed_ratio = velocity.xz().length() / run_locomotion_profile().reference_speed;
    if !speed_ratio.is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::BodyAngle,
        });
    }
    let speed_fraction = speed_ratio.clamp(0.0, 1.0);
    let startup_inertial_scale = body_response_tuning().startup_inertial_lean_scale.lerp(
        body_response_tuning().sustained_inertial_lean_scale,
        speed_fraction,
    );
    let inertial_pitch = if acceleration.z > 0.0 {
        BodyAngle::from_radians(
            acceleration.z / 12.0
                * body_response_tuning()
                    .forward_acceleration_lean_degrees
                    .to_radians(),
        )?
        .radians()
        .clamp(0.0, 14.0_f32.to_radians())
            * startup_inertial_scale
    } else {
        BodyAngle::from_radians(acceleration.z / 12.0 * 18.0_f32.to_radians())?
            .radians()
            .clamp(-22.0_f32.to_radians(), 0.0)
            * braking_scale.clamp(0.0, 1.0)
    };
    // Turning should read clearly without the extreme motorcycle-like bank of
    // the first stronger-lean pass. Keep lateral travel posture unchanged and
    // scale only acceleration-driven turning response to 60% of that tuning.
    let inertial_roll = BodyAngle::from_radians(
        -acceleration.x / 10.0
            * body_response_tuning()
                .lateral_acceleration_lean_degrees
                .to_radians(),
    )?
    .radians()
    .clamp(-8.0_f32.to_radians(), 8.0_f32.to_radians())
        * startup_inertial_scale;
    let pitch = travel_pitch + inertial_pitch;
    let roll = travel_roll + inertial_roll;
    let response = Vec2::new(pitch, roll);
    // Leave a sub-milliradian numerical margin so degree conversion cannot
    // report a value microscopically above the documented 30-degree cap.
    let maximum_response = 30.0_f32.to_radians() - 0.000001;
    if !response.length_squared().is_finite() {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::BodyAngle,
        });
    }
    let response = if response.length_squared() > maximum_response * maximum_response {
        response.normalize_or_zero() * maximum_response
    } else {
        response
    };
    BodyResponseAngles::from_radians(response)
}

pub(super) fn deceleration_lean_scale(
    previous_velocity: BodyLocalVelocity,
    velocity: BodyLocalVelocity,
    acceleration: BodyLocalAcceleration,
) -> AnimationValueResult<f32> {
    let previous_velocity = previous_velocity.metres_per_second();
    let velocity = velocity.metres_per_second();
    let acceleration = acceleration.metres_per_second_squared();
    let previous_planar = previous_velocity.xz();
    let planar = velocity.xz();
    let planar_acceleration = acceleration.xz();
    if !previous_planar.dot(planar_acceleration).is_finite()
        || !planar.length_squared().is_finite()
        || !previous_planar.length_squared().is_finite()
    {
        return Err(AnimationValueError::Overflow {
            quantity: AnimationQuantity::BodyLocalVelocity,
        });
    }
    let is_decelerating = previous_planar.dot(planar_acceleration) < 0.0
        && planar.length_squared() <= previous_planar.length_squared() + 0.000_1;
    Ok(if is_decelerating {
        (planar.length() / run_locomotion_profile().reference_speed).clamp(0.0, 1.0)
    } else {
        1.0
    })
}
