//! Shared room orientation makes seating and ward beds form coherent ensembles.
use super::{InteriorLayoutError, geometry::RoomBounds};
use crate::furniture::FurnitureKind;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::{BuildingPlan, Direction, Room, RoomKind, StoreyIndex};
use bevy::math::Vec2;

pub(super) fn preferred_facing(
    plan: &BuildingPlan,
    room: &Room,
    kind: FurnitureKind,
    bounds: RoomBounds,
) -> Result<Option<Direction>, InteriorLayoutError> {
    let min = bounds.min.metres();
    let max = bounds.max.metres();
    if kind == FurnitureKind::WardBed && room.kind == RoomKind::Ward {
        // Beds run across the ward's long axis, with heads towards the side walls.
        return Ok(Some(if max.y - min.y >= max.x - min.x {
            Direction::East
        } else {
            Direction::North
        }));
    }
    if matches!(kind, FurnitureKind::Altar | FurnitureKind::TorahShrine) {
        return room_direction(
            plan,
            RoomKind::Nave,
            ArchitecturalPlanPoint::from_metres((min + max) * 0.5)?,
        );
    }
    if !matches!(kind, FurnitureKind::ChurchBench | FurnitureKind::Pulpit) {
        return Ok(None);
    }
    let direction = room_direction(
        plan,
        RoomKind::Chancel,
        ArchitecturalPlanPoint::from_metres((min + max) * 0.5)?,
    )?
    .unwrap_or(Direction::North);
    Ok(Some(if kind == FurnitureKind::Pulpit {
        direction.opposite()
    } else {
        direction
    }))
}

fn room_direction(
    plan: &BuildingPlan,
    kind: RoomKind,
    origin: ArchitecturalPlanPoint,
) -> Result<Option<Direction>, InteriorLayoutError> {
    for storey in &plan.storeys {
        if let Some(room) = storey.rooms.iter().find(|r| r.kind == kind) {
            // The mean cell centre preserves the authored room-facing arithmetic.
            super::geometry::room_bounds(room, StoreyIndex::from_serialized(storey.level))?;
            let centre = ArchitecturalPlanPoint::from_metres(
                room.cells.iter().map(|cell| cell.centre()).sum::<Vec2>() / room.cells.len() as f32,
            )?;
            return Ok(Some(super::composition::direction_towards(
                centre.metres() - origin.metres(),
            )));
        }
    }
    Ok(None)
}

/// Liturgical fixtures retain their relationship to the audience and chancel.
pub(super) fn placement_score(
    plan: &BuildingPlan,
    placement: &super::InteriorPlacement,
    bounds: RoomBounds,
    ordinary: f32,
) -> Result<f32, InteriorLayoutError> {
    // Scores use distances and projected progress in metres; infinity rejects
    // a candidate.
    let min = bounds.min.metres();
    let max = bounds.max.metres();
    let centre = (min + max) * 0.5;
    Ok(match placement.key.kind() {
        FurnitureKind::Pulpit => {
            let forward = room_direction(
                plan,
                RoomKind::Chancel,
                ArchitecturalPlanPoint::from_metres(centre)?,
            )?
            .unwrap_or(Direction::North)
            .offset()
            .as_vec2();
            let progress = (placement.centre_metres.metres() - centre).dot(forward);
            if progress < 0.0 {
                return Ok(f32::INFINITY);
            }
            ordinary - progress
        }
        FurnitureKind::Altar | FurnitureKind::TorahShrine => {
            let back = -placement.facing.offset().as_vec2();
            let target = centre + back * (max - min) * 0.5;
            placement.centre_metres.metres().distance(target)
        }
        _ => ordinary,
    })
}

pub(super) fn facing_at(
    room: &Room,
    kind: FurnitureKind,
    facing: Direction,
    centre: ArchitecturalPlanPoint,
    room_centre: ArchitecturalPlanPoint,
) -> Direction {
    if kind == FurnitureKind::WardBed
        && room.kind == RoomKind::Ward
        && (centre.metres() - room_centre.metres()).dot(facing.offset().as_vec2()) > 0.0
    {
        facing.opposite()
    } else {
        facing
    }
}
