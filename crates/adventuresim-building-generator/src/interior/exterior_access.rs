//! Outdoor routes use the same standing body and clipped obstacles as room access.
use super::{geometry::*, obstruction::Obstruction};
use crate::CollisionCuboid;
use bevy::math::Vec2;

/// Height-clipped obstacles for repeated conservative standing-body sweeps.
/// Coordinates and elevation belong to the input cuboids' coordinate system.
pub struct StandingClearance(Vec<Rect>);

impl StandingClearance {
    pub fn new<F: crate::spatial_geometry::GeometryFrame>(
        cuboids: &[CollisionCuboid<F>],
        floor: crate::spatial_geometry::Elevation<F>,
    ) -> Result<Self, crate::CollisionError> {
        let floor_elevation_metres = floor.metres();
        let mut rectangles = Vec::new();
        for solid in cuboids {
            if let Some(rect) = Obstruction::new(*solid)?.projection(
                floor_elevation_metres + FLOOR_CLEARANCE,
                floor_elevation_metres + PERSON_HEIGHT,
            ) {
                rectangles.push(rect.expanded(PERSON_RADIUS));
            }
        }
        Ok(Self(rectangles))
    }

    pub fn is_clear(&self, start: Vec2, end: Vec2) -> bool {
        self.0
            .iter()
            .all(|&rect| !segment_intersects(rect, start, end))
    }
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
        let mut obstacle = CollisionCuboid::<crate::spatial_geometry::Architectural>::from_metres(
            crate::ResolvedItemId(1),
            Vec3::new(0.12, 0.9, 0.0),
            Vec3::new(0.01, 1.8, 2.0),
            0.0,
            0.0,
            0.0,
        )
        .unwrap();
        let floor = crate::spatial_geometry::Elevation::from_metres(0.0).unwrap();
        assert!(
            !StandingClearance::new(&[obstacle], floor)
                .unwrap()
                .is_clear(-Vec2::X, Vec2::X)
        );
        obstacle.centre =
            crate::spatial_geometry::Position::from_metres(Vec3::new(0.12, 3.0, 0.0)).unwrap();
        assert!(
            StandingClearance::new(&[obstacle], floor)
                .unwrap()
                .is_clear(-Vec2::X, Vec2::X)
        );
    }

    #[test]
    fn prepared_clearance_preserves_rotated_obstacles_at_multiple_heights() {
        let cuboids = [0.0, 0.3, 1.1].map(|angle| {
            CollisionCuboid::<crate::spatial_geometry::Architectural>::from_metres(
                crate::ResolvedItemId(1),
                Vec3::new(angle, 1.2, -angle),
                Vec3::new(2.0, 0.3, 0.2),
                angle,
                angle * 0.5,
                angle,
            )
            .unwrap()
        });
        for floor in [-1.0, 0.0, 0.2, 1.0, 3.0] {
            let clearance = StandingClearance::new(
                &cuboids,
                crate::spatial_geometry::Elevation::from_metres(floor).unwrap(),
            )
            .unwrap();
            for x in -8..=8 {
                let start = Vec2::new(x as f32 * 0.25, -2.0);
                let end = Vec2::new(-start.x, 2.0);
                let original = cuboids.iter().all(|s| {
                    Obstruction::new(*s)
                        .unwrap()
                        .projection(floor + FLOOR_CLEARANCE, floor + PERSON_HEIGHT)
                        .is_none_or(|rect| {
                            !segment_intersects(rect.expanded(PERSON_RADIUS), start, end)
                        })
                });
                assert_eq!(clearance.is_clear(start, end), original);
            }
        }
    }
}
