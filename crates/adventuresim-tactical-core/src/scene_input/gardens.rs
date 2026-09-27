//! Immutable owned planting; horizontal poses survive authority partition unchanged.
use super::*;
mod grounding;
#[cfg(test)]
mod tests;
use crate::city_layout::{CityGarden, MAX_CITY_LOTS};
pub(super) use grounding::{GardenGrounding, terrain_anchors, validate_surface};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Component, Serialize, Deserialize)]
#[component(immutable)]
#[serde(deny_unknown_fields)]
pub struct SceneGarden {
    pub garden: CityGarden,
}

#[derive(Clone, Debug)]
pub struct GeneratedGarden {
    pub scene: SceneGarden,
    pub elevation_metres: f32,
}

pub(super) fn validate(input: &TacticalSceneInput) -> Result<(), SceneInputError> {
    if input.gardens.len() > MAX_CITY_LOTS {
        return invalid("scene exceeds garden count bound");
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
            || garden.owner.0 != garden.front_building_id
            || !owners.insert(garden.owner)
            || !members.insert(garden.front_building_id)
            || garden
                .plants
                .iter()
                .any(|p| p.id.0 == 0 || !plants.insert(p.id))
        {
            return invalid("garden ownership, membership or plant identity is invalid");
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
            return invalid("garden owner must reference an occupied front building");
        };
        if !garden.plot.contains(centre) || garden.cultivated_bounds.contains(centre) {
            return invalid("garden owner geometry lies outside its property");
        }
        garden.validate_geometry(&input.streets).map_err(|issue| {
            SceneInputError::Validation(format!("garden {}: {issue:?}", garden.owner.0))
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
            return invalid("garden overlaps another owned property");
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
        .map_err(SceneInputError::Validation)
    }
}

pub(super) fn generate(
    gardens: &[CityGarden],
    buildings: &[GeneratedBuilding],
) -> Vec<GeneratedGarden> {
    gardens
        .iter()
        .filter_map(|garden| {
            let front = buildings
                .iter()
                .find(|b| b.placement.id == garden.front_building_id)?;
            Some(GeneratedGarden {
                scene: SceneGarden {
                    garden: garden.clone(),
                },
                elevation_metres: front.pad_elevation_metres,
            })
        })
        .collect()
}
