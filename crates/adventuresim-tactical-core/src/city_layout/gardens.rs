//! Property-owned cultivated ground and accepted planting recipes.
use super::{CityAccessSegment, CityPlotBounds, CityPropertyId};
use crate::scene_input::BuildingOrientation;
use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};
pub use specimen::{
    GARDEN_LEAF_WIND_CLEARANCE_METRES, GARDEN_LEAF_WIND_STRENGTH_METRES, GardenSpecimen,
    GardenSpecimenEnvelope, GardenSpecimenError,
};
mod geometry;
mod specimen;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Reflect, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GardenPlantId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize)]
#[reflect(opaque)]
#[serde(transparent)]
pub struct GardenPlantScale(f32);

/// An accepted plan position; the scene generator owns its terrain grounding.
#[derive(Clone, Copy, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GardenPlantPlacement {
    pub id: GardenPlantId,
    pub specimen: GardenSpecimen,
    pub centre_metres: crate::scene_coordinates::ScenePlanPoint,
    pub orientation: BuildingOrientation,
    pub scale: GardenPlantScale,
}

#[derive(Clone, Debug, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CityGarden {
    pub owner: CityPropertyId,
    pub front_building_id: crate::scene_input::SceneBuildingId,
    pub plot: CityPlotBounds,
    pub cultivated_bounds: CityPlotBounds,
    pub beds: Vec<CityPlotBounds>,
    pub access: Vec<CityAccessSegment>,
    pub plants: Vec<GardenPlantPlacement>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, thiserror::Error)]
#[error("garden geometry rejected: {self:?}")]
#[serde(rename_all = "snake_case")]
pub enum GardenIssue {
    Specimen(GardenSpecimenError),
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
impl GardenPlantScale {
    pub const STANDARD: Self = Self(0.75);
    pub const fn new(value: f32) -> Option<Self> {
        if value.is_finite() && value > 0.0 {
            Some(Self(value))
        } else {
            None
        }
    }
    pub const fn value(self) -> f32 {
        self.0
    }
    pub fn is_valid(self) -> bool {
        self.0.is_finite() && self.0 > 0.0
    }
}

impl<'de> Deserialize<'de> for GardenPlantScale {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::new(f32::deserialize(d)?)
            .ok_or_else(|| serde::de::Error::custom("plant scale must be finite and positive"))
    }
}
impl GardenPlantPlacement {
    pub fn world_hull(self) -> Result<Vec<crate::scene_coordinates::ScenePlanPoint>, GardenIssue> {
        let hull = self
            .specimen
            .envelope()
            .map_err(GardenIssue::Specimen)?
            .hull_metres
            .iter()
            .map(|point| {
                crate::scene_coordinates::ScenePlanPoint::try_from(
                    self.centre_metres.metres()
                        + self.orientation.local_to_world(*point * self.scale.value()),
                )
                .map_err(|_| GardenIssue::InvalidPlant)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if hull
            .iter()
            .zip(hull.iter().cycle().skip(1))
            .take(hull.len())
            .any(|(first, second)| first == second)
        {
            return Err(GardenIssue::InvalidPlant);
        }
        Ok(hull)
    }
}
