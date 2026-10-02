use super::*;

pub(super) fn append(
    walls: &[crate::WallSegment],
    requirements: &[RoomRequirement],
    archetype: BuildingArchetype,
    openings: &mut Vec<Opening>,
    occupied_walls: &mut HashSet<usize>,
) {
    let entrance_room = requirements
        .iter()
        .position(|room| matches!(room.kind, RoomKind::EntranceHall | RoomKind::Passage))
        .unwrap_or(0) as u16;
    let mut entrance_candidates = walls
        .iter()
        .enumerate()
        .filter(|(_, wall)| {
            wall.exterior()
                && wall.inside_room == entrance_room
                && wall.direction == Direction::South
        })
        .collect::<Vec<_>>();
    entrance_candidates.sort_by_key(|(_, wall)| wall.cell.x);
    let gate = matches!(
        archetype,
        BuildingArchetype::HallHouse
            | BuildingArchetype::CastleGatehouse
            | BuildingArchetype::CourtyardCastle
            | BuildingArchetype::WalledKeep
            | BuildingArchetype::ArtilleryRondelCastle
    );
    let selected_entrances = if gate {
        let middle = entrance_candidates.len() / 2;
        let start = middle.saturating_sub(1);
        &entrance_candidates[start..entrance_candidates.len().min(start + 2)]
    } else {
        let middle = entrance_candidates.len() / 2;
        &entrance_candidates[middle..entrance_candidates.len().min(middle + 1)]
    };
    for (wall_index, _) in selected_entrances {
        openings.push(Opening {
            wall: *wall_index,
            kind: if gate {
                OpeningKind::Gate
            } else {
                OpeningKind::Door
            },
            width_metres: if gate {
                1.35
            } else if archetype == BuildingArchetype::StorageRange {
                1.2
            } else {
                1.0
            },
            sill_metres: 0.0,
            height_metres: if gate { 2.8 } else { 2.15 },
        });
        occupied_walls.insert(*wall_index);
    }
    if requirements[usize::from(entrance_room)].kind == RoomKind::Passage {
        let mut exit_candidates = walls
            .iter()
            .enumerate()
            .filter(|(_, wall)| {
                wall.exterior()
                    && wall.inside_room == entrance_room
                    && wall.direction == Direction::North
            })
            .collect::<Vec<_>>();
        exit_candidates.sort_by_key(|(_, wall)| wall.cell.x);
        let middle = exit_candidates.len() / 2;
        let start = middle.saturating_sub(1);
        for (wall_index, _) in &exit_candidates[start..exit_candidates.len().min(start + 2)] {
            openings.push(Opening {
                wall: *wall_index,
                kind: OpeningKind::Gate,
                width_metres: 1.35,
                sill_metres: 0.0,
                height_metres: 2.8,
            });
            occupied_walls.insert(*wall_index);
        }
    }
    if archetype == BuildingArchetype::FachwerkMerchantHouse {
        append_court_exit(walls, requirements, openings, occupied_walls);
    }
}

/// A merchant's working ground floor opens onto its court independently of the
/// street entrance. Select the actual rear room; do not relabel the entrance hall.
fn append_court_exit(
    walls: &[crate::WallSegment],
    requirements: &[RoomRequirement],
    openings: &mut Vec<Opening>,
    occupied_walls: &mut HashSet<usize>,
) {
    let mut candidates = walls
        .iter()
        .enumerate()
        .filter(|(_, wall)| {
            wall.exterior()
                && wall.direction == Direction::North
                && requirements[usize::from(wall.inside_room)].kind != RoomKind::StairHall
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(_, wall)| wall.cell.x);
    if let Some((wall, _)) = candidates.get(candidates.len() / 2) {
        openings.push(Opening {
            wall: *wall,
            kind: OpeningKind::Door,
            width_metres: 1.0,
            sill_metres: 0.0,
            height_metres: 2.15,
        });
        occupied_walls.insert(*wall);
    }
}
