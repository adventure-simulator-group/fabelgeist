//! Authored control and navigation bounds, shared by the server and client.

use adventuresim_building_generator::spatial_geometry::PositiveLength;
use serde::{Deserialize, Serialize};

/// A finite, strictly positive duration used by control and navigation.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct PositiveControlDuration(f32);

/// Positive camera rotation in degrees per pixel of mouse movement.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct OrbitSensitivity(f32);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlConfig {
    pub arrival_radius_metres: PositiveLength,
    pub support_tolerance_metres: PositiveLength,
    pub clearance_margin_metres: PositiveLength,
    /// Horizontal voxel width resolves physically clear interior doorways.
    pub navigation_cell_size_metres: PositiveLength,
    /// Vertical voxel height preserves floor and doorway headroom.
    pub navigation_cell_height_metres: PositiveLength,
    pub perception_radius_metres: PositiveLength,
    pub pursuit_radius_metres: PositiveLength,
    pub hold_radius_metres: PositiveLength,
    pub counterattack_radius_metres: PositiveLength,
    pub talk_distance_metres: PositiveLength,
    pub no_progress_seconds: PositiveControlDuration,
    pub threat_memory_seconds: PositiveControlDuration,
    pub mesh_rebuild_interval_seconds: PositiveControlDuration,
    pub click_distance_metres: PositiveLength,
    pub orbit_distance_metres: PositiveLength,
    pub orbit_mouse_degrees_per_pixel: OrbitSensitivity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ControlConfigError {
    #[error("pursuit or counterattack distance exceeds perception distance")]
    PerceptionRadius,
    #[error("hold radius exceeds pursuit radius")]
    HoldRadius,
    #[error("talk distance exceeds perception distance")]
    TalkDistance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ControlValueError {
    #[error("control duration must be finite and positive")]
    Duration,
    #[error("orbit sensitivity must be finite and positive")]
    OrbitSensitivity,
}

impl PositiveControlDuration {
    pub fn seconds(self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for PositiveControlDuration {
    type Error = ControlValueError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_finite() && value > 0.0 {
            Ok(Self(value))
        } else {
            Err(ControlValueError::Duration)
        }
    }
}

impl From<PositiveControlDuration> for f32 {
    fn from(value: PositiveControlDuration) -> Self {
        value.0
    }
}

impl OrbitSensitivity {
    pub fn degrees_per_pixel(self) -> f32 {
        self.0
    }
}

impl TryFrom<f32> for OrbitSensitivity {
    type Error = ControlValueError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_finite() && value > 0.0 {
            Ok(Self(value))
        } else {
            Err(ControlValueError::OrbitSensitivity)
        }
    }
}

impl From<OrbitSensitivity> for f32 {
    fn from(value: OrbitSensitivity) -> Self {
        value.0
    }
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
        if self.talk_distance_metres > self.perception_radius_metres {
            return Err(ControlConfigError::TalkDistance);
        }
        Ok(())
    }
}
