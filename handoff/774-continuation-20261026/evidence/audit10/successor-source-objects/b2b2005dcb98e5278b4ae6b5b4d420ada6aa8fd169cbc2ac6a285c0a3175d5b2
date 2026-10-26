//! Collision-centre-relative reservations become scene geometry only at placement.
use super::*;
use crate::scene_coordinates::CollisionRelative;
use adventuresim_building_generator::spatial_geometry::{Displacement, GeometryError};
use bevy::math::Vec3Swizzles;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "LocalWire")]
pub(super) struct LocalReservation {
    #[serde(with = "offset_wire", rename = "centre_metres")]
    centre: Displacement<CollisionRelative>,
    #[serde(rename = "half_extents_metres")]
    half_extents: PlanExtents,
    orientation: BuildingOrientation,
}
#[derive(Deserialize)]
struct LocalWire {
    centre_metres: Vec2,
    half_extents_metres: PlanExtents,
    orientation: BuildingOrientation,
}
impl TryFrom<LocalWire> for LocalReservation {
    type Error = GeometryError;
    fn try_from(wire: LocalWire) -> Result<Self, Self::Error> {
        Self::from_metres(
            wire.centre_metres,
            wire.half_extents_metres.metres(),
            wire.orientation,
        )
    }
}
impl LocalReservation {
    /// Native opening/passage plan buffers are translated to the collision datum.
    pub(super) fn from_metres(
        centre: Vec2,
        extents: Vec2,
        orientation: BuildingOrientation,
    ) -> Result<Self, GeometryError> {
        let centre = Displacement::from_metres(Vec3::new(centre.x, 0.0, centre.y))?;
        let half_extents = PlanExtents::from_metres(extents)?;
        if !orientation.is_valid() {
            return Err(GeometryError::InvalidProjection);
        }
        for corner in [
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::ONE,
            Vec2::new(-1.0, 1.0),
        ] {
            if !(centre.metres().xz() + orientation.local_to_world(corner * extents)).is_finite() {
                return Err(GeometryError::InvalidProjection);
            }
        }
        Ok(Self {
            centre,
            half_extents,
            orientation,
        })
    }
    pub(super) fn route(start: Vec2, end: Vec2, half_width: f32) -> Result<Self, GeometryError> {
        Self::from_metres(
            (start + end) * 0.5,
            Vec2::new(start.distance(end) * 0.5 + half_width, half_width),
            BuildingOrientation::from_frontage_tangent(end - start)
                .unwrap_or(BuildingOrientation::IDENTITY),
        )
    }
    pub(super) fn place(
        &self,
        placement: &TacticalBuildingPlacement,
    ) -> Result<FurnitureFootprint, GeometryError> {
        let centre = crate::scene_coordinates::ScenePlanPoint::try_from(
            placement.centre_metres.metres()
                + placement
                    .orientation
                    .local_to_world(self.centre.metres().xz()),
        )?;
        let orientation = BuildingOrientation::from_radians(
            placement.orientation.yaw_radians() + self.orientation.yaw_radians(),
        )
        .ok_or(GeometryError::InvalidProjection)?;
        FurnitureFootprint::new(centre, self.half_extents, orientation)
    }
}
mod offset_wire {
    use super::*;
    pub(super) fn serialize<S: serde::Serializer>(
        value: &Displacement<CollisionRelative>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.metres().xz().serialize(serializer)
    }
}
