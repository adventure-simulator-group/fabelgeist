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
        self.orientation
            .world_to_local(point - self.centre_metres)
            .abs()
            .cmple(self.dimensions_metres * 0.5)
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

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct CityBoundarySegment {
    pub start_metres: Vec2,
    pub end_metres: Vec2,
    pub height_metres: f32,
    pub thickness_metres: f32,
}

/// An inward-opening timber gate, in the same horizontal coordinates as its plot.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct CityGate {
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
