//! Continuous standing-body sweeps retain the collision frame through clearance.
use super::{geometry::*, obstruction::Obstruction};
use crate::CollisionResult as Result;
use crate::spatial_geometry::{Elevation, GeometryFrame, Position};
use crate::{CollisionCuboid, CollisionError, SpatialBounds};
use bevy::math::{Vec2, Vec3};

pub struct StandingClearance<F: GeometryFrame>(Vec<SpatialBounds<F>>);
impl<F: GeometryFrame> StandingClearance<F> {
    pub fn new(cuboids: &[CollisionCuboid<F>], floor: Elevation<F>) -> Result<Self> {
        let mut rectangles = Vec::new();
        for solid in cuboids {
            let geometry = || {
                let bottom = Elevation::from_metres(floor.metres() + FLOOR_CLEARANCE)?;
                let top = Elevation::from_metres(floor.metres() + PERSON_HEIGHT)?;
                Ok::<_, crate::spatial_geometry::GeometryError>((bottom, top))
            };
            let (bottom, top) = geometry().map_err(|cause| CollisionError {
                source_id: solid.source,
                cause,
            })?;
            if let Some(bounds) = Obstruction::new(*solid)?.projection(bottom, top)? {
                let margin = Vec3::new(PERSON_RADIUS, 0.0, PERSON_RADIUS);
                rectangles.push(
                    SpatialBounds::from_metres(
                        bounds.min().metres() - margin,
                        bounds.max().metres() + margin,
                    )
                    .map_err(|cause| CollisionError {
                        source_id: solid.source,
                        cause,
                    })?,
                );
            }
        }
        Ok(Self(rectangles))
    }
    pub fn is_clear(&self, start: Position<F>, end: Position<F>) -> bool {
        self.0
            .iter()
            .all(|&rect| !segment_intersects(rect, start, end))
    }
}
/// Native slab intersection is bounded by admitted framed endpoints and bounds.
fn segment_intersects<F: GeometryFrame>(
    rect: SpatialBounds<F>,
    start: Position<F>,
    end: Position<F>,
) -> bool {
    let min = Vec2::new(rect.min().metres().x, rect.min().metres().z);
    let max = Vec2::new(rect.max().metres().x, rect.max().metres().z);
    let start = Vec2::new(start.metres().x, start.metres().z);
    let end = Vec2::new(end.metres().x, end.metres().z);
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
    fn position(p: Vec2) -> Position<crate::Architectural> {
        Position::from_metres(Vec3::new(p.x, 0.0, p.y)).unwrap()
    }
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
                .is_clear(position(-Vec2::X), position(Vec2::X))
        );
        obstacle.centre =
            crate::spatial_geometry::Position::from_metres(Vec3::new(0.12, 3.0, 0.0)).unwrap();
        assert!(
            StandingClearance::new(&[obstacle], floor)
                .unwrap()
                .is_clear(position(-Vec2::X), position(Vec2::X))
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
                    let projection = Obstruction::new(*s)
                        .unwrap()
                        .projection(
                            Elevation::from_metres(floor + FLOOR_CLEARANCE).unwrap(),
                            Elevation::from_metres(floor + PERSON_HEIGHT).unwrap(),
                        )
                        .unwrap();
                    projection.is_none_or(|bounds| {
                        let margin = Vec3::new(PERSON_RADIUS, 0.0, PERSON_RADIUS);
                        let expanded = SpatialBounds::from_metres(
                            bounds.min().metres() - margin,
                            bounds.max().metres() + margin,
                        )
                        .unwrap();
                        !segment_intersects(expanded, position(start), position(end))
                    })
                });
                assert_eq!(clearance.is_clear(position(start), position(end)), original);
            }
        }
    }
}
