//! Immutable owned scene rectangles; native corner arithmetic serves clipping.
use super::*;
use crate::scene_coordinates::{PlanDisplacement, ScenePlanPoint};
use adventuresim_building_generator::spatial_geometry::{GeometryError, PlanDimensions};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(opaque)]
#[serde(try_from = "RegionWire")]
pub struct CityPlotBounds {
    centre_metres: ScenePlanPoint,
    dimensions_metres: PlanDimensions,
    orientation: BuildingOrientation,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionWire {
    centre_metres: ScenePlanPoint,
    dimensions_metres: PlanDimensions,
    orientation: BuildingOrientation,
}
impl TryFrom<RegionWire> for CityPlotBounds {
    type Error = GeometryError;
    fn try_from(wire: RegionWire) -> Result<Self, GeometryError> {
        Self::new(wire.centre_metres, wire.dimensions_metres, wire.orientation)
    }
}
impl CityPlotBounds {
    pub const COORDINATE_TOLERANCE_METRES: f64 = 0.001;
    pub fn new(
        centre: ScenePlanPoint,
        dimensions: PlanDimensions,
        orientation: BuildingOrientation,
    ) -> Result<Self, GeometryError> {
        let region = Self {
            centre_metres: centre,
            dimensions_metres: dimensions,
            orientation,
        };
        if region.corners().iter().any(|point| !point.is_finite()) {
            return Err(GeometryError::InvalidProjection);
        }
        Ok(region)
    }
    pub fn centre(self) -> ScenePlanPoint {
        self.centre_metres
    }
    pub fn dimensions(self) -> PlanDimensions {
        self.dimensions_metres
    }
    pub fn orientation(self) -> BuildingOrientation {
        self.orientation
    }
    /// Scene east/north metre adapter for the native clipping and packing kernels.
    pub fn centre_metres(self) -> Vec2 {
        self.centre_metres.metres()
    }
    /// Scene-plane metre dimensions at the same numerical-kernel boundary.
    pub fn dimensions_metres(self) -> Vec2 {
        self.dimensions_metres.metres()
    }
    pub fn relocate(&mut self, centre: ScenePlanPoint) -> Result<(), GeometryError> {
        *self = Self::new(centre, self.dimensions_metres, self.orientation)?;
        Ok(())
    }
    pub fn resize(&mut self, dimensions: PlanDimensions) -> Result<(), GeometryError> {
        *self = Self::new(self.centre_metres, dimensions, self.orientation)?;
        Ok(())
    }
    pub fn rotate(&mut self, orientation: BuildingOrientation) -> Result<(), GeometryError> {
        *self = Self::new(self.centre_metres, self.dimensions_metres, orientation)?;
        Ok(())
    }
    pub fn relocated(self, centre: ScenePlanPoint) -> Result<Self, GeometryError> {
        Self::new(centre, self.dimensions_metres, self.orientation)
    }
    pub fn resized(self, dimensions: PlanDimensions) -> Result<Self, GeometryError> {
        Self::new(self.centre_metres, dimensions, self.orientation)
    }
    pub fn rotated(self, orientation: BuildingOrientation) -> Result<Self, GeometryError> {
        Self::new(self.centre_metres, self.dimensions_metres, orientation)
    }
    pub fn translated(self, delta: PlanDisplacement) -> Result<Self, GeometryError> {
        self.relocated(self.centre_metres.translated(delta)?)
    }
    pub(crate) fn plan_polygon(
        self,
    ) -> Result<
        crate::scene_coordinates::ScenePlanPolygon,
        adventuresim_building_generator::plan_geometry::PlanGeometryError,
    > {
        let points = self
            .corners()
            .into_iter()
            .map(ScenePlanPoint::from_metres)
            .collect::<Option<Vec<_>>>()
            .ok_or(adventuresim_building_generator::plan_geometry::PlanGeometryError::NonFinite)?;
        crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(points)
    }
    pub fn corners(self) -> [Vec2; 4] {
        [
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::ONE,
            Vec2::new(-1.0, 1.0),
        ]
        .map(|point| {
            self.centre_metres()
                + self
                    .orientation
                    .local_to_world(point * self.dimensions_metres() * 0.5)
        })
    }
    /// Native scene-plane metre containment used by exact polygon kernels.
    pub fn contains(self, point: Vec2) -> bool {
        let delta = point.as_dvec2() - self.centre_metres().as_dvec2();
        let (sine, cosine) = f64::from(self.orientation.yaw_radians()).sin_cos();
        bevy::math::DVec2::new(
            cosine * delta.x - sine * delta.y,
            sine * delta.x + cosine * delta.y,
        )
        .abs()
        .cmple(
            self.dimensions_metres().as_dvec2() * 0.5
                + bevy::math::DVec2::splat(Self::COORDINATE_TOLERANCE_METRES),
        )
        .all()
    }
    pub fn is_valid(self) -> bool {
        self.corners().iter().all(|point| point.is_finite())
    }
    pub fn intersects(self, other: Self) -> bool {
        crate::scene_input::buildings::oriented_rectangles_overlap(
            self.centre_metres(),
            self.dimensions_metres() * 0.5,
            self.orientation,
            other.centre_metres(),
            other.dimensions_metres() * 0.5,
            other.orientation,
        )
    }
}
impl TryFrom<CityBuildingLot> for CityPlotBounds {
    type Error = GeometryError;
    fn try_from(lot: CityBuildingLot) -> Result<Self, GeometryError> {
        Self::new(lot.centre_metres, lot.footprint_metres, lot.orientation)
    }
}
