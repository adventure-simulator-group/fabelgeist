//! Boundary-aware fields and ordinaries, all in a normalized square chart.
use crate::{artwork::Path, document::*};
pub(crate) fn wave(t: f32, b: Boundary) -> f32 {
    const PERIODS: f32 = 6.0;
    const AMPLITUDE: f32 = 0.018;
    let phase = (t * PERIODS).rem_euclid(1.0);
    AMPLITUDE
        * match b {
            Boundary::Straight => 0.0,
            Boundary::Wavy => (t * PERIODS * std::f32::consts::TAU).sin(),
            Boundary::Indented => 1.0 - 4.0 * (phase - 0.5).abs(),
            Boundary::Embattled => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
        }
}
pub(crate) fn division_path(d: Division, b: Boundary) -> Path {
    const SEGMENTS: usize = 192;
    let mut path = Path::new(0.0, 0.0).line(1.0, 0.0);
    match d {
        Division::Pale => {
            path = Path::new(0.0, 0.0).line(0.5, 0.0);
            for i in 0..=SEGMENTS {
                let t = i as f32 / SEGMENTS as f32;
                path = path.line(0.5 + wave(t, b), t);
            }
            path = path.line(0.0, 1.0);
        }
        Division::Fess | Division::Bend | Division::BendSinister | Division::Chevron => {
            for i in (0..=SEGMENTS).rev() {
                let t = i as f32 / SEGMENTS as f32;
                let y = match d {
                    Division::Fess => 0.5,
                    Division::Bend => t,
                    Division::BendSinister => 1.0 - t,
                    Division::Chevron => 0.22 + (t - 0.5).abs() * 1.1,
                    _ => unreachable!(),
                };
                path = path.line(t, y + wave(t, b));
            }
        }
        Division::Saltire => {
            for i in (0..=SEGMENTS).rev() {
                let t = i as f32 / SEGMENTS as f32;
                path = path.line(t, t.min(1.0 - t) + wave(t, b));
            }
            path = path.close();
            return path.clone().joined(path.mapped(|[x, y]| [x, 1.0 - y]));
        }
    }
    path.close()
}
pub(crate) fn patterns(p: Pattern, repeats: u8) -> Vec<Path> {
    let n = i32::from(repeats);
    let pitch = 1.0 / n as f32;
    let mut paths = Vec::new();
    match p {
        Pattern::Stripes => {
            for i in (0..n).step_by(2) {
                paths.push(Path::rect(i as f32 * pitch, 0.0, pitch, 1.0));
            }
        }
        Pattern::Checks => {
            for y in 0..n {
                for x in 0..n {
                    if (x + y) % 2 == 0 {
                        paths.push(Path::rect(x as f32 * pitch, y as f32 * pitch, pitch, pitch));
                    }
                }
            }
        }
        Pattern::Lozenges => {
            for y in -1..=n * 2 {
                for x in -1..=n {
                    let cx = x as f32 * pitch;
                    let cy = y as f32 * pitch * 2.0;
                    paths.push(
                        Path::new(cx, cy - pitch)
                            .line(cx + pitch * 0.5, cy)
                            .line(cx, cy + pitch)
                            .line(cx - pitch * 0.5, cy)
                            .close(),
                    );
                }
            }
        }
    }
    paths
}
pub(crate) fn ordinary(o: &Ordinary) -> Vec<Path> {
    let w = o.width.0;
    match o.kind {
        OrdinaryKind::Pale => vec![band(w, o.boundary).mapped(|[x, y]| [y, x])],
        OrdinaryKind::Fess => vec![band(w, o.boundary)],
        OrdinaryKind::Bend => vec![diagonal(w, o.boundary, false)],
        OrdinaryKind::BendSinister => vec![diagonal(w, o.boundary, true)],
        OrdinaryKind::Cross => vec![
            band(w, o.boundary),
            band(w, o.boundary).mapped(|[x, y]| [y, x]),
        ],
        OrdinaryKind::Saltire => vec![
            diagonal(w, o.boundary, false),
            diagonal(w, o.boundary, true),
        ],
        OrdinaryKind::Chief => {
            vec![division_path(Division::Fess, o.boundary).mapped(|[x, y]| [x, y * w * 2.0])]
        }
        OrdinaryKind::Chevron => {
            vec![band(w, o.boundary).mapped(|[x, y]| [x, y - 0.28 + (x - 0.5).abs() * 1.1])]
        }
        OrdinaryKind::Bordure => vec![
            Path::rect(0.0, 0.0, 1.0, w),
            Path::rect(0.0, 1.0 - w, 1.0, w),
            Path::rect(0.0, w, w, 1.0 - 2.0 * w),
            Path::rect(1.0 - w, w, w, 1.0 - 2.0 * w),
        ],
    }
}
fn band(width: f32, b: Boundary) -> Path {
    const STEPS: usize = 192;
    let mut p = Path::new(0.0, 0.5 - width * 0.5);
    for i in 0..=STEPS {
        let t = i as f32 / STEPS as f32;
        p = p.line(t, 0.5 - width * 0.5 + wave(t, b));
    }
    for i in (0..=STEPS).rev() {
        let t = i as f32 / STEPS as f32;
        p = p.line(t, 0.5 + width * 0.5 + wave(t, b));
    }
    p.close()
}
fn diagonal(w: f32, b: Boundary, reverse: bool) -> Path {
    band(w, b).mapped(|[x, y]| [x, if reverse { 1.0 - x } else { x } + y - 0.5])
}
