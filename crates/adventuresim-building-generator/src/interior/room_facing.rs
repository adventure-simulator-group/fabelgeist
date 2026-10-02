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
    if matches!(kind, FurnitureKind::Altar | FurnitureKind::TorahShrine) {
        return room_direction(plan, RoomKind::Nave, (min + max) * 0.5);
    }
    (matches!(kind, FurnitureKind::ChurchBench | FurnitureKind::Pulpit)).then(|| {
        let chancel = plan
            .storeys
            .iter()
            .flat_map(|s| &s.rooms)
            .find(|r| r.kind == RoomKind::Chancel);
        let direction = chancel.map_or(Direction::North, |r| {
            super::composition::direction_towards(
                r.cells.iter().map(|c| c.centre()).sum::<Vec2>() / r.cells.len() as f32
                    - (min + max) * 0.5,
            )
        });
        if kind == FurnitureKind::Pulpit {
            direction.opposite()
        } else {
            direction
        }
    })
}

fn room_direction(plan: &BuildingPlan, kind: RoomKind, origin: Vec2) -> Option<Direction> {
    let room = plan
        .storeys
        .iter()
        .flat_map(|s| &s.rooms)
        .find(|r| r.kind == kind)?;
    let centre =
        room.cells.iter().map(|cell| cell.centre()).sum::<Vec2>() / room.cells.len() as f32;
    Some(super::composition::direction_towards(centre - origin))
}

/// Liturgical fixtures retain their relationship to the audience and chancel.
pub(super) fn placement_score(
    plan: &BuildingPlan,
    placement: &super::InteriorPlacement,
    min: Vec2,
    max: Vec2,
    ordinary: f32,
) -> f32 {
    let centre = (min + max) * 0.5;
    match placement.key.kind() {
        FurnitureKind::Pulpit => {
            let forward = room_direction(plan, RoomKind::Chancel, centre)
                .unwrap_or(Direction::North)
                .offset()
                .as_vec2();
            let progress = (placement.centre_metres - centre).dot(forward);
            if progress < 0.0 {
                return f32::INFINITY;
            }
            ordinary - progress
        }
        FurnitureKind::Altar | FurnitureKind::TorahShrine => {
            let back = -placement.facing.offset().as_vec2();
            let target = centre + back * (max - min) * 0.5;
            placement.centre_metres.distance(target)
        }
        _ => ordinary,
    }
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
