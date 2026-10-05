//! Finite scene-local east/north positions, distinct from geographic degrees
//! and architectural building-local coordinates.
use bevy::math::Vec2;
use serde::{Deserialize, Serialize};

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
