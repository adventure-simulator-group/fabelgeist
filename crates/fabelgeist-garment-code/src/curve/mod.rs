//! A native replacement for the slice of `svgpathtools` that GarmentCode uses:
//! line / quadratic / cubic Bezier / circular-arc segments with evaluation,
//! arc length, arc-length inversion, cropping, bounding boxes and
//! intersections.
//!
//! Conventions follow `svgpathtools` exactly, including its arc-length
//! inversion by bisection and its recursive Bezier-Bezier intersection search,
//! so that results line up with the reference implementation.

pub mod arc;
pub mod bezier;
pub mod intersect;
pub mod poly;
pub mod svg_parse;

use crate::math::*;
pub use arc::Arc;

/// `svgpathtools.path.ILENGTH_S_TOL` -- and the value GarmentCode passes
/// explicitly in most places.
pub const ILENGTH_S_TOL: f64 = 1e-10;
/// The library default, used where GarmentCode does not override it.
pub const ILENGTH_S_TOL_DEFAULT: f64 = 1e-12;
const ILENGTH_MAXITS: usize = 10_000;

#[derive(Debug, Clone, PartialEq)]
pub enum Curve {
    Line { start: V2, end: V2 },
    Quad { start: V2, control: V2, end: V2 },
    Cubic { start: V2, c1: V2, c2: V2, end: V2 },
    Arc(Arc),
}

impl Curve {
    pub fn line(start: V2, end: V2) -> Self {
        Curve::Line { start, end }
    }

    pub fn quad(start: V2, control: V2, end: V2) -> Self {
        Curve::Quad {
            start,
            control,
            end,
        }
    }

    pub fn cubic(start: V2, c1: V2, c2: V2, end: V2) -> Self {
        Curve::Cubic { start, c1, c2, end }
    }

    pub fn start(&self) -> V2 {
        match self {
            Curve::Line { start, .. } | Curve::Quad { start, .. } | Curve::Cubic { start, .. } => {
                *start
            }
            Curve::Arc(a) => a.start,
        }
    }

    pub fn end(&self) -> V2 {
        match self {
            Curve::Line { end, .. } | Curve::Quad { end, .. } | Curve::Cubic { end, .. } => *end,
            Curve::Arc(a) => a.end,
        }
    }

    /// Bezier control points, start and end included.
    pub fn bpoints(&self) -> Vec<V2> {
        match self {
            Curve::Line { start, end } => vec![*start, *end],
            Curve::Quad {
                start,
                control,
                end,
            } => vec![*start, *control, *end],
            Curve::Cubic { start, c1, c2, end } => vec![*start, *c1, *c2, *end],
            Curve::Arc(_) => panic!("Curve::bpoints called on an Arc"),
        }
    }

    pub fn is_arc(&self) -> bool {
        matches!(self, Curve::Arc(_))
    }

    pub fn point(&self, t: f64) -> V2 {
        match self {
            Curve::Line { start, end } => add2(*start, scale2(sub2(*end, *start), t)),
            Curve::Quad { .. } | Curve::Cubic { .. } => bezier::point(&self.bpoints(), t),
            Curve::Arc(a) => a.point(t),
        }
    }

    pub fn derivative(&self, t: f64) -> V2 {
        match self {
            Curve::Line { start, end } => sub2(*end, *start),
            Curve::Quad { .. } | Curve::Cubic { .. } => bezier::derivative(&self.bpoints(), t),
            Curve::Arc(a) => a.derivative(t),
        }
    }

    fn second_derivative(&self, t: f64) -> V2 {
        match self {
            Curve::Line { .. } => [0.0, 0.0],
            Curve::Quad { .. } | Curve::Cubic { .. } => {
                bezier::second_derivative(&self.bpoints(), t)
            }
            Curve::Arc(a) => a.second_derivative(t),
        }
    }

    pub fn unit_tangent(&self, t: f64) -> V2 {
        normalize2(self.derivative(t))
    }

    /// `svgpathtools.path.segment_curvature`.
    pub fn curvature(&self, t: f64) -> f64 {
        let d = self.derivative(t);
        let dd = self.second_derivative(t);
        let denom = dot2(d, d).powf(1.5);
        if denom == 0.0 {
            return f64::INFINITY;
        }
        (d[0] * dd[1] - d[1] * dd[0]).abs() / denom
    }

