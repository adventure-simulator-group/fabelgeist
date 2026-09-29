//! Bezier evaluation, splitting, arc length and bounding boxes.

use super::poly;
use crate::math::*;

/// De Casteljau evaluation.
pub fn point(b: &[V2], t: f64) -> V2 {
    match b.len() {
        2 => add2(scale2(b[0], 1.0 - t), scale2(b[1], t)),
        3 => {
            let tc = 1.0 - t;
            add2(
                add2(scale2(b[0], tc * tc), scale2(b[1], 2.0 * tc * t)),
                scale2(b[2], t * t),
            )
        }
        4 => {
            let tc = 1.0 - t;
            let mut out = scale2(b[0], tc * tc * tc);
            out = add2(out, scale2(b[1], 3.0 * tc * tc * t));
            out = add2(out, scale2(b[2], 3.0 * tc * t * t));
            add2(out, scale2(b[3], t * t * t))
        }
        n => panic!("bezier::point::ERROR::unsupported degree with {n} points"),
    }
}

pub fn derivative(b: &[V2], t: f64) -> V2 {
    match b.len() {
        2 => sub2(b[1], b[0]),
        3 => scale2(
            add2(
                scale2(sub2(b[1], b[0]), 1.0 - t),
                scale2(sub2(b[2], b[1]), t),
            ),
            2.0,
        ),
        4 => {
            let tc = 1.0 - t;
            let mut out = scale2(sub2(b[1], b[0]), 3.0 * tc * tc);
            out = add2(out, scale2(sub2(b[2], b[1]), 6.0 * tc * t));
            add2(out, scale2(sub2(b[3], b[2]), 3.0 * t * t))
        }
        n => panic!("bezier::derivative::ERROR::unsupported degree with {n} points"),
    }
}

pub fn second_derivative(b: &[V2], t: f64) -> V2 {
    match b.len() {
        2 => [0.0, 0.0],
        3 => scale2(add2(sub2(b[2], scale2(b[1], 2.0)), b[0]), 2.0),
        4 => {
            let p0 = add2(sub2(b[2], scale2(b[1], 2.0)), b[0]);
            let p1 = add2(sub2(b[3], scale2(b[2], 2.0)), b[1]);
            scale2(add2(scale2(p0, 1.0 - t), scale2(p1, t)), 6.0)
        }
        n => panic!("bezier::second_derivative::ERROR::unsupported degree with {n} points"),
    }
}

/// De Casteljau split at `t`, returning the two halves' control points.
pub fn split(b: &[V2], t: f64) -> (Vec<V2>, Vec<V2>) {
    let mut left = Vec::with_capacity(b.len());
    let mut right = Vec::with_capacity(b.len());
    let mut level: Vec<V2> = b.to_vec();

    while level.len() > 1 {
        left.push(level[0]);
        right.push(level[level.len() - 1]);
        level = level
            .windows(2)
            .map(|w| add2(scale2(w[0], 1.0 - t), scale2(w[1], t)))
            .collect();
    }
    left.push(level[0]);
    right.push(level[0]);
    right.reverse();

    (left, right)
}

/// Bezier control points -> polynomial coefficients in numpy order, per
/// component. `svgpathtools.bezier.bezier2polynomial`.
pub fn to_polynomial(b: &[V2], component: usize) -> Vec<f64> {
    let p: Vec<f64> = b.iter().map(|v| v[component]).collect();
    match p.len() {
        4 => vec![
            -p[0] + 3.0 * (p[1] - p[2]) + p[3],
            3.0 * (p[0] - 2.0 * p[1] + p[2]),
            3.0 * (p[1] - p[0]),
            p[0],
        ],
        3 => vec![p[0] - 2.0 * p[1] + p[2], 2.0 * (p[1] - p[0]), p[0]],
        2 => vec![p[1] - p[0], p[0]],
        n => panic!("bezier::to_polynomial::ERROR::unsupported degree with {n} points"),
    }
}

