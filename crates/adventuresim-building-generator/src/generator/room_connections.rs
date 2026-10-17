//! Room identity and priority at the wall-source ordinal boundary.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct RoomConnection {
    pub left: RoomIndex,
    pub right: RoomIndex,
}

pub(super) struct SharedBoundary {
    pub connection: RoomConnection,
    pub candidates: Vec<usize>,
    circulation_required: bool,
    preferred: bool,
}

pub(super) fn requirement(
    requirements: &[RoomRequirement],
    room: RoomIndex,
    storey: StoreyIndex,
) -> Result<&RoomRequirement, GenerationError> {
    requirements
        .get(room.index())
        .ok_or(GenerationError::MissingRoomRequirement { storey, room })
}

pub(super) fn shared_boundaries(
    walls: &[crate::WallSegment],
    requirements: &[RoomRequirement],
    storey: StoreyIndex,
) -> Result<Vec<SharedBoundary>, GenerationError> {
    let mut shared = BTreeMap::<RoomConnection, Vec<usize>>::new();
    for (wall_index, wall) in walls.iter().enumerate() {
        if let Some(other) = wall.outside_room {
            let inside = RoomIndex::from_serialized(wall.inside_room);
            let outside = RoomIndex::from_serialized(other);
            let connection = if inside < outside {
                RoomConnection {
                    left: inside,
                    right: outside,
                }
            } else {
                RoomConnection {
                    left: outside,
                    right: inside,
                }
            };
            shared.entry(connection).or_default().push(wall_index);
        }
    }
    let mut edges = shared
        .into_iter()
        .map(|(connection, candidates)| {
            let left = requirement(requirements, connection.left, storey)?;
            let right = requirement(requirements, connection.right, storey)?;
            Ok(SharedBoundary {
                connection,
                candidates,
                preferred: left.preferred_neighbours.contains(&right.kind)
                    || right.preferred_neighbours.contains(&left.kind),
                circulation_required: left.kind == RoomKind::StairHall
                    || right.kind == RoomKind::StairHall,
            })
        })
        .collect::<Result<Vec<_>, GenerationError>>()?;
    edges.sort_by_key(|edge| (!edge.circulation_required, !edge.preferred, edge.connection));
    Ok(edges)
}
