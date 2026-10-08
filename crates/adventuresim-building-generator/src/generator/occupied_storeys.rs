//! Occupied rooms and shared circulation precede any architectural envelope.
use super::*;

const ROOM_ALLOCATION: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("building.storey.room-allocation");
const OPENING_LAYOUT: fabelgeist_determinism::StreamId =
    fabelgeist_determinism::StreamId::new("building.storey.opening-layout");

pub(super) fn generate_storeys(
    program: &BuildingProgram,
    edits: &[BuildingEdit],
) -> Result<(Vec<StoreyPlan>, Option<StraightStairCore>), GenerationError> {
    if let Some(storey) = small_church::occupied_storey(program)? {
        if !edits.is_empty() {
            return Err(GenerationError::UnsupportedEdit(
                "small church bays are edited through their service programme".to_owned(),
            ));
        }
        return Ok((vec![storey], None));
    }
    let footprint_cells = footprint_cells(program.footprint)?;
    // Preserve the public boundary's earliest programme-shape errors before
    // validating requirements that refer to those storeys.
    for (level, storey_program) in program.storeys.iter().enumerate() {
        if storey_program.rooms.is_empty() {
            return Err(GenerationError::EmptyStorey {
                level: StoreyIndex::new(level),
            });
        }
        if storey_program.rooms.len() > footprint_cells.len() {
            return Err(GenerationError::TooManyRooms {
                level: StoreyIndex::new(level),
                rooms: storey_program.rooms.len(),
                cells: footprint_cells.len(),
            });
        }
    }
    let core = resolve_straight_stair_core(program, &footprint_cells)?;
    let mut storeys = program
        .storeys
        .iter()
        .enumerate()
        .map(|(level, _)| {
            allocate_storey(
                program,
                edits,
                &footprint_cells,
                core.as_ref(),
                StoreyIndex::new(level),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    if program.church_program.is_some() || program.workplace_kind().is_some() {
        // These structural programmes own their envelopes; occupancy labels retain no duplicate grid walls.
        for storey in &mut storeys {
            storey.walls.clear();
            storey.openings.clear();
        }
    }
    Ok((storeys, core))
}

fn allocate_storey(
    program: &BuildingProgram,
    edits: &[BuildingEdit],
    footprint_cells: &[Cell],
    straight_stair_core: Option<&StraightStairCore>,
    level: StoreyIndex,
) -> Result<StoreyPlan, GenerationError> {
    let serialized_level = level.serialized_ordinal()?;
    let storey_program = &program.storeys[level.index()];
    let (width, depth) = program.footprint.dimensions();
    let layout_seed = layout_seed(program);
    let mut reservations = BTreeMap::new();
    if let Some(core) = straight_stair_core.filter(|core| core.serves(serialized_level)) {
        let Some(room_index) = storey_program
            .rooms
            .iter()
            .position(|room| room.kind == core.landing_room)
        else {
            return Err(GenerationError::UnsatisfiedVerticalCirculation {
                connection: 0,
                reason: format!(
                    "storey {level} has no {:?} to contain its stair core",
                    core.landing_room
                ),
            });
        };
        let room_index = RoomIndex::from_ordinal(room_index)?;
        reservations.extend(
            core.reserved_cells
                .iter()
                .copied()
                .map(|cell| (cell, room_index)),
        );
    }
    let keep_cells = crate::spiral_stairs::keep_reserved_cells(program, footprint_cells)?;
    if !keep_cells.is_empty() {
        let room_index = storey_program
            .rooms
            .iter()
            .position(|room| room.kind == RoomKind::StairHall)
            .ok_or_else(|| GenerationError::UnsatisfiedVerticalCirculation {
                connection: 0,
                reason: format!("storey {level} has no StairHall for its keep spiral"),
            })?;
        let room_index = RoomIndex::from_ordinal(room_index)?;
        reservations.extend(keep_cells.into_iter().map(|cell| (cell, room_index)));
    }
    let heating_storey = heated_rooms::HeatingStorey::for_program(program, level);
    if let Some(storey) = &heating_storey {
        heated_rooms::reserve(program, storey, footprint_cells, &mut reservations)?;
    }
    let assignments = room_allocation::allocate(
        footprint_cells,
        width,
        depth,
        &storey_program.rooms,
        ROOM_ALLOCATION.seed(layout_seed, &[level.index() as u64]),
        program.archetype,
        &reservations,
    )?;
    let rooms = room_allocation::collect_rooms(&assignments, &storey_program.rooms)?;
    for room in &rooms {
        if !cells_are_connected(&room.cells) {
            return Err(GenerationError::DisconnectedRoom {
                level,
                room: RoomIndex::from_serialized(room.id),
            });
        }
    }
    let walls = derive_walls(footprint_cells, &assignments)?;
    let mut openings = derive_openings(
        &walls,
        &storey_program.rooms,
        program.archetype,
        OPENING_LAYOUT.seed(layout_seed, &[level.index() as u64]),
        level,
        straight_stair_core,
    )?;
    if heating_storey.is_some() {
        heated_rooms::doorway(storey_program, &walls, &mut openings)?;
    }
    apply_opening_edits(
        storey_program,
        serialized_level,
        &walls,
        &mut openings,
        edits,
    )?;
    Ok(StoreyPlan {
        level: serialized_level,
        rooms,
        walls,
        openings,
    })
}
