//! Deterministic urban frontage around one connected, gently warped street network.

use std::collections::{BTreeMap, BTreeSet};

use bevy::math::Vec2;
use fabelgeist_determinism::mix64;

use crate::scene_input::BuildingOrientation;
use adventuresim_world_schema::{
    SettlementEconomyProfile,
    settlement_buildings::{BuildingDemand, BuildingUse, SettlementBuildingDemand},
};

mod houses;
mod services;
mod surfaces;

pub use houses::CityHouseClass;
pub use surfaces::{
    CityStreetPatch, CityStreetSurface, CityYardPatch, CityYardSurface, MAX_CITY_STREET_PATCHES,
    MAX_CITY_YARD_PATCHES,
};
use surfaces::{city_street_patches, city_yard_patches};

const STREET_LINE_COUNT: usize = 40;
const BLOCK_COUNT: usize = STREET_LINE_COUNT - 1;
const NOMINAL_BLOCK_METRES: f32 = 72.0;
const CITY_RADIUS_X_METRES: f32 = 1_300.0;
const CITY_RADIUS_Y_METRES: f32 = 1_260.0;
const STREET_LINE_JITTER_METRES: f32 = 8.0;
const BLOCK_SPACING_VARIATION_METRES: f32 = 16.0;
const STREET_CURVE_METRES: f32 = 6.0;
const ORDINARY_STREET_HALF_WIDTH_METRES: f32 = 3.5;
const SECONDARY_STREET_HALF_WIDTH_METRES: f32 = 4.5;
const PRIMARY_STREET_HALF_WIDTH_METRES: f32 = 6.0;
const FRONTAGE_CORNER_CLEARANCE_METRES: f32 = 7.0;
const PARTY_WALL_CLEARANCE_METRES: f32 = 0.12;
const REAR_COURT_EDGE_CLEARANCE_METRES: f32 = 27.5;
const REAR_COURT_PRIORITY_PENALTY: u32 = 7;
const MAXIMUM_HOUSE_DEPTH_METRES: f32 = 19.5;
const SPATIAL_BUCKET_METRES: f32 = 32.0;
const CENTRAL_MARKET_BLOCK: (usize, usize) = (BLOCK_COUNT / 2, BLOCK_COUNT / 2);
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
    block_key: u64,
    rear_court: bool,
    selection_key: u64,
}

#[derive(Clone, Copy)]
struct CityBlock {
    row: usize,
    column: usize,
    corners: [Vec2; 4],
}

impl CityBlock {
    fn centre(self) -> Vec2 {
        self.corners.into_iter().sum::<Vec2>() * 0.25
    }

    fn key(self) -> u64 {
        ((self.row as u64) << 32) | self.column as u64
    }

    fn is_market(self) -> bool {
        (self.row, self.column) == CENTRAL_MARKET_BLOCK
    }
}

