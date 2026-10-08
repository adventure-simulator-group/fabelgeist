//! One owned merchant property, including the routes and enclosure it reserves.
use super::*;
use crate::scene_coordinates::ScenePlanPoint;
pub use access::CityAccessSegment;
pub(super) use access::translate_property_access;
use adventuresim_building_generator::spatial_geometry::PositiveLength;
use bevy::prelude::Reflect;
pub use boundary::{
    BoundaryGeometryError, BoundarySupportElement, CityBoundaryMaterial, CityBoundaryMember,
    CityBoundaryPose,
};
pub use region::CityPlotBounds;
use serde::{Deserialize, Serialize};

mod boundary;

mod access;
mod region;

pub(super) const COMPOUND_EDGE_MARGIN_METRES: f32 = 1.25;
pub(super) const REAR_RANGE_DEPTH_METRES: f32 = 6.0;
pub(super) const ACCESS_HALF_WIDTH_METRES: f32 = 0.4;
pub const MAX_CITY_BUILDING_INSTANCES: usize = MAX_CITY_LOTS * 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, Reflect)]
#[serde(transparent)]
pub struct CityPropertyId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct CityBoundarySegment {
    pub start_metres: ScenePlanPoint,
    pub end_metres: ScenePlanPoint,
    pub height_metres: PositiveLength,
    pub thickness_metres: PositiveLength,
}

/// An inward-opening timber gate, in the same horizontal coordinates as its plot.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[serde(deny_unknown_fields)]
pub struct CityGate {
    pub hinge: PropertySide,
    pub centre_metres: ScenePlanPoint,
    pub orientation: BuildingOrientation,
    pub width_metres: PositiveLength,
    pub height_metres: PositiveLength,
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

impl CityBuildingLot {
    pub fn has_rear_range(self) -> bool {
        self.service.is_none() && self.house_class == CityHouseClass::MerchantHouse
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::reflect::{PartialReflect, ReflectRef};

    #[test]
    fn checked_enclosures_preserve_native_wire_geometry_and_reflection() {
        let mut gate = CityGate {
            hinge: PropertySide::Left,
            centre_metres: ScenePlanPoint::try_from(Vec2::new(12.5, -4.0)).unwrap(),
            orientation: BuildingOrientation::IDENTITY,
            width_metres: PositiveLength::from_metres(1.6).unwrap(),
            height_metres: PositiveLength::from_metres(1.8).unwrap(),
        };
        let bytes = postcard::to_allocvec(&gate).unwrap();
        assert_eq!(postcard::from_bytes::<CityGate>(&bytes).unwrap(), gate);
        let mut wire = serde_json::to_value(gate).unwrap();
        assert_eq!(wire["centre_metres"], serde_json::json!([12.5, -4.0]));
        assert_eq!(wire["width_metres"], serde_json::json!(1.6_f32));
        wire["width_metres"] = serde_json::json!(0.0);
        assert!(serde_json::from_value::<CityGate>(wire).is_err());
        assert!(matches!(
            gate.centre_metres.reflect_ref(),
            ReflectRef::Opaque(_)
        ));
        assert!(matches!(
            gate.width_metres.reflect_ref(),
            ReflectRef::Opaque(_)
        ));
        assert!(gate.width_metres.try_apply(&0.0_f32).is_err());
        assert_eq!(gate.width_metres.metres(), 1.6);
        let unadmitted = postcard::to_allocvec(&Vec2::splat(f32::NAN)).unwrap();
        assert!(postcard::from_bytes::<ScenePlanPoint>(&unadmitted).is_err());
    }
}
