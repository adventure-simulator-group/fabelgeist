//! Edges and edge sequences -- the basic building blocks of panels.
//!
//! Ports `pygarment.garmentcode.edge`.
//!
//! # Aliasing
//!
//! The reference implementation leans hard on Python's reference semantics:
//! neighbouring edges literally share the same `[x, y]` list for their common
//! vertex, so moving one moves the other, and interfaces hold the very same
//! `Edge` objects the panel does, so subdividing an edge for a stitch updates
//! both. That aliasing *is* the design, not an accident, so the port keeps it:
//! vertices are `Rc<RefCell<V2>>` and edges are `Rc<RefCell<Edge>>`, and
//! "is the same object" checks become `Rc::ptr_eq`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::curve::{Arc, Curve, ILENGTH_S_TOL};
use crate::math::*;

/// A vertex shared between the edges that meet at it.
pub type Vert = Rc<RefCell<V2>>;

pub fn vert(p: V2) -> Vert {
    Rc::new(RefCell::new(p))
}

pub fn vget(v: &Vert) -> V2 {
    *v.borrow()
}

pub fn vset(v: &Vert, p: V2) {
    *v.borrow_mut() = p;
}

/// Same vertex *object*, mirroring Python's `is`.
pub fn vsame(a: &Vert, b: &Vert) -> bool {
    Rc::ptr_eq(a, b)
}

/// The geometry of an edge, beyond its endpoints.
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeKind {
    /// A straight segment.
    Line,
    /// A circular arc, given by a third point on the arc.
    ///
    /// `control_y` is the offset of that point from the edge midpoint,
    /// expressed relative to the straight distance between the endpoints (x is
    /// pinned to 0.5 to avoid ambiguity). Keeping it relative preserves the arc
    /// angle when the endpoints move.
    Circle { control_y: f64 },
    /// A quadratic (one control point) or cubic (two) Bezier. Control points
    /// are in the edge-local frame.
    Curve { control_points: Vec<V2> },
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub start: Vert,
    pub end: Vert,
    /// Semantic label, written out as an edge property on assembly.
    pub label: String,
    /// Index of this edge within its panel's serialized loop; filled in at
    /// assembly time.
    pub geometric_id: usize,
    pub kind: EdgeKind,
}

/// A shared handle to an edge -- panels and interfaces hold the same one.
pub type EdgeRef = Rc<RefCell<Edge>>;

pub fn esame(a: &EdgeRef, b: &EdgeRef) -> bool {
    Rc::ptr_eq(a, b)
}

impl Edge {
    fn base(start: Vert, end: Vert, kind: EdgeKind) -> EdgeRef {
        Rc::new(RefCell::new(Edge {
            start,
            end,
            label: String::new(),
            geometric_id: 0,
            kind,
        }))
    }

    /// A straight edge between two fresh vertices.
    pub fn line(start: V2, end: V2) -> EdgeRef {
        assert!(
            !(close_enough(start[0], end[0], TOL) && close_enough(start[1], end[1], TOL)),
            "Edge::ERROR::start and end of an edge should differ"
        );
        Self::base(vert(start), vert(end), EdgeKind::Line)
    }

    /// A straight edge reusing existing vertex objects.
    pub fn line_v(start: Vert, end: Vert) -> EdgeRef {
        Self::base(start, end, EdgeKind::Line)
    }

    /// A circular arc; `cy` is the relative control offset (see
    /// [`EdgeKind::Circle`]).
    pub fn circle(start: V2, end: V2, cy: f64) -> EdgeRef {
        Self::base(vert(start), vert(end), EdgeKind::Circle { control_y: cy })
    }

    pub fn circle_v(start: Vert, end: Vert, cy: f64) -> EdgeRef {
        Self::base(start, end, EdgeKind::Circle { control_y: cy })
    }

