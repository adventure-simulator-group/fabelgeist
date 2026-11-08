//! Native Ahoy motor settings before tactical target-speed selection.

use crate::{
    animation::{DiveTrajectory, PostureTransitionKind, SkeletonState},
    combat_config::{CharacterMotorConfig, JumpHeightsConfig},
};
use bevy_ahoy::CharacterController;

pub(super) fn maneuver_jump_height(
    skeleton: Option<&SkeletonState>,
    heights: &JumpHeightsConfig,
) -> Option<f32> {
    skeleton.and_then(
        |skeleton| match skeleton.posture_transition().map(|state| state.kind()) {
            Some(PostureTransitionKind::DiveToDowned {
                trajectory: DiveTrajectory::Airborne,
                ..
            }) => Some(heights.dive),
            Some(PostureTransitionKind::DiveToDowned {
                trajectory: DiveTrajectory::GroundedSlide,
                ..
            }) => Some(0.0),
            _ => None,
        },
    )
}

pub(super) fn apply_motor_config(
    controller: &mut CharacterController,
    motor: &CharacterMotorConfig,
) {
    controller.acceleration_hz = 0.0;
    controller.air_acceleration_hz = 0.0;
    controller.water_acceleration_hz = 0.0;
    controller.friction_hz = 0.0;
    controller.stop_speed = 0.0;
    controller.gravity = motor.gravity_metres_per_second_squared;
    controller.step_size = motor.maximum_step_height_metres;
    controller.min_walk_cos = motor.maximum_walkable_slope_degrees.to_radians().cos();
}
