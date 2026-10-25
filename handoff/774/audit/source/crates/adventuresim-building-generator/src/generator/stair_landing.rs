//! Connect an occupied room graph to its physical straight-stair landing.
use super::*;

pub(super) struct StairLandingOpening {
    pub opening: Opening,
    pub stair_hall: RoomIndex,
    pub neighbour: RoomIndex,
}
impl StraightStairCore {
    pub(super) fn landing_opening(
        &self,
        walls: &[crate::WallSegment],
        requirements: &[RoomRequirement],
        storey: StoreyIndex,
    ) -> Result<StairLandingOpening, GenerationError> {
        let serialized_level = storey.serialized_ordinal()?;

        let stair_hall = RoomIndex::from_ordinal(
            requirements
                .iter()
                .position(|room| room.kind == self.landing_room)
                .ok_or(GenerationError::MissingLandingRoom {
                    storey,
                    role: self.landing_room,
                })?,
        )?
        .serialized_ordinal();
        let flight_index = serialized_level - self.lowest_storey;
        let axis = direction_vector(self.direction);
        let landing = if flight_index.is_multiple_of(2) {
            self.origin
        } else {
            self.origin + axis * STRAIGHT_STAIR_RUN_METRES
        };
        let reserved_cells = self.reserved_cells.iter().copied().collect::<HashSet<_>>();
        let Some((wall_index, wall)) = walls
            .iter()
            .enumerate()
            .filter(|(_, wall)| {
                reserved_cells.contains(&wall.cell)
                    && wall.outside_room.is_some()
                    && (wall.inside_room == stair_hall || wall.outside_room == Some(stair_hall))
            })
            .min_by(|(_, left), (_, right)| {
                left.centre()
                    .distance_squared(landing)
                    .total_cmp(&right.centre().distance_squared(landing))
            })
        else {
            return Err(GenerationError::UnsatisfiedVerticalCirculation {
                connection: 0,
                reason: format!(
                    "storey {storey} has no room-graph doorway adjacent to its stair landing"
                ),
            });
        };
        let other_room = wall
            .outside_room
            .filter(|room| *room != stair_hall)
            .unwrap_or(wall.inside_room);
        let opening = Opening {
            wall: wall_index,
            kind: OpeningKind::Door,
            width_metres: 0.95,
            sill_metres: 0.0,
            height_metres: 2.1,
        };
        Ok(StairLandingOpening {
            opening,
            stair_hall: RoomIndex::from_serialized(stair_hall),
            neighbour: RoomIndex::from_serialized(other_room),
        })
    }
}
