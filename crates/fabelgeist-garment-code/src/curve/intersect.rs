//! Segment intersections, following `svgpathtools`.
//!
//! Line-line and Bezier-line cases are analytic; Bezier-Bezier uses the same
//! recursive bounding-box subdivision the reference does, so that the
//! self-intersection checks agree.

use super::{Curve, bezier, poly};
use crate::math::*;

/// `Segment.intersect(other)` -- returns `(t_self, t_other)` pairs.
pub fn intersect(a: &Curve, b: &Curve) -> Vec<(f64, f64)> {
    match (a, b) {
        (Curve::Arc(_), _) | (_, Curve::Arc(_)) => arc_intersect(a, b),
        (Curve::Line { .. }, Curve::Line { .. }) => line_line(a, b),
        (Curve::Line { .. }, _) => bezier_by_line(b, a)
            .into_iter()
            .map(|(tb, tl)| (tl, tb))
            .collect(),
        (_, Curve::Line { .. }) => bezier_by_line(a, b),
        _ => {
            // Cheap bounding-box reject first, as the reference does.
            if !bpoints_overlap(a, b) {
                return vec![];
            }
            let longer = a.length().max(b.length());
            bezier_intersections(&a.bpoints(), &b.bpoints(), longer)
        }
    }
}

fn bpoints_overlap(a: &Curve, b: &Curve) -> bool {
    let (pa, pb) = (a.bpoints(), b.bpoints());
    for c in 0..2 {
        let amin = pa.iter().map(|p| p[c]).fold(f64::INFINITY, f64::min);
        let amax = pa.iter().map(|p| p[c]).fold(f64::NEG_INFINITY, f64::max);
        let bmin = pb.iter().map(|p| p[c]).fold(f64::INFINITY, f64::min);
        let bmax = pb.iter().map(|p| p[c]).fold(f64::NEG_INFINITY, f64::max);
        if bmin > amax || bmax < amin {
            return false;
        }
    }
    true
}

fn line_line(a: &Curve, b: &Curve) -> Vec<(f64, f64)> {
    let (p0, p1) = (a.start(), a.end());
    let (q0, q1) = (b.start(), b.end());

    let denom = (p1[0] - p0[0]) * (q0[1] - q1[1]) - (p1[1] - p0[1]) * (q0[0] - q1[0]);
    if poly::isclose(denom, 0.0) {
        return vec![];
    }

    let t1 = (q0[0] * (p0[1] - q1[1]) - q1[0] * (p0[1] - q0[1]) - p0[0] * (q0[1] - q1[1])) / denom;
    let t2 = -(p1[0] * (p0[1] - q0[1]) - p0[0] * (p1[1] - q0[1]) - q0[0] * (p0[1] - p1[1])) / denom;

    if (0.0..=1.0).contains(&t1) && (0.0..=1.0).contains(&t2) {
        vec![(t1, t2)]
    } else {
        vec![]
    }
}

/// `svgpathtools.bezier.bezier_by_line_intersections` -- returns
/// `(t_bezier, t_line)`.
fn bezier_by_line(bez: &Curve, line: &Curve) -> Vec<(f64, f64)> {
    let (l0, l1) = (line.start(), line.end());
    let shifted_end = sub2(l1, l0);
    let line_length = norm2(shifted_end);
    if line_length == 0.0 {
        return vec![];
    }

    // Rotate so the line lies on the positive real axis.
    let rot = scale2([shifted_end[0], -shifted_end[1]], 1.0 / line_length);
    let transformed: Vec<V2> = bez
        .bpoints()
        .iter()
        .map(|p| {
            let s = sub2(*p, l0);
            [s[0] * rot[0] - s[1] * rot[1], s[0] * rot[1] + s[1] * rot[0]]
        })
        .collect();

    let coeffs_y = bezier::to_polynomial(&transformed, 1);
    let roots_y = poly::polyroots01(&coeffs_y);

    let coeffs_x = bezier::to_polynomial(&transformed, 0);
    let mut out = Vec::new();
    for t in roots_y {
        let xval = poly::polyval(&coeffs_x, t);
        if (0.0..=line_length).contains(&xval) {
            out.push((t, xval / line_length));
        }
    }
    out
}

fn box_area(b: [f64; 4]) -> f64 {
    (b[1] - b[0]) * (b[3] - b[2])
}

fn boxes_intersect(b1: [f64; 4], b2: [f64; 4]) -> bool {
    let w = (b1[1].min(b2[1]) - b1[0].max(b2[0])).max(0.0);
    let h = (b1[3].min(b2[3]) - b1[2].max(b2[2])).max(0.0);
    w > 0.0 && h > 0.0
}

struct BPair {
    b1: Vec<V2>,
    b2: Vec<V2>,
    t1: f64,
    t2: f64,
}

