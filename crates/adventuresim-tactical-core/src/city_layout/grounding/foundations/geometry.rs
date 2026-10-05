//! Convex intersection arithmetic uses double precision at scene coordinates.
use super::*;
use crate::city_layout::grounding::planar::query::{PlanarBounds, PlanarQueryIndex};
use crate::city_layout::grounding::planar::{clip, subtract};
use std::cmp::Ordering;
mod cuts;
pub(super) use cuts::SourceCutRegions;

#[derive(Clone, Debug)]
pub(in crate::city_layout::grounding) struct GroundTriangle([Vec3; 3]);

impl GroundTriangle {
    pub fn bounds(&self) -> PlanarBounds {
        PlanarBounds::from_points(self.0.map(|p| p.xz().as_dvec2()))
            .expect("ground triangles contain three finite points")
    }

    pub fn new(mut points: [Vec3; 3]) -> Option<Self> {
        if points.iter().any(|p| !p.is_finite()) {
            return None;
        }
        let signed_area = (points[1].xz().as_dvec2() - points[0].xz().as_dvec2())
            .perp_dot(points[2].xz().as_dvec2() - points[0].xz().as_dvec2());
        if signed_area.abs() <= f64::EPSILON {
            return None;
        }
        if signed_area < 0.0 {
            points.swap(1, 2);
        }
        let first = (0..3).min_by(|a, b| compare_point(points[*a], points[*b]))?;
        points.rotate_left(first);
        Some(Self(points))
    }