    /// A Bezier edge. With `relative == false` the control points are given in
    /// panel coordinates and converted.
    pub fn curve(start: V2, end: V2, control_points: Vec<V2>, relative: bool) -> EdgeRef {
        assert!(
            control_points.len() <= 2,
            "CurveEdge::ERROR::Up to 2 control points (cubic Bezier) are supported"
        );
        let cps = if relative {
            control_points
        } else {
            control_points
                .into_iter()
                .map(|c| abs_to_rel_2d(start, end, c, false))
                .collect()
        };
        Self::base(
            vert(start),
            vert(end),
            EdgeKind::Curve {
                control_points: cps,
            },
        )
    }

    pub fn start_p(&self) -> V2 {
        vget(&self.start)
    }

    pub fn end_p(&self) -> V2 {
        vget(&self.end)
    }

    fn straight_len(&self) -> f64 {
        dist2(self.end_p(), self.start_p())
    }

    pub fn length(&self) -> f64 {
        match &self.kind {
            EdgeKind::Line => self.straight_len(),
            EdgeKind::Circle { .. } => self.rel_radius() * self.straight_len() * self.arc_angle(),
            EdgeKind::Curve { .. } => self.as_curve().length(),
        }
    }

    pub fn midpoint(&self) -> V2 {
        match &self.kind {
            EdgeKind::Line => scale2(add2(self.start_p(), self.end_p()), 0.5),
            EdgeKind::Circle { control_y } => {
                rel_to_abs_2d(self.start_p(), self.end_p(), [0.5, *control_y])
            }
            EdgeKind::Curve { .. } => {
                let curve = self.as_curve();
                let t_mid = curve.ilength(curve.length() / 2.0, ILENGTH_S_TOL);
                curve.point(t_mid)
            }
        }
    }

    /// The straight shortcut across the edge, as `[start, end]`.
    pub fn shortcut(&self) -> [V2; 2] {
        [self.start_p(), self.end_p()]
    }

    // ----- circle-arc geometry -----

    /// Relative radius (w.r.t. the straight distance), from the 3-point
    /// representation.
    pub fn rel_radius(&self) -> f64 {
        let EdgeKind::Circle { control_y } = &self.kind else {
            panic!("Edge::rel_radius::ERROR::not a circle edge");
        };
        // Circumscribed-circle radius of the triangle whose base runs from
        // (0, 0) to (1, 0) and whose apex is the control point.
        let a = 1.0;
        let b = norm2([0.5, *control_y]);
        let c = norm2([-0.5, *control_y]);
        let p = (a + b + c) / 2.0;
        a * b * c / (p * (p - a) * (p - b) * (p - c)).sqrt() / 4.0
    }

    pub fn arc_angle(&self) -> f64 {
        let rel_rad = self.rel_radius();
        let mut arc = 2.0 * (1.0 / rel_rad / 2.0).clamp(-1.0, 1.0).asin();
        if self.is_large_arc() {
            arc = 2.0 * std::f64::consts::PI - arc;
        }
        arc
    }

    pub fn is_large_arc(&self) -> bool {
        let EdgeKind::Circle { control_y } = &self.kind else {
            panic!("Edge::is_large_arc::ERROR::not a circle edge");
        };
        control_y.abs() > self.rel_radius()
    }

    /// `(radius, large_arc, right)`.
    pub fn as_radius_flag(&self) -> (f64, bool, bool) {
        let EdgeKind::Circle { control_y } = &self.kind else {
            panic!("Edge::as_radius_flag::ERROR::not a circle edge");
        };
        (
            self.rel_radius() * self.straight_len(),
            self.is_large_arc(),
            *control_y < 0.0,
        )
    }

    /// `(radius, arc angle, right)`.
    pub fn as_radius_angle(&self) -> (f64, f64, bool) {
        let EdgeKind::Circle { control_y } = &self.kind else {
            panic!("Edge::as_radius_angle::ERROR::not a circle edge");
        };
        (
            self.rel_radius() * self.straight_len(),
            self.arc_angle(),
            *control_y < 0.0,
        )
    }

    // ----- representation -----

