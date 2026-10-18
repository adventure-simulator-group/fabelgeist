//! Procedural height and body-response tuning authored in combat YAML.

use super::TacticalCombatConfigError;
use serde::{Deserialize, Serialize};

/// Radian allowance for reporting a capped body angle after degree conversion.
pub const BODY_RESPONSE_DEGREE_REPORT_MARGIN_RADIANS: f32 = 0.000001;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProceduralAnimationConfig {
    pub guarded_look_joint_limit_degrees: f32,
    pub guarded_look_joint_count: f32,
    pub dive_pelvis_lean_degrees: f32,
    pub height_transition_speed_metres_per_second: f32,
    pub locomotion_stop_height_speed_metres_per_second: f32,
    pub authored_ordinary_passing_rise_metres: f32,
    pub minimum_landing_absorption_speed_metres_per_second: f32,
    pub landing_compression_arrival_tolerance_metres: f32,
    pub landing_plant_minimum_compression_metres: f32,
    pub body_response: BodyResponseConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyResponseConfig {
    pub maximum_presentation_sample_gap_ticks: u64,
    pub steady_travel_lean_degrees: f32,
    pub maximum_travel_lean_degrees: f32,
    pub forward_reference_acceleration_metres_per_second_squared: f32,
    pub maximum_forward_acceleration_lean_degrees: f32,
    pub braking_acceleration_lean_degrees: f32,
    pub maximum_braking_lean_degrees: f32,
    pub lateral_reference_acceleration_metres_per_second_squared: f32,
    pub maximum_lateral_acceleration_lean_degrees: f32,
    pub maximum_response_degrees: f32,
    pub forward_acceleration_lean_degrees: f32,
    pub lateral_acceleration_lean_degrees: f32,
    pub startup_inertial_lean_scale: f32,
    pub sustained_inertial_lean_scale: f32,
    pub degrees_per_second: f32,
    pub smooth_time_seconds: f32,
    pub acceleration_attack_response_per_second: f32,
    pub acceleration_release_response_per_second: f32,
    pub maximum_frame_seconds: f32,
}

impl ProceduralAnimationConfig {
    pub(super) fn validate(&self) -> Result<(), TacticalCombatConfigError> {
        let finite_nonnegative = |value: f32| value.is_finite() && value >= 0.0;
        let procedural = self;
        let response = procedural.body_response;
        if ![
            procedural.guarded_look_joint_limit_degrees,
            procedural.guarded_look_joint_count,
            procedural.dive_pelvis_lean_degrees,
            procedural.height_transition_speed_metres_per_second,
            procedural.locomotion_stop_height_speed_metres_per_second,
            procedural.authored_ordinary_passing_rise_metres,
            procedural.minimum_landing_absorption_speed_metres_per_second,
            procedural.landing_compression_arrival_tolerance_metres,
            procedural.landing_plant_minimum_compression_metres,
            response.maximum_travel_lean_degrees,
            response.forward_reference_acceleration_metres_per_second_squared,
            response.maximum_forward_acceleration_lean_degrees,
            response.braking_acceleration_lean_degrees,
            response.maximum_braking_lean_degrees,
            response.lateral_reference_acceleration_metres_per_second_squared,
            response.maximum_lateral_acceleration_lean_degrees,
            response.maximum_response_degrees,
            response.steady_travel_lean_degrees,
            response.forward_acceleration_lean_degrees,
            response.lateral_acceleration_lean_degrees,
            response.startup_inertial_lean_scale,
            response.sustained_inertial_lean_scale,
            response.degrees_per_second,
            response.smooth_time_seconds,
            response.acceleration_attack_response_per_second,
            response.acceleration_release_response_per_second,
            response.maximum_frame_seconds,
        ]
        .into_iter()
        .all(finite_nonnegative)
            || procedural.guarded_look_joint_count == 0.0
            || response.maximum_presentation_sample_gap_ticks == 0
            || response.forward_reference_acceleration_metres_per_second_squared == 0.0
            || response.lateral_reference_acceleration_metres_per_second_squared == 0.0
            || response.maximum_response_degrees.to_radians()
                <= BODY_RESPONSE_DEGREE_REPORT_MARGIN_RADIANS
        {
            return Err(TacticalCombatConfigError::Validation(
                "invalid procedural animation tuning",
            ));
        }
        Ok(())
    }
}
