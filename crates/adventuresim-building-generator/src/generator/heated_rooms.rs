//! Plan a narrow town house's upper kitchen behind its heated living room.
use crate::*;
use std::collections::BTreeMap;

pub(super) fn applies(program: &BuildingProgram, level: usize) -> bool {
    program.archetype == BuildingArchetype::TownHouse
        && program.domestic_heating.is_some()
        && level <= 1
}

pub(super) fn reserve(
    program: &BuildingProgram,
    level: usize,
    footprint: &[Cell],
    reservations: &mut BTreeMap<Cell, usize>,
) -> Result<(), GenerationError> {
    let (_, depth) = program.footprint.dimensions();
    let anchors = if level == 0 {
        // A continuous storage bay below the appliance leaves the masonry pier
        // clear of lower partitions and the central stair hall.
        [(RoomKind::Storage, 3), (RoomKind::Storage, 2)]
    } else {
        [(RoomKind::Kitchen, 1), (RoomKind::CommonRoom, 3)]
    };
    for (kind, rear_offset) in anchors {
        if let Some(index) = program.storeys[level]
            .rooms
            .iter()
            .position(|room| room.kind == kind)
        {
            let cell = Cell::new(1, depth as i16 - rear_offset);
            if reservations.contains_key(&cell) || !footprint.contains(&cell) {
                return Err(GenerationError::InvalidDomesticHeating);
            }
            reservations.insert(cell, index);
        }
    }
    Ok(())
}

/// Keep the inter-room door beside the masonry bay, not in its middle.
pub(super) fn doorway(program: &StoreyProgram, walls: &[WallSegment], openings: &mut [Opening]) {
    let pair = [RoomKind::Kitchen, RoomKind::CommonRoom].map(|kind| {
        program
            .rooms
            .iter()
            .position(|r| r.kind == kind)
            .map(|index| index as u16)
    });
    let [Some(kitchen), Some(stube)] = pair else {
        return;
    };
    let shared = |wall: &WallSegment| {
        (wall.inside_room == kitchen && wall.outside_room == Some(stube))
            || (wall.inside_room == stube && wall.outside_room == Some(kitchen))
    };
    let Some((index, _)) = walls
        .iter()
        .enumerate()
        .filter(|(_, w)| shared(w) && w.is_horizontal())
        .min_by(|(_, a), (_, b)| a.centre().x.total_cmp(&b.centre().x))
    else {
        return;
    };
    for opening in openings
        .iter_mut()
        .filter(|o| o.kind == OpeningKind::Door && shared(&walls[o.wall]))
    {
        opening.wall = index;
    }
}