/// Reserves service frontage, then houses residents around the same connected street graph.
pub fn generate_city(
    seed: u64,
    resident_population: u32,
    economy: &SettlementEconomyProfile,
) -> GeneratedCityLayout {
    let nodes = street_nodes(seed);
    let mut candidates = city_blocks(nodes)
        .filter(|block| block_is_inside_city(*block) && !block.is_market())
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
        nodes,
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
        represented_population = represented_population.saturating_add(if lot.service.is_none() {
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
    GeneratedCityLayout {
        lots: selected
            .into_iter()
            .map(|candidate| candidate.lot)
            .collect(),
        streets: city_street_patches(nodes, &developed_blocks),
        yards: city_yard_patches(seed, nodes, &developed_blocks),
        unplaced_services,
        unhoused_population: target_population.saturating_sub(represented_population),
    }
}

fn street_nodes(seed: u64) -> [[Vec2; STREET_LINE_COUNT]; STREET_LINE_COUNT] {
    let column_positions = street_axis(seed);
    let row_positions = street_axis(seed.rotate_left(29));
    core::array::from_fn(|row| {
        core::array::from_fn(|column| {
            let centred_column = column as f32 - (STREET_LINE_COUNT - 1) as f32 * 0.5;
            let centred_row = row as f32 - (STREET_LINE_COUNT - 1) as f32 * 0.5;
            let column_key = mix64(seed ^ STREET_GEOMETRY_DOMAIN ^ column as u64);
            let row_key = mix64(seed ^ STREET_GEOMETRY_DOMAIN ^ (row as u64).rotate_left(31));
            let column_shift = signed_sample(column_key) * STREET_LINE_JITTER_METRES;
            let row_shift = signed_sample(row_key) * STREET_LINE_JITTER_METRES;
            let vertical_phase = signed_sample(column_key.rotate_left(17)) * core::f32::consts::PI;
            let horizontal_phase = signed_sample(row_key.rotate_left(23)) * core::f32::consts::PI;
            Vec2::new(
                column_positions[column]
                    + column_shift
                    + (centred_row * 0.48 + vertical_phase).sin() * STREET_CURVE_METRES,
                row_positions[row]
                    + row_shift
                    + (centred_column * 0.44 + horizontal_phase).sin() * STREET_CURVE_METRES,
            )
        })
    })
}

fn street_axis(seed: u64) -> [f32; STREET_LINE_COUNT] {
    let mut positions = [0.0; STREET_LINE_COUNT];
    for index in 1..STREET_LINE_COUNT {
        positions[index] = positions[index - 1]
            + NOMINAL_BLOCK_METRES
            + signed_sample(mix64(seed ^ STREET_GEOMETRY_DOMAIN ^ index as u64))
                * BLOCK_SPACING_VARIATION_METRES;
    }
    let centre = (positions[BLOCK_COUNT / 2] + positions[BLOCK_COUNT / 2 + 1]) * 0.5;
    positions.map(|position| position - centre)
}

fn city_blocks(
    nodes: [[Vec2; STREET_LINE_COUNT]; STREET_LINE_COUNT],
) -> impl Iterator<Item = CityBlock> {
    (0..BLOCK_COUNT).flat_map(move |row| {
        (0..BLOCK_COUNT).map(move |column| CityBlock {
            row,
            column,
            corners: [
                nodes[row][column],
                nodes[row][column + 1],
                nodes[row + 1][column + 1],
                nodes[row + 1][column],
            ],
        })
    })
}

fn block_is_inside_city(block: CityBlock) -> bool {
    let centre = block.centre();
    (centre.x / CITY_RADIUS_X_METRES).powi(2) + (centre.y / CITY_RADIUS_Y_METRES).powi(2) <= 1.0
}

fn block_lots(seed: u64, block: CityBlock) -> Vec<CandidateLot> {
    let mut lots = Vec::new();
    let edges = [
        (block.corners[0], block.corners[1], block.row),
        (block.corners[1], block.corners[2], block.column + 1),
        (block.corners[2], block.corners[3], block.row + 1),
        (block.corners[3], block.corners[0], block.column),
    ];
    for (edge_index, (start, end, street_line)) in edges.into_iter().enumerate() {
        append_frontage(
            &mut lots,
            seed,
            block.key() ^ (edge_index as u64).rotate_left(48),
            block.key(),
            start,
            end,
            street_half_width(street_line),
        );
    }
    append_rear_court(&mut lots, seed, block);
    lots
}

fn append_frontage(
    lots: &mut Vec<CandidateLot>,
    seed: u64,
    run_key: u64,
    block_key: u64,
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
        let frontage = house_class.frontage_width_metres();
        if cursor + frontage > length - FRONTAGE_CORNER_CLEARANCE_METRES {
            break;
        }
        let street_point = start + tangent * (cursor + frontage * 0.5);
        lots.push(candidate(
            seed,
            lot_key,
            block_key,
            street_point + inward * (street_half_width + house_class.depth_metres() * 0.5),
            tangent,
            house_class,
            false,
        ));
        cursor += frontage + PARTY_WALL_CLEARANCE_METRES;
        index += 1;
    }
}

fn append_rear_court(lots: &mut Vec<CandidateLot>, seed: u64, block: CityBlock) {
    let horizontal =
        ((block.corners[1] - block.corners[0]) + (block.corners[2] - block.corners[3])) * 0.5;
    let vertical =
        ((block.corners[3] - block.corners[0]) + (block.corners[2] - block.corners[1])) * 0.5;
    let (axis, cross_extent) = if horizontal.length() >= vertical.length() {
        (horizontal, vertical.length())
    } else {
        (vertical, horizontal.length())
    };
    let required_cross_extent = 2.0
        * (PRIMARY_STREET_HALF_WIDTH_METRES + MAXIMUM_HOUSE_DEPTH_METRES)
        + CityHouseClass::Cottage.depth_metres();
    if cross_extent < required_cross_extent {
        return;
    }
    let length = axis.length();
    let tangent = axis / length;
    let start = block.centre() - tangent * length * 0.5;
    let mut cursor = REAR_COURT_EDGE_CLEARANCE_METRES;
    let mut index = 0_u64;
    while cursor + CityHouseClass::Cottage.frontage_width_metres()
        <= length - REAR_COURT_EDGE_CLEARANCE_METRES
    {
        let lot_key = block.key() ^ 0x7265_6172_0000_0000 ^ index.rotate_left(19);
        let house_class = house_class(seed, lot_key, true);
        let frontage = house_class.frontage_width_metres();
        if cursor + frontage > length - REAR_COURT_EDGE_CLEARANCE_METRES {
            break;
        }
        lots.push(candidate(
            seed,
            lot_key,
            block.key(),
            start + tangent * (cursor + frontage * 0.5),
            tangent,
            house_class,
            true,
        ));
        cursor += frontage + PARTY_WALL_CLEARANCE_METRES;
        index += 1;
    }
}

fn candidate(
    seed: u64,
    lot_key: u64,
    block_key: u64,
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

fn remove_overlapping_candidates(candidates: Vec<CandidateLot>) -> Vec<CandidateLot> {
    let mut accepted = Vec::<CandidateLot>::with_capacity(candidates.len());
    let mut buckets = BTreeMap::<(i32, i32), Vec<usize>>::new();
    for candidate in candidates {
        let bucket = spatial_bucket(candidate.lot.centre_metres);
        let overlaps = (-1..=1).any(|row_offset| {
            (-1..=1).any(|column_offset| {
                buckets
                    .get(&(bucket.0 + column_offset, bucket.1 + row_offset))
                    .into_iter()
                    .flatten()
                    .any(|&index| lots_overlap(candidate.lot, accepted[index].lot))
            })
        });
        if !overlaps {
            let index = accepted.len();
            accepted.push(candidate);
            buckets.entry(bucket).or_default().push(index);
        }
    }
    accepted
}

fn lots_overlap(first: CityBuildingLot, second: CityBuildingLot) -> bool {
    let first_half = first.dimensions_metres() * 0.5;
    let second_half = second.dimensions_metres() * 0.5;
    let first_axes = [
        first.orientation.local_to_world(Vec2::X),
        first.orientation.local_to_world(Vec2::Y),
    ];
    let second_axes = [
        second.orientation.local_to_world(Vec2::X),
        second.orientation.local_to_world(Vec2::Y),
    ];
    let centre_delta = second.centre_metres - first.centre_metres;
    first_axes.into_iter().chain(second_axes).all(|axis| {
        let first_radius = first_half.x * axis.dot(first_axes[0]).abs()
            + first_half.y * axis.dot(first_axes[1]).abs();
        let second_radius = second_half.x * axis.dot(second_axes[0]).abs()
            + second_half.y * axis.dot(second_axes[1]).abs();
        centre_delta.dot(axis).abs() < first_radius + second_radius
    })
}

fn spatial_bucket(point: Vec2) -> (i32, i32) {
    (
        (point.x / SPATIAL_BUCKET_METRES).floor() as i32,
        (point.y / SPATIAL_BUCKET_METRES).floor() as i32,
    )
}

fn street_half_width(line_index: usize) -> f32 {
    if line_index == CENTRAL_MARKET_BLOCK.0 || line_index == CENTRAL_MARKET_BLOCK.0 + 1 {
        PRIMARY_STREET_HALF_WIDTH_METRES
    } else if line_index.is_multiple_of(4) {
        SECONDARY_STREET_HALF_WIDTH_METRES
    } else {
        ORDINARY_STREET_HALF_WIDTH_METRES
    }
}

fn street_surface(line_index: usize) -> CityStreetSurface {
    if line_index == CENTRAL_MARKET_BLOCK.0 || line_index == CENTRAL_MARKET_BLOCK.0 + 1 {
        CityStreetSurface::Fieldstone
    } else if line_index.is_multiple_of(4) {
        CityStreetSurface::Gravel
    } else {
        CityStreetSurface::CompactedEarth
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
