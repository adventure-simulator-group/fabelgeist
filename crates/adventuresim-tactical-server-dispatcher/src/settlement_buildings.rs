//! Cheap deterministic settlement layout for tactical scenes.

use std::{error::Error, fmt};

use adventuresim_building_generator::{BuildingArchetype, BuildingProgram};
use adventuresim_core::reputation::effective_population;
use adventuresim_tactical_core::prelude::{
    CityStreetPatch, CityYardPatch, DistantBuildingPlacement, TacticalBuildingPlacement,
    generate_city,
};
use adventuresim_world_schema::{SettlementEconomyProfile, settlement_buildings::BuildingUse};
use fabelgeist_determinism::mix64;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const RESIDENTIAL_RECIPE_VARIANTS: u64 = 12;
const SERVICE_RECIPE_VARIANTS: u64 = 2;
const BUILDING_RECIPE_DOMAIN: u64 = 0x7265_6369_7065_0001;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettlementBuildingError {
    Recipe {
        archetype: BuildingArchetype,
        initial_seed: u64,
        usage: BuildingUse,
        cause: adventuresim_building_generator::GenerationError,
    },
    Capacity {
        unhoused_population: u32,
        unplaced_services: usize,
    },
}

impl fmt::Display for SettlementBuildingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Recipe {
                archetype,
                usage,
                initial_seed,
                cause,
            } => write!(
                formatter,
                "no valid {usage:?} / {archetype:?} recipe from seed {initial_seed}: {cause:?}"
            ),
            Self::Capacity {
                unhoused_population,
                unplaced_services,
            } => write!(
                formatter,
                "settlement layout lacks room for {unhoused_population} residents and {unplaced_services} service buildings"
            ),
        }
    }
}

impl Error for SettlementBuildingError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementSceneProfile {
    pub id: String,
    pub population_level: i32,
    pub population_estimate: u32,
    pub economy: SettlementEconomyProfile,
}

impl SettlementSceneProfile {
    fn effective_population(&self) -> u32 {
        effective_population(self.population_level, self.population_estimate)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SettlementBuildingLayout {
    pub streets: Vec<CityStreetPatch>,
    pub yards: Vec<CityYardPatch>,
    pub playable: Vec<TacticalBuildingPlacement>,
    pub distant: Vec<DistantBuildingPlacement>,
}

pub fn place_settlement_buildings(
    settlement: &SettlementSceneProfile,
    playable_half_extent_metres: f32,
) -> Result<SettlementBuildingLayout, SettlementBuildingError> {
    let population = settlement.effective_population();
    let settlement_seed = settlement_seed(&settlement.id);
    let city = generate_city(settlement_seed, population, &settlement.economy);
    if !city.unplaced_services.is_empty() || city.unhoused_population > 0 {
        return Err(SettlementBuildingError::Capacity {
            unhoused_population: city.unhoused_population,
            unplaced_services: city.unplaced_services.len(),
        });
    }
    let mut palette = BTreeMap::new();
    let mut layout = SettlementBuildingLayout {
        streets: city.streets,
        yards: city.yards,
        ..Default::default()
    };
    for lot in city.lots {
        let selection = mix64(settlement_seed ^ BUILDING_RECIPE_DOMAIN ^ lot.id);
        let usage = lot.building_use().unwrap_or(BuildingUse::Dwelling);
        let archetype = lot.archetype();
        let variants = if usage == BuildingUse::Dwelling {
            RESIDENTIAL_RECIPE_VARIANTS
        } else {
            SERVICE_RECIPE_VARIANTS
        };
        let variant = selection % variants;
        let key = (archetype.slug(), usage, variant);
        if let std::collections::btree_map::Entry::Vacant(entry) = palette.entry(key) {
            let recipe_seed = mix64(
                settlement_seed
                    ^ BUILDING_RECIPE_DOMAIN
                    ^ (archetype as u64).rotate_left(17)
                    ^ (usage as u64).rotate_left(31)
                    ^ variant,
            );
            entry.insert(
                BuildingProgram::validated_settlement(archetype, usage, recipe_seed).map_err(
                    |cause| SettlementBuildingError::Recipe {
                        archetype,
                        usage,
                        initial_seed: recipe_seed,
                        cause,
                    },
                )?,
            );
        }
        let program = &palette[&key];
        if lot.centre_metres.abs().max_element() <= playable_half_extent_metres {
            layout.playable.push(TacticalBuildingPlacement {
                id: lot.id,
                program: program.clone(),
                centre_metres: lot.centre_metres,
                orientation: lot.orientation,
            });
        } else {
            layout.distant.push(DistantBuildingPlacement {
                usage: program.usage,
                id: lot.id,
                archetype: program.archetype,
                seed: program.seed,
                centre_metres: lot.centre_metres,
                base_elevation_metres: 0.0,
                orientation: lot.orientation,
            });
        }
    }
    Ok(layout)
}

fn settlement_seed(id: &str) -> u64 {
    let digest = Sha256::digest(id.as_bytes());
    u64::from_le_bytes(digest[..8].try_into().expect("SHA-256 prefix"))
}

#[cfg(test)]
mod tests;
