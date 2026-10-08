use super::*;
use crate::city_layout::{CityBoundary, CityCompound, CityPropertyId, MAX_CITY_LOTS};
use std::collections::{BTreeMap, BTreeSet};

const MAX_PROPERTY_EXTENT_METRES: f32 = 100.0;
const MAX_PROPERTY_ACCESS_SEGMENTS: usize = 16;
const MAX_PROPERTY_WALL_SEGMENTS: usize = 16;
const MAX_ACCESS_HALF_WIDTH_METRES: f32 = 2.0;

/// Immutable enclosure anchored at its accepted gate landing elevation.
/// Horizontal coordinates remain in the scene's shared settlement frame.
#[derive(Clone, Debug, PartialEq, Component, Serialize, Deserialize)]
#[component(immutable)]
#[serde(try_from = "SceneBoundaryWire")]
pub struct SceneBoundary {
    property_id: CityPropertyId,
    front_building_id: crate::scene_input::SceneBuildingId,
    boundary: CityBoundary,
    fixed_support: crate::city_layout::grounding::BoundarySupportMesh,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "GeneratedBoundaryWire")]
pub struct GeneratedBoundary {
    scene: SceneBoundary,
    elevation_metres: crate::city_layout::grounding::SupportElevation,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SceneBoundaryWire {
    property_id: CityPropertyId,
    front_building_id: crate::scene_input::SceneBuildingId,
    boundary: CityBoundary,
    fixed_support: crate::city_layout::grounding::BoundarySupportMesh,
}
#[derive(Deserialize)]
struct GeneratedBoundaryWire {
    scene: SceneBoundary,
    elevation_metres: crate::city_layout::grounding::SupportElevation,
}
impl TryFrom<SceneBoundaryWire> for SceneBoundary {
    type Error = crate::city_layout::grounding::enclosure::BoundaryAdmissionError;
    fn try_from(wire: SceneBoundaryWire) -> Result<Self, Self::Error> {
        wire.fixed_support.binding().validate_scene(
            wire.property_id,
            wire.front_building_id,
            &wire.boundary,
        )?;
        Ok(Self {
            property_id: wire.property_id,
            front_building_id: wire.front_building_id,
            boundary: wire.boundary,
            fixed_support: wire.fixed_support,
        })
    }
}
impl SceneBoundary {
    pub fn property_id(&self) -> CityPropertyId {
        self.property_id
    }
    pub fn front_building_id(&self) -> crate::scene_input::SceneBuildingId {
        self.front_building_id
    }
    pub fn boundary(&self) -> &CityBoundary {
        &self.boundary
    }
    pub fn fixed_support(&self) -> &crate::city_layout::grounding::BoundarySupportMesh {
        &self.fixed_support
    }
}
impl TryFrom<GeneratedBoundaryWire> for GeneratedBoundary {
    type Error = crate::city_layout::grounding::enclosure::BoundaryAdmissionError;
    fn try_from(wire: GeneratedBoundaryWire) -> Result<Self, Self::Error> {
        let expected = wire.scene.fixed_support.binding().gate();
        if wire.elevation_metres != expected {
            return Err(Self::Error::Datum {
                property: wire.scene.property_id,
                expected,
                actual: wire.elevation_metres,
            });
        }
        Ok(Self {
            scene: wire.scene,
            elevation_metres: wire.elevation_metres,
        })
    }
}
impl GeneratedBoundary {
    pub fn scene(&self) -> &SceneBoundary {
        &self.scene
    }
    pub fn elevation_metres(&self) -> crate::city_layout::grounding::SupportElevation {
        self.elevation_metres
    }
    pub fn into_scene(self) -> SceneBoundary {
        self.scene
    }
}

impl GeneratedBoundary {
    /// Project the complete enclosure atomically from its exact accepted owner.
    /// No member floor, neighbouring support or sampled-terrain fallback exists.
    pub fn project(
        compound: &CityCompound,
        terrain: &SceneTerrain,
    ) -> Result<Self, SceneInputError> {
        use crate::city_layout::grounding::enclosure::{
            BoundarySupportConstraint, BoundarySupportElement,
        };
        use crate::city_layout::grounding::{BoundarySupportError, BoundarySupportMesh};
        let foundation = terrain.property_foundation(compound.id).ok_or_else(|| {
            BoundarySupportError::new(
                compound,
                BoundarySupportElement::Owner,
                BoundarySupportConstraint::OwnerBinding,
                compound.plot.centre_metres(),
                1.0,
                0.0,
            )
        })?;
        let policy = crate::city_layout::CompoundGradingPolicy::bounded_settlement();
        let projection =
            BoundarySupportMesh::project(compound, foundation, policy.limits, policy.embedment)?;
        Ok(Self {
            scene: SceneBoundary {
                property_id: compound.id,
                front_building_id: compound.front_building_id,
                boundary: compound.boundary.clone(),
                fixed_support: projection.mesh,
            },
            elevation_metres: projection.gate_elevation,
        })
    }
}
pub(super) fn validate(input: &TacticalSceneInput) -> Result<(), SceneInputError> {
    if input.compounds.len() > MAX_CITY_LOTS {
        return invalid(SceneValidationError::CompoundCount);
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
            return invalid(SceneValidationError::SplitBuildingAuthority);
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
            return invalid(SceneValidationError::CompoundIdentity {
                owner: super::validation_error::SceneOwnerContext::compound(compound),
            });
        }
        let Some(front) = buildings.get(&compound.front_building_id) else {
            return invalid(SceneValidationError::MissingCompoundFront {
                owner: super::validation_error::SceneOwnerContext::compound(compound),
            });
        };
        let Some(rear) = buildings.get(&compound.rear_building_id) else {
            return invalid(SceneValidationError::MissingCompoundRear {
                owner: super::validation_error::SceneOwnerContext::compound(compound),
            });
        };
        use adventuresim_building_generator::BuildingArchetype;
        if front.0 != BuildingArchetype::FachwerkMerchantHouse
            || rear.0 != BuildingArchetype::StorageRange
            || rear.1.is_some()
            || front.2 != rear.2
        {
            return invalid(SceneValidationError::CompoundMemberAuthority {
                owner: super::validation_error::SceneOwnerContext::compound(compound),
            });
        }
        validate_geometry(compound)?;
    }
    Ok(())
}

