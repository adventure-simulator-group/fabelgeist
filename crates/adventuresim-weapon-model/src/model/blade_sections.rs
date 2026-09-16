//! One transverse section and closed loft authority for authored blades.
use super::*;
use std::f64::consts::TAU;

#[derive(Clone, Copy)]
pub(super) enum BladeSampling {
    Section,
    Loft(usize),
}
pub(super) fn blade(
    p: BladeProfile<'_>,
    sampling: BladeSampling,
    detail: Detail,
) -> Result<Solid, String> {
    p.validate().map_err(|e| e.to_string())?;
    let point = p.point_curve()?;
    let minimum_envelope = blade_reduction::envelope_floor(&p, detail)?;
    let ring = |y, minimum_envelope| {
        let [w, d] = p.dimensions(y, point.as_ref());
        let center = p.center(y, w);
        section(&p, y, w, d, detail, minimum_envelope)
            .into_iter()
            .map(|[x, z]| [center + x, y, z])
            .collect::<Vec<_>>()
    };
    let stations = if p.fuller.is_some() || p.point.is_some() {
        feature_stations(&p, |y| ring(y, 0.0), detail)?
    } else {
        let mut stations = baseline_stations(p.length, |y| ring(y, 0.0), sampling, detail)?;
        if p.ricasso > 0.0 {
            stations.push(p.ricasso);
        }
        stations.sort_by(f64::total_cmp);
        stations.dedup();
        stations
    };
    let rings: Vec<_> = stations
        .iter()
        .copied()
        .map(|y| ring(y, minimum_envelope))
        .collect();
    construction_budget((rings.len() * rings[0].len() * 2) as f64)?;
    let smooth = if matches!(sampling, BladeSampling::Section) {
        0
    } else {
        1
    };
    let mut solid = Solid::default();
    for (index, rows) in rings.windows(2).enumerate() {
        for side in 0..rows[0].len() {
            let next = (side + 1) % rows[0].len();
            let quad = [rows[0][side], rows[0][next], rows[1][next], rows[1][side]];
            if let Some(f) = p.fuller {
                let [a, b] = [stations[index], stations[index + 1]];
                if [a, b].into_iter().any(|y| {
                    let q = f.envelope(y);
                    q > 0.0 && q < minimum_envelope
                }) {
                    blade_reduction::check([a, b], side, &quad, |y| ring(y, 0.0), detail)?;
                }
            }
            face(
                &mut solid,
                quad,
                if p.section == BladeCrossSection::Recessed {
                    side as u32 + 1
                } else {
                    smooth
                },
            )?;
        }
    }
    for (ring, reverse) in [
        (rings.first().unwrap(), true),
        (rings.last().unwrap(), false),
    ] {
        let mut outline: Vec<_> = ring.iter().map(|p| [p[0], p[2]]).collect();
        outline.dedup();
        if outline.len() > 1 && outline.first() == outline.last() {
            outline.pop();
        }
        if outline.len() == 1 {
            continue;
        }
        let cap = Region::triangulate(
            &outline,
            p.section == BladeCrossSection::Recessed || matches!(sampling, BladeSampling::Section),
        )?;
        for [a, b, c] in cap.triangles {
            let point = |i: usize| [cap.points[i][0], ring[0][1], cap.points[i][1]];
            if reverse {
                solid.triangle(point(a), point(b), point(c), 0);
            } else {
                solid.triangle(point(a), point(c), point(b), 0);
            }
        }
    }
    Ok(solid.positive())
}

