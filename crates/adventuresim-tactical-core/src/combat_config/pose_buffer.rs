//! Pose sampling, calibration and measurement tuning authored in combat YAML.

use super::TacticalCombatConfigError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PoseBufferConfig {
    pub sample_hz: f32,
    pub maximum_observation_seconds: f32,
    pub authored_phase_calibration_intervals: u32,
    pub minimum_authored_foot_range_metres: f32,
    pub maximum_authored_foot_range_metres: f32,
    /// Seconds an inertialized transition takes to decay fully onto its new
    /// target. A joint already closing fast may finish sooner.
    pub inertial_blend_seconds: f32,
    pub cull_distance_metres: f32,
    pub cull_radius_metres: f32,
    pub authored_contact_plant_limit_metres: f32,
    pub authored_contact_height_fraction: f32,
    pub authored_contact_minimum_height_window_metres: f32,
    pub authored_contact_maximum_height_window_metres: f32,
}

impl PoseBufferConfig {
    pub(super) fn validate(&self) -> Result<(), TacticalCombatConfigError> {
        let finite_nonnegative = |value: f32| value.is_finite() && value >= 0.0;
        let pose = self;
        if ![
            pose.sample_hz,
            pose.maximum_observation_seconds,
            pose.minimum_authored_foot_range_metres,
            pose.maximum_authored_foot_range_metres,
            pose.inertial_blend_seconds,
            pose.cull_distance_metres,
            pose.cull_radius_metres,
            pose.authored_contact_plant_limit_metres,
            pose.authored_contact_height_fraction,
            pose.authored_contact_minimum_height_window_metres,
            pose.authored_contact_maximum_height_window_metres,
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
