//! Footprint coverage and physical grade include complete triangle intersections.
use super::*;
use crate::city_layout::grounding::planar::{clip, signed_area};

/// A slope observation alone cannot certify traversal across retaining edges.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct GeographicRegionMeasurement {
    pub required_area_square_metres: DiagnosticArea,
    pub covered_area_square_metres: DiagnosticArea,
    pub maximum_grade: GeographicGrade,
    pub maximum_grade_location_metres: crate::scene_coordinates::ScenePlanPoint,
    pub maximum_grade_triangle_metres: [adventuresim_building_generator::spatial_geometry::Position<
        crate::scene_coordinates::Scene,
    >; 3],
}

impl GeographicSurface {
    /// Measure all affine source triangles intersecting a convex footprint.
    /// Missing area remains explicit. Vertical risers, gate sweeps and actor
    /// clearance require separate collision checks against the composed solid.
    pub fn measure_region(
        &self,
        outline: &crate::scene_coordinates::ScenePlanPolygon,
    ) -> Option<GeographicRegionMeasurement> {
        let outline: Vec<_> = outline
            .vertices()
            .iter()
            .map(|p| p.metres().as_dvec2())
            .collect();
        if outline.len() < 3 || outline.iter().any(|p| !p.is_finite()) {
            return None;
        }
        let required_area_square_metres = signed_area(&outline);
        if required_area_square_metres <= f64::EPSILON {
            return None;
        }
        let bounds = PlanarBounds::from_points(outline.iter().copied())?;
        let mut covered_area_square_metres = 0.0;
        let mut maximum: Option<GeographicRegionMeasurement> = None;
        for index in self.query.intersections(bounds) {
            let triangle = &self.triangles[index];
            let points = triangle.points();
            let mut polygon = points.map(|p| p.xz().as_dvec2()).to_vec();
            for i in 0..outline.len() {
                let a = outline[i];
                let edge = outline[(i + 1) % outline.len()] - a;
                polygon = clip(polygon, |p| edge.perp_dot(p - a));
            }
            let area = signed_area(&polygon);
            if area <= f64::EPSILON {
                continue;
            }
            covered_area_square_metres += area;
            let [a, b, c] = points.map(Vec3::as_dvec3);
            let normal = (b - a).cross(c - a);
            let grade = normal.xz().length() / normal.y.abs();
            if maximum.is_none_or(|old| grade > old.maximum_grade.ratio()) {
                let location =
                    polygon.iter().copied().sum::<bevy::math::DVec2>() / polygon.len() as f64;
                maximum = Some(GeographicRegionMeasurement {
                    required_area_square_metres: DiagnosticArea::from_square_metres(required_area_square_metres)?,
                    covered_area_square_metres: DiagnosticArea::from_square_metres(covered_area_square_metres)?,
                    maximum_grade: GeographicGrade::from_ratio(grade)?,
                    maximum_grade_location_metres: crate::scene_coordinates::ScenePlanPoint::from_metres(location.as_vec2())?,
                    maximum_grade_triangle_metres: points.map(adventuresim_building_generator::spatial_geometry::Position::from_metres).into_iter().collect::<Result<Vec<_>, _>>().ok()?.try_into().ok()?,
                });
            }
        }
        let mut measurement = maximum?;
        measurement.covered_area_square_metres =
            DiagnosticArea::from_square_metres(covered_area_square_metres)?;
        Some(measurement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotated_footprint_measures_hidden_triangle_slope_and_missing_area() {
        let source = GeographicSurface::from_triangles([
            [
                Vec3::ZERO,
                Vec3::new(4.0, 0.0, 0.0),
                Vec3::new(4.0, 0.0, 4.0),
            ],
            [
                Vec3::ZERO,
                Vec3::new(4.0, 0.0, 4.0),
                Vec3::new(0.0, 2.0, 4.0),
            ],
        ])
        .unwrap();
        let outline = [
            Vec2::new(2.0, 0.5),
            Vec2::new(3.5, 2.0),
            Vec2::new(2.0, 3.5),
            Vec2::new(0.5, 2.0),
        ];
        let measured = source
            .measure_region(
                &crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
                    (outline)
                        .iter()
                        .copied()
                        .map(crate::scene_coordinates::ScenePlanPoint::try_from)
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        assert!((measured.required_area_square_metres.square_metres() - 4.5).abs() < 1e-9);
        assert!((measured.covered_area_square_metres.square_metres() - 4.5).abs() < 1e-9);
        assert!((measured.maximum_grade.ratio() - 0.5_f64.hypot(0.5)).abs() < 1e-9);
        let outside = outline.map(|p| p + Vec2::X * 2.0);
        let partial = source
            .measure_region(
                &crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
                    (outside)
                        .iter()
                        .copied()
                        .map(crate::scene_coordinates::ScenePlanPoint::try_from)
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        assert!(
            partial.covered_area_square_metres.square_metres()
                < partial.required_area_square_metres.square_metres()
        );
        assert!(
            crate::scene_coordinates::ScenePlanPolygon::from_ordered_vertices(
                outline
                    .into_iter()
                    .rev()
                    .map(|point| crate::scene_coordinates::ScenePlanPoint::try_from(point).unwrap())
                    .collect()
            )
            .is_err()
        );
    }
}
