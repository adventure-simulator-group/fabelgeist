use super::*;
use recipes::Recipe;

pub(super) mod clearance;
mod gate_sweep;
mod routes;
pub(super) use routes::validate_access;

const REAR_RANGE_SIDE_OFFSET_METRES: f32 = -0.5;
const BOUNDARY_HEIGHT_METRES: f32 = 1.8;
const BOUNDARY_THICKNESS_METRES: f32 = 0.3;
const GATE_WIDTH_METRES: f32 = 1.6;
const GATE_STREET_APPROACH_METRES: f32 = compound::COMPOUND_EDGE_MARGIN_METRES + 0.25;
const BOUNDARY_OUTSIDE_OFFSET_METRES: f32 = 0.45;
const AUXILIARY_BUILDING_ID_BASE: u64 = MAX_CITY_LOTS as u64;

/// Repeated merchant recipes differ only by rigid placement and identity. The
/// complete geometry/access proof is reusable; parcel fit and street connection
/// are still checked at every site. Authored scene inputs are checked separately.
#[derive(Default)]
pub(super) struct ClearanceCache {
    programmes: Vec<(
        adventuresim_building_generator::BuildingProgram,
        adventuresim_building_generator::BuildingProgram,
        Vec2,
        PropertySide,
    )>,
}

pub(super) fn compile(
    lot: CityBuildingLot,
    front: &TacticalBuildingPlacement,
    front_recipe: &Recipe,
    range_recipe: &Recipe,
    streets: &[CityStreetPatch],
    clearance_cache: &mut ClearanceCache,
) -> Result<(TacticalBuildingPlacement, CityCompound), CityCompileError> {
    let id = CityPropertyId(lot.id);
    let world = |p| lot.centre_metres + lot.orientation.local_to_world(p);
    let front_half = lot.footprint_metres * 0.5;
    let court_depth = plots::REAR_COURT_METRES;
    let rear = range_recipe.place(
        AUXILIARY_BUILDING_ID_BASE + lot.id,
        world(Vec2::new(
            lot.passage_side.sign() * REAR_RANGE_SIDE_OFFSET_METRES,
            front_half.y + court_depth + compound::REAR_RANGE_DEPTH_METRES * 0.5,
        )),
        lot.orientation,
    );
    let plot = CityPlotBounds::from(plots::reservation(lot));
    if !front_recipe.fits(front, plot) || !range_recipe.fits(&rear, plot) {
        return Err(CityCompileError::Compound {
            property: id,
            issue: CompoundIssue::GeometryOutsidePlot,
        });
    }
    let front_door = front_recipe
        .door_point(front, Vec2::Y)
        .ok_or(CityCompileError::Compound {
            property: id,
            issue: CompoundIssue::MissingCourtDoor,
        })?;
    let rear_door = range_recipe
        .door_point(&rear, -Vec2::Y)
        .ok_or(CityCompileError::Compound {
            property: id,
            issue: CompoundIssue::MissingRangeDoor,
        })?;
    let boundary = boundary(lot);
    let court = CityPlotBounds {
        centre_metres: world(Vec2::new(
            lot.passage_side.sign() * plots::SIDE_PASSAGE_METRES * 0.5,
            front_half.y + court_depth * 0.5,
        )),
        dimensions_metres: Vec2::new(
            lot.footprint_metres.x + plots::SIDE_PASSAGE_METRES,
            court_depth,
        ),
        orientation: lot.orientation,
    };
    let court_local = lot
        .orientation
        .world_to_local(court.centre_metres - lot.centre_metres);
    let gate_local = lot
        .orientation
        .world_to_local(boundary.gate.centre_metres - lot.centre_metres);
    let junction = world(Vec2::new(gate_local.x, court_local.y));
    let access = access(
        lot,
        boundary.gate,
        junction,
        court_local.y,
        [front_door, rear_door],
    );
    if !streets
        .iter()
        .any(|street| street.contains(access[0].start_metres))
    {
        return Err(CityCompileError::Compound {
            property: id,
            issue: CompoundIssue::StreetDisconnected,
        });
    }
    let compound = CityCompound {
        id,
        front_building_id: front.id,
        rear_building_id: rear.id,
        plot,
        court,
        access,
        boundary,
    };
    let key = (
        front_recipe.program.clone(),
        range_recipe.program.clone(),
        lot.footprint_metres,
        lot.passage_side,
    );
    if !clearance_cache.programmes.contains(&key) {
        clearance::validate(&compound, front, front_recipe, &rear, range_recipe)?;
        clearance_cache.programmes.push(key);
    }
    Ok((rear, compound))
}

fn access(
    lot: CityBuildingLot,
    gate: CityGate,
    junction: Vec2,
    court_local_north: f32,
    doors: [Vec2; 2],
) -> Vec<CityAccessSegment> {
    let mut routes = vec![CityAccessSegment {
        start_metres: gate.centre_metres
            + gate.orientation.local_to_world(-Vec2::Y) * GATE_STREET_APPROACH_METRES,
        end_metres: junction,
        half_width_metres: compound::ACCESS_HALF_WIDTH_METRES,
    }];
    for door in doors {
        let local = lot.orientation.world_to_local(door - lot.centre_metres);
        let turn = lot.centre_metres
            + lot
                .orientation
                .local_to_world(Vec2::new(local.x, court_local_north));
        for (start, end) in [(junction, turn), (turn, door)] {
            routes.push(CityAccessSegment {
                start_metres: start,
                end_metres: end,
                half_width_metres: compound::ACCESS_HALF_WIDTH_METRES,
            });
        }
    }
    routes
}

fn boundary(lot: CityBuildingLot) -> CityBoundary {
    let half = lot.footprint_metres * 0.5;
    let right = half.x + plots::SIDE_PASSAGE_METRES + BOUNDARY_OUTSIDE_OFFSET_METRES;
    let rear = half.y
        + plots::REAR_COURT_METRES
        + compound::REAR_RANGE_DEPTH_METRES
        + BOUNDARY_OUTSIDE_OFFSET_METRES;
    let left = -half.x - 0.12;
    let world = |p: Vec2| {
        lot.centre_metres
            + lot
                .orientation
                .local_to_world(Vec2::new(p.x * lot.passage_side.sign(), p.y))
    };
    let walls = [
        (Vec2::new(left, half.y), Vec2::new(left, rear)),
        (Vec2::new(left, rear), Vec2::new(right, rear)),
        (Vec2::new(right, rear), Vec2::new(right, -half.y)),
    ]
    .map(|(start, end)| CityBoundarySegment {
        start_metres: world(start),
        end_metres: world(end),
        height_metres: BOUNDARY_HEIGHT_METRES,
        thickness_metres: BOUNDARY_THICKNESS_METRES,
    })
    .to_vec();
    CityBoundary {
        walls,
        gate: CityGate {
            hinge: lot.passage_side.opposite(),
            centre_metres: world(Vec2::new(half.x + 1.35, -half.y)),
            orientation: lot.orientation,
            width_metres: GATE_WIDTH_METRES,
            height_metres: BOUNDARY_HEIGHT_METRES,
        },
    }
}

#[cfg(test)]
mod handedness_tests;
