//! Checked scene reservations keep dimensional extents distinct from positions.
use super::*;
use crate::scene_coordinates::ScenePlanPoint;
use adventuresim_building_generator::spatial_geometry::{GeometryError, PlanExtents};

/// Physical kit bounds plus the access/workspace that must remain unobstructed.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(opaque)]
#[serde(try_from = "FootprintWire")]
pub struct FurnitureFootprint {
    #[serde(rename = "centre_metres")]
    centre: ScenePlanPoint,
    #[serde(rename = "half_extents_metres")]
    half_extents: PlanExtents,
    orientation: BuildingOrientation,
}
#[derive(Deserialize)]
struct FootprintWire {
    centre_metres: ScenePlanPoint,
    half_extents_metres: PlanExtents,
    orientation: BuildingOrientation,
}
impl TryFrom<FootprintWire> for FurnitureFootprint {
    type Error = GeometryError;
    fn try_from(wire: FootprintWire) -> Result<Self, Self::Error> {
        Self::new(
            wire.centre_metres,
            wire.half_extents_metres,
            wire.orientation,
        )
    }
}
impl FurnitureFootprint {
    pub fn new(
        centre: ScenePlanPoint,
        half_extents: PlanExtents,
        orientation: BuildingOrientation,
    ) -> Result<Self, GeometryError> {
        let footprint = Self {
            centre,
            half_extents,
            orientation,
        };
        if !orientation.is_valid() || footprint.corners().iter().any(|p| !p.is_finite()) {
            return Err(GeometryError::InvalidProjection);
        }
        Ok(footprint)
    }
    /// Admission from native metre buffers at the reservation numerical kernel.
    pub(super) fn from_metres(
        centre: Vec2,
        half_extents: Vec2,
        orientation: BuildingOrientation,
    ) -> Result<Self, GeometryError> {
        Self::new(
            ScenePlanPoint::try_from(centre)?,
            PlanExtents::from_metres(half_extents)?,
            orientation,
        )
    }
    pub fn centre(self) -> ScenePlanPoint {
        self.centre
    }
    pub fn half_extents(self) -> PlanExtents {
        self.half_extents
    }
    pub fn orientation(self) -> BuildingOrientation {
        self.orientation
    }
    /// Native vectors are confined to the separating-axis collision kernel.
    pub fn corners(self) -> [Vec2; 4] {
        [
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::ONE,
            Vec2::new(-1.0, 1.0),
        ]
        .map(|corner| {
            self.centre.metres()
                + self
                    .orientation
                    .local_to_world(corner * self.half_extents.metres())
        })
    }
    pub fn contains(self, point: crate::scene_coordinates::ScenePlanPoint) -> bool {
        let point = point.metres();
        self.orientation
            .world_to_local(point - self.centre.metres())
            .abs()
            .cmple(self.half_extents.metres())
            .all()
    }
    pub fn intersects(self, other: Self) -> bool {
        let axes = [
            self.orientation.local_to_world(Vec2::X),
            self.orientation.local_to_world(Vec2::Y),
            other.orientation.local_to_world(Vec2::X),
            other.orientation.local_to_world(Vec2::Y),
        ];
        axes.into_iter().all(|axis| {
            let radius = |footprint: Self| {
                footprint
                    .orientation
                    .local_to_world(Vec2::X)
                    .dot(axis)
                    .abs()
                    * footprint.half_extents.metres().x
                    + footprint
                        .orientation
                        .local_to_world(Vec2::Y)
                        .dot(axis)
                        .abs()
                        * footprint.half_extents.metres().y
            };
            (self.centre.metres() - other.centre.metres())
                .dot(axis)
                .abs()
                < radius(self) + radius(other)
        })
    }
}
