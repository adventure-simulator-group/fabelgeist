//! Finite scene-local east/north positions, distinct from geographic degrees
//! and architectural building-local coordinates.
use adventuresim_building_generator::plan_geometry::{PlanPolygon, PlanVertex};
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

mod polygon;
mod spatial;
pub use polygon::{ArchitecturalPlanProjection, ScenePlanPolygon};
pub use spatial::{
    ArchitecturalFloorDatum, ArchitecturalGateDatum, CollisionCentreDatum, GateDatum, GateRelative,
    GroundRelative, PlotRelative, Scene, SceneDoorPose,
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ScenePlanPoint(Vec2);
impl ScenePlanPoint {
    pub fn from_metres(point: Vec2) -> Option<Self> {
        point.is_finite().then_some(Self(point))
    }
    pub fn metres(self) -> Vec2 {
        self.0
    }
}
impl TryFrom<Vec2> for ScenePlanPoint {
    type Error = adventuresim_building_generator::spatial_geometry::GeometryError;
    fn try_from(metres: Vec2) -> Result<Self, Self::Error> {
        use adventuresim_building_generator::spatial_geometry::{CoordinateAxis, GeometryRole};
        Self::from_metres(metres).ok_or(Self::Error::NonFinite {
            role: GeometryRole::Position,
            axis: if !metres.x.is_finite() {
                CoordinateAxis::X
            } else {
                CoordinateAxis::Z
            },
        })
    }
}
impl PlanVertex for ScenePlanPoint {
    fn plan_metres(self) -> Vec2 {
        self.metres()
    }
}
/// Finite scene-local east/north translation in metres; not a point or direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanDisplacement(Vec2);
impl PlanDisplacement {
    pub const ZERO: Self = Self(Vec2::ZERO);
    pub fn from_metres(metres: Vec2) -> Option<Self> {
        metres.is_finite().then_some(Self(metres))
    }
    pub fn metres(self) -> Vec2 {
        self.0
    }
}
impl<'de> Deserialize<'de> for ScenePlanPoint {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_metres(Vec2::deserialize(deserializer)?)
            .ok_or_else(|| serde::de::Error::custom("scene plan point must be finite"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scene_coordinates_reject_nonfinite_components_and_roundtrip_signed_positions() {
        assert!(ScenePlanPoint::from_metres(Vec2::new(0.0, f32::NAN)).is_none());
        let point = ScenePlanPoint::from_metres(Vec2::new(-3.0, 2.0)).unwrap();
        let restored: ScenePlanPoint =
            serde_json::from_str(&serde_json::to_string(&point).unwrap()).unwrap();
        assert_eq!(restored, point);
        assert!(serde_json::from_str::<ScenePlanPoint>("[0,null]").is_err());
    }
}

#[cfg(test)]
mod spatial_tests;
