//! Seeded growth assigns cells to admitted authored room ordinals.
use super::*;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllocationFailure {
    MissingCellAssignment { cell: Cell },
    UnknownRoomReservation { room: RoomIndex, rooms: usize },
    NoUnclaimedSeed { room: RoomIndex },
    MissingSeed { room: RoomIndex },
    NoExpansionEdge,
}
pub(super) fn allocate(
    footprint: &[Cell],
    width: u16,
    depth: u16,
    requirements: &[RoomRequirement],
    seed: u64,
    archetype: BuildingArchetype,
    reservations: &BTreeMap<Cell, RoomIndex>,
) -> Result<BTreeMap<Cell, RoomIndex>, GenerationError> {
    let usable = footprint.iter().copied().collect::<BTreeSet<_>>();
    let mut assignments = reservations.clone();
    let mut room_seeds = vec![None; requirements.len()];
    for (cell, room_index) in reservations {
        room_seeds
            .get_mut(room_index.index())
            .ok_or(GenerationError::RoomAllocation(
                AllocationFailure::UnknownRoomReservation {
                    room: *room_index,
                    rooms: requirements.len(),
                },
            ))?
            .get_or_insert(*cell);
    }

    if let Some(passage_index) = requirements
        .iter()
        .position(|room| room.kind == RoomKind::Passage)
    {
        let passage_width = match archetype {
            BuildingArchetype::CastleGatehouse => 2,
            BuildingArchetype::CourtyardCastle => 4,
            _ => 1,
        };
        let start_x = i16::try_from(width / 2).map_err(|_| GenerationError::InvalidFootprint)?
            - passage_width / 2;
        let passage_depth = match archetype {
            BuildingArchetype::CourtyardCastle => match requirements.len() {
                0 => 0,
                _ => 4,
            },
            _ => i16::try_from(depth).map_err(|_| GenerationError::InvalidFootprint)?,
        };
        for z in 0..passage_depth {
            for x in start_x..start_x + passage_width {
                let cell = Cell::new(x, z);
                if usable.contains(&cell) {
                    assignments.insert(cell, RoomIndex::from_ordinal(passage_index)?);
                    room_seeds[passage_index].get_or_insert(cell);
                }
            }
        }
    }

    let mut claimed_seeds = assignments.keys().copied().collect::<HashSet<_>>();
    for (room_index, requirement) in requirements.iter().enumerate() {
        if room_seeds[room_index].is_some() {
            continue;
        }
        let selected = footprint
            .iter()
            .copied()
            .filter(|cell| !claimed_seeds.contains(cell))
            .min_by_key(|cell| seed_score(*cell, requirement, width, depth, room_index, seed))
            .ok_or(GenerationError::RoomAllocation(
                AllocationFailure::NoUnclaimedSeed {
                    room: RoomIndex::from_ordinal(room_index)?,
                },
            ))?;
        assignments.insert(selected, RoomIndex::from_ordinal(room_index)?);
        claimed_seeds.insert(selected);
        room_seeds[room_index] = Some(selected);
    }

    grow_rooms(&mut assignments, footprint, requirements, &room_seeds, seed)?;

    Ok(assignments)
}

fn seed_score(
    cell: Cell,
    requirement: &RoomRequirement,
    width: u16,
    depth: u16,
    room_index: usize,
    seed: u64,
) -> u64 {
    let x = i32::from(cell.x);
    let z = i32::from(cell.z);
    let max_x = i32::from(width) - 1;
    let max_z = i32::from(depth) - 1;
    let centre_x = max_x / 2;
    let centre_z = max_z / 2;
    let exterior_distance = x.min(max_x - x).min(z).min(max_z - z).max(0) as u64;
    let centre_distance =
        (x - centre_x).unsigned_abs() as u64 + (z - centre_z).unsigned_abs() as u64;
    let south_centre = z.unsigned_abs() as u64 * 8 + (x - centre_x).unsigned_abs() as u64;
    let north_centre = (max_z - z).unsigned_abs() as u64 * 8 + (x - centre_x).unsigned_abs() as u64;
    let west_centre = x.unsigned_abs() as u64 * 8 + (z - centre_z).unsigned_abs() as u64;
    let east_centre = (max_x - x).unsigned_abs() as u64 * 8 + (z - centre_z).unsigned_abs() as u64;
    let functional = match requirement.kind {
        RoomKind::EntranceHall | RoomKind::Shop | RoomKind::Passage => south_centre,
        RoomKind::StairHall => centre_distance,
        RoomKind::Kitchen | RoomKind::Pantry => north_centre,
        RoomKind::Workshop
        | RoomKind::Armoury
        | RoomKind::MillingFloor
        | RoomKind::KilnRoom
        | RoomKind::VatRoom => west_centre,
        RoomKind::Guardroom | RoomKind::CountingRoom => east_centre,
        RoomKind::GreatHall
        | RoomKind::CommonRoom
        | RoomKind::Gallery
        | RoomKind::Chapel
        | RoomKind::Nave
        | RoomKind::Ward
        | RoomKind::Schoolroom
        | RoomKind::Stalls
        | RoomKind::Chancel => north_centre + centre_distance,
        RoomKind::Storage | RoomKind::Sacristy => west_centre + north_centre,
        RoomKind::Bedchamber | RoomKind::TowerChamber => east_centre + north_centre,
    };
    functional * 1_000
        + if requirement.needs_exterior {
            exterior_distance * 4_000
        } else {
            0
        }
        + cell_random(
            seed,
            room_index as u64,
            cell,
            fabelgeist_determinism::StreamId::new("building.room-origin"),
        )
        .index(499) as u64
}