    pub fn as_curve(&self) -> Curve {
        match &self.kind {
            EdgeKind::Line => Curve::line(self.start_p(), self.end_p()),
            EdgeKind::Circle { .. } => {
                let (radius, large_arc, right) = self.as_radius_flag();
                Curve::Arc(Arc::new(
                    self.start_p(),
                    [radius, radius],
                    0.0,
                    large_arc,
                    right,
                    self.end_p(),
                ))
            }
            EdgeKind::Curve { control_points } => {
                let (s, e) = (self.start_p(), self.end_p());
                let cp: Vec<V2> = control_points
                    .iter()
                    .map(|c| rel_to_abs_2d(s, e, *c))
                    .collect();
                if cp.len() < 2 {
                    Curve::quad(s, cp[0], e)
                } else {
                    Curve::cubic(s, cp[0], cp[1], e)
                }
            }
        }
    }

    /// The Bezier in its own local frame ([0,0] -> [1,0]).
    pub fn as_local_curve(&self) -> Curve {
        let EdgeKind::Curve { control_points } = &self.kind else {
            panic!("Edge::as_local_curve::ERROR::not a Bezier edge");
        };
        if control_points.len() < 2 {
            Curve::quad([0.0, 0.0], control_points[0], [1.0, 0.0])
        } else {
            Curve::cubic([0.0, 0.0], control_points[0], control_points[1], [1.0, 0.0])
        }
    }

    /// Default number of interior vertices used when linearizing this edge.
    pub fn default_linearization(&self) -> usize {
        match self.kind {
            EdgeKind::Line => 0,
            _ => 9,
        }
    }

    // ----- actions (in place) -----

    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.start, &mut self.end);
        match &mut self.kind {
            EdgeKind::Line => {}
            EdgeKind::Circle { control_y } => *control_y *= -1.0,
            EdgeKind::Curve { control_points } => {
                if control_points.len() == 2 {
                    control_points.swap(0, 1);
                }
                for p in control_points.iter_mut() {
                    *p = [1.0 - p[0], -p[1]];
                }
            }
        }
    }

    /// Mirror the edge's curvature to the other side of its shortcut.
    pub fn reflect_features(&mut self) {
        match &mut self.kind {
            EdgeKind::Line => {}
            EdgeKind::Circle { control_y } => *control_y *= -1.0,
            EdgeKind::Curve { control_points } => {
                for p in control_points.iter_mut() {
                    p[1] = -p[1];
                }
            }
        }
    }

    /// Translate so that `start` lands on `new_start`, moving the shared vertex
    /// objects in place.
    pub fn snap_to(&self, new_start: V2) {
        let s = self.start_p();
        let e = self.end_p();
        vset(
            &self.end,
            [e[0] - s[0] + new_start[0], e[1] - s[1] + new_start[1]],
        );
        vset(&self.start, new_start);
    }

    /// Rotate about the start vertex by `angle` radians.
    pub fn rotate(&self, angle: f64) {
        let curr_start = self.start_p();
        self.snap_to([0.0, 0.0]);
        let m = r2d(angle);
        vset(&self.end, apply_r2d(m, self.end_p()));
        self.snap_to(curr_start);
    }
}

// ----- subdivision -----
//
// Free functions rather than methods: they need the shared `EdgeRef` so the
// resulting sub-edges can keep pointing at the original end vertices.

/// Split by length. For lines and circular arcs this coincides with splitting
/// by curve parameter.
pub fn subdivide_len(edge: &EdgeRef, fractions: &[f64], connect_internal: bool) -> EdgeSequence {
    subdivide(edge, fractions, true, connect_internal)
}

/// Split by curve parameter.
pub fn subdivide_param(edge: &EdgeRef, fractions: &[f64], connect_internal: bool) -> EdgeSequence {
    subdivide(edge, fractions, false, connect_internal)
}

