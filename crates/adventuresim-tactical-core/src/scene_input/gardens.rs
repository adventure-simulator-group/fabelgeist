//! Immutable owned planting; horizontal poses survive authority partition unchanged.
use super::*;
#[cfg(test)]
mod tests;
use crate::city_layout::grounding::SupportElevation;
use crate::city_layout::{CityGarden, GardenPlantId, MAX_CITY_LOTS};
mod support;
use std::collections::BTreeSet;
pub use support::GardenSupportError;

#[derive(Clone, Debug, PartialEq, Component, Serialize, Deserialize)]
#[component(immutable)]
#[serde(deny_unknown_fields, try_from = "SceneGardenWire")]
pub struct SceneGarden {
    garden: CityGarden,
    /// Exact roots in scene coordinates; membership and horizontal poses stay
    /// in the authoritative garden descriptor. A house floor is not soil.
    plant_support: Vec<GardenPlantSupport>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SceneGardenWire {
    garden: CityGarden,
    plant_support: Vec<GardenPlantSupport>,
}
impl TryFrom<SceneGardenWire> for SceneGarden {
    type Error = GardenSupportError;
    fn try_from(wire: SceneGardenWire) -> Result<Self, Self::Error> {
        Self::from_support(wire.garden, wire.plant_support)
    }
}
impl SceneGarden {
    pub fn garden(&self) -> &CityGarden {
        &self.garden
    }
    pub fn plant_support(&self) -> &[GardenPlantSupport] {
        &self.plant_support
    }
    pub fn from_support(
        garden: CityGarden,
        plant_support: Vec<GardenPlantSupport>,
    ) -> Result<Self, GardenSupportError> {
        if garden.owner.0 == 0
            || garden.owner.0 > MAX_CITY_LOTS as u64
            || garden.front_building_id.0 == 0
            || garden.plants.iter().any(|plant| plant.id.0 == 0)
        {
            return Err(GardenSupportError::Identity {
                property: garden.owner,
                building: garden.front_building_id,
                plant: garden
                    .plants
                    .iter()
                    .find(|plant| plant.id.0 == 0)
                    .map(|plant| plant.id),
            });
        }
        let mut ids = BTreeSet::new();
        for (index, plant) in garden.plants.iter().enumerate() {
            let actual = plant_support.get(index).map(|root| root.plant_id);
            if !ids.insert(plant.id) || actual != Some(plant.id) {
                return Err(GardenSupportError::Membership {
                    property: garden.owner,
                    building: garden.front_building_id,
                    index,
                    expected: Some(plant.id),
                    actual,
                });
            }
        }
        if plant_support.len() != garden.plants.len() {
            return Err(GardenSupportError::Membership {
                property: garden.owner,
                building: garden.front_building_id,
                index: garden.plants.len(),
                expected: None,
                actual: plant_support
                    .get(garden.plants.len())
                    .map(|root| root.plant_id),
            });
        }
        Ok(Self {
            garden,
            plant_support,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GardenPlantSupport {
    pub plant_id: GardenPlantId,
    pub elevation: SupportElevation,
}

pub(super) fn validate(input: &TacticalSceneInput) -> Result<(), SceneInputError> {
    if input.gardens.len() > MAX_CITY_LOTS {
        return invalid(SceneValidationError::GardenCount);
    }
    let mut owners = input
        .compounds
        .iter()
        .map(|c| c.id)
        .collect::<BTreeSet<_>>();
    let mut members = input
        .compounds
        .iter()
        .flat_map(|c| [c.front_building_id, c.rear_building_id])
        .collect::<BTreeSet<_>>();
    let mut plants = BTreeSet::new();
    for garden in &input.gardens {
        if garden.owner.0 == 0
            || garden.owner.0 > MAX_CITY_LOTS as u64
            || garden.owner.0 != garden.front_building_id.0
            || !owners.insert(garden.owner)
            || !members.insert(garden.front_building_id)
            || garden
                .plants
                .iter()
                .any(|p| p.id.0 == 0 || !plants.insert(p.id))
        {
            return invalid(SceneValidationError::GardenOwnership {
                owner: super::validation_error::SceneOwnerContext::garden(garden),
            });
        }
        let owner = input
            .buildings
            .iter()
            .find(|b| b.id == garden.front_building_id)
            .map(|b| (b.program.usage, b.centre_metres))
            .or_else(|| {
                input
                    .distant_buildings
                    .iter()
                    .find(|b| b.id == garden.front_building_id)
                    .map(|b| (b.usage, b.centre_metres))
            });
        let Some((Some(_), centre)) = owner else {
            return invalid(SceneValidationError::GardenOwner {
                owner: super::validation_error::SceneOwnerContext::garden(garden),
            });
        };
        if !garden.plot.contains(centre.metres())
            || garden.cultivated_bounds.contains(centre.metres())
        {
            return invalid(SceneValidationError::GardenOwnerGeometry {
                owner: super::validation_error::SceneOwnerContext::garden(garden),
            });
        }
        garden.validate_geometry(&input.streets).map_err(|issue| {
            SceneInputError::Validation(SceneValidationError::Garden {
                owner: garden.owner,
                issue,
            })
        })?;
    }
    for (index, garden) in input.gardens.iter().enumerate() {
        if input.gardens[..index]
            .iter()
            .any(|g| g.plot.intersects(garden.plot))
            || input
                .compounds
                .iter()
                .any(|c| c.plot.intersects(garden.plot))
        {
            return invalid(SceneValidationError::GardenOverlap {
                owner: super::validation_error::SceneOwnerContext::garden(garden),
            });
        }
    }
    Ok(())
}

impl TacticalSceneInput {
    /// Exhaustively checks garden clearance against complete rendered buildings.
    ///
    /// Intended for scene authoring and tests. This compiles every distinct
    /// distant recipe, including its detailed meshes, and is not part of loading
    /// an accepted scene. Runtime validation retains ownership and plot checks.
    pub fn audit_garden_clearance(
        &self,
        generated: &GeneratedTacticalScene,
    ) -> Result<(), SceneInputError> {
        crate::city_layout::validate_scene_gardens(
            &self.gardens,
            &generated.buildings,
            &self.distant_buildings,
        )
        .map_err(|cause| SceneInputError::Validation(SceneValidationError::GardenClearance(cause)))
    }
}

pub(super) fn generate(
    gardens: &[CityGarden],
    buildings: &[GeneratedBuilding],
    terrain: &SceneTerrain,
) -> Result<Vec<SceneGarden>, SceneInputError> {
    gardens
        .iter()
        .filter(|garden| {
            buildings
                .iter()
                .any(|b| b.placement.id == garden.front_building_id)
        })
        .map(|garden| SceneGarden::project(garden.clone(), terrain).map_err(SceneInputError::from))
        .collect()
}
