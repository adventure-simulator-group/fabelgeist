//! Shared spatial attachment chart for the German drawing and its painted marks.
//! Every contour uses the same continuous warp: fills, lines and highlights
//! remain attached even where the source combines anatomy into a single path.
use crate::{artwork::Path, document::DrawingStyle};

const HIP: [f32; 2] = [0.62, 0.62];
const NECK: [f32; 2] = [0.55, 0.29];
const HEAD: [f32; 2] = [0.55, 0.12];
const TAIL_ROOT: [f32; 2] = [0.72, 0.56];
const PAWS: [[f32; 2]; 4] = [[0.26, 0.11], [0.11, 0.49], [0.30, 0.78], [0.70, 0.92]];

pub(super) fn deform(q: [f32; 2], style: &DrawingStyle) -> [f32; 2] {
    let d = &style.lion;
    let mut out = q;
    for (root, tip, reach, radius) in [
        ([0.46, 0.27], PAWS[0], d.foreleg_reach.0, [0.25, 0.22]),
        ([0.41, 0.36], PAWS[1], d.foreleg_reach.0, [0.30, 0.22]),
        (HIP, PAWS[2], d.hindleg_spread.0, [0.23, 0.23]),
        (HIP, PAWS[3], d.hindleg_spread.0, [0.19, 0.25]),
    ] {
        let weight = influence(q, tip, radius);
        out[0] += (q[0] - root[0]) * (reach - 1.0) * weight * 0.55;
        out[1] += (q[1] - root[1]) * (reach - 1.0) * weight * 0.20;
    }
    for tip in PAWS {
        let weight = influence(q, tip, [0.18, 0.15]);
        for i in 0..2 {
            out[i] += (q[i] - tip[i]) * (d.paw_size.0 - 1.0) * weight;
        }
    }
    let face = influence(q, HEAD, [0.23, 0.24]);
    let mane = influence(q, [0.48, 0.35], [0.28, 0.25]);
    for i in 0..2 {
        out[i] += (q[i] - HEAD[i]) * (d.head_size.0 - 1.0) * face;
    }
    out[0] += (q[0] - NECK[0]) * (d.mane_fullness.0 - 1.0) * mane;
    out[1] += (q[1] - NECK[1]) * (d.mane_fullness.0 - 1.0) * mane * 0.3;
    let tail = smooth((q[0] - 0.66) / 0.08) * smooth((TAIL_ROOT[1] - q[1]) / 0.4);
    let curled = rotate(q, TAIL_ROOT, (d.tail_curl.0 - 1.0) * 0.65 * tail);
    for i in 0..2 {
        out[i] += curled[i] - q[i];
    }
    let waist = (1.0 - ((q[1] - 0.54) / 0.30).powi(2)).max(0.0);
    out[0] += (q[0] - HIP[0]) * (d.body_width.0 - 1.0) * waist * 0.65;
    out[0] += (d.spine_arch.0 - 1.0) * 0.12 * waist;
    out[1] += style.asymmetry.0 * (q[0] - 0.5) * 0.04;
    out
}
/// Isolate the tail from the compound contour at its narrow root. This clip is
/// transformed with the drawing, including facing and heraldic composition.
pub(super) fn tail_clip() -> Path {
    Path::new(230.0, -10.0)
        .line(350.0, -10.0)
        .line(350.0, 226.0)
        .line(248.0, 226.0)
        .line(226.0, 200.0)
        .line(226.0, 180.0)
        .line(230.0, 140.0)
        .close()
        .mapped(|q| std::array::from_fn(|i| q[i] / super::source::SOURCE_SIZE[i]))
}
pub(super) fn second_tail(q: [f32; 2]) -> [f32; 2] {
    let q = std::array::from_fn(|i| TAIL_ROOT[i] + (q[i] - TAIL_ROOT[i]) * 0.82);
    rotate(q, TAIL_ROOT, -0.28)
}
fn influence(q: [f32; 2], center: [f32; 2], radius: [f32; 2]) -> f32 {
    smooth(1.0 - ((q[0] - center[0]) / radius[0]).hypot((q[1] - center[1]) / radius[1]))
}
fn rotate(q: [f32; 2], root: [f32; 2], angle: f32) -> [f32; 2] {
    if angle == 0.0 {
        return q;
    }
    let (s, c) = angle.sin_cos();
    let x = q[0] - root[0];
    let y = q[1] - root[1];
    [root[0] + x * c - y * s, root[1] + x * s + y * c]
}
fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn neutral_anatomy_preserves_the_original_contours() {
        let mut style = DrawingStyle::default();
        style.asymmetry.0 = 0.0;
        for path in super::super::source::paths() {
            let p = super::super::source::convert(path.data(), path.abs_transform());
            assert_eq!(p.svg(), p.mapped(|q| deform(q, &style)).svg());
        }
    }
}
