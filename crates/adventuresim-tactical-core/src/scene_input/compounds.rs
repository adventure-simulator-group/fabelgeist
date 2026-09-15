use super::*;
use crate::city_layout::{CityBoundary, CityCompound, CityPropertyId, MAX_CITY_LOTS};
use std::collections::{BTreeMap, BTreeSet};
#[cfg(test)]
mod tests;

const MAX_PROPERTY_EXTENT_METRES: f32 = 100.0;
const MAX_PROPERTY_ACCESS_SEGMENTS: usize = 16;
const MAX_PROPERTY_WALL_SEGMENTS: usize = 16;
const MAX_ACCESS_HALF_WIDTH_METRES: f32 = 2.0;

/// Immutable enclosure anchored at its property's level ground elevation.
/// Horizontal coordinates remain in the scene's shared settlement frame.
#[derive(Clone, Debug, PartialEq, Component, Serialize, Deserialize)]
#[component(immutable)]
#[serde(deny_unknown_fields)]
pub struct SceneBoundary {
    pub property_id: CityPropertyId,
    pub front_building_id: u64,
    pub boundary: CityBoundary,
}

#[derive(Clone, Debug)]
pub struct GeneratedBoundary {
    pub scene: SceneBoundary,
    pub elevation_metres: f32,
}

pub(super) fn validate(input: &TacticalSceneInput) -> Result<(), SceneInputError> {
    if input.compounds.len() > MAX_CITY_LOTS {
        return invalid("scene exceeds compound count bound");
    }
    let mut buildings = BTreeMap::new();
    for (id, archetype, usage, playable) in input
        .buildings
        .iter()
        .map(|b| (b.id, b.program.archetype, b.program.usage, true))
        .chain(
            input
                .distant_buildings
                .iter()
                .map(|b| (b.id, b.archetype, b.usage, false)),
        )
    {
        if buildings.insert(id, (archetype, usage, playable)).is_some() {
            return invalid("building identity occurs in both simulation and distant presentation");
        }
    }
    let mut ids = BTreeSet::new();
    let mut members = BTreeSet::new();
    for compound in &input.compounds {
        if compound.id.0 == 0
            || compound.id.0 > MAX_CITY_LOTS as u64
            || !ids.insert(compound.id)
            || !members.insert(compound.front_building_id)
            || !members.insert(compound.rear_building_id)
        {
            return invalid("compound identity or membership is invalid");
        }
        let Some(front) = buildings.get(&compound.front_building_id) else {
            return invalid("compound front building is missing");
        };
        let Some(rear) = buildings.get(&compound.rear_building_id) else {
            return invalid("compound rear building is missing");
        };
        use adventuresim_building_generator::BuildingArchetype;
        if front.0 != BuildingArchetype::FachwerkMerchantHouse
            || rear.0 != BuildingArchetype::StorageRange
            || rear.1.is_some()
            || front.2 != rear.2
        {
            return invalid("compound members have invalid roles or split simulation authority");
        }
        validate_geometry(compound)?;
    }
    Ok(())
}

fn validate_geometry(compound: &CityCompound) -> Result<(), SceneInputError> {
    let bounded = |v: f32| v.is_finite() && v > 0.0 && v <= MAX_PROPERTY_EXTENT_METRES;
    if !compound.plot.is_valid()
        || !compound.court.is_valid()
        || compound.plot.dimensions_metres.max_element() > MAX_PROPERTY_EXTENT_METRES
        || compound
            .court
            .corners()
            .iter()
            .any(|p| !compound.plot.contains(*p))
        || compound.access.is_empty()
        || compound.access.len() > MAX_PROPERTY_ACCESS_SEGMENTS
        || compound.boundary.walls.len() > MAX_PROPERTY_WALL_SEGMENTS
    {
        return invalid("compound plot, court or route count is invalid");
    }
    for route in &compound.access {
        if !route.start_metres.is_finite()
            || !route.end_metres.is_finite()
            || !bounded(route.half_width_metres)
            || route.half_width_metres > MAX_ACCESS_HALF_WIDTH_METRES
            || route.start_metres.distance(route.end_metres) > MAX_PROPERTY_EXTENT_METRES
            || !compound.plot.contains(route.end_metres)
        {
            return invalid("compound access route is invalid");
        }
    }
    for wall in &compound.boundary.walls {
        if !wall.start_metres.is_finite()
            || !wall.end_metres.is_finite()
            || !bounded(wall.start_metres.distance(wall.end_metres))
            || !bounded(wall.height_metres)
            || !bounded(wall.thickness_metres)
            || !compound.plot.contains(wall.start_metres)
            || !compound.plot.contains(wall.end_metres)
        {
            return invalid("compound boundary wall is invalid");
        }
    }
    let gate = compound.boundary.gate;
    if !gate.orientation.is_valid()
        || !compound.plot.contains(gate.centre_metres)
        || !bounded(gate.width_metres)
        || !bounded(gate.height_metres)
    {
        return invalid("compound gate is invalid");
    }
    Ok(())
}

pub(super) fn generate(
    compounds: &[CityCompound],
    buildings: &[GeneratedBuilding],
) -> Vec<GeneratedBoundary> {
    compounds
        .iter()
        .filter_map(|compound| {
            let front = buildings
                .iter()
                .find(|b| b.placement.id == compound.front_building_id)?;
            Some(GeneratedBoundary {
                scene: SceneBoundary {
                    property_id: compound.id,
                    front_building_id: compound.front_building_id,
                    boundary: compound.boundary.clone(),
                },
                elevation_metres: front.pad_elevation_metres,
            })
        })
        .collect()
}

pub(super) fn validate_generated(
    compounds: &[CityCompound],
    buildings: &[GeneratedBuilding],
    streets: &[CityStreetPatch],
) -> Result<(), SceneInputError> {
    for compound in compounds {
        let Some(front) = buildings
            .iter()
            .find(|b| b.placement.id == compound.front_building_id)
        else {
            continue;
        };
        let rear = buildings
            .iter()
            .find(|b| b.placement.id == compound.rear_building_id)
            .expect("input validation requires both members to share authority");
        crate::city_layout::validate_scene_compound(compound, front, rear, streets)
            .map_err(|error| SceneInputError::Validation(error.to_string()))?;
    }
    Ok(())
}
