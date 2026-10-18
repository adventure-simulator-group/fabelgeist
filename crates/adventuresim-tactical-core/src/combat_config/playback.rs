//! Playback timing and locomotion presentation thresholds authored in YAML.

use super::TacticalCombatConfigError;
use adventuresim_building_generator::spatial_geometry::SignedLength;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationPlaybackConfig {
    pub frames_per_second: f32,
    pub player_visual_y_offset_metres: SignedLength,
    pub velocity_response_per_second: f32,
    pub phase_correction_rate_per_second: f32,
    pub phase_drift_deadband: f32,
    pub phase_drift_measurement_blend: f32,
    pub phase_snap_error: f32,
    pub maximum_source_gap_ticks: u64,
    pub maximum_authored_step_cadence_per_second: f32,
    pub maximum_authored_stance_slip_metres: f32,
    pub authored_cadence_cap_transition_width: f32,
    pub maximum_coalesced_events: u64,
    pub initial_locomotion_speed_metres_per_second: f32,
    pub locomotion_deactivation_speed_metres_per_second: f32,
    pub locomotion_activation_speed_metres_per_second: f32,
}

impl AnimationPlaybackConfig {
    pub(super) fn validate(&self) -> Result<(), TacticalCombatConfigError> {
        let playback = self;
        let finite_nonnegative = |value: f32| value.is_finite() && value >= 0.0;
        if ![
            playback.frames_per_second,
            playback.initial_locomotion_speed_metres_per_second,
            playback.locomotion_deactivation_speed_metres_per_second,
            playback.locomotion_activation_speed_metres_per_second,
            playback.velocity_response_per_second,
            playback.phase_correction_rate_per_second,
            playback.phase_drift_deadband,
            playback.phase_drift_measurement_blend,
            playback.phase_snap_error,
            playback.maximum_authored_step_cadence_per_second,
            playback.maximum_authored_stance_slip_metres,
            playback.authored_cadence_cap_transition_width,
        ]
        .into_iter()
        .all(finite_nonnegative)
            || playback.phase_drift_measurement_blend > 1.0
            || playback.frames_per_second == 0.0
            || playback.maximum_source_gap_ticks == 0
            || playback.maximum_coalesced_events == 0
            || playback.locomotion_deactivation_speed_metres_per_second
                > playback.initial_locomotion_speed_metres_per_second
            || playback.initial_locomotion_speed_metres_per_second
                > playback.locomotion_activation_speed_metres_per_second
        {
            return Err(TacticalCombatConfigError::Validation(
                "invalid animation playback tuning",
            ));
        }
        Ok(())
    }
}