pub(super) fn generate(
    compounds: &[CityCompound],
    buildings: &[GeneratedBuilding],
    terrain: &SceneTerrain,
) -> Result<Vec<GeneratedBoundary>, SceneInputError> {
    compounds
        .iter()
        .filter(|compound| {
            buildings
                .iter()
                .any(|b| b.placement.id == compound.front_building_id)
        })
        .map(|compound| GeneratedBoundary::project(compound, terrain))
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
            .ok_or_else(|| {
                SceneInputError::Validation(SceneValidationError::MissingCompoundRear {
                    owner: super::validation_error::SceneOwnerContext::compound(compound),
                })
            })?;
        crate::city_layout::validate_scene_compound(compound, front, rear, streets)
            .map_err(|error| SceneInputError::Validation(SceneValidationError::City(error)))?;
    }
    Ok(())
}

fn validate_geometry(compound: &CityCompound) -> Result<(), SceneInputError> {
    let bounded = |v: f32| v.is_finite() && v > 0.0 && v <= MAX_PROPERTY_EXTENT_METRES;
    if !compound.plot.is_valid()
        || !compound.court.is_valid()
        || compound.plot.dimensions_metres().max_element() > MAX_PROPERTY_EXTENT_METRES
        || compound
            .court
            .corners()
            .iter()
            .any(|p| !compound.plot.contains(*p))
        || compound.access.is_empty()
        || compound.access.len() > MAX_PROPERTY_ACCESS_SEGMENTS
        || compound.boundary.walls.len() > MAX_PROPERTY_WALL_SEGMENTS
    {
        return invalid(SceneValidationError::CompoundPlot {
            owner: super::validation_error::SceneOwnerContext::compound(compound),
        });
    }
    for route in &compound.access {
        if !route.start_metres().is_finite()
            || !route.end_metres().is_finite()
            || !bounded(route.half_width_metres())
            || route.half_width_metres() > MAX_ACCESS_HALF_WIDTH_METRES
            || route.start_metres().distance(route.end_metres()) > MAX_PROPERTY_EXTENT_METRES
            || !compound.plot.contains(route.end_metres())
        {
            return invalid(SceneValidationError::CompoundAccess {
                owner: super::validation_error::SceneOwnerContext::compound(compound),
            });
        }
    }
    for wall in &compound.boundary.walls {
        if !wall.start_metres.metres().is_finite()
            || !wall.end_metres.metres().is_finite()
            || !bounded(
                wall.start_metres
                    .metres()
                    .distance(wall.end_metres.metres()),
            )
            || !bounded(wall.height_metres.metres())
            || !bounded(wall.thickness_metres.metres())
            || !compound.plot.contains(wall.start_metres.metres())
            || !compound.plot.contains(wall.end_metres.metres())
        {
            return invalid(SceneValidationError::CompoundWall {
                owner: super::validation_error::SceneOwnerContext::compound(compound),
            });
        }
    }
    let gate = compound.boundary.gate;
    if !gate.orientation.is_valid()
        || !compound.plot.contains(gate.centre_metres.metres())
        || !bounded(gate.width_metres.metres())
        || !bounded(gate.height_metres.metres())
    {
        return invalid(SceneValidationError::CompoundGate {
            owner: super::validation_error::SceneOwnerContext::compound(compound),
        });
    }
    Ok(())
}
#[cfg(test)]
mod tests;
