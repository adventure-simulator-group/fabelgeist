//! Leaf blade sections and their longitudinal outline.
use super::*;

pub(super) fn spear(p: &SpearParameters, detail: Detail) -> Result<Solid, String> {
    if p.socket.is_some() {
        return super::spear_socket::construct(p, detail);
    }
    let length = p.length.get();
    let thickness = p.thickness.get();
    let belly = p.belly_position.or(p.shoulder).map_or(0.18, Ratio::get);
    let samples = detail.samples(p.samples.map_or(12, |n| n.0 as usize), 4);
    let mut stops: Vec<_> = (0..samples)
        .map(|i| i as f64 / samples as f64)
        .chain([0.0, belly])
        .collect();
    if p.shoulder_roundness
        .is_some_and(|roundness| roundness.get() > 0.0)
    {
        stops.extend(outline_stations(p, detail).into_iter().filter(|t| *t < 1.0));
    }
    stops.sort_by(f64::total_cmp);
    stops.dedup();
    if p.section == Some(SpearSection::Flat) {
        let mut outline: Vec<_> = stops
            .iter()
            .map(|&t| {
                let half = half_width(p, t);
                [-half, t * length]
            })
            .collect();
        outline.push([0.0, length]);
        outline.extend(stops.iter().rev().map(|&t| {
            let half = half_width(p, t);
            [half, t * length]
        }));
        return Solid::prism(&outline, thickness, detail);
    }
    let ring = |t: f64| {
        let half = half_width(p, t);
        let depth = thickness / 2.0 * (1.0 - t * 0.9);
        [
            [-half, t * length, 0.0],
            [0.0, t * length, depth],
            [half, t * length, 0.0],
            [0.0, t * length, -depth],
        ]
    };
    let mut solid = Solid::default();
    for row in 0..stops.len() {
        let a = ring(stops[row]);
        for side in 0..4 {
            let next = (side + 1) % 4;
            if row + 1 == stops.len() {
                solid.triangle(a[side], [0.0, length, 0.0], a[next], side as u32 + 1);
            } else {
                let b = ring(stops[row + 1]);
                solid.triangle(a[side], b[next], a[next], side as u32 + 1);
                solid.triangle(a[side], b[side], b[next], side as u32 + 1);
            }
        }
    }
    let base = ring(0.0);
    for side in 0..4 {
        solid.triangle([0.0; 3], base[side], base[(side + 1) % 4], 0);
    }
    Ok(solid.positive())
}

/// Roundness alters tangents without moving the root, maximum-width station or tip.
pub(super) fn half_width(p: &SpearParameters, t: f64) -> f64 {
    let width = p.width.get();
    let root = p.root_width.map_or(width * 0.4, Metres::get);
    let belly = p.belly_position.or(p.shoulder).map_or(0.18, Ratio::get);
    let acuteness = p.acuteness.map_or(1.0, Ratio::get);
    let roundness = p.shoulder_roundness.map_or(0.0, Ratio::get);
    if t <= belly {
        let u = t / belly;
        let original = u.powf(0.8);
        let rounded = u * u * (3.0 - 2.0 * u);
        (root + (width - root) * (original + roundness * (rounded - original))) / 2.0
    } else {
        let u = (t - belly) / (1.0 - belly);
        let rounding_span = roundness * belly.min(1.0 - belly);
        let u = if u < rounding_span {
            let progress = u / rounding_span;
            rounding_span * progress * progress * (2.0 - progress)
        } else {
            u
        };
        // Retain the existing arithmetic when no rounding was requested.
        let taper = if roundness == 0.0 {
            (1.0 - t) / (1.0 - belly)
        } else {
            1.0 - u
        };
        width * taper.powf(acuteness) / 2.0
    }
}

/// Physical chord and sagitta limits preserve the leaf at the lowest display LOD.
pub(super) fn outline_stations(p: &SpearParameters, detail: Detail) -> Vec<f64> {
    let belly = p.belly_position.or(p.shoulder).map_or(0.18, Ratio::get);
    let quality = CurveQuality {
        minimum_segments: p.samples.map_or(12, |n| n.0 as usize),
        max_chord: p.length.get() / 16.0,
        max_deviation: p.width.get() / 800.0,
    };
    [[0.0, belly], [belly, 1.0]]
        .into_iter()
        .flat_map(|[a, b]| {
            adaptive_curve(
                |u| {
                    let t = a + (b - a) * u;
                    [half_width(p, t), t * p.length.get()]
                },
                quality,
                detail,
            )
            .into_iter()
            .map(|point| point[1] / p.length.get())
        })
        .collect()
}
