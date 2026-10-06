//! Admitted replicated window geometry; mutable controller state belongs to the server.
use adventuresim_building_generator::spatial_geometry::GeometryResult as Result;
use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
#[error("building {building_id}, window {opening_id}: {cause}")]
pub struct SceneWindowError {
    pub building_id: super::SceneBuildingId,
    pub opening_id: adventuresim_building_generator::OpeningAssemblyId,
    #[source]
    pub cause: adventuresim_building_generator::spatial_geometry::GeometryError,
}

/// Compact identity and dimensions for one server-authoritative window casement.
#[derive(Clone, Copy, Debug, PartialEq, Component, Serialize)]
#[component(immutable)]
pub struct SceneWindow {
    pub leaf: adventuresim_building_generator::WindowLeafKind,
    pub building_id: super::SceneBuildingId,
    pub opening_id: adventuresim_building_generator::OpeningAssemblyId,
    pub size_metres: adventuresim_building_generator::spatial_geometry::LeafDimensions,
    pub opening_centre_metres: adventuresim_building_generator::spatial_geometry::Position<
        crate::scene_coordinates::Scene,
    >,
    pub tangent: adventuresim_building_generator::spatial_geometry::SpatialDirection<
        crate::scene_coordinates::Scene,
    >,
    pub outward: adventuresim_building_generator::spatial_geometry::SpatialDirection<
        crate::scene_coordinates::Scene,
    >,
    pub bars: adventuresim_building_generator::WindowBarPresence,
}

impl<'de> Deserialize<'de> for SceneWindow {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        use adventuresim_building_generator::spatial_geometry::{
            LeafDimensions, Position, SpatialDirection,
        };
        // Scene-frame positions are metres; dimensions are leaf-local width,
        // height and thickness. Directions are normalized scene XYZ vectors.
        // Decode native leaves together to attach IDs to admission failures.
        #[derive(Deserialize)]
        struct NativeWindow {
            leaf: adventuresim_building_generator::WindowLeafKind,
            building_id: u64,
            opening_id: u64,
            size_metres: bevy::math::Vec3,
            opening_centre_metres: bevy::math::Vec3,
            tangent: bevy::math::Vec3,
            outward: bevy::math::Vec3,
            bars: adventuresim_building_generator::WindowBarPresence,
        }
        let v = NativeWindow::deserialize(d)?;
        let admit = || -> Result<Self> {
            Ok(Self {
                leaf: v.leaf,
                building_id: v.building_id.into(),
                opening_id: adventuresim_building_generator::OpeningAssemblyId(v.opening_id),
                size_metres: LeafDimensions::from_metres(v.size_metres)?,
                opening_centre_metres: Position::from_metres(v.opening_centre_metres)?,
                tangent: SpatialDirection::from_normalized(v.tangent)?,
                outward: SpatialDirection::from_normalized(v.outward)?,
                bars: v.bars,
            })
        };
        admit().map_err(|cause| {
            serde::de::Error::custom(SceneWindowError {
                building_id: crate::scene_input::SceneBuildingId::from(v.building_id),
                opening_id: adventuresim_building_generator::OpeningAssemblyId(v.opening_id),
                cause,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_building_generator::spatial_geometry::{
        LeafDimensions, Position, SpatialDirection,
    };
    use bevy::math::Vec3;
    #[test]
    fn replicated_window_round_trip_admits_named_bars_and_rejects_invalid_leaves() {
        let window = SceneWindow {
            leaf: adventuresim_building_generator::WindowLeafKind::TimberShutter,
            building_id: crate::scene_input::SceneBuildingId::from(8),
            opening_id: adventuresim_building_generator::OpeningAssemblyId(17),
            size_metres: LeafDimensions::from_metres(Vec3::new(1.0, 2.0, 0.001)).unwrap(),
            opening_centre_metres: Position::from_metres(Vec3::new(-2.0, 3.0, 4.0)).unwrap(),
            tangent: SpatialDirection::from_normalized(Vec3::X).unwrap(),
            outward: SpatialDirection::from_normalized(Vec3::Z).unwrap(),
            bars: adventuresim_building_generator::WindowBarPresence::Present,
        };
        for (bars, name) in [
            (
                adventuresim_building_generator::WindowBarPresence::Absent,
                "Absent",
            ),
            (
                adventuresim_building_generator::WindowBarPresence::Present,
                "Present",
            ),
        ] {
            let window = SceneWindow { bars, ..window };
            let bytes = postcard::to_allocvec(&window).unwrap();
            assert_eq!(postcard::from_bytes::<SceneWindow>(&bytes).unwrap(), window);
            let json = serde_json::to_value(window).unwrap();
            assert_eq!(json["building_id"], 8);
            assert_eq!(json["opening_id"], 17);
            assert_eq!(json["bars"], name);
            assert!(json.get("barred").is_none());
            assert_eq!(
                serde_json::from_value::<SceneWindow>(json.clone()).unwrap(),
                window
            );
            let mut invalid = json;
            invalid["bars"] = serde_json::json!(true);
            assert!(serde_json::from_value::<SceneWindow>(invalid).is_err());
        }
        let mut value = serde_json::to_value(window).unwrap();
        value["size_metres"] = serde_json::json!([1, 2, 0]);
        let error = serde_json::from_value::<SceneWindow>(value)
            .unwrap_err()
            .to_string();
        assert!(error.contains("building 8, window 17"));
    }
}
