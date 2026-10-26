//! Reserve a continuous rear heating bay through the occupied town-house floors.
use crate::*;
use std::collections::BTreeMap;

/// Architectural role selected from the occupied storey ordinal. Upper
/// storeys remain arbitrary ordinals; they share one heating reservation role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HeatingStoreyRole {
    MasonrySupport,
    HeatedRooms,
    UpperStorage,
}

pub(super) struct HeatingStorey<'a> {
    programme: &'a StoreyProgram,
    role: HeatingStoreyRole,
    index: StoreyIndex,
}
impl<'a> HeatingStorey<'a> {
    pub fn for_program(program: &'a BuildingProgram, index: StoreyIndex) -> Option<Self> {
        if !matches!(
            program.archetype,
            BuildingArchetype::TownHouse | BuildingArchetype::FachwerkMerchantHouse
        ) || program.domestic_heating.is_none()
        {
            return None;
        }
        let programme = program.storeys.get(index.index())?;
        let role = if index == StoreyIndex::GROUND {
            HeatingStoreyRole::MasonrySupport
        } else if index == StoreyIndex::FIRST_UPPER {
            HeatingStoreyRole::HeatedRooms
        } else {
            HeatingStoreyRole::UpperStorage
        };
        Some(Self {
            programme,
            role,
            index,
        })
    }
}