/// `svgpathtools.bezier.bezier_intersections`.
fn bezier_intersections(bez1: &[V2], bez2: &[V2], longer_length: f64) -> Vec<(f64, f64)> {
    const TOL: f64 = 1e-8;
    const TOL_DEC: f64 = 1e-8;

    let maxits = (1.0 - (TOL_DEC / longer_length).ln() / 2.0_f64.ln()).ceil() as usize;

    let mut pair_list = vec![BPair {
        b1: bez1.to_vec(),
        b2: bez2.to_vec(),
        t1: 0.5,
        t2: 0.5,
    }];
    let mut intersections: Vec<(f64, f64)> = Vec::new();
    let mut approx_points: Vec<V2> = Vec::new();

    let mut k = 0usize;
    while !pair_list.is_empty() && k < maxits {
        let mut new_pairs = Vec::new();
        let delta = 0.5_f64.powi(k as i32 + 2);

        for pair in &pair_list {
            let bb1 = bezier::bbox(&pair.b1);
            let bb2 = bezier::bbox(&pair.b2);
            if !boxes_intersect(bb1, bb2) {
                continue;
            }
            if box_area(bb1) < TOL_DEC && box_area(bb2) < TOL_DEC {
                let point = bezier::point(bez1, pair.t1);
                if !approx_points.iter().any(|p| dist2(*p, point) < TOL) {
                    approx_points.push(point);
                    intersections.push((pair.t1, pair.t2));
                }
            } else {
                let (c11, c12) = bezier::split(&pair.b1, 0.5);
                let (c21, c22) = bezier::split(&pair.b2, 0.5);
                let (t11, t12) = (pair.t1 - delta, pair.t1 + delta);
                let (t21, t22) = (pair.t2 - delta, pair.t2 + delta);
                new_pairs.push(BPair {
                    b1: c11.clone(),
                    b2: c21.clone(),
                    t1: t11,
                    t2: t21,
                });
                new_pairs.push(BPair {
                    b1: c11,
                    b2: c22.clone(),
                    t1: t11,
                    t2: t22,
                });
                new_pairs.push(BPair {
                    b1: c12.clone(),
                    b2: c21,
                    t1: t12,
                    t2: t21,
                });
                new_pairs.push(BPair {
                    b1: c12,
                    b2: c22,
                    t1: t12,
                    t2: t22,
                });
            }
        }
        pair_list = new_pairs;
        k += 1;
    }

    intersections
}

/// Arc intersections.
///
/// `svgpathtools`' own arc intersection code is documented as unreliable and
/// GarmentCode works around it by linearising arcs before intersecting them.
/// This does the same: sample the arc densely and intersect the linearisation,
/// mapping the parameters back.
fn arc_intersect(a: &Curve, b: &Curve) -> Vec<(f64, f64)> {
    const N: usize = 64;

    let (segs_a, is_arc_a) = match a {
        Curve::Arc(_) => (a.linearized(N), true),
        _ => (vec![a.clone()], false),
    };
    let (segs_b, is_arc_b) = match b {
        Curve::Arc(_) => (b.linearized(N), true),
        _ => (vec![b.clone()], false),
    };

    let mut out = Vec::new();
    let na = segs_a.len() as f64;
    let nb = segs_b.len() as f64;
    for (ia, sa) in segs_a.iter().enumerate() {
        for (ib, sb) in segs_b.iter().enumerate() {
            for (t1, t2) in intersect(sa, sb) {
                let ta = if is_arc_a { (ia as f64 + t1) / na } else { t1 };
                let tb = if is_arc_b { (ib as f64 + t2) / nb } else { t2 };
                if !out
                    .iter()
                    .any(|(x, y): &(f64, f64)| (x - ta).abs() < 1e-6 && (y - tb).abs() < 1e-6)
                {
                    out.push((ta, tb));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crossing_lines() {
        let a = Curve::line([0.0, 0.0], [2.0, 2.0]);
        let b = Curve::line([0.0, 2.0], [2.0, 0.0]);
        let hits = intersect(&a, &b);
        assert_eq!(hits.len(), 1);
        assert!((hits[0].0 - 0.5).abs() < 1e-12);
        assert!((hits[0].1 - 0.5).abs() < 1e-12);
    }

    #[test]
    fn parallel_lines_miss() {
        let a = Curve::line([0.0, 0.0], [2.0, 0.0]);
        let b = Curve::line([0.0, 1.0], [2.0, 1.0]);
        assert!(intersect(&a, &b).is_empty());
    }

    #[test]
    fn disjoint_lines_miss() {
        let a = Curve::line([0.0, 0.0], [1.0, 0.0]);
        let b = Curve::line([5.0, -1.0], [5.0, 1.0]);
        assert!(intersect(&a, &b).is_empty());
    }

    #[test]
    fn quadratic_meets_line() {
        // Peak of the arch is at (1, 2); a horizontal line at y = 1 cuts it twice.
        let q = Curve::quad([0.0, 0.0], [1.0, 4.0], [2.0, 0.0]);
        let l = Curve::line([-1.0, 1.0], [3.0, 1.0]);
        let hits = intersect(&q, &l);
        assert_eq!(hits.len(), 2, "{hits:?}");
        for (t, _) in hits {
            assert!((q.point(t)[1] - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn cubics_cross() {
        let c1 = Curve::cubic([0.0, 0.0], [1.0, 3.0], [2.0, -3.0], [3.0, 0.0]);
        let c2 = Curve::cubic([0.0, -1.0], [1.0, 1.0], [2.0, 1.0], [3.0, 3.0]);
        let hits = intersect(&c1, &c2);
        assert!(!hits.is_empty(), "expected the cubics to cross");
        for (t1, t2) in hits {
            assert!(dist2(c1.point(t1), c2.point(t2)) < 1e-3);
        }
    }

    #[test]
    fn arc_meets_line() {
        let a = Curve::Arc(super::super::Arc::new(
            [-1.0, 0.0],
            [1.0, 1.0],
            0.0,
            false,
            false,
            [1.0, 0.0],
        ));
        let l = Curve::line([0.0, -2.0], [0.0, 2.0]);
        let hits = intersect(&a, &l);
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert!(dist2(a.point(hits[0].0), [0.0, 1.0]) < 1e-2);
    }
}
