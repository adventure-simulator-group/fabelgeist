//! Native horizontal velocity projection for an admitted posture launch.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DiveLaunch {
    pub(super) travel_direction: DiveDirection,
    pub(super) trajectory: DiveTrajectory,
}

pub(super) fn dive_horizontal_velocity(yaw: f32, direction: DiveDirection, speed: f32) -> Vec3 {
    let local = match direction {
        DiveDirection::Forward => Vec3::NEG_Z,
        DiveDirection::Backward => Vec3::Z,
        DiveDirection::Left => Vec3::NEG_X,
        DiveDirection::Right => Vec3::X,
    };
    Quat::from_rotation_y(yaw) * local * speed
}

pub(super) fn apply_dive_launch_velocity(
    velocity: &mut LinearVelocity,
    yaw: f32,
    launch: DiveLaunch,
    speed: f32,
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
        let yaw = std::f32::consts::FRAC_PI_2;
        assert!(dive_horizontal_velocity(yaw, DiveDirection::Forward, 7.0).x < -6.9);
        assert!(dive_horizontal_velocity(yaw, DiveDirection::Backward, 7.0).x > 6.9);
        assert!(dive_horizontal_velocity(0.0, DiveDirection::Left, 7.0).x < -6.9);
        assert!(dive_horizontal_velocity(0.0, DiveDirection::Right, 7.0).x > 6.9);
    }
    #[test]
    fn grounded_slide_preserves_the_complete_sprint_velocity() {
        let mut velocity = LinearVelocity(Vec3::new(1.0, 4.0, 2.0));
        apply_dive_launch_velocity(
            &mut velocity,
            0.0,
            DiveLaunch {
                travel_direction: DiveDirection::Forward,
                trajectory: DiveTrajectory::GroundedSlide,
            },
            7.0,
        );

        assert_eq!(velocity.0, Vec3::new(1.0, 4.0, 2.0));
    }
}