    pub fn length(&self) -> f64 {
        self.length_range(0.0, 1.0)
    }

    pub fn length_range(&self, t0: f64, t1: f64) -> f64 {
        match self {
            Curve::Line { start, end } => dist2(*end, *start) * (t1 - t0),
            Curve::Quad {
                start,
                control,
                end,
            } => bezier::quad_length(*start, *control, *end, t0, t1),
            Curve::Cubic { .. } => bezier::adaptive_length(|t| self.derivative(t), t0, t1),
            Curve::Arc(a) => a.length_range(t0, t1),
        }
    }

    /// `svgpathtools.path.inv_arclength` -- bisection on `t` until
    /// `|s(t) - s| < s_tol`.
    pub fn ilength(&self, s: f64, s_tol: f64) -> f64 {
        let curve_length = self.length();
        assert!(curve_length > 0.0, "ilength on a zero-length curve");
        assert!(
            (-1e-9..=curve_length + 1e-9).contains(&s),
            "ilength::ERROR::s ({s}) is not in [0, {curve_length}]"
        );

        if s <= 0.0 {
            return 0.0;
        }
        if s >= curve_length {
            return 1.0;
        }
        if let Curve::Line { .. } = self {
            return s / curve_length;
        }

        let (mut t_lower, mut t_upper) = (0.0_f64, 1.0_f64);
        for _ in 0..ILENGTH_MAXITS {
            let t = (t_lower + t_upper) / 2.0;
            let s_t = self.length_range(0.0, t);
            if (s_t - s).abs() < s_tol {
                return t;
            } else if s_t < s {
                t_lower = t;
            } else {
                t_upper = t;
            }
            if t_upper == t_lower {
                return t;
            }
        }
        panic!("ilength::ERROR::maximum iterations reached");
    }

    /// Split into the `[0, t]` and `[t, 1]` halves.
    pub fn split(&self, t: f64) -> (Curve, Curve) {
        match self {
            Curve::Line { start, end } => {
                let mid = self.point(t);
                (Curve::line(*start, mid), Curve::line(mid, *end))
            }
            Curve::Quad { .. } | Curve::Cubic { .. } => {
                let (l, r) = bezier::split(&self.bpoints(), t);
                (from_bpoints(&l), from_bpoints(&r))
            }
            Curve::Arc(a) => (Curve::Arc(a.cropped(0.0, t)), Curve::Arc(a.cropped(t, 1.0))),
        }
    }

    /// `Segment.cropped(t0, t1)`.
    pub fn cropped(&self, t0: f64, t1: f64) -> Curve {
        assert!(t0 < t1, "cropped::ERROR::t0 ({t0}) must be < t1 ({t1})");
        match self {
            Curve::Arc(a) => Curve::Arc(a.cropped(t0, t1)),
            _ => {
                if t0 <= 0.0 {
                    self.split(t1).0
                } else if t1 >= 1.0 {
                    self.split(t0).1
                } else {
                    // De Casteljau twice: crop the tail, then re-map t1.
                    let trimmed = self.split(t0).1;
                    let t1_adj = (t1 - t0) / (1.0 - t0);
                    trimmed.split(t1_adj).0
                }
            }
        }
    }

    /// `(xmin, xmax, ymin, ymax)`.
    pub fn bbox(&self) -> [f64; 4] {
        match self {
            Curve::Arc(a) => a.bbox(),
            Curve::Line { start, end } => [
                start[0].min(end[0]),
                start[0].max(end[0]),
                start[1].min(end[1]),
                start[1].max(end[1]),
            ],
            _ => bezier::bbox(&self.bpoints()),
        }
    }

    pub fn reversed(&self) -> Curve {
        match self {
            Curve::Line { start, end } => Curve::line(*end, *start),
            Curve::Quad {
                start,
                control,
                end,
            } => Curve::quad(*end, *control, *start),
            Curve::Cubic { start, c1, c2, end } => Curve::cubic(*end, *c2, *c1, *start),
            Curve::Arc(a) => Curve::Arc(a.reversed()),
        }
    }

