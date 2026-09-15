//! Closed solid construction from shared planar and radial boundaries.
use super::{Detail, PlanarPoint, Region, Solid, signed_area, subdivide};
use std::f64::consts::{PI, TAU};

fn planar_region(outline: &[PlanarPoint], max_edge: f64) -> Result<(Region, f64), String> {
    let region = Region::triangulate(outline, false)?;
    let budget = max_edge.max((signed_area(&region.points).abs() / 800.0).sqrt());
    Ok((region.refine(budget), budget))
}

impl Solid {
    /// A polygonal outer barrel with a circular bore. Display detail subdivides
    /// the same authored flats while independently resolving the bore circle.
    pub(crate) fn faceted_socket(
        profile: &[PlanarPoint],
        bore: f64,
        sides: usize,
        detail: Detail,
    ) -> Self {
        let largest = profile.iter().map(|p| p[1]).fold(0.0, f64::max);
        let segments = detail.radial(largest, 16).div_ceil(sides) * sides;
        let point = |row: usize, index: usize, inner: bool| {
            let angle = index as f64 / segments as f64 * TAU;
            let sector = TAU / sides as f64;
            let local = (angle % sector) - sector / 2.0;
            let radius = if inner {
                bore
            } else {
                profile[row][1] * (sector / 2.0).cos() / local.cos()
            };
            [angle.cos() * radius, profile[row][0], angle.sin() * radius]
        };
        let mut solid = Self::default();
        for inner in [false, true] {
            for row in 0..profile.len() - 1 {
                for segment in 0..segments {
                    let next = (segment + 1) % segments;
                    let [a, b, c, d] = [
                        point(row, segment, inner),
                        point(row, next, inner),
                        point(row + 1, next, inner),
                        point(row + 1, segment, inner),
                    ];
                    if inner {
                        solid.quad(a, b, c, d, sides as u32 + 1);
                    } else {
                        solid.quad(a, d, c, b, (segment / (segments / sides)) as u32 + 1);
                    }
                }
            }
        }
        for row in [0, profile.len() - 1] {
            for segment in 0..segments {
                let next = (segment + 1) % segments;
                let [a, b, c, d] = [
                    point(row, segment, false),
                    point(row, next, false),
                    point(row, next, true),
                    point(row, segment, true),
                ];
                if row == 0 {
                    solid.quad(a, b, c, d, 0);
                } else {
                    solid.quad(a, d, c, b, 0);
                }
            }
        }
        solid.positive()
    }
    pub(crate) fn prism(
        outline: &[PlanarPoint],
        thickness: f64,
        detail: Detail,
    ) -> Result<Self, String> {
        let (region, max_edge) = planar_region(outline, detail.error(0.03))?;
        let mut solid = Self::default();
        let half = thickness / 2.0;
        let vertex = |i: usize, z| [region.points[i][0], region.points[i][1], z];
        for [a, b, c] in region.triangles {
            solid.triangle(vertex(a, half), vertex(b, half), vertex(c, half), 0);
            solid.triangle(vertex(c, -half), vertex(b, -half), vertex(a, -half), 0);
        }
        let layers = (thickness / max_edge).ceil().max(1.0) as usize;
        for edge in 0..region.boundary.len() {
            for row in 0..layers {
                let a = region.boundary[edge];
                let b = region.boundary[(edge + 1) % region.boundary.len()];
                let z0 = -half + thickness * row as f64 / layers as f64;
                let z1 = -half + thickness * (row + 1) as f64 / layers as f64;
                solid.quad(
                    vertex(a, z0),
                    vertex(b, z0),
                    vertex(b, z1),
                    vertex(a, z1),
                    0,
                );
            }
        }
        Ok(solid.positive())
    }

    pub(crate) fn cuboid([x, y, z]: [f64; 3], detail: Detail) -> Result<Self, String> {
        Self::prism(
            &[
                [-x / 2.0, -y / 2.0],
                [x / 2.0, -y / 2.0],
                [x / 2.0, y / 2.0],
                [-x / 2.0, y / 2.0],
            ],
            z,
            detail,
        )
    }

