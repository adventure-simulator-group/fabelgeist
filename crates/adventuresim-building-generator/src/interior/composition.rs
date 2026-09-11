//! Atomic arrangements keep seating related to the surface it serves.
use super::InteriorPlacement;
use super::geometry::local_rotate;
use crate::Direction;
use crate::furniture::{FurnitureKey, FurnitureKind, InteriorFurnitureSpec};
use bevy::math::Vec2;

const TABLE_SEATING_GAP_METRES: f32 = 0.4;
const DESK_APPROACH_MARGIN_METRES: f32 = 0.05;

pub(super) fn compose(primary: InteriorPlacement) -> Vec<InteriorPlacement> {
    let mut group = vec![primary.clone()];
    let size = primary.key.interior_spec().unwrap().size_metres;
    match primary.key.kind {
        FurnitureKind::DiningTable => {
            let key = FurnitureKey {
                kind: FurnitureKind::Bench,
                ..primary.key
            };
            let bench = key.interior_spec().unwrap().size_metres;
            for sign in [-1.0, 1.0] {
                let offset = Vec2::new(
                    0.0,
                    sign * (size.z * 0.5 + bench.z * 0.5 + TABLE_SEATING_GAP_METRES),
                );
                group.push(InteriorPlacement {
                    key,
                    centre_metres: primary.centre_metres
                        + local_rotate(offset, primary.yaw_radians()),
                    facing: if sign < 0.0 {
                        primary.facing
                    } else {
                        primary.facing.opposite()
                    },
                    ..primary.clone()
                });
            }
        }
        FurnitureKind::WritingDesk => {
            let key = FurnitureKey {
                kind: FurnitureKind::Chair,
                ..primary.key
            };
            let chair = key.interior_spec().unwrap().size_metres;
            let offset = Vec2::new(
                0.0,
                -(size.z * 0.5
                    + InteriorFurnitureSpec::ACCESS_DEPTH_METRES
                    + chair.z * 0.5
                    + DESK_APPROACH_MARGIN_METRES),
            );
            group.push(InteriorPlacement {
                key,
                centre_metres: primary.centre_metres + local_rotate(offset, primary.yaw_radians()),
                facing: primary.facing.opposite(),
                ..primary.clone()
            });
        }
        FurnitureKind::Counter => {
            group.clear();
            for (index, kind) in [
                FurnitureKind::CounterLeftEnd,
                FurnitureKind::Counter,
                FurnitureKind::CounterRightEnd,
            ]
            .into_iter()
            .enumerate()
            {
                group.push(InteriorPlacement {
                    key: FurnitureKey {
                        kind,
                        ..primary.key
                    },
                    centre_metres: primary.centre_metres
                        + local_rotate(
                            Vec2::new((index as f32 - 1.0) * size.x, 0.0),
                            primary.yaw_radians(),
                        ),
                    ..primary.clone()
                });
            }
        }
        _ => {}
    }
    group
}

pub(super) fn direction_towards(vector: Vec2) -> Direction {
    if vector.x.abs() > vector.y.abs() {
        if vector.x > 0.0 {
            Direction::East
        } else {
            Direction::West
        }
    } else if vector.y > 0.0 {
        Direction::North
    } else {
        Direction::South
    }
}