    pub fn translated(&self, d: V2) -> Curve {
        self.map_points(|p| add2(p, d))
    }

    /// Rotate by `degs` degrees about `origin`.
    pub fn rotated(&self, degs: f64, origin: V2) -> Curve {
        let m = r2d(degs.to_radians());
        self.map_points(|p| add2(apply_r2d(m, sub2(p, origin)), origin))
    }

    fn map_points<F: Fn(V2) -> V2>(&self, f: F) -> Curve {
        match self {
            Curve::Line { start, end } => Curve::line(f(*start), f(*end)),
            Curve::Quad {
                start,
                control,
                end,
            } => Curve::quad(f(*start), f(*control), f(*end)),
            Curve::Cubic { start, c1, c2, end } => Curve::cubic(f(*start), f(*c1), f(*c2), f(*end)),
            Curve::Arc(a) => Curve::Arc(a.map_points(f)),
        }
    }

    /// Linear approximation with `n_inside` interior sample points.
    pub fn linearized(&self, n_inside: usize) -> Vec<Curve> {
        let n = n_inside + 1;
        let mut verts = vec![self.start()];
        for i in 1..n {
            verts.push(self.point(i as f64 / n as f64));
        }
        verts.push(self.end());
        verts.windows(2).map(|w| Curve::line(w[0], w[1])).collect()
    }

    /// `Segment.intersect(other)` -- pairs `(t_self, t_other)`.
    pub fn intersect(&self, other: &Curve) -> Vec<(f64, f64)> {
        intersect::intersect(self, other)
    }
}

/// Build a Bezier `Curve` from its control points.
pub fn from_bpoints(b: &[V2]) -> Curve {
    match b.len() {
        2 => Curve::line(b[0], b[1]),
        3 => Curve::quad(b[0], b[1], b[2]),
        4 => Curve::cubic(b[0], b[1], b[2], b[3]),
        n => panic!("from_bpoints::ERROR::unsupported degree with {n} points"),
    }
}

/// A chain of segments -- `svgpathtools.Path`.
#[derive(Debug, Clone, Default)]
pub struct Path {
    pub segments: Vec<Curve>,
}

