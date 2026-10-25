//! Constructors for edges and common edge sequences.
//!
//! Ports `pygarment.garmentcode.edge_factory`.

use crate::curve::Curve;
use crate::math::*;
use crate::optimize;

use super::edge::{Edge, EdgeRef, EdgeSequence, vert};

// ----- Circle arcs -----

/// Circle arc through two points spanning a given angle (radians).
pub fn circle_from_points_angle(start: V2, end: V2, arc_angle: f64, right: bool) -> EdgeRef {
    let (arc_angle, to_sum) = if arc_angle > std::f64::consts::PI {
        (2.0 * std::f64::consts::PI - arc_angle, true)
    } else {
        (arc_angle, false)
    };

    let radius = 1.0 / (arc_angle / 2.0).sin() / 2.0;
    let h = 1.0 / (arc_angle / 2.0).tan() / 2.0;

    let mut control_y = if to_sum { radius + h } else { radius - h };
    if right {
        control_y *= -1.0;
    }

    Edge::circle(start, end, control_y)
}

/// Circle arc through two points with a given absolute radius.
pub fn circle_from_points_radius(
    start: V2,
    end: V2,
    radius: f64,
    large_arc: bool,
    right: bool,
) -> EdgeRef {
    let str_dist = dist2(end, start);

    // Values that are close enough can go slightly negative under the root.
    let center_r = if close_enough(radius * radius, str_dist * str_dist / 4.0, 1e-3) {
        0.0
    } else {
        (radius * radius - str_dist * str_dist / 4.0)
            .max(0.0)
            .sqrt()
    };

    let mut control_y = if large_arc {
        radius + center_r
    } else {
        radius - center_r
    };
    control_y /= str_dist;
    if right {
        control_y *= -1.0;
    }

    Edge::circle(start, end, control_y)
}

/// Circle arc of a given radius and arc length.
///
/// Without `start` both vertices are created to match; with it the arc is
/// snapped so that it begins there.
pub fn circle_from_rad_length(
    rad: f64,
    length: f64,
    right: bool,
    start: Option<&super::edge::Vert>,
) -> EdgeRef {
    let max_len = 2.0 * std::f64::consts::PI * rad;
    assert!(
        length <= max_len,
        "CircleEdge::ERROR::Incorrect length for specified radius"
    );

    let large_arc = length > max_len / 2.0;
    let length = if large_arc { max_len - length } else { length };

    let w_half = rad * (length / rad / 2.0).sin();

    let edge = circle_from_points_radius([-w_half, 0.0], [w_half, 0.0], rad, large_arc, right);

    if let Some(s) = start {
        edge.borrow().snap_to(super::edge::vget(s));
        edge.borrow_mut().start = s.clone();
    }
    edge
}

/// Circle arc through three points. `point_on_arc` may be given relative to the
/// `start -> end` frame.
pub fn circle_from_three_points(start: V2, end: V2, point_on_arc: V2, relative: bool) -> EdgeRef {
    let point_on_arc = if relative {
        rel_to_abs_2d(start, end, point_on_arc)
    } else {
        point_on_arc
    };

    // Circumcentre via the complex-number construction the reference uses.
    let (x, y, z) = (start, point_on_arc, end);
    let w = cdiv(sub2(z, x), sub2(y, x));
    let w_abs_sq = dot2(w, w);
    // c = (x - y) * (w - |w|^2) / (2i * w.imag) - x
    let num = cmul(sub2(x, y), sub2(w, [w_abs_sq, 0.0]));
    let denom = [0.0, 2.0 * w[1]];
    let c = sub2(cdiv(num, denom), x);
    let rad = norm2(add2(c, x));

    let mid_dist = norm2(sub2(point_on_arc, scale2(add2(start, end), 0.5)));
    let angle = vector_angle(sub2(point_on_arc, start), sub2(end, start));

    circle_from_points_radius(start, end, rad, mid_dist > rad, angle > 0.0)
}

fn cmul(a: V2, b: V2) -> V2 {
    [a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0]]
}

fn cdiv(a: V2, b: V2) -> V2 {
    let d = b[0] * b[0] + b[1] * b[1];
    [
        (a[0] * b[0] + a[1] * b[1]) / d,
        (a[1] * b[0] - a[0] * b[1]) / d,
    ]
}

// ----- Bezier curves -----

/// Penalty used when an optimiser probe produces no curve/line intersection.
///
/// The reference would raise an `IndexError` here; in practice the fit never
/// reaches such a point for a valid design, and a large finite value keeps the
/// search well-behaved instead of aborting the whole pattern.
const NO_INTERSECTION_PENALTY: f64 = 1e6;

