//! Pose sampling, calibration and measurement tuning authored in combat YAML.

use super::TacticalCombatConfigError;
use crate::animation::PositiveAnimationDuration;
use adventuresim_building_generator::spatial_geometry::PositiveLength;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PoseBufferConfig {
    pub sample_hz: f32,
    /// Minimum extent of the numerical bake grid, after source-duration admission.
    pub minimum_baked_grid_seconds: PositiveAnimationDuration,
    /// Minimum signed stance displacement needed for a stable phase fit.
    pub minimum_phase_fit_displacement_metres: PositiveLength,
    pub maximum_observation_seconds: PositiveAnimationDuration,
    pub authored_phase_calibration_intervals: u32,
    pub minimum_authored_foot_range_metres: PositiveLength,
    pub maximum_authored_foot_range_metres: PositiveLength,
    /// Seconds an inertialized transition takes to decay fully onto its new
    /// target. A joint already closing fast may finish sooner.
    pub inertial_blend_seconds: PositiveAnimationDuration,
    pub cull_distance_metres: PositiveLength,
    pub cull_radius_metres: PositiveLength,
    pub authored_contact_plant_limit_metres: PositiveLength,
    pub authored_contact_height_fraction: f32,
    pub authored_contact_minimum_height_window_metres: PositiveLength,
    pub authored_contact_maximum_height_window_metres: PositiveLength,
}

impl PoseBufferConfig {
    pub(super) fn validate(&self) -> Result<(), TacticalCombatConfigError> {
        let finite_nonnegative = |value: f32| value.is_finite() && value >= 0.0;
        let pose = self;
        if ![
            pose.sample_hz,
            pose.maximum_observation_seconds.seconds(),
            pose.minimum_authored_foot_range_metres.metres(),
            pose.maximum_authored_foot_range_metres.metres(),
            pose.inertial_blend_seconds.seconds(),
            pose.cull_distance_metres.metres(),
            pose.cull_radius_metres.metres(),
            pose.authored_contact_plant_limit_metres.metres(),
            pose.authored_contact_height_fraction,
            pose.authored_contact_minimum_height_window_metres.metres(),
            pose.authored_contact_maximum_height_window_metres.metres(),
        ]
        .into_iter()
        .all(|value| finite_nonnegative(value) && value > 0.0)
            || pose.authored_phase_calibration_intervals < 3
            || pose.maximum_authored_foot_range_metres < pose.minimum_authored_foot_range_metres
            || pose.authored_contact_maximum_height_window_metres
                < pose.authored_contact_minimum_height_window_metres
        {
            return Err(TacticalCombatConfigError::Validation(
                "invalid pose buffer tuning",
            ));
        }
        Ok(())
    }
}
