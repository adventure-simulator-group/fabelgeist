//! Native horizontal velocity projection for an admitted posture launch.

use super::*;
use adventuresim_building_generator::spatial_geometry::Radians;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DiveLaunch {
    pub(super) travel_direction: DiveDirection,
    pub(super) trajectory: DiveTrajectory,
}

pub(super) fn dive_horizontal_velocity(
    yaw: Radians,
    direction: DiveDirection,
    speed: adventuresim_core::linear_speed::NonnegativeSpeed,
) -> Vec3 {
    let local = match direction {
        DiveDirection::Forward => Vec3::NEG_Z,
        DiveDirection::Backward => Vec3::Z,
        DiveDirection::Left => Vec3::NEG_X,
        DiveDirection::Right => Vec3::X,
    };
    Quat::from_rotation_y(yaw.radians()) * local * speed.metres_per_second()
}

pub(super) fn apply_dive_launch_velocity(
    velocity: &mut LinearVelocity,
    yaw: Radians,
    launch: DiveLaunch,
    speed: adventuresim_core::linear_speed::NonnegativeSpeed,
) {
    // Sliding is a posture change, not a fresh launch. Preserve the complete
    // sprint velocity; gravity and the later body-drag phase own its evolution.
    if launch.trajectory == DiveTrajectory::GroundedSlide {
        return;
    }
    let horizontal = dive_horizontal_velocity(yaw, launch.travel_direction, speed);
    velocity.x = horizontal.x;
    velocity.z = horizontal.z;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dive_launch_velocity_follows_the_requested_camera_relative_direction() {
        let yaw = Radians::new(std::f32::consts::FRAC_PI_2).unwrap();
        assert!(
            dive_horizontal_velocity(
                yaw,
                DiveDirection::Forward,
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(7.0)
                    .unwrap()
            )
            .x < -6.9
        );
        assert!(
            dive_horizontal_velocity(
                yaw,
                DiveDirection::Backward,
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(7.0)
                    .unwrap()
            )
            .x > 6.9
        );
        assert!(
            dive_horizontal_velocity(
                Radians::new(0.0).unwrap(),
                DiveDirection::Left,
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(7.0)
                    .unwrap()
            )
            .x < -6.9
        );
        assert!(
            dive_horizontal_velocity(
                Radians::new(0.0).unwrap(),
                DiveDirection::Right,
                adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(7.0)
                    .unwrap()
            )
            .x > 6.9
        );
    }
    #[test]
    fn grounded_slide_preserves_the_complete_sprint_velocity() {
        let mut velocity = LinearVelocity(Vec3::new(1.0, 4.0, 2.0));
        apply_dive_launch_velocity(
            &mut velocity,
            Radians::new(0.0).unwrap(),
            DiveLaunch {
                travel_direction: DiveDirection::Forward,
                trajectory: DiveTrajectory::GroundedSlide,
            },
            adventuresim_core::linear_speed::NonnegativeSpeed::from_metres_per_second(7.0).unwrap(),
        );

        assert_eq!(velocity.0, Vec3::new(1.0, 4.0, 2.0));
    }
}
