//! Compare complete terrain-triangle overlays, including subdivision seams.
use super::*;
use crate::city_layout::grounding::planar::{clip, signed_area};

/// A difference control belongs to an actual intersection of both surfaces.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct SurfaceDifferenceControl {
    pub position_metres: Vec2,
    pub first_elevation: SupportElevation,
    pub second_elevation: SupportElevation,
    /// First surface minus second surface, evaluated before coordinate rounding.
    pub difference_metres: f64,
}

/// Extremal differences and explicit coverage over a convex footprint.
/// Missing area does not count as agreement between the two surfaces.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct GeographicSurfaceComparison {
    pub minimum: SurfaceDifferenceControl,
    pub maximum: SurfaceDifferenceControl,
    pub covered_area_square_metres: f64,
    pub required_area_square_metres: f64,
}

impl GeographicSurface {
    /// Compare affine heights at every vertex of the complete intersection of
    /// the footprint and both triangulations. Their difference is affine inside
    /// each such polygon, so these controls include all possible extrema.
    /// An empty, clockwise or nonfinite footprint yields no comparison.
    pub fn compare_in_outline(
        &self,
        other: &Self,
        outline: &[Vec2],
    ) -> Option<GeographicSurfaceComparison> {
        let outline: Vec<_> = outline.iter().map(|p| p.as_dvec2()).collect();
        if outline.len() < 3 || outline.iter().any(|p| !p.is_finite()) {
            return None;
        }
        let required_area_square_metres = signed_area(&outline);
        if required_area_square_metres <= f64::EPSILON {
            return None;
        }
        let bounds = PlanarBounds::from_points(outline.iter().copied())?;
        let mut minimum: Option<SurfaceDifferenceControl> = None;
        let mut maximum: Option<SurfaceDifferenceControl> = None;
        let mut covered_area_square_metres = 0.0;
        for index in self.query.intersections(bounds) {
            let first = &self.triangles[index];
            let mut clipped = first.points().map(|p| p.xz().as_dvec2()).to_vec();
            for i in 0..outline.len() {
                let a = outline[i];
                let edge = outline[(i + 1) % outline.len()] - a;
                clipped = clip(clipped, |p| edge.perp_dot(p - a));
            }
            if signed_area(&clipped) <= f64::EPSILON {
                continue;
            }
            for second in other.intersecting(first) {
                let points = second.points().map(|p| p.xz().as_dvec2());
                let mut polygon = clipped.clone();
                for i in 0..3 {
                    let edge = points[(i + 1) % 3] - points[i];
                    polygon = clip(polygon, |p| edge.perp_dot(p - points[i]));
                }
                let area = signed_area(&polygon);
                if area <= f64::EPSILON {
                    continue;
                }
                covered_area_square_metres += area;
                for p in polygon {
                    let first_height = first.height_f64(p);
                    let second_height = second.height_f64(p);
                    let control = SurfaceDifferenceControl {
                        position_metres: p.as_vec2(),
                        first_elevation: SupportElevation(first_height as f32),
                        second_elevation: SupportElevation(second_height as f32),
                        difference_metres: first_height - second_height,
                    };
                    if minimum.is_none_or(|old| control.difference_metres < old.difference_metres) {
                        minimum = Some(control);
                    }
                    if maximum.is_none_or(|old| control.difference_metres > old.difference_metres) {
                        maximum = Some(control);
                    }
                }
            }
        }
        Some(GeographicSurfaceComparison {
            minimum: minimum?,
            maximum: maximum?,
            covered_area_square_metres,
            required_area_square_metres,
        })
    }
}