// Refine the complete section together. Unrelated uniform rows inside the
// vanishing groove tail can create features below float32 resolution.
fn feature_stations(
    p: &BladeProfile<'_>,
    ring: impl Fn(f64) -> Vec<Point>,
    detail: Detail,
) -> Result<Vec<f64>, String> {
    let mut features = vec![0.0, p.length];
    if p.ricasso > 0.0 {
        features.push(p.ricasso);
    }
    if let Some(start) = p.point_start() {
        features.push(start);
    }
    if let Some(f) = p.fuller {
        features.extend([
            f.start.get(),
            f.start.get() + f.entry_length.get(),
            f.end.get() - f.exit_length.get(),
            f.end.get(),
        ]);
    }
    features.sort_by(f64::total_cmp);
    features.dedup();
    let mut stations = vec![0.0];
    for pair in features.windows(2) {
        refine_sections(pair[0], pair[1], &ring, detail, 0, &mut stations)?;
    }
    Ok(stations)
}
fn refine_sections(
    a: f64,
    b: f64,
    ring: &impl Fn(f64) -> Vec<Point>,
    detail: Detail,
    depth: usize,
    out: &mut Vec<f64>,
) -> Result<(), String> {
    let (left, right) = (ring(a), ring(b));
    let mut deviation = 0.0_f64;
    for t in [0.25, 0.5, 0.75] {
        let sampled = ring(a + (b - a) * t);
        for i in 0..left.len() {
            let line = sub(right[i], left[i]);
            let length2 = dot(line, line);
            let u = if length2 > 0.0 {
                (dot(sub(sampled[i], left[i]), line) / length2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            deviation = deviation.max(magnitude(sub(sampled[i], add(left[i], mul(line, u)))));
        }
    }
    if (b - a) > detail.error(0.015)
        || deviation > detail.error(blade_reduction::BLADE_SURFACE_ERROR) / 2.0
    {
        if depth >= f64::MANTISSA_DIGITS as usize {
            return Err("blade sampling exceeds its surface-error budget".into());
        }
        let mid = (a + b) / 2.0;
        refine_sections(a, mid, ring, detail, depth + 1, out)?;
        refine_sections(mid, b, ring, detail, depth + 1, out)?;
    } else {
        construction_budget(((out.len() + 1) * left.len() * 2) as f64)?;
        out.push(b);
    }
    Ok(())
}

fn baseline_stations(
    length: f64,
    ring: impl Fn(f64) -> Vec<Point>,
    sampling: BladeSampling,
    detail: Detail,
) -> Result<Vec<f64>, String> {
    Ok(match sampling {
        BladeSampling::Loft(n) => (0..=n)
            .map(|i| length * i as f64 / n as f64)
            .collect::<Vec<_>>(),
        BladeSampling::Section => {
            let mut ys = vec![length];
            loop {
                let y = *ys.last().unwrap();
                if y <= 0.0 {
                    break;
                }
                let profile = ring(y);
                let edge = (0..profile.len())
                    .map(|i| magnitude(sub(profile[i], profile[(i + 1) % profile.len()])))
                    .filter(|&d| d > 0.0)
                    .fold(f64::INFINITY, f64::min);
                let step = detail.error(0.03).min(edge * 40.0);
                if !step.is_finite() || step <= 0.0 {
                    return Err("blade section has no area".into());
                }
                construction_budget(((ys.len() + 1) * profile.len() * 2) as f64)?;
                ys.push(if y <= step * (1.0 + 1e-8) {
                    0.0
                } else {
                    y - step
                });
            }
            ys.reverse();
            ys
        }
    })
}

fn face(solid: &mut Solid, quad: [Point; 4], surface: u32) -> Result<(), String> {
    let mut vertices = quad.to_vec();
    vertices.dedup();
    if vertices.first() == vertices.last() {
        vertices.pop();
    }
    if vertices.len() < 3 {
        return Ok(());
    }
    if vertices.iter().flatten().any(|x| !x.is_finite()) {
        return Err("blade section contains a nonfinite vertex".into());
    }
    for i in 1..vertices.len() - 1 {
        solid.triangle(vertices[0], vertices[i], vertices[i + 1], surface);
    }
    Ok(())
}

fn section(
    p: &BladeProfile<'_>,
    y: f64,
    w: f64,
    d: f64,
    detail: Detail,
    minimum_envelope: f64,
) -> Vec<PlanarPoint> {
    match p.section {
        BladeCrossSection::Diamond => vec![[-w, 0.0], [0.0, d], [w, 0.0], [0.0, -d]],
        BladeCrossSection::Fullered => vec![
            [-w, 0.0],
            [-w * 0.72, d],
            [-w * 0.28, d * 0.32],
            [0.0, d * 0.22],
            [w * 0.28, d * 0.32],
            [w * 0.72, d],
            [w, 0.0],
            [0.0, -d],
        ],
        BladeCrossSection::Hexagonal => vec![
            [-w, 0.0],
            [-w * 0.72, d],
            [w * 0.72, d],
            [w, 0.0],
            [w * 0.72, -d],
            [-w * 0.72, -d],
        ],
        BladeCrossSection::Lenticular => {
            // Curved faces remain real geometry rather than a diamond alias.
            let n = detail.samples(24, 12).div_ceil(4) * 4;
            (0..n)
                .map(|i| {
                    let a = TAU * i as f64 / n as f64;
                    [-w * a.cos(), d * a.sin()]
                })
                .collect()
        }
        BladeCrossSection::Recessed => {
            let f = p.fuller.unwrap();
            let q = f.envelope(y);
            let q = if q < minimum_envelope { 0.0 } else { q };
            let mouth = f.mouth_width.get() * q / 2.0;
            let floor = mouth * f.floor_width_ratio.get();
            let recess = f.depth.get() * q * q;
            let flat = w * (1.0 - f.bevel_width_ratio.get());
            let mut result = vec![[-w, 0.0]];
            for front in [true, false] {
                let sign = if front { 1.0 } else { -1.0 };
                let cut = if f.faces == FullerFaces::Both
                    || (front && f.faces == FullerFaces::Front)
                    || (!front && f.faces == FullerFaces::Back)
                {
                    recess
                } else {
                    0.0
                };
                let mut face = vec![
                    [-flat, sign * d],
                    [-mouth, sign * d],
                    [-floor, sign * (d - cut)],
                    [floor, sign * (d - cut)],
                    [mouth, sign * d],
                    [flat, sign * d],
                ];
                if !front {
                    face.reverse();
                }
                result.extend(face);
                result.push([if front { w } else { -w }, 0.0]);
            }
            result.pop();
            result
        }
    }
}
