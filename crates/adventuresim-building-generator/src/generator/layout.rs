fn footprint_cells(footprint: Footprint) -> Result<Vec<Cell>, GenerationError> {
    let (width, depth) = footprint.dimensions();
    if width < 3 || depth < 3 || width > i16::MAX as u16 || depth > i16::MAX as u16 {
        return Err(GenerationError::InvalidFootprint);
    }
    let mut cells = Vec::new();
    match footprint {
        Footprint::Rectangle { .. } => {
            for z in 0..depth {
                for x in 0..width {
                    cells.push(Cell::new(x as i16, z as i16));
                }
            }
        }
        Footprint::Courtyard {
            wing, gate_width, ..
        } => {
            if wing < 2
                || wing * 2 >= width
                || wing * 2 >= depth
                || gate_width == 0
                || gate_width > width - wing * 2
            {
                return Err(GenerationError::InvalidFootprint);
            }
            for z in 0..depth {
                for x in 0..width {
                    if x < wing || x >= width - wing || z < wing || z >= depth - wing {
                        cells.push(Cell::new(x as i16, z as i16));
                    }
                }
            }
        }
    }
    Ok(cells)
}

fn resolve_straight_stair_core(
    program: &BuildingProgram,
    footprint: &[Cell],
) -> Result<Option<StraightStairCore>, GenerationError> {
    validate_vertical_connections(program)?;
    let straight = program
        .vertical_connections
        .iter()
        .enumerate()
        .filter_map(|(index, requirement)| match *requirement {
            VerticalConnectionRequirement::StraightStair {
                lowest_storey,
                highest_storey,
                landing_room,
            } => Some((index, lowest_storey, highest_storey, landing_room)),
            VerticalConnectionRequirement::TowerSpiral { .. } => None,
        })
        .collect::<Vec<_>>();
    let Some(&(connection, lowest_storey, highest_storey, landing_room)) = straight.first() else {
        return Ok(None);
    };
    if straight.len() > 1 {
        return Err(GenerationError::UnsatisfiedVerticalCirculation {
            connection,
            reason: "the bounded civilian solver supports one shared straight stair core"
                .to_owned(),
        });
    }
    if lowest_storey != 0 || usize::from(highest_storey) + 1 != program.storeys.len() {
        return Err(GenerationError::UnsatisfiedVerticalCirculation {
            connection,
            reason: "the bounded civilian stair core must serve every occupied storey".to_owned(),
        });
    }
    for level in lowest_storey..=highest_storey {
        if !program.storeys[usize::from(level)]
            .rooms
            .iter()
            .any(|room| room.kind == landing_room)
        {
            return Err(GenerationError::UnsatisfiedVerticalCirculation {
                connection,
                reason: format!("storey {level} has no {landing_room:?} landing room"),
            });
        }
    }

    // A 4 x 2 cell core contains the 3.2 m flight, its 1.0 m clear width,
    // stringers, and a landing at both ends.  Reserving the same room cells on
    // every served storey prevents later wall derivation from boxing in an
    // otherwise physically valid stair.
    let usable = footprint.iter().copied().collect::<HashSet<_>>();
    let (width, depth) = program.footprint.dimensions();
    let building_centre = Vec2::new(f32::from(width), f32::from(depth)) * 0.5;
    let mut candidates = Vec::new();
    for z in 0..i16::try_from(depth).map_err(|_| GenerationError::InvalidFootprint)? {
        for x in 0..i16::try_from(width).map_err(|_| GenerationError::InvalidFootprint)? {
            let anchor = Cell::new(x, z);
            for direction in Direction::ALL {
                let (long, lateral) = match direction {
                    Direction::North | Direction::South => ((0_i16, 1_i16), (1_i16, 0_i16)),
                    Direction::East | Direction::West => ((1_i16, 0_i16), (0_i16, 1_i16)),
                };
                let cells = (0..4_i16)
                    .flat_map(|along| {
                        (0..2_i16).map(move |across| {
                            Cell::new(
                                anchor.x + long.0 * along + lateral.0 * across,
                                anchor.z + long.1 * along + lateral.1 * across,
                            )
                        })
                    })
                    .collect::<Vec<_>>();
                if !cells.iter().all(|cell| usable.contains(cell)) {
                    continue;
                }
                let rectangle_centre =
                    cells.iter().map(|cell| cell.centre()).sum::<Vec2>() / cells.len() as f32;
                let origin = match direction {
                    Direction::North => Vec2::new(
                        (f32::from(anchor.x) + 1.0) * CELL_SIZE_METRES,
                        (f32::from(anchor.z) + 0.5) * CELL_SIZE_METRES,
                    ),
                    Direction::South => Vec2::new(
                        (f32::from(anchor.x) + 1.0) * CELL_SIZE_METRES,
                        (f32::from(anchor.z) + 3.5) * CELL_SIZE_METRES,
                    ),
                    Direction::East => Vec2::new(
                        (f32::from(anchor.x) + 0.5) * CELL_SIZE_METRES,
                        (f32::from(anchor.z) + 1.0) * CELL_SIZE_METRES,
                    ),
                    Direction::West => Vec2::new(
                        (f32::from(anchor.x) + 3.5) * CELL_SIZE_METRES,
                        (f32::from(anchor.z) + 1.0) * CELL_SIZE_METRES,
                    ),
                };
                let centre_distance =
                    (rectangle_centre / CELL_SIZE_METRES - building_centre).length_squared();
                let direction_salt = match direction {
                    Direction::North => 0,
                    Direction::East => 1,
                    Direction::South => 2,
                    Direction::West => 3,
                };
                candidates.push((
                    centre_distance,
                    cell_random(
                        layout_seed(program),
                        direction_salt,
                        anchor,
                        fabelgeist_determinism::StreamId::new("building.stair-placement"),
                    )
                    .next_u64(),
                    origin,
                    direction,
                    cells,
                ));
            }
        }
    }
    candidates.sort_by(|left, right| {
        left.0
            .total_cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
    });
    let Some((_, _, origin, direction, reserved_cells)) = candidates.into_iter().next() else {
        return Err(GenerationError::UnsatisfiedVerticalCirculation {
            connection,
            reason: "the footprint has no contiguous 4 x 2 cell stair-and-landing core".to_owned(),
        });
    };
    Ok(Some(StraightStairCore {
        lowest_storey,
        highest_storey,
        landing_room,
        origin,
        direction,
        reserved_cells,
    }))
}

