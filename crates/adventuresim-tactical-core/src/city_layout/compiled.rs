//! Compile accepted properties once before handing their exact recipes to a scene.
use super::*;
use crate::scene_input::{DistantBuildingPlacement, TacticalBuildingPlacement};
use adventuresim_building_generator::BuildingArchetype;
use adventuresim_world_schema::settlement_buildings::BusinessKey;

mod assembly;
mod church;
mod gardens;
mod grounded;
mod homes;
mod packing;
mod single_support;
pub use grounded::{
    CityGroundingError, CityGroundingProjection, CityGroundingProjectionError,
    GroundedCitySceneLayout, SelectedCityGrounding,
};
mod support;
pub(crate) use gardens::validate_scene_gardens;
pub use single_support::{CitySingleProperty, SinglePropertyGradingPolicy};
pub use support::{CitySupportError, CompoundGradingPolicy, StreetApronDimensions};
mod property;
pub use church::ChurchSitingIssue;
mod recipes;
pub use recipes::CityRecipePalette;
#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum CityCompileError {
    #[error("property {property:?} cannot be packed: {issue:?}")]
    Packing {
        property: CityPropertyId,
        issue: CityPackingIssue,
    },
    #[error("church {building} is not buildable: {issue:?}")]
    Church {
        building: u64,
        issue: ChurchSitingIssue,
    },
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
    pub prosperity: adventuresim_world_schema::ProsperityTier,
    pub parishes: Vec<CityParish>,
    pub buildings: Vec<TacticalBuildingPlacement>,
    pub compounds: Vec<CityCompound>,
    pub single_properties: Vec<CitySingleProperty>,
    pub streets: Vec<CityStreetPatch>,
    pub yards: Vec<CityYardPatch>,
    pub gardens: Vec<CityGarden>,
    pub businesses: Vec<CityBusinessSite>,
    /// Transient memo of the accepted physical recipes, retained for grounding.
    pub support_recipes: CityRecipePalette,
}

/// The physical building selected for one settlement-local business key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CityBusinessSite {
    pub building_id: u64,
    pub key: BusinessKey,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CitySceneLayout {
    pub parishes: Vec<CityParish>,
    pub playable: Vec<TacticalBuildingPlacement>,
    pub distant: Vec<DistantBuildingPlacement>,
    pub compounds: Vec<CityCompound>,
    pub single_properties: Vec<CitySingleProperty>,
    pub streets: Vec<CityStreetPatch>,
    pub yards: Vec<CityYardPatch>,
    pub gardens: Vec<CityGarden>,
    pub businesses: Vec<CityBusinessSite>,
    /// Transient memo of the accepted physical recipes, retained for grounding.
    pub support_recipes: CityRecipePalette,
}

impl GeneratedCityLayout {
    /// Compile the selected roster, then solve its measured physical packing.
    pub fn compile(mut self, seed: u64) -> Result<CompiledCityLayout, CityCompileError> {
        let context = std::mem::replace(
            &mut self.packing,
            Ok(super::packing::CityPackingContext::default()),
        )?;
        let mut compiled = self.compile_properties(seed)?;
        compiled.finalize_packing(&context)?;
        Ok(compiled)
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
        for garden in &self.gardens {
            compound_members.insert(garden.front_building_id);
            if playable_half_extent_metres.is_some_and(|extent| {
                garden.plot.intersects(CityPlotBounds {
                    centre_metres: Vec2::ZERO,
                    dimensions_metres: Vec2::splat(extent * 2.0),
                    orientation: BuildingOrientation::IDENTITY,
                })
            }) {
                playable_members.insert(garden.front_building_id);
            }
        }
        let mut result = CitySceneLayout {
            parishes: self.parishes,
            compounds: self.compounds,
            single_properties: self.single_properties,
            gardens: self.gardens,
            streets: self.streets,
            yards: self.yards,
            businesses: self.businesses,
            support_recipes: self.support_recipes,
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
                    prosperity: self.prosperity,
                    id: building.id,
                    archetype: building.program.archetype,
                    usage: building.program.usage,
                    service_size: building.program.service_size,
                    seed: building.program.seed,
                    centre_metres: building.centre_metres,
                    orientation: building.orientation,
                    base_elevation_metres: building.base_elevation_metres,
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
