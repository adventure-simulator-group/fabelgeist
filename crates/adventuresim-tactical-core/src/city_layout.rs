//! Deterministic urban plots around a surveyed street graph and staggered old-quarter lanes.

use std::collections::BTreeSet;

use bevy::math::Vec2;
use fabelgeist_determinism::mix64;

use crate::scene_input::BuildingOrientation;
use adventuresim_world_schema::{
    SettlementEconomyProfile,
    settlement_buildings::{BuildingDemand, BuildingUse, SettlementBuildingDemand},
};

mod compiled;
mod compound;
mod graph;
pub(crate) use compiled::validate_scene_compound;
pub use compiled::{CityCompileError, CitySceneLayout, CompiledCityLayout, CompoundIssue};
pub use compound::{
    CityAccessSegment, CityBoundary, CityBoundaryMaterial, CityBoundaryMember, CityBoundarySegment,
    CityCompound, CityGate, CityPlotBounds, CityPropertyId, MAX_CITY_BUILDING_INSTANCES,
};
mod houses;
mod subdivision;
use graph::{BlockId, CityBlock, StreetClass, StreetGraph};
mod plots;
#[cfg(test)]
use plots::lots_overlap;
use plots::remove_overlapping_candidates;
mod services;
mod site;
pub use site::CitySite;
use site::DevelopmentExtent;
mod surfaces;

pub use houses::CityHouseClass;
pub use surfaces::{
    CityStreetPatch, CityStreetSurface, CityYardPatch, CityYardSurface, MAX_CITY_STREET_PATCHES,
    MAX_CITY_YARD_PATCHES,
};
use surfaces::{city_street_patches, city_yard_patches};

const NOMINAL_BLOCK_METRES: f32 = 72.0;
const CITY_RADIUS_X_METRES: f32 = 1_300.0;
const CITY_RADIUS_Y_METRES: f32 = 1_260.0;
const ORDINARY_STREET_HALF_WIDTH_METRES: f32 = 3.5;
const SECONDARY_STREET_HALF_WIDTH_METRES: f32 = 4.5;
const PRIMARY_STREET_HALF_WIDTH_METRES: f32 = 6.0;
const FRONTAGE_CORNER_CLEARANCE_METRES: f32 = 7.0;
const PARTY_WALL_CLEARANCE_METRES: f32 = 0.12;
const REAR_COURT_PRIORITY_PENALTY: u32 = 7;
const SPATIAL_BUCKET_METRES: f32 = 32.0;
const STREET_GEOMETRY_DOMAIN: u64 = 0x7374_7265_6574_6765;
const HOUSE_CLASS_DOMAIN: u64 = 0x686f_7573_655f_636c;
const DEVELOPMENT_DOMAIN: u64 = 0x6465_7665_6c6f_706d;
pub const MAX_CITY_LOTS: usize = 16_384;

#[derive(Clone, Debug, PartialEq)]
pub struct GeneratedCityLayout {
    pub lots: Vec<CityBuildingLot>,
    pub streets: Vec<CityStreetPatch>,
    pub yards: Vec<CityYardPatch>,
    pub unplaced_services: Vec<BuildingDemand>,
    pub unhoused_population: u32,
}

/// One rectangular building lot aligned to one locally straight street frontage.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CityBuildingLot {
    pub id: u64,
    pub centre_metres: Vec2,
    pub orientation: BuildingOrientation,
    pub house_class: CityHouseClass,
    pub footprint_metres: Vec2,
    pub service: Option<BuildingDemand>,
}

#[derive(Clone, Copy)]
struct CandidateLot {
    lot: CityBuildingLot,
    block_key: BlockId,
    rear_court: bool,
    selection_key: u64,
}

/// Reserves service frontage, then houses residents around the same connected street graph.
impl CitySite {
    pub fn generate(
        &self,
        seed: u64,
        resident_population: u32,
        economy: &SettlementEconomyProfile,
    ) -> GeneratedCityLayout {
        let extent = DevelopmentExtent::for_population(resident_population);
        let graph = self.street_graph(seed, extent);
        let mut candidates = graph
            .blocks
            .iter()
            .copied()
            .filter(|block| block_is_inside_city(*block, extent) && !block.is_market())
            .flat_map(|block| block_lots(seed, block))
            .collect::<Vec<_>>();
        candidates.sort_by_key(|candidate| {
            (
                candidate.rear_court,
                candidate.selection_key,
                candidate.block_key,
            )
        });
        let demand = SettlementBuildingDemand::new(seed, resident_population, economy);
        if !demand.shortfalls.is_empty() {
            let mut unplaced_services = demand.buildings;
            unplaced_services.extend(demand.shortfalls.into_iter().map(|(usage, capacity)| {
                BuildingDemand {
                    usage,
                    ordinal: u32::MAX,
                    capacity,
                }
            }));
            return GeneratedCityLayout {
                lots: Vec::new(),
                streets: Vec::new(),
                yards: Vec::new(),
                unplaced_services,
                unhoused_population: resident_population,
            };
        }

        let (service_lots, unplaced_services) = services::place_services(
            seed,
            resident_population,
            &graph.blocks,
            &candidates,
            &demand.buildings,
        );
        let mut candidates =
            remove_overlapping_candidates(service_lots.into_iter().chain(candidates).collect());
        candidates.sort_by_key(|candidate| {
            let radial_band = (candidate.lot.centre_metres.length() / NOMINAL_BLOCK_METRES) as u32
                + u32::from(candidate.rear_court) * REAR_COURT_PRIORITY_PENALTY;
            (
                candidate.lot.service.is_none(),
                radial_band,
                candidate.block_key,
                candidate.selection_key,
            )
        });

        let target_population = resident_population;
        let mut represented_population = 0_u32;
        let mut selected = Vec::new();
        for candidate in candidates.into_iter().take(MAX_CITY_LOTS) {
            if candidate.lot.service.is_none() && represented_population >= target_population {
                break;
            }
            let mut lot = candidate.lot;
            lot.id = selected.len() as u64 + 1;
            represented_population =
                represented_population.saturating_add(if lot.service.is_none() {
                    lot.house_class.resident_capacity()
                } else {
                    0
                });
            selected.push(CandidateLot { lot, ..candidate });
        }
        let developed_blocks = selected
            .iter()
            .map(|candidate| candidate.block_key)
            .collect::<BTreeSet<_>>();
        let yards = city_yard_patches(seed, &selected);
        let streets = city_street_patches(&graph, &developed_blocks);
        GeneratedCityLayout {
            lots: selected
                .into_iter()
                .map(|candidate| candidate.lot)
                .collect(),
            streets,
            yards,
            unplaced_services,
            unhoused_population: target_population.saturating_sub(represented_population),
        }
    }
}