impl Path {
    pub fn new(segments: Vec<Curve>) -> Self {
        Self { segments }
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn length(&self) -> f64 {
        self.segments.iter().map(|s| s.length()).sum()
    }

    pub fn bbox(&self) -> [f64; 4] {
        let mut out = [
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ];
        for s in &self.segments {
            let b = s.bbox();
            out[0] = out[0].min(b[0]);
            out[1] = out[1].max(b[1]);
            out[2] = out[2].min(b[2]);
            out[3] = out[3].max(b[3]);
        }
        out
    }

    pub fn translated(&self, d: V2) -> Path {
        Path::new(self.segments.iter().map(|s| s.translated(d)).collect())
    }

    pub fn rotated(&self, degs: f64, origin: V2) -> Path {
        Path::new(
            self.segments
                .iter()
                .map(|s| s.rotated(degs, origin))
                .collect(),
        )
    }

    pub fn scaled(&self, s: f64) -> Path {
        Path::new(
            self.segments
                .iter()
                .map(|c| c.map_points(|p| scale2(p, s)))
                .collect(),
        )
    }

    /// Global parameter `T` in `[0, 1]` -> `(segment index, local t)`.
    pub fn t_to_local(&self, big_t: f64) -> (usize, f64) {
        let total = self.length();
        let target = big_t * total;
        let mut covered = 0.0;
        for (i, seg) in self.segments.iter().enumerate() {
            let l = seg.length();
            if covered + l >= target - 1e-12 || i == self.segments.len() - 1 {
                let s = (target - covered).clamp(0.0, l);
                let t = if l == 0.0 {
                    0.0
                } else {
                    seg.ilength(s, ILENGTH_S_TOL_DEFAULT)
                };
                return (i, t);
            }
            covered += l;
        }
        (self.segments.len() - 1, 1.0)
    }

    /// `(segment index, local t)` -> global parameter `T`.
    pub fn local_to_t(&self, seg_idx: usize, t: f64) -> f64 {
        let total = self.length();
        let mut covered: f64 = self.segments[..seg_idx].iter().map(|s| s.length()).sum();
        covered += self.segments[seg_idx].length_range(0.0, t);
        covered / total
    }

    /// `Path.cropped(T0, T1)` over the global (arc-length) parameter.
    pub fn cropped(&self, big_t0: f64, big_t1: f64) -> Path {
        let (i0, t0) = self.t_to_local(big_t0);
        let (i1, t1) = self.t_to_local(big_t1);

        if i0 == i1 {
            return Path::new(vec![self.segments[i0].cropped(t0, t1)]);
        }

        let mut out = Vec::new();
        if t0 < 1.0 - 1e-12 {
            out.push(self.segments[i0].cropped(t0, 1.0));
        }
        for seg in &self.segments[i0 + 1..i1] {
            out.push(seg.clone());
        }
        if t1 > 1e-12 {
            out.push(self.segments[i1].cropped(0.0, t1));
        }
        Path::new(out)
    }

    /// Intersections with a segment, as `(T_self, t_other)`.
    pub fn intersect_segment(&self, other: &Curve) -> Vec<(f64, f64)> {
        let mut out = Vec::new();
        for (i, seg) in self.segments.iter().enumerate() {
            for (t1, t2) in seg.intersect(other) {
                out.push((self.local_to_t(i, t1), t2));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn line_basics() {
        let l = Curve::line([0.0, 0.0], [3.0, 4.0]);
        assert!(approx(l.length(), 5.0, 1e-12));
        assert!(approx(l.ilength(2.5, 1e-12), 0.5, 1e-12));
        assert_eq!(l.point(0.5), [1.5, 2.0]);
    }

    /// Oracle: `svgpathtools.QuadraticBezier(0, 1+2j, 3).length()`.
    #[test]
    fn quad_length_matches_svgpathtools() {
        let q = Curve::quad([0.0, 0.0], [1.0, 2.0], [3.0, 0.0]);
        assert!(
            approx(q.length(), 3.7546364123171068, 1e-9),
            "{}",
            q.length()
        );
    }

    /// Oracle: `svgpathtools.CubicBezier(0, 1+2j, 2-1j, 3).length()`.
    #[test]
    fn cubic_length_matches_svgpathtools() {
        let c = Curve::cubic([0.0, 0.0], [1.0, 2.0], [2.0, -1.0], [3.0, 0.0]);
        assert!(
            approx(c.length(), 3.6437962789530105, 1e-9),
            "{}",
            c.length()
        );
    }

    #[test]
    fn cropped_endpoints() {
        let c = Curve::cubic([0.0, 0.0], [1.0, 2.0], [2.0, -1.0], [3.0, 0.0]);
        let sub = c.cropped(0.25, 0.75);
        assert!(dist2(sub.start(), c.point(0.25)) < 1e-12);
        assert!(dist2(sub.end(), c.point(0.75)) < 1e-12);
        // Midpoint of the crop must lie on the original curve.
        assert!(dist2(sub.point(0.5), c.point(0.5)) < 1e-12);
    }

    #[test]
    fn cropped_lengths_sum() {
        let c = Curve::cubic([0.0, 0.0], [1.0, 2.0], [2.0, -1.0], [3.0, 0.0]);
        let total: f64 = [(0.0, 0.3), (0.3, 0.8), (0.8, 1.0)]
            .iter()
            .map(|(a, b)| c.cropped(*a, *b).length())
            .sum();
        assert!(approx(total, c.length(), 1e-9), "{total}");
    }

    #[test]
    fn ilength_is_inverse_of_length() {
        let c = Curve::cubic([0.0, 0.0], [1.0, 2.0], [2.0, -1.0], [3.0, 0.0]);
        let target = c.length() * 0.37;
        let t = c.ilength(target, 1e-12);
        assert!(approx(c.length_range(0.0, t), target, 1e-10));
    }

    #[test]
    fn path_crop_roundtrip() {
        let p = Path::new(vec![
            Curve::line([0.0, 0.0], [1.0, 0.0]),
            Curve::line([1.0, 0.0], [1.0, 1.0]),
            Curve::line([1.0, 1.0], [0.0, 1.0]),
        ]);
        let c = p.cropped(0.25, 0.75);
        assert!(approx(c.length(), p.length() * 0.5, 1e-9), "{}", c.length());
    }
}
