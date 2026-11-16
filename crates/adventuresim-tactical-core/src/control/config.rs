//! Authored control and navigation bounds, shared by the server and client.

use adventuresim_building_generator::spatial_geometry::PositiveLength;
use serde::{Deserialize, Serialize};

use crate::animation::{AngularMagnitudeDegrees, PositiveAnimationDuration};

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ControlConfigError {
    #[error("pursuit or counterattack distance exceeds perception distance")]
    PerceptionRadius,
    #[error("hold radius exceeds pursuit radius")]
    HoldRadius,
    #[error("orbit sensitivity must be positive")]
    OrbitSensitivity,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlConfig {
    pub arrival_radius_metres: PositiveLength,
    pub support_tolerance_metres: PositiveLength,
    pub clearance_margin_metres: PositiveLength,
    pub perception_radius_metres: PositiveLength,
    pub pursuit_radius_metres: PositiveLength,
    pub hold_radius_metres: PositiveLength,
    pub counterattack_radius_metres: PositiveLength,
    pub no_progress_seconds: PositiveAnimationDuration,
    pub threat_memory_seconds: PositiveAnimationDuration,
    pub mesh_rebuild_interval_seconds: PositiveAnimationDuration,
    pub click_distance_metres: PositiveLength,
    pub orbit_distance_metres: PositiveLength,
    pub orbit_mouse_degrees_per_pixel: AngularMagnitudeDegrees,
}

impl Default for ControlConfig {
    fn default() -> Self {
        crate::combat_config::TacticalCombatConfig::default()
            .ai
            .control
    }
}

impl ControlConfig {
    pub fn validate(&self) -> Result<(), ControlConfigError> {
        if self.pursuit_radius_metres > self.perception_radius_metres
            || self.counterattack_radius_metres > self.perception_radius_metres
        {
            return Err(ControlConfigError::PerceptionRadius);
        }
        if self.hold_radius_metres > self.pursuit_radius_metres {
            return Err(ControlConfigError::HoldRadius);
        }
        if self.orbit_mouse_degrees_per_pixel.degrees() == 0.0 {
            return Err(ControlConfigError::OrbitSensitivity);
        }
        Ok(())
    }
}
