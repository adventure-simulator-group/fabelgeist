//! One reserved keep stair hall owns both the authored flight and its occupied approaches.
use super::*;
use crate::{BuildingProgram, CELL_SIZE_METRES, Cell};

const KEEP_INNER_RADIUS_METRES: f32 = 0.25;
const KEEP_OUTER_RADIUS_METRES: f32 = 1.25;
const KEEP_BASE_HEIGHT_METRES: f32 = 0.15;
const KEEP_FLIGHT_TURNS: f32 = 2.8;
const LANDING_WALL_CLEARANCE_METRES: f32 = 0.45;
const STAIR_AISLE_WIDTH_METRES: f32 = 0.75;

pub(super) fn owns_occupied_storeys(archetype: BuildingArchetype) -> bool {
    matches!(
        archetype,
        BuildingArchetype::WalledKeep | BuildingArchetype::ArtilleryRondelCastle
    )
}

pub(crate) fn stair(program: &BuildingProgram) -> Option<Stair> {
    if !owns_occupied_storeys(program.archetype) || program.storeys.len() < 2 {
        return None;
    }
    let (width, depth) = program.footprint.dimensions();
    let rise =
        program.storeys.len() as f32 * program.storey_height_metres - KEEP_BASE_HEIGHT_METRES;
    Some(Stair::Spiral {
        centre: Vec2::new(f32::from(width), f32::from(depth)) * CELL_SIZE_METRES * 0.5,
        base_height_metres: KEEP_BASE_HEIGHT_METRES,
        rise_metres: rise,
        inner_radius_metres: KEEP_INNER_RADIUS_METRES,
        outer_radius_metres: KEEP_OUTER_RADIUS_METRES,
        turns: KEEP_FLIGHT_TURNS,
        clockwise: true,
        tread_count: required_treads(KEEP_BASE_HEIGHT_METRES, rise, program.storey_height_metres),
    })
}

pub(crate) fn reserved_cells(program: &BuildingProgram, footprint: &[Cell]) -> Vec<Cell> {
    let Some(stair) = stair(program) else {
        return Vec::new();
    };
    let flight =
        compile_flight(stair, program.storey_height_metres).expect("authored keep flight is valid");
    let (well_min, well_max) = well_bounds(stair).expect("keep circulation uses a spiral");
    // Reserve walking space around the shaft before walls and their doors exist.
    let mut min = well_min - Vec2::splat(STAIR_AISLE_WIDTH_METRES);
    let mut max = well_max + Vec2::splat(STAIR_AISLE_WIDTH_METRES);
    for landing in flight
        .landings
        .iter()
        .filter(|landing| usize::from(landing.storey) < program.storeys.len())
    {
        min = min.min(landing.position_metres - Vec2::splat(LANDING_WALL_CLEARANCE_METRES));
        max = max.max(landing.position_metres + Vec2::splat(LANDING_WALL_CLEARANCE_METRES));
    }
    footprint
        .iter()
        .copied()
        .filter(|cell| {
            let centre = cell.centre();
            let half = Vec2::splat(CELL_SIZE_METRES * 0.5);
            (centre + half).cmpgt(min).all() && (centre - half).cmplt(max).all()
        })
        .collect()
}