    pub(crate) fn lathe(
        profile: &[PlanarPoint],
        requested: usize,
        radial_scale: f64,
        exact_segments: bool,
        detail: Detail,
    ) -> Result<Self, String> {
        if profile.len() < 2 || requested < 3 || radial_scale <= 0.0 {
            return Err("radial solid requires stations and a positive section".into());
        }
        let largest = profile.iter().map(|p| p[1]).fold(0.0, f64::max);
        let segments = detail.lathe_radial(largest, requested, exact_segments);
        let minimum = profile
            .iter()
            .map(|p| p[1])
            .filter(|&r| r > 0.0)
            .fold(f64::INFINITY, f64::min);
        let chord = detail
            .error(0.025)
            .min(minimum * (PI / segments as f64).sin() * 64.0 * radial_scale.min(1.0));
        let stations = profile
            .windows(2)
            .map(|p| {
                ((p[1][0] - p[0][0]).hypot(p[1][1] - p[0][1]) / chord)
                    .ceil()
                    .max(1.0)
            })
            .sum::<f64>()
            + 1.0;
        super::construction_budget(stations * segments as f64 * 2.0)?;
        let profile = subdivide(profile, chord);
        let mut solid = Self::default();
        let mut band = 1;
        let vertex = |row: usize, index: usize| {
            let [y, r] = profile[row];
            let angle = index as f64 / segments as f64 * TAU;
            [angle.cos() * r, y, angle.sin() * r * radial_scale]
        };
        for ring in 0..profile.len() - 1 {
            if ring > 0 {
                let slope = |i: usize| {
                    (profile[i + 1][1] - profile[i][1]).atan2(profile[i + 1][0] - profile[i][0])
                };
                if (slope(ring) - slope(ring - 1)).abs() > PI / 3.0 {
                    band += 1;
                }
            }
            for segment in 0..segments {
                let next = (segment + 1) % segments;
                let [a, b, c, d] = [
                    vertex(ring, segment),
                    vertex(ring, next),
                    vertex(ring + 1, next),
                    vertex(ring + 1, segment),
                ];
                let surface = if exact_segments && segments <= 8 {
                    0
                } else {
                    band
                };
                if profile[ring][1] > 0.0 {
                    solid.triangle(a, c, b, surface);
                }
                if profile[ring + 1][1] > 0.0 {
                    solid.triangle(a, d, c, surface);
                }
            }
        }
        for (row, reverse) in [(0, true), (profile.len() - 1, false)] {
            if profile[row][1] == 0.0 {
                continue;
            }
            for segment in 0..segments {
                let center = [0.0, profile[row][0], 0.0];
                let a = vertex(row, segment);
                let b = vertex(row, (segment + 1) % segments);
                if reverse {
                    solid.triangle(center, a, b, 0);
                } else {
                    solid.triangle(center, b, a, 0);
                }
            }
        }
        Ok(solid.positive())
    }

    pub(crate) fn hollow_socket(
        profile: &[PlanarPoint],
        inner_radii: &[f64],
        segments_override: Option<usize>,
        detail: Detail,
    ) -> Self {
        let largest = profile.iter().map(|p| p[1]).fold(0.0, f64::max);
        let segments =
            segments_override.map_or_else(|| detail.radial(largest, 16), |n| detail.samples(n, 6));
        Self::hollow_profile(profile, inner_radii, segments)
    }

