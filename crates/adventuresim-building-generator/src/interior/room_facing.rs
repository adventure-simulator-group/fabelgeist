//! Shared room orientation makes seating and ward beds form coherent ensembles.
use crate::furniture::FurnitureKind;
use crate::{BuildingPlan, Direction, Room, RoomKind};
use bevy::math::Vec2;

pub(super) fn preferred_facing(
    plan: &BuildingPlan,
    room: &Room,
    kind: FurnitureKind,
    min: Vec2,
    max: Vec2,
) -> Option<Direction> {
    if kind == FurnitureKind::WardBed && room.kind == RoomKind::Ward {
        // Beds run across the ward's long axis, with heads towards the side walls.
        return Some(if max.y - min.y >= max.x - min.x {
            Direction::East
        } else {
            Direction::North
        });
    }
    (kind == FurnitureKind::ChurchBench).then(|| {
        let chancel = plan
            .storeys
            .iter()
            .flat_map(|s| &s.rooms)
            .find(|r| r.kind == RoomKind::Chancel);
        chancel.map_or(Direction::North, |r| {
            super::composition::direction_towards(
                r.cells.iter().map(|c| c.centre()).sum::<Vec2>() / r.cells.len() as f32
                    - (min + max) * 0.5,
            )
        })
    })
}

pub(super) fn facing_at(
    room: &Room,
    kind: FurnitureKind,
    facing: Direction,
    centre: Vec2,
    room_centre: Vec2,
) -> Direction {
    if kind == FurnitureKind::WardBed
        && room.kind == RoomKind::Ward
        && (centre - room_centre).dot(facing.offset().as_vec2()) > 0.0
    {
        facing.opposite()
    } else {
        facing
    }
}