fn room_counts(room_count: usize, assignments: &BTreeMap<Cell, RoomIndex>) -> Vec<usize> {
    let mut counts = vec![0; room_count];
    for room in assignments.values().copied() {
        counts[room.index()] += 1;
    }
    counts
}

pub(super) fn collect_rooms(
    assignments: &BTreeMap<Cell, RoomIndex>,
    requirements: &[RoomRequirement],
) -> Result<Vec<Room>, GenerationError> {
    requirements
        .iter()
        .enumerate()
        .map(|(room_index, requirement)| {
            let room_index = RoomIndex::from_ordinal(room_index)?;
            Ok(Room {
                id: room_index.serialized_ordinal(),
                kind: requirement.kind,
                cells: assignments
                    .iter()
                    .filter_map(|(cell, assigned)| (*assigned == room_index).then_some(*cell))
                    .collect(),
            })
        })
        .collect()
}

/// Lexicographic seeded growth ordering; the first three fields are unitless ranks.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct GrowthCandidate {
    fill_ratio: u64,
    geometry_score: u64,
    random_rank: u64,
    cell: Cell,
    room: RoomIndex,
}

fn grow_rooms(
    assignments: &mut BTreeMap<Cell, RoomIndex>,
    footprint: &[Cell],
    requirements: &[RoomRequirement],
    room_seeds: &[Option<Cell>],
    seed: u64,
) -> Result<(), GenerationError> {
    while assignments.len() < footprint.len() {
        let room_counts = room_counts(requirements.len(), assignments);
        let mut best: Option<GrowthCandidate> = None;
        for cell in footprint.iter().copied() {
            if assignments.contains_key(&cell) {
                continue;
            }
            let neighbouring_rooms = Direction::ALL
                .into_iter()
                .filter_map(|direction| assignments.get(&cell.neighbour(direction)).copied())
                .collect::<BTreeSet<_>>();
            for room_index in neighbouring_rooms {
                if requirements[room_index.index()].kind == RoomKind::Passage {
                    continue;
                }
                let preferred = u64::from(requirements[room_index.index()].preferred_cells.max(1));
                let fill_ratio = room_counts[room_index.index()] as u64 * 10_000 / preferred;
                let seed_cell =
                    room_seeds[room_index.index()].ok_or(GenerationError::RoomAllocation(
                        AllocationFailure::MissingSeed { room: room_index },
                    ))?;
                let distance =
                    cell.x.abs_diff(seed_cell.x) as u64 + cell.z.abs_diff(seed_cell.z) as u64;
                let same_room_neighbours = Direction::ALL
                    .into_iter()
                    .filter(|direction| {
                        assignments.get(&cell.neighbour(*direction)) == Some(&room_index)
                    })
                    .count() as u64;
                let geometry_score = distance * 8 + (4 - same_room_neighbours) * 12;
                let candidate = GrowthCandidate {
                    fill_ratio,
                    geometry_score,
                    random_rank: cell_random(
                        seed,
                        room_index.index() as u64,
                        cell,
                        fabelgeist_determinism::StreamId::new("building.room-growth"),
                    )
                    .index(97) as u64,
                    cell,
                    room: room_index,
                };
                if best.is_none_or(|current| candidate < current) {
                    best = Some(candidate);
                }
            }
        }
        let candidate = best.ok_or(GenerationError::RoomAllocation(
            AllocationFailure::NoExpansionEdge,
        ))?;
        assignments.insert(candidate.cell, candidate.room);
    }

    Ok(())
}
