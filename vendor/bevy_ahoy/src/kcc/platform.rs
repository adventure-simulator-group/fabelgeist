//! Point motion is measured relative to the platform centre, not world origin.
use bevy_math::{Quat, Vec3};

pub(super) fn point_displacement(
    offset_from_centre: Vec3,
    linear_velocity: Vec3,
    angular_velocity: Vec3,
    delta_seconds: f32,
) -> Vec3 {
    // Equivalent rigid-body point motion avoids inverse/forward pose rounding.
    // In particular, a stationary platform contributes exactly zero drift.
    let rotation = Quat::from_scaled_axis(angular_velocity * delta_seconds);
    linear_velocity * delta_seconds + (rotation * offset_from_centre - offset_from_centre)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_platforms_have_no_motion_at_near_and_geographic_offsets() {
        for offset in [
            Vec3::ZERO,
            Vec3::new(-236.68556, 25.364708, -344.1464),
            Vec3::new(24000.31, -511.127, -17000.68),
        ] {
            for delta in [1.0 / 64.0, 1.0 / 120.0] {
                assert_eq!(
                    point_displacement(offset, Vec3::ZERO, Vec3::ZERO, delta),
                    Vec3::ZERO
                );
            }
        }
    }

    #[test]
    fn translating_platforms_carry_the_point_without_world_coordinate_cancellation() {
        let velocity = Vec3::new(0.25, -0.125, 1.5);
        let delta = 1.0 / 64.0;
        assert_eq!(
            point_displacement(Vec3::splat(24000.0), velocity, Vec3::ZERO, delta),
            velocity * delta
        );
    }

    #[test]
    fn rotating_platforms_use_the_world_angular_velocity_about_the_centre() {
        let moved = point_displacement(
            Vec3::X * 2.0,
            Vec3::Y,
            Vec3::Y * std::f32::consts::FRAC_PI_2,
            1.0,
        );
        assert!(moved.distance(Vec3::new(-2.0, 1.0, -2.0)) < 0.00001);
    }
}
