//! Whole occupied floors intentionally fill the convex envelope between bearings.
use super::*;
use crate::plan_geometry::{ArchitecturalPlanPoint, PlanGeometryError, PlanHull, PlanPolygon};

#[derive(Clone, Debug, PartialEq)]
pub struct GroundFloorFootprint(PlanPolygon<ArchitecturalPlanPoint>);
impl GroundFloorFootprint {
    pub fn vertices(&self) -> &[ArchitecturalPlanPoint] {
        self.0.vertices()
    }
    pub fn polygon(&self) -> &PlanPolygon<ArchitecturalPlanPoint> {
        &self.0
    }
}
impl BuildingCollision {
    pub fn ground_floor_footprint(
        &self,
    ) -> Result<Option<GroundFloorFootprint>, PlanGeometryError> {
        let mut points = Vec::new();
        for solid in &self.cuboids {
            points.extend(solid.ground_contact()?.points());
        }
        Ok(match PlanHull::from_points(points)? {
            PlanHull::Area(polygon) => Some(GroundFloorFootprint(polygon)),
            PlanHull::Empty | PlanHull::Point(_) | PlanHull::Segment(_) => None,
        })
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
        let footprint = collision.ground_floor_footprint().unwrap().unwrap();
        assert_eq!(footprint.vertices().len(), 4);
        let area = footprint
            .vertices()
            .iter()
            .zip(footprint.vertices().iter().cycle().skip(1))
            .map(|(a, b)| a.metres().as_dvec2().perp_dot(b.metres().as_dvec2()))
            .sum::<f64>()
            * 0.5;
        assert!((area - 16.0).abs() < 0.00001);
        let corner = collision
            .ground_floor_contact_bounds()
            .unwrap()
            .unwrap()
            .max
            .xz();
        assert!(
            footprint
                .vertices()
                .iter()
                .zip(footprint.vertices().iter().cycle().skip(1))
                .any(|(a, b)| (b.metres() - a.metres()).perp_dot(corner - a.metres()) < 0.0)
        );
        collision.cuboids.push(solid);
        collision.cuboids.reverse();
        assert_eq!(
            footprint,
            collision.ground_floor_footprint().unwrap().unwrap()
        );
    }
}
