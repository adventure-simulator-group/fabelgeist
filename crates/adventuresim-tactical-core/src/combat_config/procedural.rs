//! Procedural height and body-response tuning authored in combat YAML.

use super::TacticalCombatConfigError;
use crate::animation::{
    AngularMagnitudeDegrees, AngularSpeedDegrees, AnimationElapsed, AnimationSampleSpan,
    HeightDistance, HeightSpeed, InertialLeanGain, LookDistribution, PositiveAnimationDuration,
    ReferenceAcceleration, ResponseRate,
};
use serde::{Deserialize, Serialize};

/// Radian allowance for reporting a capped body angle after degree conversion.
pub const BODY_RESPONSE_DEGREE_REPORT_MARGIN_RADIANS: f32 = 0.000001;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProceduralAnimationConfig {
    pub guarded_look_joint_limit_degrees: AngularMagnitudeDegrees,
    pub guarded_look_joint_count: LookDistribution,
    pub dive_pelvis_lean_degrees: AngularMagnitudeDegrees,
    pub height_transition_speed_metres_per_second: HeightSpeed,
    pub locomotion_stop_height_speed_metres_per_second: HeightSpeed,
    pub authored_ordinary_passing_rise_metres: HeightDistance,
    pub minimum_landing_absorption_speed_metres_per_second: HeightSpeed,
    pub landing_compression_arrival_tolerance_metres: HeightDistance,
    pub landing_plant_minimum_compression_metres: HeightDistance,
    pub body_response: BodyResponseConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyResponseConfig {
    pub maximum_presentation_sample_gap_ticks: AnimationSampleSpan,
    pub steady_travel_lean_degrees: AngularMagnitudeDegrees,
    pub maximum_travel_lean_degrees: AngularMagnitudeDegrees,
    pub forward_reference_acceleration_metres_per_second_squared: ReferenceAcceleration,
    pub maximum_forward_acceleration_lean_degrees: AngularMagnitudeDegrees,
    pub braking_acceleration_lean_degrees: AngularMagnitudeDegrees,
    pub maximum_braking_lean_degrees: AngularMagnitudeDegrees,
    pub lateral_reference_acceleration_metres_per_second_squared: ReferenceAcceleration,
    pub maximum_lateral_acceleration_lean_degrees: AngularMagnitudeDegrees,
    pub maximum_response_degrees: AngularMagnitudeDegrees,
    pub forward_acceleration_lean_degrees: AngularMagnitudeDegrees,
    pub lateral_acceleration_lean_degrees: AngularMagnitudeDegrees,
    pub startup_inertial_lean_scale: InertialLeanGain,
    pub sustained_inertial_lean_scale: InertialLeanGain,
    pub degrees_per_second: AngularSpeedDegrees,
    pub smooth_time_seconds: PositiveAnimationDuration,
    pub acceleration_attack_response_per_second: ResponseRate,
    pub acceleration_release_response_per_second: ResponseRate,
    pub maximum_frame_seconds: AnimationElapsed,
}

impl ProceduralAnimationConfig {
    pub(super) fn validate(&self) -> Result<(), TacticalCombatConfigError> {
        let response = self.body_response;
        if response.maximum_presentation_sample_gap_ticks == AnimationSampleSpan::ZERO
            || response.maximum_response_degrees.radians()
                <= BODY_RESPONSE_DEGREE_REPORT_MARGIN_RADIANS
        {
            return Err(TacticalCombatConfigError::Validation(
                "invalid procedural animation tuning",
            ));
        }
        Ok(())
    }
}
