//! Reserve a continuous rear heating bay through the occupied town-house floors.
use crate::*;
use std::collections::BTreeMap;

pub(super) fn applies(program: &BuildingProgram, level: usize) -> bool {
    matches!(
        program.archetype,
        BuildingArchetype::TownHouse | BuildingArchetype::FachwerkMerchantHouse
    ) && program.domestic_heating.is_some()
        && level < program.storeys.len()
}

pub(super) fn reserve(
    program: &BuildingProgram,
    level: usize,
    footprint: &[Cell],
    reservations: &mut BTreeMap<Cell, usize>,
) -> Result<(), GenerationError> {
    let (_, depth) = program.footprint.dimensions();
    let anchors: &[(RoomKind, i16)] = match level {
        0 => &[
            (RoomKind::Storage, 1),
            (RoomKind::Storage, 2),
            (RoomKind::Storage, 3),
        ],
        1 => &[
            (RoomKind::Kitchen, 1),
            (RoomKind::Kitchen, 2),
            (RoomKind::CommonRoom, 3),
        ],
        _ => &[(RoomKind::Storage, 1), (RoomKind::Storage, 2)],
    };
    let mut reserve = |kind, x, rear_offset| {
        if let Some(index) = program.storeys[level]
            .rooms
            .iter()
            .position(|room| room.kind == kind)
        {
            let cell = Cell::new(x, depth as i16 - rear_offset);
            if reservations.contains_key(&cell) || !footprint.contains(&cell) {
                return Err(GenerationError::InvalidDomesticHeating);
            }
            reservations.insert(cell, index);
        }
        Ok(())
    };
    // Reserve the complete two-cell bay. Isolated anchor cells allow seeded
    // growth to turn the kitchen/Stube partition across the floor joists or
    // put a lower partition through the continuous masonry support.
    for &(kind, rear_offset) in anchors {
        for x in [0, 1] {
            reserve(kind, x, rear_offset)?;
        }
    }
    if level == 1 {
        // A full pantry boundary beside the rear kitchen leaves its doorway
        // behind the hearth, rather than at the end of a one-cell notch.
        for rear_offset in [1, 2] {
            reserve(RoomKind::Pantry, 2, rear_offset)?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum HeatingDoorSetOut {
    BesideBay,
    BehindHearth,
}

/// Heating-bay doors use the full room boundary, not a seeded one-cell notch.
pub(super) fn doorway(program: &StoreyProgram, walls: &[WallSegment], openings: &mut [Opening]) {
    for (kinds, set_out) in [
        (
            [RoomKind::Kitchen, RoomKind::CommonRoom],
            HeatingDoorSetOut::BesideBay,
        ),
        (
            [RoomKind::Storage, RoomKind::Gallery],
            HeatingDoorSetOut::BesideBay,
        ),
        (
            [RoomKind::Kitchen, RoomKind::Pantry],
            HeatingDoorSetOut::BehindHearth,
        ),
    ] {
        let pair = kinds.map(|kind| {
            program
                .rooms
                .iter()
                .position(|r| r.kind == kind)
                .map(|index| index as u16)
        });
        let [Some(first), Some(second)] = pair else {
            continue;
        };
        let shared = |wall: &WallSegment| {
            (wall.inside_room == first && wall.outside_room == Some(second))
                || (wall.inside_room == second && wall.outside_room == Some(first))
        };
        let candidates = walls.iter().enumerate().filter(|(_, wall)| shared(wall));
        let selected = match set_out {
            HeatingDoorSetOut::BesideBay => candidates
                .filter(|(_, wall)| wall.is_horizontal())
                .min_by(|(_, a), (_, b)| a.centre().x.total_cmp(&b.centre().x)),
            HeatingDoorSetOut::BehindHearth => candidates
                .filter(|(_, wall)| !wall.is_horizontal())
                .max_by(|(_, a), (_, b)| a.centre().y.total_cmp(&b.centre().y)),
        };
        let Some((index, _)) = selected else {
            continue;
        };
        for opening in openings
            .iter_mut()
            .filter(|o| o.kind == OpeningKind::Door && shared(&walls[o.wall]))
        {
            opening.wall = index;
        }
    }
}
