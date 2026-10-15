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
impl CollisionCuboid<Architectural> {
    /// Classify exact signs after the existing f32 pose calculation. Edge
    /// intersections use the existing f32 interpolation. No tolerance merges
    /// different contacts. Computed corners also own the broad phase, avoiding
    /// disagreement with a separately rounded centre/half-extent AABB.
    pub fn ground_contact(self) -> Result<GroundContact, PlanGeometryError> {
        let corners = self
            .corners()
            .map_err(|error| PlanGeometryError::SolidConstruction {
                solid: error.source_id,
                cause: error.cause,
            })?;
        if corners.points().iter().all(|p| p.metres().y > 0.0)
            || corners.points().iter().all(|p| p.metres().y < 0.0)
        {
            return Ok(GroundContact::Empty);
        }
        let point = |p: Vec3| {
            ArchitecturalPlanPoint::from_metres(Vec2::new(p.x, p.z)).map_err(|cause| {
                PlanGeometryError::SolidConstruction {
                    solid: self.source,
                    cause,
                }
            })
        };
        let mut points = Vec::new();
        for &corner in corners.points() {
            if corner.metres().y == 0.0 {
                points.push(point(corner.metres())?);
            }
        }
        for edge in corners.edges() {
            let a = edge.start.metres();
            let b = edge.end.metres();
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

    fn solid() -> CollisionCuboid<crate::spatial_geometry::Architectural> {
        CollisionCuboid::<crate::spatial_geometry::Architectural>::from_metres(
            ResolvedItemId(1),
            Vec3::ZERO,
            Vec3::splat(2.0),
            0.0,
            0.0,
            0.0,
        )
        .unwrap()
    }

    #[test]
    fn pitched_solid_contact_is_smaller_than_its_projected_envelope() {
        let solid = CollisionCuboid {
            centre: crate::spatial_geometry::Position::from_metres(Vec3::Y).unwrap(),
            crossfall_radians: crate::spatial_geometry::Radians::new(core::f32::consts::FRAC_PI_4)
                .unwrap(),
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
        assert!(
            solid.bounds().unwrap().max().metres().z - solid.bounds().unwrap().min().metres().z
                > 2.0
        );
        let collision = BuildingCollision {
            bounds: solid.bounds().unwrap(),
            cuboids: vec![solid],
        };
        let contact = collision.ground_floor_contact_bounds().unwrap().unwrap();
        let expected_half_depth = 2.0_f32.sqrt() - 1.0;
        assert!((contact.min().metres().z + expected_half_depth).abs() < 0.000001);
        assert!((contact.max().metres().z - expected_half_depth).abs() < 0.000001);
        assert_eq!(contact.min().metres().y, 0.0);
        assert_eq!(contact.max().metres().y, 0.0);
        assert_eq!(contact.min().metres().x, -1.0);
        assert_eq!(contact.max().metres().x, 1.0);
    }

    #[test]
    fn buried_slab_keeps_its_top_contact_and_upper_projection_has_none() {
        let slab = CollisionCuboid {
            centre: crate::spatial_geometry::Position::from_metres(Vec3::new(0.0, -0.5, 0.0))
                .unwrap(),
            size: crate::spatial_geometry::CuboidDimensions::from_metres(Vec3::new(2.0, 1.0, 3.0))
                .unwrap(),
            yaw_radians: crate::spatial_geometry::Radians::new(0.73).unwrap(),
            ..solid()
        };
        assert_eq!(slab.ground_contact().unwrap().points().count(), 4);
        let upper = CollisionCuboid {
            centre: crate::spatial_geometry::Position::from_metres(Vec3::Y * 4.0).unwrap(),
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
    fn cube() -> CollisionCuboid<crate::spatial_geometry::Architectural> {
        CollisionCuboid::<crate::spatial_geometry::Architectural>::from_metres(
            ResolvedItemId(1),
            Vec3::ZERO,
            Vec3::splat(2.0),
            0.0,
            0.0,
            0.0,
        )
        .unwrap()
    }
    #[test]
    fn rotated_point_and_edge_tangencies_keep_their_dimensionality() {
        let mut point = CollisionCuboid {
            yaw_radians: crate::spatial_geometry::Radians::new(0.49).unwrap(),
            crossfall_radians: crate::spatial_geometry::Radians::new(0.37).unwrap(),
            longfall_radians: crate::spatial_geometry::Radians::new(0.61).unwrap(),
            ..cube()
        };
        let minimum = point
            .corners()
            .unwrap()
            .points()
            .iter()
            .map(|p| p.metres().y)
            .fold(f32::INFINITY, f32::min);
        point.centre = Position::from_metres(Vec3::Y * -minimum).unwrap();
        assert!(matches!(
            point.ground_contact().unwrap(),
            GroundContact::Point(_)
        ));
        let mut edge = CollisionCuboid {
            crossfall_radians: crate::spatial_geometry::Radians::new(0.41).unwrap(),
            ..cube()
        };
        let minimum = edge
            .corners()
            .unwrap()
            .points()
            .iter()
            .map(|p| p.metres().y)
            .fold(f32::INFINITY, f32::min);
        edge.centre = Position::from_metres(Vec3::Y * -minimum).unwrap();
        assert!(matches!(
            edge.ground_contact().unwrap(),
            GroundContact::Segment(_)
        ));
    }
    #[test]
    fn adjacent_representable_elevations_do_not_acquire_a_contact_epsilon() {
        let contact = CollisionCuboid {
            centre: crate::spatial_geometry::Position::from_metres(Vec3::Y).unwrap(),
            ..cube()
        };
        assert!(matches!(
            contact.ground_contact().unwrap(),
            GroundContact::Area(_)
        ));
        let above = CollisionCuboid {
            centre: crate::spatial_geometry::Position::from_metres(
                Vec3::Y * f32::from_bits(1.0_f32.to_bits() + 1),
            )
            .unwrap(),
            ..cube()
        };
        assert!(matches!(
            above.ground_contact().unwrap(),
            GroundContact::Empty
        ));
        let below = CollisionCuboid {
            centre: crate::spatial_geometry::Position::from_metres(
                Vec3::Y * f32::from_bits(1.0_f32.to_bits() - 1),
            )
            .unwrap(),
            ..cube()
        };
        assert!(matches!(
            below.ground_contact().unwrap(),
            GroundContact::Area(_)
        ));
        let point = CollisionCuboid {
            size: crate::spatial_geometry::CuboidDimensions::from_metres(Vec3::ZERO).unwrap(),
            ..cube()
        };
        assert!(matches!(
            point.ground_contact().unwrap(),
            GroundContact::Point(_)
        ));
        let segment = CollisionCuboid {
            size: crate::spatial_geometry::CuboidDimensions::from_metres(Vec3::X).unwrap(),
            ..cube()
        };
        assert!(matches!(
            segment.ground_contact().unwrap(),
            GroundContact::Segment(_)
        ));
    }
    #[test]
    fn malformed_solids_report_errors_instead_of_absent_support() {
        use crate::spatial_geometry::{CoordinateAxis, GeometryError, GeometryRole};
        let construct = |centre, size| {
            CollisionCuboid::<Architectural>::from_metres(
                ResolvedItemId(17),
                centre,
                size,
                0.0,
                0.0,
                0.0,
            )
        };
        let error = construct(Vec3::splat(f32::NAN), Vec3::ONE).unwrap_err();
        assert_eq!(error.source_id, ResolvedItemId(17));
        assert_eq!(
            error.cause,
            GeometryError::NonFinite {
                role: GeometryRole::Position,
                axis: CoordinateAxis::X
            }
        );
        assert_eq!(
            construct(Vec3::ZERO, -Vec3::ONE).unwrap_err().cause,
            GeometryError::NegativeExtent {
                role: GeometryRole::CuboidDimensions,
                axis: CoordinateAxis::X
            }
        );
        let huge = construct(Vec3::splat(f32::MAX), Vec3::splat(f32::MAX)).unwrap();
        assert!(matches!(
            huge.ground_contact(),
            Err(PlanGeometryError::SolidConstruction {
                solid: ResolvedItemId(17),
                cause: GeometryError::NonFinite {
                    role: GeometryRole::Position,
                    ..
                }
            })
        ));
    }

    #[test]
    fn rotated_church_bearing_survives_inconsistent_aabb_rounding() {
        // Goslar building 14, parish-church programme seed 42, large service
        // size. The separately rounded AABB misses a real datum contact.
        let solid = CollisionCuboid::<crate::spatial_geometry::Architectural>::from_metres(
            ResolvedItemId(1153222152317568514),
            Vec3::new(41.132202, 5.675, 10.5),
            Vec3::new(2.7502518, 11.35, 0.9),
            -core::f32::consts::FRAC_PI_2,
            0.0,
            0.0,
        )
        .unwrap();
        assert!(solid.bounds().unwrap().min().metres().y > 0.0);
        let corners = solid.corners().unwrap();
        assert_eq!(
            corners
                .points()
                .iter()
                .filter(|point| point.metres().y == 0.0)
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

#[cfg(test)]
mod admission_tests {
    use super::*;
    #[test]
    fn thin_sections_survive_both_ground_contact_consumers() {
        for thickness in [0.000_001, f32::MIN_POSITIVE] {
            let solid = CollisionCuboid::<Architectural>::from_metres(
                ResolvedItemId(271),
                Vec3::ZERO,
                Vec3::new(2.0, thickness, 3.0),
                0.0,
                0.0,
                0.0,
            )
            .unwrap();
            let collision = BuildingCollision {
                bounds: solid.bounds().unwrap(),
                cuboids: vec![solid],
            };
            assert!(matches!(
                solid.ground_contact().unwrap(),
                GroundContact::Area(_)
            ));
            let bounds = collision.ground_floor_contact_bounds().unwrap().unwrap();
            assert_eq!(bounds.min().metres(), Vec3::new(-1.0, 0.0, -1.5));
            assert_eq!(bounds.max().metres(), Vec3::new(1.0, 0.0, 1.5));
            assert_eq!(
                collision
                    .ground_floor_footprint()
                    .unwrap()
                    .unwrap()
                    .polygon()
                    .vertices()
                    .len(),
                4
            );
        }
    }
    #[test]
    fn contact_consumers_preserve_overflow_identity_and_cause() {
        let solid = CollisionCuboid::<Architectural>::from_metres(
            ResolvedItemId(272),
            Vec3::splat(f32::MAX),
            Vec3::splat(f32::MAX),
            0.0,
            0.0,
            0.0,
        )
        .unwrap();
        let collision = BuildingCollision {
            bounds: SpatialBounds::at(solid.centre),
            cuboids: vec![solid],
        };
        for error in [
            collision.ground_floor_contact_bounds().unwrap_err(),
            collision.ground_floor_footprint().unwrap_err(),
        ] {
            assert!(matches!(
                error,
                PlanGeometryError::SolidConstruction {
                    solid: ResolvedItemId(272),
                    cause: GeometryError::NonFinite { .. }
                }
            ));
        }
    }
}
