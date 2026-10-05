//! One owned merchant property, including the routes and enclosure it reserves.
use super::*;
use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

mod boundary;
pub use boundary::{CityBoundaryMaterial, CityBoundaryMember};

pub(super) const COMPOUND_EDGE_MARGIN_METRES: f32 = 1.25;
pub(super) const REAR_RANGE_DEPTH_METRES: f32 = 6.0;
pub(super) const ACCESS_HALF_WIDTH_METRES: f32 = 0.4;
pub const MAX_CITY_BUILDING_INSTANCES: usize = MAX_CITY_LOTS * 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Reflect)]
#[serde(transparent)]
pub struct CityPropertyId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct CityPlotBounds {
    pub centre_metres: Vec2,
    pub dimensions_metres: Vec2,
    pub orientation: BuildingOrientation,
}

impl CityPlotBounds {
    pub(crate) fn plan_polygon(
        self,
    ) -> Result<
        crate::scene_coordinates::ScenePlanPolygon,
        adventuresim_building_generator::plan_geometry::PlanGeometryError,
    > {
        use crate::scene_coordinates::{ScenePlanPoint, ScenePlanPolygon};
        use adventuresim_building_generator::plan_geometry::PlanGeometryError;
        let points = self
            .corners()
            .into_iter()
            .map(ScenePlanPoint::from_metres)
            .collect::<Option<Vec<_>>>()
            .ok_or(PlanGeometryError::NonFinite)?;
        ScenePlanPolygon::from_ordered_vertices(points)
    }

    /// Roundoff of bounded near-city f32 world poses, not extra owned land.
    /// Translation independently rounds a plot and its coincident child edge.
    pub const COORDINATE_TOLERANCE_METRES: f64 = 0.001;

    pub fn corners(self) -> [Vec2; 4] {
        [
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::ONE,
            Vec2::new(-1.0, 1.0),
        ]
        .map(|p| {
            self.centre_metres
                + self
                    .orientation
                    .local_to_world(p * self.dimensions_metres * 0.5)
        })
    }

    pub fn contains(self, point: Vec2) -> bool {
        let delta = point.as_dvec2() - self.centre_metres.as_dvec2();
        let (sine, cosine) = f64::from(self.orientation.yaw_radians()).sin_cos();
        bevy::math::DVec2::new(
            cosine * delta.x - sine * delta.y,
            sine * delta.x + cosine * delta.y,
        )
        .abs()
        .cmple(
            self.dimensions_metres.as_dvec2() * 0.5
                + bevy::math::DVec2::splat(Self::COORDINATE_TOLERANCE_METRES),
        )
        .all()
    }

    pub fn is_valid(self) -> bool {
        self.centre_metres.is_finite()
            && self.orientation.is_valid()
            && self.dimensions_metres.is_finite()
            && self.dimensions_metres.cmpgt(Vec2::ZERO).all()
    }

    pub fn intersects(self, other: Self) -> bool {
        crate::scene_input::buildings::oriented_rectangles_overlap(
            self.centre_metres,
            self.dimensions_metres * 0.5,
            self.orientation,
            other.centre_metres,
            other.dimensions_metres * 0.5,
            other.orientation,
        )
    }
}

impl From<CityBuildingLot> for CityPlotBounds {
    fn from(lot: CityBuildingLot) -> Self {
        Self {
            centre_metres: lot.centre_metres,
            dimensions_metres: lot.footprint_metres,
            orientation: lot.orientation,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct CityAccessSegment {
    pub start_metres: Vec2,
    pub end_metres: Vec2,
    pub half_width_metres: f32,
}

impl CityAccessSegment {
    /// Endpoint agreement for the generated property access graph.
    pub const JOIN_TOLERANCE_METRES: f32 = 0.02;

    pub fn ends_at(self, point: Vec2) -> bool {
        self.end_metres.distance(point) <= Self::JOIN_TOLERANCE_METRES
    }

    pub fn contains_centreline(self, point: Vec2) -> bool {
        let delta = self.end_metres - self.start_metres;
        if delta.length_squared() <= f32::EPSILON {
            return false;
        }
        let fraction = (point - self.start_metres).dot(delta) / delta.length_squared();
        (0.0..=1.0).contains(&fraction)
            && point.distance(self.start_metres + delta * fraction) <= Self::JOIN_TOLERANCE_METRES
    }
}

/// Carry the owned route while keeping its first hook on the original street.
pub(super) fn translate_property_access(
    access: &mut [CityAccessSegment],
    delta: Vec2,
    tangent: Vec2,
) {
    for (index, segment) in access.iter_mut().enumerate() {
        segment.start_metres += if index == 0 {
            tangent * delta.dot(tangent)
        } else {
            delta
        };
        segment.end_metres += delta;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct CityBoundarySegment {
    pub start_metres: Vec2,
    pub end_metres: Vec2,
    pub height_metres: f32,
    pub thickness_metres: f32,
}

/// Side in the property's local frontage coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(rename_all = "snake_case")]
pub enum PropertySide {
    Left,
    Right,
}
impl PropertySide {
    pub const fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
    pub const fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

/// An inward-opening timber gate, in the same horizontal coordinates as its plot.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct CityGate {
    pub hinge: PropertySide,
    pub centre_metres: Vec2,
    pub orientation: BuildingOrientation,
    pub width_metres: f32,
    pub height_metres: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct CityBoundary {
    pub walls: Vec<CityBoundarySegment>,
    pub gate: CityGate,
}

/// Front and rear are members of one property; the rear adds no service or residents.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct CityCompound {
    pub id: CityPropertyId,
    pub front_building_id: u64,
    pub rear_building_id: u64,
    pub plot: CityPlotBounds,
    pub court: CityPlotBounds,
    pub access: Vec<CityAccessSegment>,
    pub boundary: CityBoundary,
}

impl CityBuildingLot {
    pub fn has_rear_range(self) -> bool {
        self.service.is_none() && self.house_class == CityHouseClass::MerchantHouse
    }
}
