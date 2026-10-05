//! Physical contact dimensionality at the authored architectural ground datum.
use super::*;
use crate::plan_geometry::{
    ArchitecturalPlanPoint, PlanGeometryError, PlanHull, PlanPolygon, PlanSegment,
};

#[derive(Clone, Debug, PartialEq)]
pub enum GroundContact {
    Empty,
    Point(ArchitecturalPlanPoint),
    Segment(PlanSegment<ArchitecturalPlanPoint>),
    Area(PlanPolygon<ArchitecturalPlanPoint>),
}
impl GroundContact {
    pub fn points(&self) -> impl Iterator<Item = ArchitecturalPlanPoint> + '_ {
        let (fixed, area) = match self {
            Self::Empty => ([None, None], &[][..]),
            Self::Point(point) => ([Some(*point), None], &[][..]),
            Self::Segment(segment) => ([Some(segment.start()), Some(segment.end())], &[][..]),
            Self::Area(polygon) => ([None, None], polygon.vertices()),
        };
        fixed.into_iter().flatten().chain(area.iter().copied())
    }
}
impl CollisionCuboid {
    /// Classify exact signs after the existing f32 pose calculation. Edge
    /// intersections use the existing f32 interpolation. No tolerance merges
    /// different contacts. Computed corners also own the broad phase, avoiding
    /// disagreement with a separately rounded centre/half-extent AABB.
    pub fn ground_contact(self) -> Result<GroundContact, PlanGeometryError> {
        let corners = self.corners()?;
        if corners.points().iter().all(|p| p.y > 0.0) || corners.points().iter().all(|p| p.y < 0.0)
        {
            return Ok(GroundContact::Empty);
        }
        let point = |p: Vec3| {
            ArchitecturalPlanPoint::from_metres(Vec2::new(p.x, p.z))
                .ok_or(PlanGeometryError::NonFinite)
        };
        let mut points = Vec::new();
        for &corner in corners.points() {
            if corner.y == 0.0 {
                points.push(point(corner)?);
            }
        }
        for edge in corners.edges() {
            let a = edge.start;
            let b = edge.end;
            if (a.y < 0.0 && b.y > 0.0) || (a.y > 0.0 && b.y < 0.0) {
                points.push(point(a + (b - a) * (-a.y / (b.y - a.y)))?);
            }
        }
        Ok(match PlanHull::from_points(points)? {
            PlanHull::Empty => GroundContact::Empty,
            PlanHull::Point(point) => GroundContact::Point(point),
            PlanHull::Segment(segment) => GroundContact::Segment(segment),
            PlanHull::Area(polygon) => GroundContact::Area(polygon),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid() -> CollisionCuboid {
        CollisionCuboid {
            source: ResolvedItemId(1),
            centre: Vec3::ZERO,
            size: Vec3::splat(2.0),
            yaw_radians: 0.0,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
        }
    }

    #[test]
    fn pitched_solid_contact_is_smaller_than_its_projected_envelope() {
        let solid = CollisionCuboid {
            centre: Vec3::Y,
            crossfall_radians: core::f32::consts::FRAC_PI_4,
            ..solid()
        };
        let polygon = solid.ground_contact().unwrap();
        assert_eq!(polygon.points().count(), 4);
        let min = polygon
            .points()
            .map(|p| p.metres().y)
            .fold(f32::INFINITY, f32::min);
        let max = polygon
            .points()
            .map(|p| p.metres().y)
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(max - min < 1.0, "the floor cuts only the lower tip");
        assert!(solid.bounds().max.z - solid.bounds().min.z > 2.0);
        let collision = BuildingCollision {
            bounds: solid.bounds(),
            cuboids: vec![solid],
        };
        let contact = collision.ground_floor_contact_bounds().unwrap().unwrap();
        let expected_half_depth = 2.0_f32.sqrt() - 1.0;
        assert!((contact.min.z + expected_half_depth).abs() < 0.000001);
        assert!((contact.max.z - expected_half_depth).abs() < 0.000001);
        assert_eq!(contact.min.y, 0.0);
        assert_eq!(contact.max.y, 0.0);
        assert_eq!(contact.min.x, -1.0);
        assert_eq!(contact.max.x, 1.0);
    }

    #[test]
    fn buried_slab_keeps_its_top_contact_and_upper_projection_has_none() {
        let slab = CollisionCuboid {
            centre: Vec3::new(0.0, -0.5, 0.0),
            size: Vec3::new(2.0, 1.0, 3.0),
            yaw_radians: 0.73,
            ..solid()
        };
        assert_eq!(slab.ground_contact().unwrap().points().count(), 4);
        let upper = CollisionCuboid {
            centre: Vec3::Y * 4.0,
            ..slab
        };
        assert!(matches!(
            upper.ground_contact().unwrap(),
            GroundContact::Empty
        ));
    }
}

#[cfg(test)]
mod datum_regressions {
    use super::*;
    fn cube() -> CollisionCuboid {
        CollisionCuboid {
            source: ResolvedItemId(1),
            centre: Vec3::ZERO,
            size: Vec3::splat(2.0),
            yaw_radians: 0.0,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
        }
    }
    #[test]
    fn rotated_point_and_edge_tangencies_keep_their_dimensionality() {
        let mut point = CollisionCuboid {
            yaw_radians: 0.49,
            crossfall_radians: 0.37,
            longfall_radians: 0.61,
            ..cube()
        };
        let minimum = point
            .corners()
            .unwrap()
            .points()
            .iter()
            .map(|p| p.y)
            .fold(f32::INFINITY, f32::min);
        point.centre.y = -minimum;
        assert!(matches!(
            point.ground_contact().unwrap(),
            GroundContact::Point(_)
        ));
        let mut edge = CollisionCuboid {
            crossfall_radians: 0.41,
            ..cube()
        };
        let minimum = edge
            .corners()
            .unwrap()
            .points()
            .iter()
            .map(|p| p.y)
            .fold(f32::INFINITY, f32::min);
        edge.centre.y = -minimum;
        assert!(matches!(
            edge.ground_contact().unwrap(),
            GroundContact::Segment(_)
        ));
    }
    #[test]
    fn adjacent_representable_elevations_do_not_acquire_a_contact_epsilon() {
        let contact = CollisionCuboid {
            centre: Vec3::Y,
            ..cube()
        };
        assert!(matches!(
            contact.ground_contact().unwrap(),
            GroundContact::Area(_)
        ));
        let above = CollisionCuboid {
            centre: Vec3::Y * f32::from_bits(1.0_f32.to_bits() + 1),
            ..cube()
        };
        assert!(matches!(
            above.ground_contact().unwrap(),
            GroundContact::Empty
        ));
        let below = CollisionCuboid {
            centre: Vec3::Y * f32::from_bits(1.0_f32.to_bits() - 1),
            ..cube()
        };
        assert!(matches!(
            below.ground_contact().unwrap(),
            GroundContact::Area(_)
        ));
        let point = CollisionCuboid {
            size: Vec3::ZERO,
            ..cube()
        };
        assert!(matches!(
            point.ground_contact().unwrap(),
            GroundContact::Point(_)
        ));
        let segment = CollisionCuboid {
            size: Vec3::X,
            ..cube()
        };
        assert!(matches!(
            segment.ground_contact().unwrap(),
            GroundContact::Segment(_)
        ));
    }
    #[test]
    fn malformed_solids_report_errors_instead_of_absent_support() {
        let invalid = CollisionCuboid {
            centre: Vec3::splat(f32::NAN),
            ..cube()
        };
        assert_eq!(invalid.ground_contact(), Err(PlanGeometryError::NonFinite));
        let negative = CollisionCuboid {
            size: -Vec3::ONE,
            ..cube()
        };
        assert_eq!(
            negative.ground_contact(),
            Err(PlanGeometryError::NegativeSolidDimension)
        );
    }

    #[test]
    fn rotated_church_bearing_survives_inconsistent_aabb_rounding() {
        // Goslar building 14, parish-church programme seed 42, large service
        // size. The separately rounded AABB misses a real datum contact.
        let solid = CollisionCuboid {
            source: ResolvedItemId(1153222152317568514),
            centre: Vec3::new(41.132202, 5.675, 10.5),
            size: Vec3::new(2.7502518, 11.35, 0.9),
            yaw_radians: -core::f32::consts::FRAC_PI_2,
            crossfall_radians: 0.0,
            longfall_radians: 0.0,
        };
        assert!(solid.bounds().min.y > 0.0);
        let corners = solid.corners().unwrap();
        assert_eq!(
            corners
                .points()
                .iter()
                .filter(|point| point.y == 0.0)
                .count(),
            4
        );
        let GroundContact::Area(contact) = solid.ground_contact().unwrap() else {
            panic!("the computed bottom face must remain an area bearing")
        };
        assert_eq!(contact.vertices().len(), 4);
        assert_eq!(
            contact
                .vertices()
                .iter()
                .map(|p| p.metres().x)
                .fold(f32::NEG_INFINITY, f32::max),
            41.582203
        );
    }
}
