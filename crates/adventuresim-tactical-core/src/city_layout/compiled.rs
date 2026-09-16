//! Compile accepted properties once before handing their exact recipes to a scene.
use super::*;
use crate::scene_input::{DistantBuildingPlacement, TacticalBuildingPlacement};
use adventuresim_building_generator::BuildingArchetype;

mod property;
mod recipes;
use recipes::RecipePalette;
#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CityCompileError {
    #[error("parish {parish:?} lacks its precinct or resident catchment")]
    Parish {
        parish: adventuresim_world_schema::settlement_buildings::ParishId,
    },
    #[error("playable city needs {required} building instances, exceeding {maximum}")]
    PlayableCapacity { required: usize, maximum: usize },
    #[error("city lacks room for {residents} residents and {services} service buildings")]
    Capacity { residents: u32, services: usize },
    #[error("{archetype:?} recipe from seed {seed} failed: {source}")]
    Recipe {
        archetype: BuildingArchetype,
        seed: u64,
        source: adventuresim_building_generator::GenerationError,
    },
    #[error("property {property:?} is not buildable: {issue:?}")]
    Compound {
        property: CityPropertyId,
        issue: CompoundIssue,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompoundIssue {
    MissingCourtDoor,
    MissingRangeDoor,
    GeometryOutsidePlot,
    AccessBlocked { building: u64 },
    GateBlocksOpenPassage,
    GateSweepBlocked,
    StreetDisconnected,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledCityLayout {
    pub parishes: Vec<CityParish>,
    pub buildings: Vec<TacticalBuildingPlacement>,
    pub compounds: Vec<CityCompound>,
    pub streets: Vec<CityStreetPatch>,
    pub yards: Vec<CityYardPatch>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CitySceneLayout {
    pub parishes: Vec<CityParish>,
    pub playable: Vec<TacticalBuildingPlacement>,
    pub distant: Vec<DistantBuildingPlacement>,
    pub compounds: Vec<CityCompound>,
    pub streets: Vec<CityStreetPatch>,
    pub yards: Vec<CityYardPatch>,
}

impl GeneratedCityLayout {
    /// Packing reserves complete properties cheaply. This stage validates actual
    /// generated envelopes and access before exposing any placement to a consumer.
    pub fn compile(self, seed: u64) -> Result<CompiledCityLayout, CityCompileError> {
        if self.unhoused_population > 0
            || !self.unplaced_services.is_empty()
            || !self.demand_shortfalls.is_empty()
        {
            return Err(CityCompileError::Capacity {
                residents: self.unhoused_population,
                services: self.unplaced_services.len() + self.demand_shortfalls.len(),
            });
        }
        let mut palette = RecipePalette::default();
        let parishes = self.parish_layout()?;
        let mut clearance_cache = property::ClearanceCache::default();
        let mut buildings = Vec::new();
        let mut compounds = Vec::new();
        for lot in self.lots {
            let recipe = palette.front(seed, lot)?;
            let front = recipe.place(lot.id, lot.centre_metres, lot.orientation);
            if lot.has_rear_range() {
                let range = palette.range()?;
                let (rear, compound) = property::compile(
                    lot,
                    &front,
                    &recipe,
                    &range,
                    &self.streets,
                    &mut clearance_cache,
                )?;
                buildings.push(rear);
                compounds.push(compound);
            }
            buildings.push(front);
        }
        Ok(CompiledCityLayout {
            parishes,
            buildings,
            compounds,
            streets: self.streets,
            yards: self.yards,
        })
    }
}

/// Recheck authored scene descriptors against their actual generated members.
pub(crate) fn validate_scene_compound(
    compound: &CityCompound,
    front: &crate::scene_input::GeneratedBuilding,
    rear: &crate::scene_input::GeneratedBuilding,
    streets: &[CityStreetPatch],
) -> Result<(), CityCompileError> {
    let front_recipe = recipes::Recipe::from_generated(front);
    let rear_recipe = recipes::Recipe::from_generated(rear);
    if !front_recipe.fits(&front.placement, compound.plot)
        || !rear_recipe.fits(&rear.placement, compound.plot)
    {
        return Err(CityCompileError::Compound {
            property: compound.id,
            issue: CompoundIssue::GeometryOutsidePlot,
        });
    }
    property::validate_access(
        compound,
        &front.placement,
        &front_recipe,
        &rear.placement,
        &rear_recipe,
        streets,
    )?;
    property::clearance::validate(
        compound,
        &front.placement,
        &front_recipe,
        &rear.placement,
        &rear_recipe,
    )
}

impl CompiledCityLayout {
    /// Every compound intersecting playable terrain retains collision for both
    /// members. Terrain pads extend to the clipped property boundary.
    /// `None` selects a presentation-only city, as used by the art viewer.
    pub fn partition(
        self,
        playable_half_extent_metres: Option<f32>,
    ) -> Result<CitySceneLayout, CityCompileError> {
        let mut playable_members = BTreeSet::new();
        let mut compound_members = BTreeSet::new();
        for compound in &self.compounds {
            let playable = playable_half_extent_metres.is_some_and(|extent| {
                compound.plot.intersects(CityPlotBounds {
                    centre_metres: Vec2::ZERO,
                    dimensions_metres: Vec2::splat(extent * 2.0),
                    orientation: BuildingOrientation::IDENTITY,
                })
            });
            compound_members.extend([compound.front_building_id, compound.rear_building_id]);
            if playable {
                playable_members.extend([compound.front_building_id, compound.rear_building_id]);
            }
        }
        let mut result = CitySceneLayout {
            parishes: self.parishes,
            compounds: self.compounds,
            streets: self.streets,
            yards: self.yards,
            ..Default::default()
        };
        for building in self.buildings {
            if playable_members.contains(&building.id)
                || (!compound_members.contains(&building.id)
                    && playable_half_extent_metres
                        .is_some_and(|extent| building.centre_metres.abs().max_element() <= extent))
            {
                result.playable.push(building);
            } else {
                result.distant.push(DistantBuildingPlacement {
                    id: building.id,
                    archetype: building.program.archetype,
                    usage: building.program.usage,
                    service_size: building.program.service_size,
                    seed: building.program.seed,
                    centre_metres: building.centre_metres,
                    orientation: building.orientation,
                    base_elevation_metres: 0.0,
                });
            }
        }
        let maximum = crate::scene_input::buildings::MAX_TACTICAL_BUILDINGS;
        if result.playable.len() > maximum {
            return Err(CityCompileError::PlayableCapacity {
                required: result.playable.len(),
                maximum,
            });
        }
        Ok(result)
    }
}
