//! Property-owned cultivated ground and accepted planting recipes.
use super::{CityAccessSegment, CityPlotBounds, CityPropertyId};
use crate::scene_input::BuildingOrientation;
use bevy::{math::Vec2, prelude::Reflect};
use serde::{Deserialize, Serialize};
mod geometry;
mod specimen;
pub use specimen::{
    GARDEN_LEAF_WIND_CLEARANCE_METRES, GARDEN_LEAF_WIND_STRENGTH_METRES, GardenSpecimen,
    GardenSpecimenEnvelope,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Reflect, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GardenPlantId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GardenPlantScale(f32);
impl GardenPlantScale {
    pub const fn new(value: f32) -> Self {
        Self(value)
    }
    pub const fn value(self) -> f32 {
        self.0
    }
    pub fn is_valid(self) -> bool {
        self.0.is_finite() && self.0 > 0.0
    }
}

/// An accepted plan position; the scene generator owns its terrain grounding.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GardenPlantPlacement {
    pub id: GardenPlantId,
    pub specimen: GardenSpecimen,
    pub centre_metres: Vec2,
    pub orientation: BuildingOrientation,
    pub scale: GardenPlantScale,
}
impl GardenPlantPlacement {
    pub fn world_hull(self) -> Vec<Vec2> {
        self.specimen
            .envelope()
            .hull_metres
            .iter()
            .map(|point| {
                self.centre_metres + self.orientation.local_to_world(*point * self.scale.value())
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CityGarden {
    pub owner: CityPropertyId,
    pub front_building_id: u64,
    pub plot: CityPlotBounds,
    pub cultivated_bounds: CityPlotBounds,
    pub beds: Vec<CityPlotBounds>,
    pub access: Vec<CityAccessSegment>,
    pub plants: Vec<GardenPlantPlacement>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GardenIssue {
    MissingOwner,
    DuplicateProperty,
    InvalidBounds,
    StreetDisconnected,
    InvalidAccess,
    DisconnectedTendingLane,
    ObstructedAccess,
    InvalidPlant,
    PlantOutsidePlot,
    PlantObstructsWorkingSpace,
    OverlappingPlants,
    BuildingObstruction,
    InvalidGrounding,
}