fn cells_are_connected(cells: &[Cell]) -> bool {
    let Some(first) = cells.first().copied() else {
        return false;
    };
    let all = cells.iter().copied().collect::<HashSet<_>>();
    let mut reached = HashSet::from([first]);
    let mut pending = VecDeque::from([first]);
    while let Some(cell) = pending.pop_front() {
        for direction in Direction::ALL {
            let neighbour = cell.neighbour(direction);
            if all.contains(&neighbour) && reached.insert(neighbour) {
                pending.push_back(neighbour);
            }
        }
    }
    reached.len() == cells.len()
}

// Room indices identify authored programme slots; cells identify spatial samples.
fn cell_random(
    seed: u64,
    room_slot: u64,
    cell: Cell,
    stream: fabelgeist_determinism::StreamId,
) -> fabelgeist_determinism::DeterministicRng {
    stream.rng(
        seed.into(),
        &[room_slot, cell.x as u16 as u64, cell.z as u16 as u64],
    )
}

fn validate_vertical_connections(program: &BuildingProgram) -> Result<(), GenerationError> {
    if program.storeys.len() > 1 && program.vertical_connections.is_empty() {
        return Err(GenerationError::UnsatisfiedVerticalCirculation {
            connection: 0,
            reason: "a multi-storey programme declares no vertical connection".to_owned(),
        });
    }
    for (connection, requirement) in program.vertical_connections.iter().enumerate() {
        let (lowest_storey, highest_storey) = match *requirement {
            VerticalConnectionRequirement::StraightStair {
                lowest_storey,
                highest_storey,
                ..
            }
            | VerticalConnectionRequirement::TowerSpiral {
                lowest_storey,
                highest_storey,
            } => (lowest_storey, highest_storey),
        };
        if lowest_storey >= highest_storey || usize::from(highest_storey) >= program.storeys.len() {
            return Err(GenerationError::UnsatisfiedVerticalCirculation {
                connection,
                reason: format!(
                    "invalid served-storey range {lowest_storey}..={highest_storey} for {} storeys",
                    program.storeys.len()
                ),
            });
        }
    }
    for lower_storey in 0..program.storeys.len().saturating_sub(1) {
        let lower_storey = StoreyIndex::new(lower_storey).serialized_ordinal()?;
        let covered = program.vertical_connections.iter().any(|requirement| {
            let (lowest_storey, highest_storey) = match *requirement {
                VerticalConnectionRequirement::StraightStair {
                    lowest_storey,
                    highest_storey,
                    ..
                }
                | VerticalConnectionRequirement::TowerSpiral {
                    lowest_storey,
                    highest_storey,
                } => (lowest_storey, highest_storey),
            };
            lowest_storey <= lower_storey && highest_storey > lower_storey
        });
        if !covered {
            return Err(GenerationError::UnsatisfiedVerticalCirculation {
                connection: 0,
                reason: format!(
                    "no declared connector crosses storeys {lower_storey} and {}",
                    lower_storey + 1
                ),
            });
        }
    }
    Ok(())
}