/// Objective: the quadratic Bezier `[0,0] -> cp -> [1,0]` should pass through
/// `target` (given in the edge-local frame).
fn fit_pass_point(cp: &[f64], target: V2) -> f64 {
    let curve = Curve::quad([0.0, 0.0], [cp[0], cp[1]], [1.0, 0.0]);
    let inter = Curve::line([target[0], target[1] * 2.0], [target[0], -target[1] * 2.0]);

    let hits = curve.intersect(&inter);
    let Some((t, _)) = hits.first() else {
        return NO_INTERSECTION_PENALTY;
    };
    let point = curve.point(*t);
    let diff = dist2(point, target);
    diff * diff
}

/// A quadratic curve edge between `start` and `end` passing through `target`.
pub fn curve_3_points(start: V2, end: V2, target: V2) -> EdgeRef {
    let rel_target = abs_to_rel_2d(start, end, target, false);

    assert!(
        (0.0..=1.0).contains(&rel_target[0]),
        "CurveEdgeFactory::curve_3_points::ERROR::the target point's projection \
         is outside of the base edge, which is not supported"
    );

    // Starting from the target itself gives the smoothest solution.
    let out = optimize::minimize(|cp| fit_pass_point(cp, rel_target), &rel_target);

    Edge::curve(start, end, vec![[out.x[0], out.x[1]]], true)
}

fn fit_tangents(cp: &[f64], tan0: Option<V2>, tan1: Option<V2>) -> f64 {
    let curve = Curve::quad([0.0, 0.0], [cp[0], cp[1]], [1.0, 0.0]);
    let mut fin = 0.0;
    if let Some(t0) = tan0 {
        let d = dist2(curve.unit_tangent(0.0), t0);
        fin += d * d;
    }
    if let Some(t1) = tan1 {
        let d = dist2(curve.unit_tangent(1.0), t1);
        fin += d * d;
    }
    fin
}

/// A quadratic curve edge with prescribed endpoint tangents (either may be
/// omitted). Tangents are given in panel coordinates and normalised here.
pub fn curve_from_tangents(
    start: V2,
    end: V2,
    target_tan0: Option<V2>,
    target_tan1: Option<V2>,
    initial_guess: Option<V2>,
) -> EdgeRef {
    let to_local = |t: V2| normalize2(abs_to_rel_2d(start, end, t, true));
    let tan0 = target_tan0.map(to_local);
    let tan1 = target_tan1.map(to_local);

    let x0 = initial_guess.unwrap_or([0.5, 0.0]);
    let out = optimize::minimize(|cp| fit_tangents(cp, tan0, tan1), &x0);

    Edge::curve(start, end, vec![[out.x[0], out.x[1]]], true)
}

// ----- Edge sequences -----

/// A chain of straight edges through the given vertices, optionally closed.
pub fn from_verts(verts: &[V2], close: bool) -> EdgeSequence {
    assert!(
        verts.len() >= 2,
        "EdgeSeqFactory::from_verts::ERROR::need at least 2 vertices"
    );

    let mut seq = EdgeSequence::one(Edge::line(verts[0], verts[1]));
    for v in &verts[2..] {
        let prev = seq.last().borrow().end.clone();
        seq.push(Edge::line_v(prev, vert(*v)));
    }
    if close {
        let a = seq.last().borrow().end.clone();
        let b = seq.first().borrow().start.clone();
        seq.push(Edge::line_v(a, b));
    }
    seq
}

/// Straight edges between `start` and `end` whose lengths follow `frac`.
pub fn from_fractions(start: V2, end: V2, frac: &[f64]) -> EdgeSequence {
    let frac: Vec<f64> = frac.iter().map(|f| f.abs()).collect();
    let fsum: f64 = frac.iter().sum();
    assert!(
        close_enough(fsum, 1.0, 1e-4),
        "EdgeSequence::ERROR::fraction is incorrect. The sum {fsum} is not 1"
    );

    let vec = sub2(end, start);
    let mut verts = vec![start];
    for f in &frac[..frac.len() - 1] {
        let last = *verts.last().unwrap();
        verts.push([last[0] + f * vec[0], last[1] + f * vec[1]]);
    }
    verts.push(end);

    from_verts(&verts, false)
}

