//! Outdoor routes use the same standing body and clipped obstacles as room access.
use super::{geometry::*, obstruction::Obstruction};
use crate::CollisionCuboid;
use bevy::math::Vec2;

/// Conservative continuous sweep in the input cuboids' coordinate system.
pub fn standing_path_clear(
    cuboids: &[CollisionCuboid],
    start: Vec2,
    end: Vec2,
    floor_elevation_metres: f32,
) -> bool {
    cuboids.iter().all(|solid| {
        let obstruction = Obstruction::new(
            solid.centre,
            solid.size,
            solid.yaw_radians,
            solid.crossfall_radians,
            solid.longfall_radians,
        );
        obstruction
            .projection(
                floor_elevation_metres + FLOOR_CLEARANCE,
                floor_elevation_metres + PERSON_HEIGHT,
            )
            .is_none_or(|rect| !segment_intersects(rect.expanded(PERSON_RADIUS), start, end))
    })
}

fn segment_intersects(rect: Rect, start: Vec2, end: Vec2) -> bool {
    let min = rect.centre - rect.half;
    let max = rect.centre + rect.half;
    let delta = end - start;
    let (mut enter, mut leave) = (0.0_f32, 1.0_f32);
    for axis in 0..2 {
        if delta[axis].abs() <= f32::EPSILON {
            if start[axis] < min[axis] || start[axis] > max[axis] {
                return false;
            }
        } else {
            let a = (min[axis] - start[axis]) / delta[axis];
            let b = (max[axis] - start[axis]) / delta[axis];
            enter = enter.max(a.min(b));
            leave = leave.min(a.max(b));
            if enter > leave {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec3;
    #[test]
    fn continuous_route_detects_thin_obstacle_between_sampling_stations() {
        let mut obstacle = CollisionCuboid {
            source: crate::ResolvedItemId(1),
            centre: Vec3::new(0.12, 0.9, 0.0),
            size: Vec3::new(0.01, 1.8, 2.0),
            yaw_radians: 0.0,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
        };
        assert!(!standing_path_clear(&[obstacle], -Vec2::X, Vec2::X, 0.0));
        obstacle.centre.y = 3.0;
        assert!(standing_path_clear(&[obstacle], -Vec2::X, Vec2::X, 0.0));
    }
}
