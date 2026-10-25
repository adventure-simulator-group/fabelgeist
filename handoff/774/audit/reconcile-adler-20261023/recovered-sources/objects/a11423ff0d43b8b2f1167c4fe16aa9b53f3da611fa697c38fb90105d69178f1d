//! Rejected numerical coordinates remain evidence, never usable scene geometry.
use crate::scene_coordinates::ScenePlanPoint;
use bevy::math::Vec2;
use serde::Serialize;

/// Location of a support rejection. Finite attempts retain the checked scene
/// frame; non-finite attempts preserve the solver's original diagnostic input.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportDiagnosticLocation {
    Scene(ScenePlanPoint),
    Rejected(SceneCoordinateAttempt),
}

/// Native east/north metres captured only when coordinate admission failed.
/// This record has no conversion to a position or displacement.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SceneCoordinateAttempt {
    east_metres: f32,
    north_metres: f32,
}
impl SupportDiagnosticLocation {
    /// Explicit error boundary from the native scene-plane numerical kernel.
    pub(crate) fn from_attempt_metres(point: Vec2) -> Self {
        match ScenePlanPoint::try_from(point) {
            Ok(point) => Self::Scene(point),
            Err(_) => Self::Rejected(SceneCoordinateAttempt {
                east_metres: point.x,
                north_metres: point.y,
            }),
        }
    }
    pub fn scene_point(self) -> Option<ScenePlanPoint> {
        match self {
            Self::Scene(point) => Some(point),
            Self::Rejected(_) => None,
        }
    }
    /// Original diagnostic coordinates for rendering and numerical comparison.
    pub fn attempted_metres(self) -> Vec2 {
        match self {
            Self::Scene(point) => point.metres(),
            Self::Rejected(point) => Vec2::new(point.east_metres, point.north_metres),
        }
    }
}