/// A side edge carrying extra vertices, so that only part of it takes a stitch.
pub fn side_with_cut(start: V2, end: V2, start_cut: f64, end_cut: f64) -> EdgeSequence {
    let vec = sub2(end, start);
    let mut verts = vec![start];
    if start_cut > 0.0 {
        verts.push(add2(start, scale2(vec, start_cut)));
    }
    if end_cut > 0.0 {
        verts.push(sub2(end, scale2(vec, end_cut)));
    }
    verts.push(end);

    from_verts(&verts, false)
}

/// A simple triangular dart, specified by width plus either the side length or
/// the perpendicular depth.
pub fn dart_shape(width: f64, side_len: Option<f64>, depth: Option<f64>) -> EdgeSequence {
    let depth = match (depth, side_len) {
        (Some(d), _) => d,
        (None, Some(side)) => {
            assert!(
                width / 2.0 <= side,
                "EdgeFactory::ERROR::Requested dart shape (w={width}, side={side}) \
                 does not form a valid triangle"
            );
            (side * side - (width / 2.0) * (width / 2.0)).sqrt()
        }
        (None, None) => panic!(
            "EdgeFactory::ERROR::dart shape is not fully specified. \
             Add dart side length or dart perpendicular"
        ),
    };

    from_verts(&[[0.0, 0.0], [width / 2.0, -depth], [width, 0.0]], false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garment::edge::EdgeKind;

    #[test]
    fn arc_from_angle_is_a_semicircle() {
        let e = circle_from_points_angle([0.0, 0.0], [2.0, 0.0], std::f64::consts::PI, false);
        let EdgeKind::Circle { control_y } = e.borrow().kind else {
            panic!()
        };
        assert!((control_y - 0.5).abs() < 1e-9, "{control_y}");
        assert!((e.borrow().length() - std::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn arc_from_radius_roundtrips() {
        let e = circle_from_points_radius([0.0, 0.0], [2.0, 0.0], 1.0, false, false);
        let (r, large, right) = e.borrow().as_radius_flag();
        assert!((r - 1.0).abs() < 1e-9, "{r}");
        assert!(!large);
        assert!(!right);
    }

    #[test]
    fn arc_from_rad_length() {
        let e = circle_from_rad_length(5.0, 3.0, true, None);
        assert!(
            (e.borrow().length() - 3.0).abs() < 1e-9,
            "{}",
            e.borrow().length()
        );
    }

    #[test]
    fn arc_through_three_points() {
        let e = circle_from_three_points([0.0, 0.0], [2.0, 0.0], [1.0, 1.0], false);
        // The apex is the top of a unit semicircle.
        assert!(
            dist2(e.borrow().midpoint(), [1.0, 1.0]) < 1e-6,
            "{:?}",
            e.borrow().midpoint()
        );
    }

    #[test]
    fn curve_through_three_points_passes_through_it() {
        let target = [3.0, 2.0];
        let e = curve_3_points([0.0, 0.0], [10.0, 0.0], target);
        let curve = e.borrow().as_curve();

        // The curve must cross the vertical line at the target's x, at its y.
        let vline = Curve::line([target[0], -10.0], [target[0], 10.0]);
        let hits = curve.intersect(&vline);
        assert_eq!(hits.len(), 1, "{hits:?}");
        let p = curve.point(hits[0].0);
        assert!(dist2(p, target) < 1e-4, "{p:?} != {target:?}");
    }

    #[test]
    fn curve_matches_requested_tangents() {
        let e = curve_from_tangents([0.0, 0.0], [10.0, 0.0], Some([1.0, 1.0]), None, None);
        let curve = e.borrow().as_curve();
        let t = curve.unit_tangent(0.0);
        let want = normalize2([1.0, 1.0]);
        assert!(dist2(t, want) < 1e-3, "{t:?} != {want:?}");
    }

    #[test]
    fn from_verts_chains_and_closes() {
        let seq = from_verts(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]], true);
        assert_eq!(seq.len(), 3);
        assert!(seq.is_chained());
        assert!(seq.is_loop());
    }

    #[test]
    fn fractions_split_a_line() {
        let seq = from_fractions([0.0, 0.0], [10.0, 0.0], &[0.2, 0.5, 0.3]);
        let lens = seq.lengths();
        assert!((lens[0] - 2.0).abs() < 1e-12, "{lens:?}");
        assert!((lens[1] - 5.0).abs() < 1e-12, "{lens:?}");
        assert!((lens[2] - 3.0).abs() < 1e-12, "{lens:?}");
    }

    #[test]
    fn dart_from_side_length() {
        let d = dart_shape(6.0, Some(5.0), None);
        assert_eq!(d.len(), 2);
        // 3-4-5 triangle -> depth 4
        assert_eq!(d[0].borrow().end_p(), [3.0, -4.0]);
    }
}