    /// Annular profile using an explicitly shared receiving polygon section.
    pub(crate) fn hollow_profile(
        profile: &[PlanarPoint],
        inner_radii: &[f64],
        segments: usize,
    ) -> Self {
        let vertex = |row: usize, index: usize, inner: bool| {
            let radius = if inner {
                inner_radii[row]
            } else {
                profile[row][1]
            };
            let angle = index as f64 / segments as f64 * TAU;
            [angle.cos() * radius, profile[row][0], angle.sin() * radius]
        };
        let mut solid = Self::default();
        for inner in [false, true] {
            for row in 0..profile.len() - 1 {
                for segment in 0..segments {
                    let next = (segment + 1) % segments;
                    let [a, b, c, d] = [
                        vertex(row, segment, inner),
                        vertex(row, next, inner),
                        vertex(row + 1, next, inner),
                        vertex(row + 1, segment, inner),
                    ];
                    if inner {
                        solid.quad(a, b, c, d, 2);
                    } else {
                        solid.quad(a, d, c, b, 1);
                    }
                }
            }
        }
        for row in [0, profile.len() - 1] {
            for segment in 0..segments {
                let next = (segment + 1) % segments;
                let [a, b, c, d] = [
                    vertex(row, segment, false),
                    vertex(row, next, false),
                    vertex(row, next, true),
                    vertex(row, segment, true),
                ];
                if row == 0 {
                    solid.quad(a, b, c, d, 0);
                } else {
                    solid.quad(a, d, c, b, 0);
                }
            }
        }
        solid.positive()
    }

    pub(crate) fn shaped_plate(
        outline: &[PlanarPoint],
        thickness_at: impl Fn(f64, f64) -> f64,
        detail: Detail,
    ) -> Result<Self, String> {
        let minimum = outline
            .iter()
            .map(|&[x, y]| thickness_at(x, y))
            .fold(f64::INFINITY, f64::min);
        let (region, _) = planar_region(outline, detail.error(0.03).min(minimum * 40.0))?;
        let vertex = |i: usize, side: f64| {
            let [x, y] = region.points[i];
            [x, y, side * thickness_at(x, y) / 2.0]
        };
        let mut solid = Self::default();
        for [a, b, c] in region.triangles {
            solid.triangle(vertex(a, 1.0), vertex(b, 1.0), vertex(c, 1.0), 1);
            solid.triangle(vertex(a, -1.0), vertex(c, -1.0), vertex(b, -1.0), 2);
        }
        for i in 0..region.boundary.len() {
            let a = region.boundary[i];
            let b = region.boundary[(i + 1) % region.boundary.len()];
            solid.quad(
                vertex(a, -1.0),
                vertex(b, -1.0),
                vertex(b, 1.0),
                vertex(a, 1.0),
                0,
            );
        }
        Ok(solid.positive())
    }

    pub(crate) fn rounded_plate(
        outline: &[PlanarPoint],
        thickness: f64,
        bevel_fraction: f64,
    ) -> Result<Self, String> {
        let region = Region::triangulate(outline, false)?;
        let center_y = (region
            .points
            .iter()
            .map(|p| p[1])
            .fold(f64::INFINITY, f64::min)
            + region
                .points
                .iter()
                .map(|p| p[1])
                .fold(f64::NEG_INFINITY, f64::max))
            / 2.0;
        let vertex = |i: usize, side: f64, inset: bool| {
            let scale = if inset { 1.0 - bevel_fraction } else { 1.0 };
            [
                region.points[i][0] * scale,
                center_y + (region.points[i][1] - center_y) * scale,
                side * thickness * if inset { 0.5 } else { 0.28 },
            ]
        };
        let mut solid = Self::default();
        for side in [-1.0, 1.0] {
            for &[a, b, c] in &region.triangles {
                let [a, b, c] = [
                    vertex(a, side, true),
                    vertex(b, side, true),
                    vertex(c, side, true),
                ];
                if side < 0.0 {
                    solid.triangle(c, b, a, 0);
                } else {
                    solid.triangle(a, b, c, 0);
                }
            }
            for i in 0..region.points.len() {
                let j = (i + 1) % region.points.len();
                let [a, b, c, d] = [
                    vertex(i, side, false),
                    vertex(j, side, false),
                    vertex(j, side, true),
                    vertex(i, side, true),
                ];
                if side > 0.0 {
                    solid.quad(a, b, c, d, 1);
                } else {
                    solid.quad(a, d, c, b, 2);
                }
            }
        }
        for i in 0..region.points.len() {
            let j = (i + 1) % region.points.len();
            solid.quad(
                vertex(i, -1.0, false),
                vertex(j, -1.0, false),
                vertex(j, 1.0, false),
                vertex(i, 1.0, false),
                3,
            );
        }
        Ok(solid.positive())
    }
}