/// Positive whole-cell distance from the rear footprint edge. This uses
/// Cell's unit, not the finer GridLength construction unit.
#[derive(Clone, Copy)]
struct RearCellOffset(i16);
impl RearCellOffset {
    const fn from_cells(cells: i16) -> Option<Self> {
        if cells > 0 { Some(Self(cells)) } else { None }
    }
    const fn cells(self) -> i16 {
        self.0
    }
}
struct HeatingBayAnchor {
    room: RoomKind,
    rear_offset: RearCellOffset,
}
impl HeatingBayAnchor {
    fn new(room: RoomKind, rear_cells: i16) -> Result<Self, GenerationError> {
        Ok(Self {
            room,
            rear_offset: RearCellOffset::from_cells(rear_cells)
                .ok_or(GenerationError::HeatingAnchor { cells: rear_cells })?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservationFailure {
    AlreadyReserved,
    OutsideFootprint,
}

pub(super) fn reserve(
    program: &BuildingProgram,
    storey: &HeatingStorey<'_>,
    footprint: &[Cell],
    reservations: &mut BTreeMap<Cell, RoomIndex>,
) -> Result<(), GenerationError> {
    let (_, depth) = program.footprint.dimensions();
    let anchors: &[HeatingBayAnchor] = match storey.role {
        HeatingStoreyRole::MasonrySupport => &[
            HeatingBayAnchor::new(RoomKind::Storage, 1)?,
            HeatingBayAnchor::new(RoomKind::Storage, 2)?,
            HeatingBayAnchor::new(RoomKind::Storage, 3)?,
        ],
        HeatingStoreyRole::HeatedRooms => &[
            HeatingBayAnchor::new(RoomKind::Kitchen, 1)?,
            HeatingBayAnchor::new(RoomKind::Kitchen, 2)?,
            HeatingBayAnchor::new(RoomKind::CommonRoom, 3)?,
        ],
        HeatingStoreyRole::UpperStorage => &[
            HeatingBayAnchor::new(RoomKind::Storage, 1)?,
            HeatingBayAnchor::new(RoomKind::Storage, 2)?,
        ],
    };
    let mut reserve = |kind, x, rear_offset: RearCellOffset| {
        if let Some(index) = storey
            .programme
            .rooms
            .iter()
            .position(|room| room.kind == kind)
        {
            let cell = Cell::new(
                x,
                i16::try_from(depth).map_err(|_| GenerationError::InvalidFootprint)?
                    - rear_offset.cells(),
            );
            let room = RoomIndex::from_ordinal(index)?;
            let reason = if reservations.contains_key(&cell) {
                Some(ReservationFailure::AlreadyReserved)
            } else if !footprint.contains(&cell) {
                Some(ReservationFailure::OutsideFootprint)
            } else {
                None
            };
            if let Some(reason) = reason {
                return Err(GenerationError::HeatingReservation {
                    storey: storey.index,
                    room,
                    cell,
                    reason,
                });
            }
            reservations.insert(cell, room);
        }
        Ok(())
    };
    // Reserve the complete two-cell bay. Isolated anchor cells allow seeded
    // growth to turn the kitchen/Stube partition across the floor joists or
    // put a lower partition through the continuous masonry support.
    for anchor in anchors {
        for x in [0, 1] {
            reserve(anchor.room, x, anchor.rear_offset)?;
        }
    }
    if storey.role == HeatingStoreyRole::HeatedRooms {
        // A full pantry boundary beside the rear kitchen leaves its doorway
        // behind the hearth, rather than at the end of a one-cell notch.
        for rear_offset in [1, 2] {
            reserve(
                RoomKind::Pantry,
                2,
                RearCellOffset::from_cells(rear_offset)
                    .ok_or(GenerationError::HeatingAnchor { cells: rear_offset })?,
            )?;
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum HeatingDoorSetOut {
    BesideBay,
    BehindHearth,
}

struct HeatingDoorRecipe {
    first_room: RoomKind,
    second_room: RoomKind,
    set_out: HeatingDoorSetOut,
}

/// Heating-bay doors use the full room boundary, not a seeded one-cell notch.
pub(super) fn doorway(
    program: &StoreyProgram,
    walls: &[WallSegment],
    openings: &mut [Opening],
) -> Result<(), GenerationError> {
    for recipe in [
        HeatingDoorRecipe {
            first_room: RoomKind::Kitchen,
            second_room: RoomKind::CommonRoom,
            set_out: HeatingDoorSetOut::BesideBay,
        },
        HeatingDoorRecipe {
            first_room: RoomKind::Storage,
            second_room: RoomKind::Gallery,
            set_out: HeatingDoorSetOut::BesideBay,
        },
        HeatingDoorRecipe {
            first_room: RoomKind::Kitchen,
            second_room: RoomKind::Pantry,
            set_out: HeatingDoorSetOut::BehindHearth,
        },
    ] {
        let mut pair = [None; 2];
        for (slot, kind) in [recipe.first_room, recipe.second_room]
            .into_iter()
            .enumerate()
        {
            pair[slot] = program
                .rooms
                .iter()
                .position(|room| room.kind == kind)
                .map(RoomIndex::from_ordinal)
                .transpose()?
                .map(RoomIndex::serialized_ordinal);
        }
        let [Some(first), Some(second)] = pair else {
            continue;
        };
        let shared = |wall: &WallSegment| {
            (wall.inside_room == first && wall.outside_room == Some(second))
                || (wall.inside_room == second && wall.outside_room == Some(first))
        };
        let candidates = walls.iter().enumerate().filter(|(_, wall)| shared(wall));
        let selected = match recipe.set_out {
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn occupied_storey_indices_classify_roles_and_reject_absent_storeys() {
        let mut program = BuildingProgram::fixture(
            BuildingArchetype::TownHouse,
            fabelgeist_determinism::Seed::from_u64(11),
        );
        program.domestic_heating = Some(DomesticHeatingProgramme::HearthAndRearFedStove);
        let upper = program.storeys[1].clone();
        program.storeys.extend([upper.clone(), upper]);
        assert_eq!(
            HeatingStorey::for_program(&program, StoreyIndex::GROUND)
                .unwrap()
                .role,
            HeatingStoreyRole::MasonrySupport
        );
        assert_eq!(
            HeatingStorey::for_program(&program, StoreyIndex::FIRST_UPPER)
                .unwrap()
                .role,
            HeatingStoreyRole::HeatedRooms
        );
        for ordinal in 2..program.storeys.len() {
            assert_eq!(
                HeatingStorey::for_program(&program, StoreyIndex::new(ordinal))
                    .unwrap()
                    .role,
                HeatingStoreyRole::UpperStorage
            );
        }
        assert!(
            HeatingStorey::for_program(&program, StoreyIndex::new(program.storeys.len())).is_none()
        );
        program.domestic_heating = None;
        assert!(HeatingStorey::for_program(&program, StoreyIndex::GROUND).is_none());
    }
    #[test]
    fn rear_offsets_reject_nonpositive_cell_distances() {
        assert!(RearCellOffset::from_cells(0).is_none());
        assert!(RearCellOffset::from_cells(-1).is_none());
    }
}
