//! Intersections of offset perimeter lines determine wall and frame end points.
use bevy::math::Vec2;

use crate::{CELL_SIZE_METRES, StoreyPlan, WallAssembly, WallSegment, WallSourceId};

use super::direction_vector;

const ENDPOINT_TOLERANCE_METRES: f32 = 0.001;

pub(super) struct WallSpan {
    pub start: Vec2,
    pub end: Vec2,
}

impl WallSpan {
    pub fn for_source(wall: WallSegment, storey: &StoreyPlan, offset: f32) -> Self {
        let tangent = if wall.is_horizontal() {
            Vec2::X
        } else {
            Vec2::Y
        };
        let outward = direction_vector(wall.direction);
        let endpoint = |sign: f32| {
            let source = wall.centre() + tangent * sign * CELL_SIZE_METRES * 0.5;
            let return_offset = if wall.exterior() {
                storey
                    .walls
                    .iter()
                    .find(|other| {
                        other.exterior()
                            && other.is_horizontal() != wall.is_horizontal()
                            && [-1.0, 1.0].into_iter().any(|side| {
                                let axis = if other.is_horizontal() {
                                    Vec2::X
                                } else {
                                    Vec2::Y
                                };
                                (other.centre() + axis * side * CELL_SIZE_METRES * 0.5)
                                    .distance(source)
                                    < ENDPOINT_TOLERANCE_METRES
                            })
                    })
                    .map_or(Vec2::ZERO, |other| {
                        direction_vector(other.direction) * offset
                    })
            } else {
                Vec2::ZERO
            };
            source + outward * offset + return_offset
        };
        Self {
            start: endpoint(-1.0),
            end: endpoint(1.0),
        }
    }

    pub fn centre(&self) -> Vec2 {
        (self.start + self.end) * 0.5
    }

    pub fn length(&self) -> f32 {
        self.start.distance(self.end)
    }

    /// Timber lies slightly outside the wall centre plane. Intersect its own
    /// planes at each return so both facades reference the same physical post.
    pub fn for_frame(wall: &WallAssembly, walls: &[WallAssembly], inset: f32) -> Self {
        let endpoint = |sign: f32| {
            let source = wall.frame.origin + wall.frame.tangent * sign * wall.length_metres * 0.5;
            let return_offset = walls
                .iter()
                .find(|other| {
                    other.storey_level == wall.storey_level
                        && other.frame.outside_room.is_none()
                        && matches!(other.source, WallSourceId::StoreyWall { .. })
                        && other.frame.tangent.dot(wall.frame.tangent).abs()
                            < ENDPOINT_TOLERANCE_METRES
                        && [-1.0, 1.0].into_iter().any(|side| {
                            (other.frame.origin
                                + other.frame.tangent * side * other.length_metres * 0.5)
                                .distance(source)
                                < ENDPOINT_TOLERANCE_METRES
                        })
                })
                .map_or(Vec2::ZERO, |other| other.frame.outward * inset);
            source + wall.frame.outward * inset + return_offset
        };
        Self {
            start: endpoint(-1.0),
            end: endpoint(1.0),
        }
    }
}