fn block_is_inside_city(block: CityBlock, extent: DevelopmentExtent) -> bool {
    let centre = block.centre();
    (centre.x / CITY_RADIUS_X_METRES).powi(2) + (centre.y / extent.radius_y_metres).powi(2) <= 1.0
}

fn block_lots(seed: u64, block: CityBlock) -> Vec<CandidateLot> {
    let mut lots = Vec::new();
    for edge_index in 0..4 {
        append_frontage(
            &mut lots,
            seed,
            block.key().0 ^ (edge_index as u64).rotate_left(48),
            block.key(),
            block.corners[edge_index],
            block.corners[(edge_index + 1) % 4],
            block.streets[edge_index].half_width(),
        );
    }
    lots.retain(|candidate| plots::inside_block(candidate.lot, block));
    lots
}

fn append_frontage(
    lots: &mut Vec<CandidateLot>,
    seed: u64,
    run_key: u64,
    block_key: BlockId,
    start: Vec2,
    end: Vec2,
    street_half_width: f32,
) {
    let displacement = end - start;
    let length = displacement.length();
    let tangent = displacement / length;
    let inward = Vec2::new(-tangent.y, tangent.x);
    let mut cursor = FRONTAGE_CORNER_CLEARANCE_METRES;
    let mut index = 0_u64;
    loop {
        let lot_key = run_key ^ index.rotate_left(19);
        let house_class = house_class(seed, lot_key, false);
        let compound_margin = if house_class == CityHouseClass::MerchantHouse {
            compound::COMPOUND_EDGE_MARGIN_METRES
        } else {
            0.0
        };
        let frontage = house_class.frontage_width_metres() + compound_margin * 2.0;
        if cursor + frontage > length - FRONTAGE_CORNER_CLEARANCE_METRES {
            break;
        }
        let street_point = start + tangent * (cursor + frontage * 0.5);
        lots.push(candidate(
            seed,
            lot_key,
            block_key,
            street_point
                + inward * (street_half_width + compound_margin + house_class.depth_metres() * 0.5),
            tangent,
            house_class,
            false,
        ));
        cursor += frontage + plots::SIDE_PASSAGE_METRES + PARTY_WALL_CLEARANCE_METRES;
        index += 1;
    }
}

fn candidate(
    seed: u64,
    lot_key: u64,
    block_key: BlockId,
    centre_metres: Vec2,
    frontage_tangent: Vec2,
    house_class: CityHouseClass,
    rear_court: bool,
) -> CandidateLot {
    CandidateLot {
        lot: CityBuildingLot {
            id: lot_key,
            centre_metres,
            orientation: BuildingOrientation::from_frontage_tangent(frontage_tangent)
                .expect("street frontage tangent is finite and nonzero"),
            house_class,
            footprint_metres: Vec2::new(
                house_class.frontage_width_metres(),
                house_class.depth_metres(),
            ),
            service: None,
        },
        block_key,
        rear_court,
        selection_key: mix64(seed ^ DEVELOPMENT_DOMAIN ^ lot_key),
    }
}

fn house_class(seed: u64, lot_key: u64, rear_court: bool) -> CityHouseClass {
    let sample = mix64(seed ^ HOUSE_CLASS_DOMAIN ^ lot_key);
    if rear_court {
        return if sample.is_multiple_of(5) {
            CityHouseClass::CraftTownHouse
        } else {
            CityHouseClass::Cottage
        };
    }
    match sample % 16 {
        0..=4 => CityHouseClass::Cottage,
        5..=11 => CityHouseClass::CraftTownHouse,
        12..=13 => CityHouseClass::HallHouse,
        _ => CityHouseClass::MerchantHouse,
    }
}

fn signed_sample(sample: u64) -> f32 {
    (sample as u32 as f32 / u32::MAX as f32) * 2.0 - 1.0
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod capacity_tests;
