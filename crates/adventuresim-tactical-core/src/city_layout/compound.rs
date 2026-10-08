//! One owned merchant property, including the routes and enclosure it reserves.
use super::*;
use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

mod boundary;
pub use boundary::{
    BoundaryGeometryError, BoundarySupportElement, CityBoundaryMaterial, CityBoundaryMember,
    CityBoundaryPose,
};

pub(super) const COMPOUND_EDGE_MARGIN_METRES: f32 = 1.25;
pub(super) const REAR_RANGE_DEPTH_METRES: f32 = 6.0;
pub(super) const ACCESS_HALF_WIDTH_METRES: f32 = 0.4;
pub const MAX_CITY_BUILDING_INSTANCES: usize = MAX_CITY_LOTS * 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Reflect)]
#[serde(transparent)]
pub struct CityPropertyId(pub u64);

mod access;
mod region;
pub use access::CityAccessSegment;
pub(super) use access::translate_property_access;
pub use region::CityPlotBounds;

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
    pub front_building_id: crate::scene_input::SceneBuildingId,
    pub rear_building_id: crate::scene_input::SceneBuildingId,
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
