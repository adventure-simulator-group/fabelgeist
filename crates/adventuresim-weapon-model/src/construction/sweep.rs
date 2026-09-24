//! Parallel-transport swept sections with bounded path sampling.
mod path;
use super::*;
use path::{SampledPath, TransportFrames};
use std::f64::consts::{PI, TAU};

#[cfg(test)]
#[path = "sweep_tests.rs"]
mod tests;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Section {
    #[default]
    Round,
    Oval,
    Diamond,
    Flat,
    Triangular,
    DShape,
    Beveled,
}

#[derive(Clone, Debug)]
pub(crate) struct Sweep {
    pub(crate) section: Section,
    pub(crate) width: f64,
    pub(crate) depth: f64,
    pub(crate) radial_segments: usize,
    pub(crate) twist: f64,
    pub(crate) tip_scale: f64,
    pub(crate) terminal_swell: f64,
    pub(crate) centered_taper: bool,
    pub(crate) fit_bends: bool,
    pub(crate) ring_scales: Option<Vec<f64>>,
}
impl Default for Sweep {
    fn default() -> Self {
        Self {
            section: Section::Round,
            width: 0.012,
            depth: 0.012,
            radial_segments: 12,
            twist: 0.0,
            tip_scale: 1.0,
            terminal_swell: 0.0,
            centered_taper: false,
            fit_bends: false,
            ring_scales: None,
        }
    }
}

pub(crate) fn tube_segments(radius: f64, requested: usize, detail: Detail) -> usize {
    let chord = if 0.006 >= radius * 2.0 {
        3.0
    } else {
        (PI / (0.006 / (radius * 2.0)).asin()).ceil()
    };
    let sagitta = if 0.0003 >= radius {
        3.0
    } else {
        (PI / (1.0 - 0.0003 / radius).acos()).ceil()
    };
    detail
        .radial(radius, requested)
        .max((chord / detail.error(1.0)).ceil() as usize)
        .max((sagitta / detail.error(1.0).sqrt()).ceil() as usize)
}

impl Section {
    fn outline(
        self,
        width: f64,
        mut depth: f64,
        requested: usize,
        detail: Detail,
    ) -> Vec<PlanarPoint> {
        if self == Self::Round {
            depth = width;
        }
        match self {
            Self::Beveled => {
                const CORNER_CUT_FRACTION: f64 = 0.25;
                let x = width / 2.0;
                let y = depth / 2.0;
                let cut_x = width * CORNER_CUT_FRACTION;
                let cut_y = depth * CORNER_CUT_FRACTION;
                vec![
                    [-x + cut_x, -y],
                    [x - cut_x, -y],
                    [x, -y + cut_y],
                    [x, y - cut_y],
                    [x - cut_x, y],
                    [-x + cut_x, y],
                    [-x, y - cut_y],
                    [-x, -y + cut_y],
                ]
            }
            Self::Round | Self::Oval => {
                let count = tube_segments(width.max(depth) / 2.0, requested, detail);
                (0..count)
                    .map(|i| {
                        let angle = i as f64 / count as f64 * TAU;
                        [angle.cos() * width / 2.0, angle.sin() * depth / 2.0]
                    })
                    .collect()
            }
            Self::Diamond => vec![
                [width / 2.0, 0.0],
                [0.0, depth / 2.0],
                [-width / 2.0, 0.0],
                [0.0, -depth / 2.0],
            ],
            Self::DShape => {
                let count = tube_segments(width.max(depth) / 2.0, requested, detail).max(8);
                let mut points = vec![[-width / 2.0, -depth / 2.0]];
                points.extend((0..=count).map(|i| {
                    let angle = -PI / 2.0 + i as f64 / count as f64 * PI;
                    [angle.cos() * width / 2.0, angle.sin() * depth / 2.0]
                }));
                points.push([-width / 2.0, depth / 2.0]);
                points
            }
            Self::Triangular => vec![
                [0.0, depth * 0.58],
                [-width / 2.0, -depth * 0.42],
                [width / 2.0, -depth * 0.42],
            ],
            Self::Flat => vec![
                [-width / 2.0, -depth / 2.0],
                [width / 2.0, -depth / 2.0],
                [width / 2.0, depth / 2.0],
                [-width / 2.0, depth / 2.0],
            ],
        }
    }
}

