//! Occupied rooms and shared circulation precede any architectural envelope.
use super::*;

pub(super) fn generate_storeys(
    program: &BuildingProgram,
    edits: &[BuildingEdit],
) -> Result<(Vec<StoreyPlan>, Option<StraightStairCore>), GenerationError> {
    if let Some(storey) = small_church::occupied_storey(program) {
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
            return Err(GenerationError::EmptyStorey { level });
        }
        if storey_program.rooms.len() > footprint_cells.len() {
            return Err(GenerationError::TooManyRooms {
                level,
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
        .map(|(level, _)| allocate_storey(program, edits, &footprint_cells, core.as_ref(), level))
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
    level: usize,
) -> Result<StoreyPlan, GenerationError> {
    let storey_program = &program.storeys[level];
    let (width, depth) = program.footprint.dimensions();
    let layout_seed = layout_seed(program);
    let mut reservations = BTreeMap::new();
    if let Some(core) = straight_stair_core.filter(|core| core.serves(level as u16)) {
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
        reservations.extend(
            core.reserved_cells
                .iter()
                .copied()
                .map(|cell| (cell, room_index)),
        );
    }
    let assignments = allocate_rooms(
        footprint_cells,
        width,
        depth,
        &storey_program.rooms,
        layout_seed.wrapping_add(level as u64 * 0x9e37_79b9),
        program.archetype,
        &reservations,
    );
    let rooms = collect_rooms(&assignments, &storey_program.rooms);
    for room in &rooms {
        if !cells_are_connected(&room.cells) {
            return Err(GenerationError::DisconnectedRoom {
                level,
                room: room.id,
            });
        }
    }
    let walls = derive_walls(footprint_cells, &assignments);
    let mut openings = derive_openings(
        &walls,
        &storey_program.rooms,
        program.archetype,
        layout_seed.wrapping_add(level as u64),
        level,
        straight_stair_core,
    )?;
    apply_opening_edits(storey_program, level as u16, &walls, &mut openings, edits)?;
    Ok(StoreyPlan {
        level: level as u16,
        rooms,
        walls,
        openings,
    })
}