/// Closed-form quadratic Bezier arc length, matching
/// `svgpathtools.QuadraticBezier.length`.
pub fn quad_length(start: V2, control: V2, end: V2, t0: f64, t1: f64) -> f64 {
    let a = add2(sub2(start, scale2(control, 2.0)), end);
    let b = scale2(sub2(control, start), 2.0);
    let a_dot_b = dot2(a, b);
    let abs_a = norm2(a);
    let abs_b = norm2(b);

    if abs_a < 1e-12 {
        return abs_b * (t1 - t0);
    }

    let c2 = 4.0 * dot2(a, a);
    let c1 = 4.0 * a_dot_b;
    let c0 = dot2(b, b);

    let beta = c1 / (2.0 * c2);
    let gamma = c0 / c2 - beta * beta;

    let dq1_mag = (c2 * t1 * t1 + c1 * t1 + c0).max(0.0).sqrt();
    let dq0_mag = (c2 * t0 * t0 + c1 * t0 + c0).max(0.0).sqrt();
    let rand_num = c2.sqrt() * (t1 + beta) + dq1_mag;
    let rand_den = c2.sqrt() * (t0 + beta) + dq0_mag;
    let logarand = if rand_den != 0.0 && rand_num / rand_den > 0.0 {
        (rand_num / rand_den).ln()
    } else {
        0.0
    };

    let s = ((t1 + beta) * dq1_mag - (t0 + beta) * dq0_mag + gamma * c2.sqrt() * logarand) / 2.0;

    if s.is_nan() {
        // Degenerate case: the control points are collinear and the curve
        // reverses direction at `tstar`.
        let tstar = abs_b / (2.0 * abs_a);
        return if t1 < tstar {
            abs_a * (t0 * t0 - t1 * t1) - abs_b * (t0 - t1)
        } else if tstar < t0 {
            abs_a * (t1 * t1 - t0 * t0) - abs_b * (t1 - t0)
        } else {
            abs_a * (t1 * t1 + t0 * t0) - abs_b * (t1 + t0) + abs_b * abs_b / (2.0 * abs_a)
        };
    }
    s
}

// ----- Adaptive Gauss-Kronrod quadrature -----
//
// Stands in for `scipy.integrate.quad(..., epsabs=1e-12)`, which is what
// svgpathtools uses for cubic and arc lengths.

const GK_X: [f64; 8] = [
    0.000000000000000000000000000000000,
    0.207_784_955_007_898_48,
    0.405_845_151_377_397_2,
    0.586_087_235_467_691_1,
    0.741_531_185_599_394_5,
    0.864_864_423_359_769_1,
    0.949_107_912_342_758_5,
    0.991_455_371_120_812_6,
];

const GK_WK: [f64; 8] = [
    0.209_482_141_084_727_82,
    0.204_432_940_075_298_89,
    0.190_350_578_064_785_42,
    0.169_004_726_639_267_9,
    0.140_653_259_715_525_92,
    0.104_790_010_322_250_19,
    0.063_092_092_629_978_56,
    0.022_935_322_010_529_224,
];

const GK_WG: [f64; 4] = [
    0.417_959_183_673_469_4,
    0.381_830_050_505_118_9,
    0.279_705_391_489_276_64,
    0.129_484_966_168_869_7,
];

fn gk15<F: Fn(f64) -> f64>(f: &F, a: f64, b: f64) -> (f64, f64) {
    let center = 0.5 * (a + b);
    let half = 0.5 * (b - a);

    let mut res_k = GK_WK[0] * f(center);
    let mut res_g = GK_WG[0] * f(center);

    for j in 1..8 {
        let dx = half * GK_X[j];
        let fsum = f(center - dx) + f(center + dx);
        res_k += GK_WK[j] * fsum;
        if j % 2 == 0 {
            res_g += GK_WG[j / 2] * fsum;
        }
    }

    (res_k * half, ((res_k - res_g) * half).abs())
}