    /// Offset all three half-planes by a distance. The resulting triangle is
    /// homothetic about its incenter; the farthest vertex displacement bounds
    /// both coordinate expansions, including acute corners.
    pub fn boundary_extension_factor(&self) -> f64 {
        let points = self.0.map(|p| p.xz().as_dvec2());
        let lengths = [
            points[1].distance(points[2]),
            points[2].distance(points[0]),
            points[0].distance(points[1]),
        ];
        let perimeter = lengths.iter().sum::<f64>();
        let incenter = (0..3)
            .map(|i| points[i] * lengths[i])
            .sum::<bevy::math::DVec2>()
            / perimeter;
        let inradius = self.area() * 2.0 / perimeter;
        points
            .into_iter()
            .map(|p| p.distance(incenter) / inradius)
            .fold(0.0, f64::max)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| compare_point(*a, *b))
            .find(|order| !order.is_eq())
            .unwrap_or(Ordering::Equal)
    }

    pub fn height_at(&self, point: Vec2) -> f32 {
        self.height_f64(point.as_dvec2()) as f32
    }

    pub(in crate::city_layout::grounding) fn height_f64(&self, point: bevy::math::DVec2) -> f64 {
        let [a, b, c] = self.0;
        let ab = b.xz().as_dvec2() - a.xz().as_dvec2();
        let ac = c.xz().as_dvec2() - a.xz().as_dvec2();
        let ap = point - a.xz().as_dvec2();
        let det = ab.perp_dot(ac);
        f64::from(a.y)
            + (f64::from(b.y) - f64::from(a.y)) * ap.perp_dot(ac) / det
            + (f64::from(c.y) - f64::from(a.y)) * ab.perp_dot(ap) / det
    }

    /// Lift the represented horizontal point, after final f32 quantization.
    /// Lifting a pre-rounded intersection gives narrow triangles a different
    /// plane from their geographic source and can invent an unwalkable normal.
    fn represented_point(&self, point: bevy::math::DVec2) -> Vec3 {
        let point = point.as_vec2();
        Vec3::new(point.x, self.height_at(point), point.y)
    }

    pub fn area(&self) -> f64 {
        area(&self.0.map(|p| p.xz()))
    }

    pub fn perimeter(&self) -> f64 {
        (0..3)
            .map(|i| {
                self.0[i]
                    .xz()
                    .as_dvec2()
                    .distance(self.0[(i + 1) % 3].xz().as_dvec2())
            })
            .sum()
    }

    pub fn centre(&self) -> Vec2 {
        (self.0[0].xz() + self.0[1].xz() + self.0[2].xz()) / 3.0
    }

    /// A float query denotes its rounding cell. Admit that cell at a declared
    /// edge without extending ownership by an architectural contact margin.
    /// This matters when a represented midpoint rounds just outside its face.
    /// The rounding-cell diagonal is independent of triangle geometry, so a
    /// multi-face query computes it once while retaining the same edge rule.
    pub fn represented_point_tolerance(point: Vec2) -> f32 {
        let half_quantum = |coordinate: f32| {
            let upper = f64::from(coordinate.next_up()) - f64::from(coordinate);
            let lower = f64::from(coordinate) - f64::from(coordinate.next_down());
            upper.max(lower) * 0.5
        };
        let uncertainty = half_quantum(point.x).hypot(half_quantum(point.y));
        (uncertainty as f32).next_up()
    }

    pub fn contains(&self, point: Vec2, tolerance_metres: f32) -> bool {
        (0..3).all(|i| {
            let a = self.0[i].xz().as_dvec2();
            let edge = self.0[(i + 1) % 3].xz().as_dvec2() - a;
            edge.perp_dot(point.as_dvec2() - a) >= -f64::from(tolerance_metres) * edge.length()
        })
    }

    pub fn points(&self) -> [Vec3; 3] {
        self.0
    }

    pub(in crate::city_layout::grounding::foundations) fn outside_regions(
        &self,
        regions: &SourceCutRegions,
    ) -> Vec<Self> {
        let mut pieces = vec![self.0.map(|p| p.xz().as_dvec2()).to_vec()];
        for region in regions.intersecting(self) {
            pieces = pieces
                .into_iter()
                .flat_map(|polygon| subtract(polygon, region))
                .collect();
        }
        pieces
            .into_iter()
            .flat_map(|polygon| {
                (1..polygon.len().saturating_sub(1)).filter_map(move |i| {
                    let points =
                        [polygon[0], polygon[i], polygon[i + 1]].map(|p| self.represented_point(p));
                    Self::new(points)
                })
            })
            .collect()
    }

    /// Remove an already owned convex floor while retaining this triangle's
    /// affine elevation. This is a same-property surface union, not a change to
    /// grading ownership or a proximity-based terrace merge.
    pub fn outside_outline(&self, outline: &[bevy::math::DVec2]) -> Vec<Self> {
        subtract(self.0.map(|p| p.xz().as_dvec2()).to_vec(), outline)
            .into_iter()
            .flat_map(|polygon| {
                (1..polygon.len().saturating_sub(1)).filter_map(move |i| {
                    Self::new(
                        [polygon[0], polygon[i], polygon[i + 1]].map(|p| self.represented_point(p)),
                    )
                })
            })
            .collect()
    }

    pub fn intersection(&self, other: &Self) -> Vec<Vec2> {
        let mut polygon = self.0.map(|p| p.xz().as_dvec2()).to_vec();
        for index in 0..3 {
            let a = other.0[index].xz().as_dvec2();
            let edge = other.0[(index + 1) % 3].xz().as_dvec2() - a;
            polygon = clip(polygon, |p| edge.perp_dot(p - a));
        }
        polygon.into_iter().map(|p| p.as_vec2()).collect()
    }
}

pub(super) fn area(polygon: &[Vec2]) -> f64 {
    let Some(origin) = polygon.first() else {
        return 0.0;
    };
    polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
        .map(|(a, b)| (a.as_dvec2() - origin.as_dvec2()).perp_dot(b.as_dvec2() - origin.as_dvec2()))
        .sum::<f64>()
        .abs()
        * 0.5
}

pub(super) fn height_sections(
    polygon: &[Vec2],
    support: &GroundTriangle,
    source: &GroundTriangle,
) -> Vec<Vec<Vec2>> {
    let difference = |point| support.height_f64(point) - source.height_f64(point);
    let differences: Vec<_> = polygon.iter().map(|p| difference(p.as_dvec2())).collect();
    if differences.iter().all(|h| *h >= 0.0) || differences.iter().all(|h| *h <= 0.0) {
        return vec![polygon.to_vec()];
    }
    [1.0, -1.0]
        .map(|sign| {
            clip(polygon.iter().map(|p| p.as_dvec2()).collect(), |p| {
                difference(p) * sign
            })
            .into_iter()
            .map(|p| p.as_vec2())
            .collect()
        })
        .to_vec()
}

fn compare_point(a: Vec3, b: Vec3) -> Ordering {
    a.x.total_cmp(&b.x)
        .then(a.z.total_cmp(&b.z))
        .then(a.y.total_cmp(&b.y))
}

#[cfg(test)]
mod tests;