fn subdivide(
    edge: &EdgeRef,
    fractions: &[f64],
    by_length: bool,
    connect_internal: bool,
) -> EdgeSequence {
    let frac: Vec<f64> = fractions.iter().map(|f| f.abs()).collect();
    let fsum: f64 = frac.iter().sum();
    assert!(
        close_enough(fsum, 1.0, 1e-4),
        "Edge Subdivision::ERROR::fraction is incorrect. The sum {fsum} is not 1"
    );

    let kind = edge.borrow().kind.clone();
    let seq = match kind {
        EdgeKind::Line => subdivide_line(edge, &frac),
        // For circular arcs the parametrisation is uniform in arc length, so
        // both requests take the same path.
        EdgeKind::Circle { .. } | EdgeKind::Curve { .. } => {
            subdivide_curved(edge, &frac, by_length)
        }
    };

    if connect_internal {
        for i in 1..seq.len() {
            let prev_end = seq[i - 1].borrow().end.clone();
            seq[i].borrow_mut().start = prev_end;
        }
    }
    seq
}

fn subdivide_line(edge: &EdgeRef, frac: &[f64]) -> EdgeSequence {
    let e = edge.borrow();
    let vec = sub2(e.end_p(), e.start_p());

    let mut verts: Vec<Vert> = vec![e.start.clone()];
    let mut seq = EdgeSequence::new();
    for f in &frac[..frac.len() - 1] {
        let last = vget(verts.last().unwrap());
        let next = vert([last[0] + f * vec[0], last[1] + f * vec[1]]);
        seq.push(Edge::line_v(verts.last().unwrap().clone(), next.clone()));
        verts.push(next);
    }
    seq.push(Edge::line_v(verts.last().unwrap().clone(), e.end.clone()));
    seq
}

fn subdivide_curved(edge: &EdgeRef, frac: &[f64], by_length: bool) -> EdgeSequence {
    let e = edge.borrow();
    let curve = e.as_curve();

    let mut subcurves = Vec::new();
    if by_length && matches!(e.kind, EdgeKind::Curve { .. }) {
        let clen = curve.length();
        let (mut covered, mut prev_t) = (0.0, 0.0);
        for f in frac {
            covered += f;
            let next_t = if covered >= 1.0 {
                1.0
            } else {
                curve.ilength(clen * covered, ILENGTH_S_TOL)
            };
            subcurves.push(curve.cropped(prev_t, next_t));
            prev_t = next_t;
        }
    } else {
        let mut covered = 0.0;
        for f in frac {
            let next = (covered + f).min(1.0);
            subcurves.push(curve.cropped(covered, next));
            covered = next;
        }
    }

    let mut seq = EdgeSequence::new();
    for c in subcurves {
        seq.push(from_svg_curve(&c));
    }
    // Re-attach the outer endpoints to the original vertex objects.
    seq[0].borrow_mut().start = e.start.clone();
    let last = seq.len() - 1;
    seq[last].borrow_mut().end = e.end.clone();
    seq
}

/// `EdgeFactory.from_svg_curve` -- build an edge from a curve segment.
pub fn from_svg_curve(seg: &Curve) -> EdgeRef {
    match seg {
        Curve::Line { start, end } => Edge::line(*start, *end),
        Curve::Arc(a) => {
            // Circular arcs only, as in the reference.
            crate::garment::edge_factory::circle_from_points_radius(
                a.start,
                a.end,
                a.radius[0],
                a.large_arc,
                a.sweep,
            )
        }
        Curve::Quad {
            start,
            control,
            end,
        } => Edge::curve(*start, *end, vec![*control], false),
        Curve::Cubic { start, c1, c2, end } => Edge::curve(*start, *end, vec![*c1, *c2], false),
    }
}

/// Linear approximation of an edge, sharing its endpoint vertices.
pub fn linearize(edge: &EdgeRef, n_verts_inside: usize) -> EdgeSequence {
    let e = edge.borrow();
    if n_verts_inside == 0 && matches!(e.kind, EdgeKind::Line) {
        // The reference returns the edge itself here, so downstream vertex
        // mutations still land on the original.
        return EdgeSequence::from_edges(vec![edge.clone()]);
    }

    let n = n_verts_inside + 1;
    let curve = e.as_curve();
    let interior: Vec<V2> = (1..n).map(|i| curve.point(i as f64 / n as f64)).collect();
    to_edge_sequence(&e, &interior)
}

/// Linearization using this edge kind's default resolution.
pub fn linearize_default(edge: &EdgeRef) -> EdgeSequence {
    let n = edge.borrow().default_linearization();
    linearize(edge, n)
}