/// Integrate `|derivative(t)|` over `[t0, t1]` to an absolute tolerance of
/// 1e-12.
pub fn adaptive_length<F: Fn(f64) -> V2>(derivative: F, t0: f64, t1: f64) -> f64 {
    if t0 == t1 {
        return 0.0;
    }
    let speed = |t: f64| norm2(derivative(t));

    // Interval stack with per-interval error estimates; always subdivide the
    // worst interval, as QAGS does.
    let mut intervals: Vec<(f64, f64, f64, f64)> = Vec::with_capacity(64);
    let (v, e) = gk15(&speed, t0, t1);
    intervals.push((t0, t1, v, e));

    for _ in 0..200 {
        let total_err: f64 = intervals.iter().map(|i| i.3).sum();
        if total_err < 1e-12 || intervals.len() > 512 {
            break;
        }
        let (worst, _) = intervals
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.3.partial_cmp(&b.1.3).unwrap())
            .map(|(i, v)| (i, v.3))
            .unwrap();
        let (a, b, _, _) = intervals[worst];
        let mid = 0.5 * (a + b);
        if mid <= a || mid >= b {
            break;
        }
        let (v1, e1) = gk15(&speed, a, mid);
        let (v2, e2) = gk15(&speed, mid, b);
        intervals[worst] = (a, mid, v1, e1);
        intervals.push((mid, b, v2, e2));
    }

    intervals.iter().map(|i| i.2).sum()
}

/// `(xmin, xmax, ymin, ymax)` of a Bezier segment.
pub fn bbox(b: &[V2]) -> [f64; 4] {
    let mut out = [0.0; 4];
    for c in 0..2 {
        let poly = to_polynomial(b, c);
        let d = poly::polyder(&poly);
        let mut ts = vec![0.0, 1.0];
        ts.extend(poly::polyroots_open01(&d));
        let vals: Vec<f64> = ts.iter().map(|t| poly::polyval(&poly, *t)).collect();
        out[c * 2] = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        out[c * 2 + 1] = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    }
    out
}

/// Extreme points (excluding the endpoints) of a Bezier along one axis.
pub fn extreme_points(b: &[V2], on_x: bool, on_y: bool) -> Vec<f64> {
    let mut ts = Vec::new();
    if on_x {
        let poly = to_polynomial(b, 0);
        ts.extend(poly::polyroots_open01(&poly::polyder(&poly)));
    }
    if on_y {
        let poly = to_polynomial(b, 1);
        ts.extend(poly::polyroots_open01(&poly::polyder(&poly)));
    }
    ts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_preserves_curve() {
        let b = [[0.0, 0.0], [1.0, 2.0], [2.0, -1.0], [3.0, 0.0]];
        let (l, r) = split(&b, 0.3);
        assert!(dist2(point(&l, 1.0), point(&b, 0.3)) < 1e-12);
        assert!(dist2(point(&r, 0.0), point(&b, 0.3)) < 1e-12);
        assert!(dist2(point(&l, 0.5), point(&b, 0.15)) < 1e-12);
        assert!(dist2(point(&r, 0.5), point(&b, 0.65)) < 1e-12);
    }

    #[test]
    fn quad_length_straight_line() {
        // Collinear control points -> the closed form must still give the
        // straight distance.
        let l = quad_length([0.0, 0.0], [1.0, 0.0], [2.0, 0.0], 0.0, 1.0);
        assert!((l - 2.0).abs() < 1e-9, "{l}");
    }

    /// Oracle: `quad(lambda t: abs(CubicBezier(...).derivative(t)), 0, 1)`.
    #[test]
    fn adaptive_length_matches_quad() {
        let b = [[0.0, 0.0], [1.0, 2.0], [2.0, -1.0], [3.0, 0.0]];
        let l = adaptive_length(|t| derivative(&b, t), 0.0, 1.0);
        assert!((l - 3.6437962789530105).abs() < 1e-10, "{l}");
    }

    #[test]
    fn bbox_includes_interior_extremum() {
        let b = [[0.0, 0.0], [1.0, 4.0], [2.0, 0.0]];
        let bb = bbox(&b);
        assert!((bb[0] - 0.0).abs() < 1e-12);
        assert!((bb[1] - 2.0).abs() < 1e-12);
        assert!((bb[2] - 0.0).abs() < 1e-12);
        // Peak of the quadratic is at t=0.5 -> y = 2
        assert!((bb[3] - 2.0).abs() < 1e-12, "{bb:?}");
    }
}
