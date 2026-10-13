use super::*;
use recipes::Recipe;
pub(super) use routes::validate_access;

pub(super) mod clearance;
mod gate_sweep;
mod routes;

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
    programmes: Vec<ClearanceKey>,
}

#[derive(PartialEq)]
struct ClearanceKey {
    front: adventuresim_building_generator::BuildingProgram,
    rear: adventuresim_building_generator::BuildingProgram,
    dimensions: PlanDimensions,
    passage: PropertySide,
}

struct CourtDoors {
    front: ScenePlanPoint,
    rear: ScenePlanPoint,
}

pub(super) struct CompiledCompound {
    pub rear: TacticalBuildingPlacement,
    pub compound: CityCompound,
}

pub(super) fn compile(
    lot: CityBuildingLot,
    front: &TacticalBuildingPlacement,
    front_recipe: &Recipe,
    range_recipe: &Recipe,
    streets: &[CityStreetPatch],
    clearance_cache: &mut ClearanceCache,
) -> CityCompileResult<CompiledCompound> {
    let id = lot.id;
    let world = |p| lot.centre_metres.metres() + lot.orientation.local_to_world(p);
    let front_half = lot.footprint_metres.metres() * 0.5;
    let court_depth = plots::REAR_COURT_METRES;
    let rear = range_recipe.place(
        (AUXILIARY_BUILDING_ID_BASE + lot.id.0).into(),
        lot.centre_metres.translated(PlanDisplacement::try_from(
            lot.orientation.local_to_world(Vec2::new(
                lot.passage_side.sign() * REAR_RANGE_SIDE_OFFSET_METRES,
                front_half.y + court_depth + compound::REAR_RANGE_DEPTH_METRES * 0.5,
            )),
        )?)?,
        lot.orientation,
    )?;
    let plot = plots::reservation(lot)?;
    if !front_recipe.fits(front, plot) || !range_recipe.fits(&rear, plot) {
        return Err(CityCompileError::Compound {
            property: id,
            issue: CompoundIssue::GeometryOutsidePlot,
        });
    }
    let front_door = front_recipe
        .door_point(front, adventuresim_building_generator::Direction::North)?
        .ok_or(CityCompileError::Compound {
            property: id,
            issue: CompoundIssue::MissingCourtDoor,
        })?;
    let rear_door = range_recipe
        .door_point(&rear, adventuresim_building_generator::Direction::South)?
        .ok_or(CityCompileError::Compound {
            property: id,
            issue: CompoundIssue::MissingRangeDoor,
        })?;
    let boundary = boundary(lot)?;
    let court = CityPlotBounds::new(
        crate::scene_coordinates::ScenePlanPoint::try_from(world(Vec2::new(
            lot.passage_side.sign() * plots::SIDE_PASSAGE_METRES * 0.5,
            front_half.y + court_depth * 0.5,
        )))?,
        adventuresim_building_generator::spatial_geometry::PlanDimensions::from_metres(Vec2::new(
            lot.footprint_metres.metres().x + plots::SIDE_PASSAGE_METRES,
            court_depth,
        ))?,
        lot.orientation,
    )?;
    let court_local = lot
        .orientation
        .world_to_local(court.centre_metres() - lot.centre_metres.metres());
    let gate_local = lot
        .orientation
        .world_to_local(boundary.gate.centre_metres.metres() - lot.centre_metres.metres());
    let junction = ScenePlanPoint::try_from(world(Vec2::new(gate_local.x, court_local.y)))?;
    let access = access(
        lot,
        boundary.gate,
        junction,
        court_local.y,
        CourtDoors {
            front: front_door,
            rear: rear_door,
        },
    )?;
    if !streets
        .iter()
        .any(|street| street.contains(access[0].start_metres()))
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
    let key = ClearanceKey {
        front: front_recipe.program.clone(),
        rear: range_recipe.program.clone(),
        dimensions: lot.footprint_metres,
        passage: lot.passage_side,
    };
    if !clearance_cache.programmes.contains(&key) {
        clearance::validate(&compound, front, front_recipe, &rear, range_recipe)?;
        clearance_cache.programmes.push(key);
    }
    Ok(CompiledCompound { rear, compound })
}

fn access(
    lot: CityBuildingLot,
    gate: CityGate,
    junction: ScenePlanPoint,
    court_local_north: f32,
    doors: CourtDoors,
) -> CityCompileResult<Vec<CityAccessSegment>> {
    let mut routes = vec![CityAccessSegment::new(
        crate::scene_coordinates::ScenePlanPoint::try_from(
            gate.centre_metres.metres()
                + gate.orientation.local_to_world(-Vec2::Y) * GATE_STREET_APPROACH_METRES,
        )?,
        junction,
        adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
            compound::ACCESS_HALF_WIDTH_METRES,
        )?,
    )?];
    for door in [doors.front, doors.rear] {
        let local = lot
            .orientation
            .world_to_local(door.metres() - lot.centre_metres.metres());
        let turn = ScenePlanPoint::try_from(
            lot.centre_metres.metres()
                + lot
                    .orientation
                    .local_to_world(Vec2::new(local.x, court_local_north)),
        )?;
        for (start, end) in [(junction, turn), (turn, door)] {
            routes.push(CityAccessSegment::new(
                start,
                end,
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    compound::ACCESS_HALF_WIDTH_METRES,
                )?,
            )?);
        }
    }
    Ok(routes)
}

fn boundary(lot: CityBuildingLot) -> GeometryResult<CityBoundary> {
    let half = lot.footprint_metres.metres() * 0.5;
    let right = half.x + plots::SIDE_PASSAGE_METRES + BOUNDARY_OUTSIDE_OFFSET_METRES;
    let rear = half.y
        + plots::REAR_COURT_METRES
        + compound::REAR_RANGE_DEPTH_METRES
        + BOUNDARY_OUTSIDE_OFFSET_METRES;
    let left = -half.x - 0.12;
    let world = |p: Vec2| {
        lot.centre_metres.metres()
            + lot
                .orientation
                .local_to_world(Vec2::new(p.x * lot.passage_side.sign(), p.y))
    };
    let walls = [
        (Vec2::new(left, half.y), Vec2::new(left, rear)),
        (Vec2::new(left, rear), Vec2::new(right, rear)),
        (Vec2::new(right, rear), Vec2::new(right, -half.y)),
    ]
    .into_iter()
    .map(|(start, end)| {
        Ok(CityBoundarySegment {
            start_metres: ScenePlanPoint::try_from(world(start))?,
            end_metres: ScenePlanPoint::try_from(world(end))?,
            height_metres:
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    BOUNDARY_HEIGHT_METRES,
                )?,
            thickness_metres:
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    BOUNDARY_THICKNESS_METRES,
                )?,
        })
    })
    .collect::<GeometryResult<Vec<_>>>()?;
    Ok(CityBoundary {
        walls,
        gate: CityGate {
            hinge: lot.passage_side.opposite(),
            centre_metres: ScenePlanPoint::try_from(world(Vec2::new(half.x + 1.35, -half.y)))?,
            orientation: lot.orientation,
            width_metres:
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    GATE_WIDTH_METRES,
                )?,
            height_metres:
                adventuresim_building_generator::spatial_geometry::PositiveLength::from_metres(
                    BOUNDARY_HEIGHT_METRES,
                )?,
        },
    })
}

#[cfg(test)]
mod handedness_tests;