/// Chain of straight edges through `interior`, keeping the original endpoints.
fn to_edge_sequence(e: &Edge, interior: &[V2]) -> EdgeSequence {
    let mut seq = EdgeSequence::new();
    let mut prev = e.start.clone();
    for p in interior {
        let v = vert(*p);
        seq.push(Edge::line_v(prev, v.clone()));
        prev = v;
    }
    seq.push(Edge::line_v(prev, e.end.clone()));
    seq
}

// ----- EdgeSequence -----

/// A (usually chained) run of edges.
#[derive(Debug, Clone, Default)]
pub struct EdgeSequence {
    pub edges: Vec<EdgeRef>,
}

impl std::ops::Index<usize> for EdgeSequence {
    type Output = EdgeRef;
    fn index(&self, i: usize) -> &EdgeRef {
        &self.edges[i]
    }
}

impl EdgeSequence {
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    pub fn from_edges(edges: Vec<EdgeRef>) -> Self {
        Self { edges }
    }

    pub fn one(edge: EdgeRef) -> Self {
        Self { edges: vec![edge] }
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    pub fn first(&self) -> &EdgeRef {
        &self.edges[0]
    }

    pub fn last(&self) -> &EdgeRef {
        self.edges.last().unwrap()
    }

    /// Index of an edge *by identity*.
    pub fn index_of(&self, e: &EdgeRef) -> Option<usize> {
        self.edges.iter().position(|x| esame(x, e))
    }

    pub fn contains(&self, e: &EdgeRef) -> bool {
        self.index_of(e).is_some()
    }

    /// A sub-range, sharing the same edge objects.
    pub fn slice(&self, from: usize, to: usize) -> EdgeSequence {
        EdgeSequence::from_edges(self.edges[from..to].to_vec())
    }

    pub fn push(&mut self, e: EdgeRef) {
        self.edges.push(e);
    }

    pub fn extend(&mut self, other: &EdgeSequence) {
        self.edges.extend(other.edges.iter().cloned());
    }

    pub fn insert(&mut self, i: usize, other: &EdgeSequence) {
        for (j, e) in other.edges.iter().enumerate() {
            self.edges.insert(i + j, e.clone());
        }
    }

    pub fn insert_one(&mut self, i: usize, e: EdgeRef) {
        self.edges.insert(i, e);
    }

    pub fn pop_at(&mut self, i: usize) {
        self.edges.remove(i);
    }

    pub fn pop_edge(&mut self, e: &EdgeRef) {
        if let Some(i) = self.index_of(e) {
            self.edges.remove(i);
        }
    }

    /// Replace the edge at `i` with a whole sequence.
    pub fn substitute_at(&mut self, i: usize, new: &EdgeSequence) {
        self.edges.remove(i);
        self.insert(i, new);
    }

    /// Replace `orig` (matched by identity) with a sequence.
    pub fn substitute(&mut self, orig: &EdgeRef, new: &EdgeSequence) {
        let i = self
            .index_of(orig)
            .expect("EdgeSequence::substitute::ERROR::edge is not in this sequence");
        self.substitute_at(i, new);
    }

    pub fn substitute_one(&mut self, orig: &EdgeRef, new: EdgeRef) {
        self.substitute(orig, &EdgeSequence::one(new));
    }

    pub fn length(&self) -> f64 {
        self.edges.iter().map(|e| e.borrow().length()).sum()
    }

    pub fn lengths(&self) -> Vec<f64> {
        self.edges.iter().map(|e| e.borrow().length()).collect()
    }

    pub fn fractions(&self) -> Vec<f64> {
        let total = self.length();
        self.lengths().into_iter().map(|l| l / total).collect()
    }

    pub fn is_loop(&self) -> bool {
        self.len() > 1 && vsame(&self.first().borrow().start, &self.last().borrow().end)
    }

    pub fn is_chained(&self) -> bool {
        if self.len() < 2 {
            return false;
        }
        (1..self.len()).all(|i| {
            vsame(
                &self.edges[i].borrow().start,
                &self.edges[i - 1].borrow().end,
            )
        })
    }

    /// Every vertex object in the sequence, without double-counting shared ones.
    pub fn verts(&self) -> Vec<Vert> {
        let mut verts: Vec<Vert> = vec![self.first().borrow().start.clone()];
        for e in &self.edges {
            let e = e.borrow();
            if !vsame(&e.start, verts.last().unwrap()) {
                verts.push(e.start.clone());
            }
            verts.push(e.end.clone());
        }
        if verts.len() > 1 && vsame(&verts[0], verts.last().unwrap()) {
            verts.pop();
        }
        verts
    }

    /// `[start of first, end of last]`.
    pub fn shortcut(&self) -> [V2; 2] {
        [
            self.first().borrow().start_p(),
            self.last().borrow().end_p(),
        ]
    }

    /// 2D bounding box plus the vertices that sit on it.
    ///
    /// Returns `([min_x, max_x, min_y, max_y], boundary_points)`.
    pub fn bbox(&self) -> ([f64; 4], Vec<V2>) {
        let mut lin = EdgeSequence::new();
        for e in &self.edges {
            lin.extend(&linearize_default(e));
        }
        let verts: Vec<V2> = lin.verts().iter().map(vget).collect();

        let mi = [
            verts.iter().map(|v| v[0]).fold(f64::INFINITY, f64::min),
            verts.iter().map(|v| v[1]).fold(f64::INFINITY, f64::min),
        ];
        let ma = [
            verts.iter().map(|v| v[0]).fold(f64::NEG_INFINITY, f64::max),
            verts.iter().map(|v| v[1]).fold(f64::NEG_INFINITY, f64::max),
        ];

        let mut b_points: Vec<V2> = verts
            .iter()
            .filter(|v| v[0] == mi[0] || v[0] == ma[0] || v[1] == mi[1] || v[1] == ma[1])
            .cloned()
            .collect();

        if b_points.len() == 2 {
            // Two boundary points cannot describe a normal; add a corner.
            let extra = if b_points.contains(&mi) {
                [mi[0], ma[1]]
            } else {
                mi
            };
            b_points = vec![b_points[0], extra, b_points[1]];
        }

        ([mi[0], ma[0], mi[1], ma[1]], b_points)
    }

    // ----- modifiers -----

    pub fn reverse(&mut self) {
        self.edges.reverse();
        for e in &self.edges {
            e.borrow_mut().reverse();
        }
    }

    /// Reverse the order only, leaving edge directions alone.
    pub fn reverse_order(&mut self) {
        self.edges.reverse();
    }

    pub fn translate_by(&self, shift: V2) {
        for v in self.verts() {
            let p = vget(&v);
            vset(&v, [p[0] + shift[0], p[1] + shift[1]]);
        }
    }

    pub fn snap_to(&self, new_origin: V2) -> &Self {
        let start = self.first().borrow().start_p();
        self.translate_by([new_origin[0] - start[0], new_origin[1] - start[1]]);
        self
    }

    /// Close the loop with a straight edge, if it is not closed already.
    pub fn close_loop(&mut self) {
        if !self.is_loop() {
            let a = self.last().borrow().end.clone();
            let b = self.first().borrow().start.clone();
            self.push(Edge::line_v(a, b));
        }
    }

    pub fn rotate(&self, angle: f64) {
        let curr_start = self.first().borrow().start_p();
        self.snap_to([0.0, 0.0]);
        let m = r2d(angle);
        for v in self.verts() {
            vset(&v, apply_r2d(m, vget(&v)));
        }
        self.snap_to(curr_start);
    }

    /// Stretch (or shrink) along the line from the first start to the last end,
    /// keeping the first vertex fixed.
    pub fn extend_by(&self, factor: f64) {
        let chained_owned;
        let chained = if self.is_chained() {
            self
        } else {
            chained_owned = self.chained_order();
            if chained_owned.is_loop() {
                // Looped sequences have no meaningful extension direction.
                return;
            }
            &chained_owned
        };

        let target_line = normalize2(sub2(
            chained.last().borrow().end_p(),
            chained.first().borrow().start_p(),
        ));

        let verts = self.verts();
        let coords: Vec<V2> = verts.iter().map(vget).collect();
        let fixed = coords[0];

        for (i, v) in verts.iter().enumerate() {
            let proj = scale2(target_line, dot2(sub2(coords[i], fixed), target_line));
            vset(v, sub2(coords[i], scale2(proj, 1.0 - factor)));
        }
    }

    /// Reflect across the line through `v0` and `v1`.
    pub fn reflect(&self, v0: V2, v1: V2) -> &Self {
        let vec = normalize2(sub2(v1, v0));
        let m = [
            [1.0 - 2.0 * vec[1] * vec[1], 2.0 * vec[0] * vec[1]],
            [2.0 * vec[0] * vec[1], -1.0 + 2.0 * vec[1] * vec[1]],
        ];

        for v in self.verts() {
            let p = sub2(vget(&v), v0);
            vset(&v, add2(apply_r2d(m, p), v0));
        }
        for e in &self.edges {
            e.borrow_mut().reflect_features();
        }
        self
    }

    pub fn propagate_label(&self, label: &str) {
        for e in &self.edges {
            e.borrow_mut().label = label.to_string();
        }
    }

    // ----- copies -----

    /// A deep copy that preserves vertex sharing between neighbouring edges,
    /// matching Python's memoised `deepcopy`.
    pub fn copy(&self) -> EdgeSequence {
        let mut memo: HashMap<usize, Vert> = HashMap::new();
        let mut copy_vert = |v: &Vert| -> Vert {
            let key = Rc::as_ptr(v) as usize;
            memo.entry(key).or_insert_with(|| vert(vget(v))).clone()
        };

        let edges = self
            .edges
            .iter()
            .map(|e| {
                let e = e.borrow();
                Rc::new(RefCell::new(Edge {
                    start: copy_vert(&e.start),
                    end: copy_vert(&e.end),
                    label: e.label.clone(),
                    geometric_id: e.geometric_id,
                    kind: e.kind.clone(),
                }))
            })
            .collect();

        EdgeSequence::from_edges(edges)
    }

    /// A copy with edge directions aligned into a chain.
    ///
    /// Useful when the edges were reversed externally and the sequence lost its
    /// chaining property.
    pub fn chained_order(&self) -> EdgeSequence {
        let chained = self.copy();

        for i in 0..chained.len() {
            // Already sorted up to i-1: if this edge ends where the previous
            // one ends, it is pointing backwards.
            let follows_previous =
                i > 0 && vsame(&chained[i].borrow().end, &chained[i - 1].borrow().end);

            // Otherwise, if it *starts* at the next edge, it is backwards too.
            let reverse = follows_previous
                || (i + 1 < chained.len() && {
                    let s = chained[i].borrow().start.clone();
                    vsame(&s, &chained[i + 1].borrow().start)
                        || vsame(&s, &chained[i + 1].borrow().end)
                });

            if reverse {
                chained[i].borrow_mut().reverse();
            }
        }
        chained
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_vertices_move_together() {
        let a = Edge::line([0.0, 0.0], [1.0, 0.0]);
        let b = Edge::line_v(a.borrow().end.clone(), vert([1.0, 1.0]));

        // Moving the shared vertex through `a` must be visible from `b`.
        vset(&a.borrow().end, [2.0, 0.0]);
        assert_eq!(b.borrow().start_p(), [2.0, 0.0]);
        assert!((b.borrow().length() - 1.0_f64.hypot(2.0 - 1.0)).abs() < 1e-12);
    }

    #[test]
    fn snap_to_moves_both_ends() {
        let e = Edge::line([1.0, 1.0], [4.0, 5.0]);
        e.borrow().snap_to([0.0, 0.0]);
        assert_eq!(e.borrow().start_p(), [0.0, 0.0]);
        assert_eq!(e.borrow().end_p(), [3.0, 4.0]);
    }

    #[test]
    fn copy_preserves_sharing_but_not_identity() {
        let mut seq = EdgeSequence::new();
        let a = Edge::line([0.0, 0.0], [1.0, 0.0]);
        let b = Edge::line_v(a.borrow().end.clone(), vert([1.0, 1.0]));
        seq.push(a.clone());
        seq.push(b);

        let c = seq.copy();
        assert!(c.is_chained(), "copy lost the chain");
        assert!(!vsame(&c[0].borrow().end, &seq[0].borrow().end));

        // Moving the copy must not disturb the original.
        c.translate_by([10.0, 0.0]);
        assert_eq!(a.borrow().start_p(), [0.0, 0.0]);
        assert_eq!(c[0].borrow().start_p(), [10.0, 0.0]);
    }

    /// `control_y == relative radius` gives a semicircle.
    ///
    /// The expected values are the reference implementation's own, taken from
    /// `CircleEdge([0,0], [2,0], cy=0.5)`. They are a hair off pi: the
    /// circumscribed-radius formula lands on 0.5000000000000003, and `arcsin`
    /// near 1 amplifies that to ~1e-8. Matching the reference here matters more
    /// than matching pi, so the test pins its numbers.
    #[test]
    fn circle_edge_half_turn() {
        let e = Edge::circle([0.0, 0.0], [2.0, 0.0], 0.5);
        assert!((e.borrow().rel_radius() - 0.5000000000000003).abs() < 1e-15);
        assert!(
            (e.borrow().arc_angle() - 3.14159258058931).abs() < 1e-13,
            "{}",
            e.borrow().arc_angle()
        );
        assert!(
            (e.borrow().length() - 3.1415925805893123).abs() < 1e-12,
            "{}",
            e.borrow().length()
        );
        assert_eq!(e.borrow().midpoint(), [1.0, 1.0]);
    }

    #[test]
    fn subdivision_lengths_add_up() {
        let e = Edge::curve([0.0, 0.0], [10.0, 0.0], vec![[0.5, 0.4]], true);
        let total = e.borrow().length();
        let sub = subdivide_len(&e, &[0.3, 0.7], true);
        assert_eq!(sub.len(), 2);
        assert!((sub[0].borrow().length() - 0.3 * total).abs() < 1e-6);
        assert!((sub.length() - total).abs() < 1e-6);
        // Outer endpoints stay the original vertex objects.
        assert!(vsame(&sub[0].borrow().start, &e.borrow().start));
        assert!(vsame(&sub[1].borrow().end, &e.borrow().end));
        // ... and the internal vertex is shared.
        assert!(vsame(&sub[0].borrow().end, &sub[1].borrow().start));
    }

    #[test]
    fn reverse_roundtrips() {
        let e = Edge::curve([0.0, 0.0], [4.0, 1.0], vec![[0.3, 0.2], [0.7, -0.1]], true);
        let before = e.borrow().length();
        let mid_before = e.borrow().midpoint();
        e.borrow_mut().reverse();
        assert!((e.borrow().length() - before).abs() < 1e-9);
        assert!(dist2(e.borrow().midpoint(), mid_before) < 1e-9);
        e.borrow_mut().reverse();
        assert_eq!(e.borrow().start_p(), [0.0, 0.0]);
    }

    #[test]
    fn extend_scales_along_the_shortcut() {
        let mut seq = EdgeSequence::new();
        let a = Edge::line([0.0, 0.0], [2.0, 0.0]);
        let b = Edge::line_v(a.borrow().end.clone(), vert([4.0, 0.0]));
        seq.push(a);
        seq.push(b);

        seq.extend_by(2.0);
        assert!((seq.length() - 8.0).abs() < 1e-9, "{}", seq.length());
        assert_eq!(seq.first().borrow().start_p(), [0.0, 0.0]);
    }

    #[test]
    fn reflect_flips_curvature() {
        let e = Edge::circle([0.0, 0.0], [2.0, 0.0], 0.5);
        let seq = EdgeSequence::one(e.clone());
        seq.reflect([0.0, 0.0], [1.0, 0.0]);
        let EdgeKind::Circle { control_y } = e.borrow().kind else {
            panic!()
        };
        assert!((control_y + 0.5).abs() < 1e-12);
    }
}
