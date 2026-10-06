//! Admitted replicated window geometry; mutable controller state belongs to the server.
use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
#[error("building {building_id}, window {opening_id}: {cause}")]
pub struct SceneWindowError {
    pub building_id: u64,
    pub opening_id: u64,
    #[source]
    pub cause: adventuresim_building_generator::spatial_geometry::GeometryError,
}

/// Compact identity and dimensions for one server-authoritative window casement.
#[derive(Clone, Copy, Debug, PartialEq, Component, Serialize)]
#[component(immutable)]
pub struct SceneWindow {
    pub leaf: adventuresim_building_generator::WindowLeafKind,
    pub building_id: u64,
    pub opening_id: u64,
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
    pub barred: bool,
}

impl<'de> Deserialize<'de> for SceneWindow {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use adventuresim_building_generator::spatial_geometry::{
            GeometryError, LeafDimensions, Position, SpatialDirection,
        };
        #[derive(Deserialize)]
        struct NativeWindow {
            leaf: adventuresim_building_generator::WindowLeafKind,
            building_id: u64,
            opening_id: u64,
            size_metres: bevy::math::Vec3,
            opening_centre_metres: bevy::math::Vec3,
            tangent: bevy::math::Vec3,
            outward: bevy::math::Vec3,
            barred: bool,
        }
        let v = NativeWindow::deserialize(d)?;
        let admit = || -> Result<Self, GeometryError> {
            Ok(Self {
                leaf: v.leaf,
                building_id: v.building_id,
                opening_id: v.opening_id,
                size_metres: LeafDimensions::from_metres(v.size_metres)?,
                opening_centre_metres: Position::from_metres(v.opening_centre_metres)?,
                tangent: SpatialDirection::from_normalized(v.tangent)?,
                outward: SpatialDirection::from_normalized(v.outward)?,
                barred: v.barred,
            })
        };
        admit().map_err(|cause| {
            serde::de::Error::custom(SceneWindowError {
                building_id: v.building_id,
                opening_id: v.opening_id,
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
    fn replicated_window_round_trip_and_invalid_leaf_keep_the_wire_contract() {
        let window = SceneWindow {
            leaf: adventuresim_building_generator::WindowLeafKind::TimberShutter,
            building_id: 8,
            opening_id: 17,
            size_metres: LeafDimensions::from_metres(Vec3::new(1.0, 2.0, 0.001)).unwrap(),
            opening_centre_metres: Position::from_metres(Vec3::new(-2.0, 3.0, 4.0)).unwrap(),
            tangent: SpatialDirection::from_normalized(Vec3::X).unwrap(),
            outward: SpatialDirection::from_normalized(Vec3::Z).unwrap(),
            barred: true,
        };
        let wire = postcard::to_allocvec(&window).unwrap();
        assert_eq!(postcard::from_bytes::<SceneWindow>(&wire).unwrap(), window);
        let mut value = serde_json::to_value(window).unwrap();
        value["size_metres"] = serde_json::json!([1, 2, 0]);
        let error = serde_json::from_value::<SceneWindow>(value)
            .unwrap_err()
            .to_string();
        assert!(error.contains("building 8, window 17"));
    }
}
