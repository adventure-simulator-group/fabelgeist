//! Indexed exact geographic triangles; no resampling or support substitutions.
use super::*;
use crate::city_layout::grounding::planar::query::{PlanarBounds, PlanarQueryIndex};
mod difference;
mod presented;
mod region;
pub use difference::{GeographicSurfaceComparison, SurfaceDifferenceControl};
pub use region::GeographicRegionMeasurement;

/// Finite, nonvertical geographic triangles with canonical order and winding.
/// Coverage is checked against each complete support triangle during compilation.
#[derive(Clone, Debug)]
pub struct GeographicSurface {
    pub(super) triangles: Vec<GroundTriangle>,
    query: PlanarQueryIndex,
    boundary_extension_factor: f64,
}

/// Extremal source controls at complete footprint/terrain intersections.
#[derive(Clone, Copy, Debug)]
pub struct GeographicHeightRange {
    pub minimum: (Vec2, SupportElevation),
    pub maximum: (Vec2, SupportElevation),
}

impl GeographicSurface {
    pub(super) fn intersecting(
        &self,
        triangle: &GroundTriangle,
    ) -> impl Iterator<Item = &GroundTriangle> {
        self.query
            .intersections(triangle.bounds())
            .into_iter()
            .map(|i| &self.triangles[i])
    }

    pub(super) fn intersecting_boundary_segment(
        &self,
        segment: [bevy::math::DVec2; 2],
        tolerance_metres: f32,
    ) -> impl Iterator<Item = &GroundTriangle> {
        let bounds = PlanarBounds::from_points(segment)
            .expect("an owned boundary has finite endpoints")
            .expanded(f64::from(tolerance_metres) * self.boundary_extension_factor);
        self.query
            .intersections(bounds)
            .into_iter()
            .map(|i| &self.triangles[i])
    }

    pub fn height_range_in(&self, region: CityPlotBounds) -> Option<GeographicHeightRange> {
        self.height_range_in_outline(&region.corners())
    }

    pub fn height_range_in_outline(&self, outline: &[Vec2]) -> Option<GeographicHeightRange> {
        let support: Vec<_> = (1..outline.len().saturating_sub(1))
            .filter_map(|i| {
                GroundTriangle::new(
                    [outline[0], outline[i], outline[i + 1]].map(|p| Vec3::new(p.x, 0.0, p.y)),
                )
            })
            .collect();
        let controls: Vec<_> = support
            .iter()
            .flat_map(|support| {
                self.intersecting(support).flat_map(move |source| {
                    support
                        .intersection(source)
                        .into_iter()
                        .map(|p| (p, SupportElevation(source.height_at(p))))
                })
            })
            .collect();
        let minimum = *controls
            .iter()
            .min_by(|a, b| a.1.metres().total_cmp(&b.1.metres()))?;
        let maximum = *controls
            .iter()
            .max_by(|a, b| a.1.metres().total_cmp(&b.1.metres()))?;
        Some(GeographicHeightRange { minimum, maximum })
    }

    /// Exact ungraded vista triangles with the presentation's a--c diagonal.
    /// Selecting a LOD is explicit; this does not infer finer source resolution.
    pub fn from_vista_lod(lod: &crate::scene_input::VistaLod) -> Option<Self> {
        let width = usize::from(lod.width);
        let depth = usize::from(lod.depth);
        if width < 2
            || depth < 2
            || lod.heights_metres.len() != width.checked_mul(depth)?
            || !lod.spacing_metres.is_finite()
            || lod.spacing_metres <= 0.0
            || !lod.origin_east_metres.is_finite()
            || !lod.origin_north_metres.is_finite()
        {
            return None;
        }
        let half = Vec2::new((width - 1) as f32, (depth - 1) as f32) * 0.5;
        let origin = Vec2::new(
            lod.origin_east_metres as f32,
            lod.origin_north_metres as f32,
        );
        let point = |x: usize, z: usize| {
            let position = (Vec2::new(x as f32, z as f32) - half) * lod.spacing_metres + origin;
            Vec3::new(position.x, lod.heights_metres[z * width + x], position.y)
        };
        Self::from_triangles((0..depth - 1).flat_map(|z| {
            (0..width - 1).flat_map(move |x| {
                let [a, b, c, d] = [
                    point(x, z),
                    point(x + 1, z),
                    point(x + 1, z + 1),
                    point(x, z + 1),
                ];
                [[a, b, c], [a, c, d]]
            })
        }))
    }

    pub fn from_triangles(triangles: impl IntoIterator<Item = [Vec3; 3]>) -> Option<Self> {
        let mut triangles = triangles
            .into_iter()
            .map(GroundTriangle::new)
            .collect::<Option<Vec<_>>>()?;
        triangles.sort_by(GroundTriangle::compare);
        if triangles.is_empty() {
            return None;
        }
        let query =
            PlanarQueryIndex::from_bounds(triangles.iter().map(GroundTriangle::bounds).collect());
        let boundary_extension_factor = triangles
            .iter()
            .map(GroundTriangle::boundary_extension_factor)
            .fold(0.0, f64::max);
        Some(Self {
            triangles,
            query,
            boundary_extension_factor,
        })
    }

    pub fn triangles(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        self.triangles.iter().map(GroundTriangle::points)
    }

    /// Sample the canonical source triangle, without bilinear interpolation.
    /// Complete coverage and overlap validation still belongs to compilation.
    pub fn elevation_at(&self, point: Vec2) -> Option<SupportElevation> {
        self.query
            .intersections(PlanarBounds::from_points([point.as_dvec2()])?)
            .into_iter()
            .map(|i| &self.triangles[i])
            .find(|triangle| triangle.contains(point, 0.0))
            .map(|triangle| SupportElevation(triangle.height_at(point)))
    }
}

#[cfg(test)]
mod tests;
