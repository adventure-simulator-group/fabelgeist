//! The occupied floor envelope follows actual fixed-solid contacts.
use super::*;

/// Counterclockwise convex envelope of fixed geometry intersecting Y=0.
/// Filling the space between bearings supports the occupied floor without
/// assigning empty corners of its rectangular bounding box to this building.
#[derive(Clone, Debug, PartialEq)]
pub struct GroundFloorFootprint(Vec<Vec2>);

impl GroundFloorFootprint {
    pub fn vertices(&self) -> &[Vec2] {
        &self.0
    }
}

impl BuildingCollision {
    pub fn ground_floor_footprint(&self) -> Option<GroundFloorFootprint> {
        let mut points: Vec<_> = self
            .cuboids
            .iter()
            .copied()
            .flat_map(CollisionCuboid::ground_contact_polygon)
            .collect();
        points.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
        points.dedup();
        let cross = |a: Vec2, b: Vec2, c: Vec2| {
            (b.as_dvec2() - a.as_dvec2()).perp_dot(c.as_dvec2() - a.as_dvec2())
        };
        let half = |points: &mut dyn Iterator<Item = Vec2>| {
            let mut hull: Vec<Vec2> = Vec::new();
            for point in points {
                while hull.len() >= 2
                    && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0
                {
                    hull.pop();
                }
                hull.push(point);
            }
            hull.pop();
            hull
        };
        let mut hull = half(&mut points.iter().copied());
        hull.extend(half(&mut points.iter().rev().copied()));
        (hull.len() >= 3).then_some(GroundFloorFootprint(hull))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec3Swizzles;
    #[test]
    fn rotated_contacts_do_not_acquire_empty_bounding_box_corners() {
        let solid = CollisionCuboid {
            source: ResolvedItemId(1),
            centre: Vec3::ZERO,
            size: Vec3::new(4.0, 2.0, 4.0),
            yaw_radians: core::f32::consts::FRAC_PI_4,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
        };
        let mut collision = BuildingCollision {
            bounds: solid.bounds(),
            cuboids: vec![solid],
        };
        let footprint = collision.ground_floor_footprint().unwrap();
        assert_eq!(footprint.vertices().len(), 4);
        let area = footprint
            .vertices()
            .iter()
            .zip(footprint.vertices().iter().cycle().skip(1))
            .map(|(a, b)| a.as_dvec2().perp_dot(b.as_dvec2()))
            .sum::<f64>()
            * 0.5;
        assert!((area - 16.0).abs() < 0.00001);
        let corner = collision.ground_floor_contact_bounds().unwrap().max.xz();
        assert!(
            footprint
                .vertices()
                .iter()
                .zip(footprint.vertices().iter().cycle().skip(1))
                .any(|(a, b)| (*b - *a).perp_dot(corner - *a) < 0.0)
        );
        collision.cuboids.push(solid);
        collision.cuboids.reverse();
        assert_eq!(footprint, collision.ground_floor_footprint().unwrap());
    }
}