fn rotate_around(vector: Point, axis: Point, angle: f64) -> Point {
    add(
        add(
            mul(vector, angle.cos()),
            mul(cross(axis, vector), angle.sin()),
        ),
        mul(axis, dot(axis, vector) * (1.0 - angle.cos())),
    )
}

fn segment_distance(a: Point, b: Point, c: Point, d: Point) -> f64 {
    let u = sub(b, a);
    let v = sub(d, c);
    let w = sub(a, c);
    let aa = dot(u, u);
    let bb = dot(u, v);
    let cc = dot(v, v);
    let dd = dot(u, w);
    let ee = dot(v, w);
    let denominator = aa * cc - bb * bb;
    let mut s = if denominator > aa * cc * 1e-14 {
        ((bb * ee - cc * dd) / denominator).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut t = if cc > 0.0 { (bb * s + ee) / cc } else { 0.0 };
    if t < 0.0 {
        t = 0.0;
        s = if aa > 0.0 {
            (-dd / aa).clamp(0.0, 1.0)
        } else {
            0.0
        };
    }
    if t > 1.0 {
        t = 1.0;
        s = if aa > 0.0 {
            ((bb - dd) / aa).clamp(0.0, 1.0)
        } else {
            0.0
        };
    }
    magnitude(sub(add(w, mul(u, s)), mul(v, t)))
}

pub(crate) fn bend_scales(points: &[Point], radius: f64, closed: bool) -> Vec<f64> {
    let n = points.len();
    let count = if closed { n - 1 } else { n };
    let mut scales = vec![1.0_f64; n];
    let mut arc = vec![0.0];
    for i in 1..n {
        arc.push(arc[i - 1] + magnitude(sub(points[i], points[i - 1])));
    }
    for i in 0..count {
        if !closed && (i == 0 || i == count - 1) {
            continue;
        }
        let incoming = sub(points[i], points[(i + count - 1) % count]);
        let outgoing = sub(points[(i + 1) % count], points[i]);
        let a = magnitude(incoming);
        let b = magnitude(outgoing);
        let cosine = (dot(incoming, outgoing) / (a * b)).clamp(-1.0, 1.0);
        let tangent_half = ((1.0 - cosine) / (1.0 + cosine).max(1e-12)).sqrt();
        if tangent_half > 1e-8 {
            scales[i] = (0.3 * a.min(b) / (radius * tangent_half)).min(1.0);
        }
    }
    for i in 0..n - 1 {
        for j in i + 2..n - 1 {
            let forward = arc[j] - arc[i + 1];
            let separated = if closed {
                forward.min(arc[n - 1] - arc[j + 1] + arc[i])
            } else {
                forward
            };
            if separated <= radius * 4.0 {
                continue;
            }
            let scale = (segment_distance(points[i], points[i + 1], points[j], points[j + 1])
                * 0.3
                / radius)
                .min(1.0);
            for vertex in [i, i + 1, j, j + 1] {
                scales[vertex] = scales[vertex].min(scale);
            }
        }
    }
    let limits = scales.clone();
    for i in 0..count {
        for j in 0..count {
            let forward = (arc[i] - arc[j]).abs();
            let distance = if closed {
                forward.min(arc[n - 1] - forward)
            } else {
                forward
            };
            scales[i] = scales[i].min(limits[j] + distance / (radius * 3.0));
        }
    }
    if closed {
        scales[n - 1] = scales[0].min(scales[n - 1]);
        scales[0] = scales[n - 1];
    }
    scales
}

/// A single authored tube path may close, but may not cross itself.
pub(crate) fn validate_simple_path(input: &[Point]) -> Result<(), String> {
    if input.len() < 2 {
        return Err("member needs at least two stations".into());
    }
    let closed = magnitude(sub(input[0], *input.last().unwrap())) < 1e-8;
    for i in 0..input.len() - 1 {
        for j in i + 2..input.len() - 1 {
            if closed && i == 0 && j == input.len() - 2 {
                continue;
            }
            if segment_distance(input[i], input[i + 1], input[j], input[j + 1]) < 1e-10 {
                return Err("member centerline intersects itself".into());
            }
        }
    }
    Ok(())
}

impl Solid {
    pub(crate) fn sweep(input: &[Point], sweep: &Sweep, detail: Detail) -> Result<Self, String> {
        if input.len() < 2 || sweep.width <= 0.0 || sweep.depth <= 0.0 || sweep.tip_scale <= 0.0 {
            return Err("sweep needs a positive section and at least two points".into());
        }
        if input.windows(2).any(|p| magnitude(sub(p[1], p[0])) < 1e-12) {
            return Err("member has repeated adjacent stations".into());
        }
        let closed = magnitude(sub(input[0], *input.last().unwrap())) < 1e-8;
        if closed && (sweep.twist % 360.0).abs() > 1e-8 {
            return Err("closed member needs whole-turn section twist".into());
        }
        let outline =
            sweep
                .section
                .outline(sweep.width, sweep.depth, sweep.radial_segments, detail);
        let SampledPath {
            points,
            scales,
            progress,
        } = SampledPath::new(input, sweep, outline.len(), detail, closed)?;
        let n = points.len();
        let TransportFrames {
            tangents,
            normals: frames,
        } = TransportFrames::along(&points, closed)?;
        let vertex = |mut row: usize, side: usize| {
            if closed && row == n - 1 {
                row = 0;
            }
            let normal = frames[row];
            let binormal = normalize(cross(tangents[row], normal));
            let twist = (sweep.twist * progress[row]).to_radians();
            let t = if sweep.centered_taper {
                (progress[row] * 2.0 - 1.0).abs()
            } else {
                progress[row]
            };
            let taper = (1.0 + (sweep.tip_scale - 1.0) * t)
                * (1.0 + sweep.terminal_swell * ((t - 0.72) / 0.28).max(0.0).powi(2));
            let [u, v] = outline[side].map(|value| value * taper * scales[row]);
            add(
                add(points[row], mul(normal, u * twist.cos() - v * twist.sin())),
                mul(binormal, u * twist.sin() + v * twist.cos()),
            )
        };
        let mut solid = Self::default();
        for row in 0..n - 1 {
            for side in 0..outline.len() {
                let next = (side + 1) % outline.len();
                let group = match sweep.section {
                    Section::Round | Section::Oval => 1,
                    Section::DShape => {
                        if side == outline.len() - 1 {
                            2
                        } else {
                            1
                        }
                    }
                    _ => side as u32 + 1,
                };
                let corners = [
                    vertex(row, side),
                    vertex(row, next),
                    vertex(row + 1, next),
                    vertex(row + 1, side),
                ];
                if sweep.twist == 0.0 {
                    solid.quad(corners[0], corners[1], corners[2], corners[3], group);
                } else {
                    // A fixed diagonal of a twisted face adds a handedness-
                    // dependent material wedge. Its bilinear center treats
                    // both diagonals equally and preserves mirrored volumes.
                    let center = mul(corners.into_iter().fold([0.0; 3], add), 0.25);
                    for side in 0..4 {
                        solid.triangle(corners[side], corners[(side + 1) % 4], center, group);
                    }
                }
            }
        }
        if !closed {
            for (row, reverse) in [(0, true), (n - 1, false)] {
                for side in 0..outline.len() {
                    let a = vertex(row, side);
                    let b = vertex(row, (side + 1) % outline.len());
                    if reverse {
                        solid.triangle(points[row], b, a, 0);
                    } else {
                        solid.triangle(points[row], a, b, 0);
                    }
                }
            }
        }
        Ok(solid.positive())
    }
}
