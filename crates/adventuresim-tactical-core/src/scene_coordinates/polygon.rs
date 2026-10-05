//! Scene vertices are rounded projections, not newly authored exact polygons.
use super::*;
use crate::scene_input::BuildingOrientation;
use adventuresim_building_generator::plan_geometry::{ArchitecturalPlanPoint, PlanGeometryError};

/// The rigid transform from the authored floor datum to one scene placement.
#[derive(Clone, Copy, Debug)]
pub struct ArchitecturalPlanProjection {
    pub centre: ScenePlanPoint,
    pub origin: ArchitecturalPlanPoint,
    pub orientation: BuildingOrientation,
}

/// A convex physical footprint projected to finite f32 scene coordinates.
/// Architectural convexity and CCW topology are established before projection.
/// Rotation/translation preserve that topology mathematically; f32 rounding can
/// give nearly collinear edges tiny signed bends. These are the actual existing
/// render/collision coordinates, retained in order without re-hulling or epsilon
/// merging. An arbitrary scene polygon must pass the shared exact constructor.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ScenePlanPolygon(Vec<ScenePlanPoint>);
impl ArchitecturalPlanProjection {
    pub fn from_placement(
        placement: &crate::scene_input::TacticalBuildingPlacement,
        bounds: adventuresim_building_generator::spatial_geometry::SpatialBounds<
            adventuresim_building_generator::spatial_geometry::Architectural,
        >,
    ) -> Result<Self, adventuresim_building_generator::spatial_geometry::GeometryError> {
        use bevy::math::Vec3Swizzles;
        if !placement.orientation.is_valid() {
            return Err(
                adventuresim_building_generator::spatial_geometry::GeometryError::InvalidProjection,
            );
        }
        Ok(Self {
            centre: ScenePlanPoint::try_from(placement.centre_metres)?,
            origin: ArchitecturalPlanPoint::try_from(bounds.centre()?.metres().xz())?,
            orientation: placement.orientation,
        })
    }

    /// Project one admitted architectural point with the established f32 order.
    pub fn point(
        self,
        point: ArchitecturalPlanPoint,
    ) -> Result<ScenePlanPoint, adventuresim_building_generator::spatial_geometry::GeometryError>
    {
        if !self.orientation.is_valid() {
            return Err(
                adventuresim_building_generator::spatial_geometry::GeometryError::InvalidProjection,
            );
        }
        ScenePlanPoint::try_from(
            self.centre.metres()
                + self
                    .orientation
                    .local_to_world(point.metres() - self.origin.metres()),
        )
    }
}

impl ScenePlanPolygon {
    pub fn from_ordered_vertices(vertices: Vec<ScenePlanPoint>) -> Result<Self, PlanGeometryError> {
        let polygon = PlanPolygon::from_ordered_vertices(vertices)?;
        Ok(Self(polygon.vertices().to_vec()))
    }

    pub fn from_architectural(
        source: &PlanPolygon<ArchitecturalPlanPoint>,
        projection: ArchitecturalPlanProjection,
    ) -> Result<Self, PlanGeometryError> {
        if !projection.orientation.is_valid() {
            return Err(PlanGeometryError::InvalidProjection);
        }
        let vertices = source
            .vertices()
            .iter()
            .map(|point| {
                ScenePlanPoint::from_metres(
                    projection.centre.metres()
                        + projection
                            .orientation
                            .local_to_world(point.metres() - projection.origin.metres()),
                )
            })
            .collect::<Option<Vec<_>>>()
            .ok_or(PlanGeometryError::NonFinite)?;
        Self::from_rigid_projection(vertices)
    }

    pub fn vertices(&self) -> &[ScenePlanPoint] {
        &self.0
    }

    pub fn translated(&self, delta: PlanDisplacement) -> Result<Self, PlanGeometryError> {
        let vertices = self
            .0
            .iter()
            .map(|point| ScenePlanPoint::from_metres(point.metres() + delta.metres()))
            .collect::<Option<Vec<_>>>()
            .ok_or(PlanGeometryError::NonFinite)?;
        Self::from_rigid_projection(vertices)
    }

    /// Only validated rigid transforms may enter here. Distinct vertices and
    /// positive represented area reject coordinate collapse/overflow; no general
    /// closure or deserialization can bypass the authored convexity boundary.
    fn from_rigid_projection(vertices: Vec<ScenePlanPoint>) -> Result<Self, PlanGeometryError> {
        let Some(first) = vertices.first() else {
            return Err(PlanGeometryError::DegeneratePolygon);
        };
        let origin = first.metres().as_dvec2();
        let area = vertices
            .iter()
            .zip(vertices.iter().cycle().skip(1))
            .map(|(a, b)| (a.metres().as_dvec2() - origin).perp_dot(b.metres().as_dvec2() - origin))
            .sum::<f64>();
        if vertices.len() < 3 || area <= 0.0 {
            return Err(PlanGeometryError::DegeneratePolygon);
        }
        for (vertex, point) in vertices.iter().enumerate() {
            if vertices[..vertex].contains(point) {
                return Err(PlanGeometryError::DuplicateVertex { vertex });
            }
        }
        Ok(Self(vertices))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rigid_projection_retains_vertices_and_rejects_coordinate_collapse() {
        let p = |x, y| ArchitecturalPlanPoint::from_metres(Vec2::new(x, y)).unwrap();
        let source = PlanPolygon::from_ordered_vertices(vec![
            p(0.0, 0.0),
            p(2.0, 0.0),
            p(2.0, 2.0),
            p(0.0, 2.0),
        ])
        .unwrap();
        let projection = ArchitecturalPlanProjection {
            centre: ScenePlanPoint::from_metres(Vec2::new(90.0, -350.0)).unwrap(),
            origin: p(1.0, 1.0),
            orientation: BuildingOrientation::from_radians(0.37).unwrap(),
        };
        let projected = ScenePlanPolygon::from_architectural(&source, projection).unwrap();
        assert_eq!(
            projected
                .vertices()
                .iter()
                .map(|p| p.metres())
                .collect::<Vec<_>>(),
            source
                .vertices()
                .iter()
                .map(|p| projection.centre.metres()
                    + projection
                        .orientation
                        .local_to_world(p.metres() - projection.origin.metres()))
                .collect::<Vec<_>>()
        );
        let delta = PlanDisplacement::from_metres(Vec2::new(-1.0, 2.0)).unwrap();
        let translated = projected.translated(delta).unwrap();
        assert_eq!(
            translated
                .vertices()
                .iter()
                .map(|p| p.metres())
                .collect::<Vec<_>>(),
            projected
                .vertices()
                .iter()
                .map(|p| p.metres() + delta.metres())
                .collect::<Vec<_>>()
        );
        let collapsed = ArchitecturalPlanProjection {
            centre: ScenePlanPoint::from_metres(Vec2::splat(1e20)).unwrap(),
            ..projection
        };
        assert!(ScenePlanPolygon::from_architectural(&source, collapsed).is_err());
        assert!(
            ScenePlanPolygon::from_ordered_vertices(
                projected.vertices().iter().rev().copied().collect()
            )
            .is_err()
        );
    }
}
