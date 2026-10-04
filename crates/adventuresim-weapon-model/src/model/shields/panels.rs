//! Conforming front/back shield surfaces and their actual rim boundary.
use super::*;

fn ending(shape: &ShapedShieldTopShape, u: f64, depth: f64, roundness: f64) -> f64 {
    let d = u.abs().min(1.0);
    let rounded = 0.5 + 0.5 * (d * PI).cos();
    match shape {
        ShapedShieldTopShape::Flat => 0.0,
        ShapedShieldTopShape::Rounded => {
            depth * ((1.0 - d * d).max(0.0).sqrt() * (1.0 - roundness) + rounded * roundness)
        }
        ShapedShieldTopShape::DoublePeak => {
            let distance = ((u.abs() - 0.48).abs() / 0.52).min(1.0);
            depth
                * ((1.0 - distance) * (1.0 - roundness)
                    + (0.5 + 0.5 * (distance * PI).cos()) * roundness)
                    .max(0.0)
        }
        ShapedShieldTopShape::SinglePeak => {
            depth * ((1.0 - d) * (1.0 - roundness) + rounded * roundness)
        }
    }
}
fn resolution(p: &ShapedShieldParameters, detail: Detail) -> usize {
    p.outline_resolution(detail)
}
impl Shield<'_> {
    pub(super) fn outline(&self, detail: Detail) -> Vec<PlanarPoint> {
        match self.panel {
            Panel::Round(p) => {
                let segments = detail.radial(p.radius.get(), p.radial_segments.0 as usize);
                (0..segments)
                    .map(|i| {
                        let angle = i as f64 / segments as f64 * TAU;
                        [angle.cos() * p.radius.get(), angle.sin() * p.radius.get()]
                    })
                    .collect()
            }
            Panel::Shaped(p) => {
                let count = resolution(p, detail);
                let hw = p.width.get() / 2.0;
                let hh = p.height.get() / 2.0;
                let fraction = (p.corner_radius.get() / hw).min(0.45);
                let corner = |u: f64| {
                    if fraction > 0.0 {
                        let t = ((u.abs() - (1.0 - fraction)) / fraction).clamp(0.0, 1.0);
                        p.corner_radius.get() * (1.0 - (1.0 - t * t).max(0.0).sqrt())
                    } else {
                        0.0
                    }
                };
                let bottom = match p.bottom_shape {
                    ShapedShieldBottomShape::Flat => ShapedShieldTopShape::Flat,
                    ShapedShieldBottomShape::Rounded => ShapedShieldTopShape::Rounded,
                    ShapedShieldBottomShape::Point => ShapedShieldTopShape::SinglePeak,
                };
                let mut outline: Vec<_> = (0..=count)
                    .map(|i| {
                        let u = -1.0 + 2.0 * i as f64 / count as f64;
                        [
                            u * hw,
                            hh + ending(&p.top_shape, u, p.top_depth.get(), p.top_roundness.get())
                                - corner(u),
                        ]
                    })
                    .collect();
                outline.extend((0..=count).rev().map(|i| {
                    let u = -1.0 + 2.0 * i as f64 / count as f64;
                    [
                        u * hw * (1.0 - p.side_taper.get()),
                        -hh - ending(&bottom, u, p.bottom_depth.get(), p.bottom_roundness.get())
                            + corner(u) * (1.0 - p.side_taper.get()),
                    ]
                }));
                outline
            }
        }
    }
    pub(super) fn body(&self, detail: Detail) -> Result<(Solid, Vec<PlanarPoint>), String> {
        let outline = self.outline(detail);
        let solid = match self.panel {
            Panel::Round(p) => self.round_body(p, detail),
            Panel::Shaped(p) => {
                let max_edge = (p.width.get().min(p.height.get()) / resolution(p, detail) as f64)
                    .max((signed_area(&outline).abs() / 800.0).sqrt());
                let region = Region::triangulate(&outline, false)?.refine(max_edge);
                let mut solid = Solid::default();
                let point = |index: usize, front: bool| {
                    let [x, y] = region.points[index];
                    [x, y, self.surface([x, y], front)]
                };
                for &[a, b, c] in &region.triangles {
                    solid.triangle(point(a, true), point(b, true), point(c, true), 1);
                    solid.triangle(point(a, false), point(c, false), point(b, false), 2);
                }
                for i in 0..region.boundary.len() {
                    let a = region.boundary[i];
                    let b = region.boundary[(i + 1) % region.boundary.len()];
                    solid.quad(
                        point(a, true),
                        point(a, false),
                        point(b, false),
                        point(b, true),
                        0,
                    );
                }
                return Ok((
                    solid.positive(),
                    region.boundary.iter().map(|&i| region.points[i]).collect(),
                ));
            }
        };
        Ok((solid, outline))
    }
    fn round_body(&self, p: &RoundShieldParameters, detail: Detail) -> Solid {
        let segments = detail.radial(p.radius.get(), p.radial_segments.0 as usize);
        let rings = detail.samples(p.rings.0 as usize, 3);
        let aperture = self.aperture();
        let point = |ring: usize, segment: usize, front: bool| {
            let radius = aperture + (p.radius.get() - aperture) * ring as f64 / rings as f64;
            let angle = segment as f64 / segments as f64 * TAU;
            let x = radius * angle.cos();
            let y = radius * angle.sin();
            [x, y, self.surface([x, y], front)]
        };
        let mut solid = Solid::default();
        for front in [true, false] {
            if aperture == 0.0 {
                for i in 0..segments {
                    let a = point(0, 0, front);
                    let b = point(1, i, front);
                    let c = point(1, (i + 1) % segments, front);
                    if front {
                        solid.triangle(a, b, c, 1);
                    } else {
                        solid.triangle(a, c, b, 2);
                    }
                }
            }
            for ring in if aperture > 0.0 { 0 } else { 1 }..rings {
                for i in 0..segments {
                    let next = (i + 1) % segments;
                    let [a, b, c, d] = [
                        point(ring, i, front),
                        point(ring + 1, i, front),
                        point(ring + 1, next, front),
                        point(ring, next, front),
                    ];
                    if front {
                        solid.quad(a, b, c, d, 1);
                    } else {
                        solid.quad(a, d, c, b, 2);
                    }
                }
            }
        }
        for i in 0..segments {
            let next = (i + 1) % segments;
            solid.quad(
                point(rings, i, true),
                point(rings, i, false),
                point(rings, next, false),
                point(rings, next, true),
                0,
            );
            if aperture > 0.0 {
                solid.quad(
                    point(0, i, true),
                    point(0, next, true),
                    point(0, next, false),
                    point(0, i, false),
                    0,
                );
            }
        }
        solid.positive()
    }
}
