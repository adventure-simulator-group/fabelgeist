//! Playback timing and locomotion presentation thresholds authored in YAML.

use super::TacticalCombatConfigError;
use crate::animation::{AnimationFrequency, AnimationSampleSpan, NonnegativeLength, UnitInterval};
use adventuresim_building_generator::spatial_geometry::SignedLength;
use adventuresim_core::linear_speed::NonnegativeSpeed;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationPlaybackConfig {
    /// Authored clip frames per second; no representable reciprocal is required.
    pub frames_per_second: AnimationFrequency,
    pub player_visual_y_offset_metres: SignedLength,
    pub velocity_response_per_second: f32,
    pub phase_correction_rate_per_second: f32,
    pub phase_drift_deadband: f32,
    pub phase_drift_measurement_blend: UnitInterval,
    pub phase_snap_error: f32,
    pub maximum_source_gap_ticks: AnimationSampleSpan,
    pub maximum_authored_step_cadence_per_second: f32,
    pub maximum_authored_stance_slip_metres: NonnegativeLength,
    pub authored_cadence_cap_transition_width: f32,
    pub maximum_coalesced_events: u64,
    pub initial_locomotion_speed_metres_per_second: NonnegativeSpeed,
    pub locomotion_deactivation_speed_metres_per_second: NonnegativeSpeed,
    pub locomotion_activation_speed_metres_per_second: NonnegativeSpeed,
}

impl AnimationPlaybackConfig {
    pub(super) fn validate(&self) -> Result<(), TacticalCombatConfigError> {
        let playback = self;
        let finite_nonnegative = |value: f32| value.is_finite() && value >= 0.0;
        if ![
            playback.velocity_response_per_second,
            playback.phase_correction_rate_per_second,
            playback.phase_drift_deadband,
            playback.phase_snap_error,
            playback.maximum_authored_step_cadence_per_second,
            playback.authored_cadence_cap_transition_width,
        ]
        .into_iter()
        .all(finite_nonnegative)
            || playback.maximum_source_gap_ticks == AnimationSampleSpan::ZERO
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
