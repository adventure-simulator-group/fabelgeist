//! Panels -- one flat piece of fabric.
//!
//! Ports `pygarment.garmentcode.panel`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::curve::Curve;
use crate::math::*;
use crate::pattern::spec::{EdgeSpec, PanelSpec, PatternSpec};

use super::connector::Stitches;
use super::edge::{EdgeKind, EdgeSequence, linearize, linearize_default, vget, vset};
use super::interface::InterfaceRef;

/// Named interfaces, in insertion order.
#[derive(Debug, Clone, Default)]
pub struct InterfaceMap {
    entries: Vec<(String, InterfaceRef)>,
}

impl InterfaceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: &str, value: InterfaceRef) {
        match self.entries.iter_mut().find(|(k, _)| k == key) {
            Some((_, slot)) => *slot = value,
            None => self.entries.push((key.to_string(), value)),
        }
    }

    pub fn try_get(&self, key: &str) -> Option<InterfaceRef> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    /// Panics with the reference's style of message when the key is missing.
    pub fn get(&self, key: &str) -> InterfaceRef {
        self.try_get(key)
            .unwrap_or_else(|| panic!("Interfaces::ERROR::no interface named '{key}'"))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn values(&self) -> impl Iterator<Item = &InterfaceRef> {
        self.entries.iter().map(|(_, v)| v)
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|(k, _)| k.as_str())
    }
}

/// How a panel reports its length.
///
/// Most panels use their 2D bounding box; sleeve and circle-arc panels report
/// the length of a specific interface instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PanelLength {
    BoundingBox,
    Interface(String),
}

/// How a bodice panel computes its width at a given level.
///
/// The level is measured down from the top of the panel; the width is taken
/// from the slope of the panel's outside edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WidthRule {
    /// Follow the outside edge's slope from half the shoulder width.
    Slope { shoulder_w: f64 },
    /// As `Slope`, plus the panel's own half-width offset -- used by the
    /// non-fitted (T-shirt) torso panels, which are wider than the shoulder.
    SlopeOffset { shoulder_w: f64, width: f64 },
    /// A fixed width, regardless of level -- the fitted bodice's back panel.
    Constant(f64),
}

#[derive(Debug)]
pub struct Panel {
    /// Unique identifier of the panel.
    pub name: String,
    /// Additional, non-unique label (e.g. a body-segment tag).
    pub label: String,
    pub translation: V3,
    pub rotation: Rotation,
    pub edges: EdgeSequence,
    pub interfaces: InterfaceMap,
    pub stitching_rules: Stitches,
    pub verbose: bool,
    pub length_source: PanelLength,
    pub width_rule: Option<WidthRule>,
}

/// Panels are referenced from interfaces and components alike.
pub type PanelRef = Rc<RefCell<Panel>>;

impl Panel {
    pub fn new(name: &str) -> PanelRef {
        Rc::new(RefCell::new(Panel {
            name: name.to_string(),
            label: String::new(),
            translation: [0.0, 0.0, 0.0],
            rotation: Rotation::identity(),
            edges: EdgeSequence::new(),
            interfaces: InterfaceMap::new(),
            stitching_rules: Stitches::new(),
            verbose: false,
            length_source: PanelLength::BoundingBox,
            width_rule: None,
        }))
    }

    /// Panel width at `level`, counted from the top of the panel.
    ///
    /// The reference only defines this for bodice panels; for a fitted bodice
    /// it is valid between 0 and the bust level.
    pub fn get_width(&self, level: f64) -> f64 {
        let rule = self
            .width_rule
            .unwrap_or_else(|| panic!("Panel::{}::ERROR::get_width has no rule", self.name));

        match rule {
            WidthRule::Constant(w) => w,
            WidthRule::Slope { shoulder_w } => self.slope_width(level, shoulder_w),
            WidthRule::SlopeOffset { shoulder_w, width } => {
                self.slope_width(level, shoulder_w) + width - shoulder_w / 2.0
            }
        }
    }

    fn slope_width(&self, level: f64, shoulder_w: f64) -> f64 {
        // Assumes the top edge is as wide as the shoulder.
        let side_edge = self.interfaces.get("outside");
        let side_edge = side_edge.borrow();
        let e = side_edge.edges.last().borrow();

        let mut x = e.end_p()[0] - e.start_p()[0];
        let mut y = e.end_p()[1] - e.start_p()[1];
        // Flip if the edge points downwards instead of up.
        if y < 0.0 {
            x = -x;
            y = -y;
        }

        (level * x / y) + shoulder_w / 2.0
    }

    // ----- info -----

    pub fn pivot_3d(&self) -> V3 {
        self.point_to_3d([0.0, 0.0])
    }

    /// 3D position of a point given in the panel's local 2D frame.
    pub fn point_to_3d(&self, point_2d: V2) -> V3 {
        let p = [point_2d[0], point_2d[1], 0.0];
        add3(self.rotation.apply(p), self.translation)
    }

    /// Panel size in cm: the vertical extent of the 2D bounding box, or the
    /// longest dimension when `longest_dim` is set.
    ///
    /// Panels with a [`PanelLength::Interface`] source report that interface's
    /// edge length instead, ignoring `longest_dim`.
    pub fn length(&self, longest_dim: bool) -> f64 {
        if let PanelLength::Interface(key) = &self.length_source {
            return self.interfaces.get(key).borrow().edges.length();
        }
        let (lo, hi) = self.bbox();
        let x = (hi[0] - lo[0]).abs();
        let y = (hi[1] - lo[1]).abs();
        if longest_dim { x.max(y) } else { y }
    }

    pub fn bbox(&self) -> (V2, V2) {
        let verts = self.linearized_verts();
        let mut lo = [f64::INFINITY; 2];
        let mut hi = [f64::NEG_INFINITY; 2];
        for v in verts {
            lo = [lo[0].min(v[0]), lo[1].min(v[1])];
            hi = [hi[0].max(v[0]), hi[1].max(v[1])];
        }
        (lo, hi)
    }

    pub fn bbox3d(&self) -> (V3, V3) {
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        for v in self.linearized_verts() {
            let p = self.point_to_3d(v);
            lo = min3(lo, p);
            hi = max3(hi, p);
        }
        (lo, hi)
    }

    fn linearized_verts(&self) -> Vec<V2> {
        let mut lin = EdgeSequence::new();
        for e in &self.edges.edges {
            lin.extend(&linearize_default(e));
        }
        lin.verts().iter().map(vget).collect()
    }

    /// Approximate panel centre (crude linearisation of curved edges).
    fn center_2d(&self, n_verts_inside: usize) -> V2 {
        let mut lin = EdgeSequence::new();
        for e in &self.edges.edges {
            lin.extend(&linearize(e, n_verts_inside));
        }
        let verts: Vec<V2> = lin.verts().iter().map(vget).collect();
        let n = verts.len() as f64;
        [
            verts.iter().map(|v| v[0]).sum::<f64>() / n,
            verts.iter().map(|v| v[1]).sum::<f64>() / n,
        ]
    }

    /// Outward normal of the panel in 3D.
    ///
    /// Averaging the normals of consecutive bounding-box vertices makes this
    /// work for non-convex panels too: the dominant direction wins.
    pub fn norm(&self) -> V3 {
        let (_, b_verts_2d) = self.edges.bbox();
        let b_verts_3d: Vec<V3> = b_verts_2d.iter().map(|v| self.point_to_3d(*v)).collect();

        let n = b_verts_3d.len() as f64;
        let b_center_3d = [
            b_verts_3d.iter().map(|v| v[0]).sum::<f64>() / n,
            b_verts_3d.iter().map(|v| v[1]).sum::<f64>() / n,
            b_verts_3d.iter().map(|v| v[2]).sum::<f64>() / n,
        ];

        let mut norms: Vec<V3> = Vec::new();
        for i in 0..b_verts_3d.len() {
            let v0 = b_verts_3d[i];
            let v1 = b_verts_3d[(i + 1) % b_verts_3d.len()];
            let cr = cross3(sub3(v0, b_center_3d), sub3(v1, b_center_3d));
            norms.push(scale3(cr, 1.0 / norm3(cr)));
        }

        let mut avg = [0.0; 3];
        for nv in &norms {
            avg = add3(avg, *nv);
        }
        avg = scale3(avg, 1.0 / norms.len() as f64);

        if close_to_zero(norm3(avg)) {
            // Indecisive averaging (thin arcs) -- fall back to the first normal.
            avg = norms[0];
        }

        let mut final_norm = scale3(avg, 1.0 / norm3(avg));
        for c in final_norm.iter_mut() {
            if c.abs() < 1e-8 {
                *c = 0.0;
            }
        }
        final_norm
    }

    /// Do any two edges of this panel cross?
    pub fn is_self_intersecting(&self) -> bool {
        let mut edge_curves: Vec<Curve> = Vec::new();
        for e in &self.edges.edges {
            if matches!(e.borrow().kind, EdgeKind::Circle { .. }) {
                // Arc intersections are unreliable, so approximate them.
                for seg in linearize(e, 10).edges {
                    edge_curves.push(seg.borrow().as_curve());
                }
            } else {
                edge_curves.push(e.borrow().as_curve());
            }
        }

        for i1 in 0..edge_curves.len() {
            for i2 in (i1 + 1)..edge_curves.len() {
                let hits = edge_curves[i1].intersect(&edge_curves[i2]);
                let real: Vec<_> = hits
                    .into_iter()
                    .filter(|(a, b)| {
                        let (t1, t2) = if b < a { (*b, *a) } else { (*a, *b) };
                        // Touching at a shared vertex is not an intersection.
                        !(close_to_zero(t1) && close_enough(t2, 1.0, TOL))
                    })
                    .collect();
                if !real.is_empty() {
                    return true;
                }
            }
        }
        false
    }

    // ----- operations -----

    pub fn set_panel_label(&mut self, label: &str, overwrite: bool) {
        if self.label.is_empty() || overwrite {
            self.label = label.to_string();
        }
    }

    /// Move the panel's local origin to `point_2d`.
    ///
    /// With `replicate_placement` the 3D position is adjusted so the panel does
    /// not jump.
    pub fn set_pivot(&mut self, point_2d: V2, replicate_placement: bool) {
        if replicate_placement {
            self.translation = self.point_to_3d(point_2d);
        }

        // NOTE: the reference truncates the shift to whole centimetres here.
        let shift = [point_2d[0].trunc(), point_2d[1].trunc()];
        for v in self.edges.verts() {
            let p = vget(&v);
            vset(&v, [p[0] - shift[0], p[1] - shift[1]]);
        }
    }

    /// Put the pivot at the middle of whichever bounding-box side is highest
    /// in 3D -- the most useful default.
    pub fn top_center_pivot(&mut self) {
        let verts: Vec<V2> = self.edges.verts().iter().map(vget).collect();
        let top_right = [
            verts.iter().map(|v| v[0]).fold(f64::NEG_INFINITY, f64::max),
            verts.iter().map(|v| v[1]).fold(f64::NEG_INFINITY, f64::max),
        ];
        let low_left = [
            verts.iter().map(|v| v[0]).fold(f64::INFINITY, f64::min),
            verts.iter().map(|v| v[1]).fold(f64::INFINITY, f64::min),
        ];
        let mid_x = (top_right[0] + low_left[0]) / 2.0;
        let mid_y = (top_right[1] + low_left[1]) / 2.0;
        let mid_points_2d = [
            [mid_x, top_right[1]],
            [mid_x, low_left[1]],
            [top_right[0], mid_y],
            [low_left[0], mid_y],
        ];

        // NOTE: first maximum, not last -- see `pattern::core::argmax_first`.
        let heights: Vec<f64> = mid_points_2d
            .iter()
            .map(|p| self.point_to_3d(*p)[1])
            .collect();
        let best = crate::pattern::core::argmax_first(&heights);

        self.set_pivot(mid_points_2d[best], false);
    }

    pub fn translate_by(&mut self, delta: V3) {
        self.translation = add3(self.translation, delta);
        self.autonorm();
    }

    pub fn translate_to(&mut self, new_translation: V3) {
        self.translation = new_translation;
        self.autonorm();
    }

    pub fn rotate_by(&mut self, delta: Rotation) {
        self.rotation = delta.mul(&self.rotation);
        self.autonorm();
    }

    pub fn rotate_to(&mut self, new_rot: Rotation) {
        self.rotation = new_rot;
        self.autonorm();
    }

    /// Rotate so the panel normal points along `vector`.
    pub fn rotate_align(&mut self, vector: V3) {
        let v = scale3(vector, 1.0 / norm3(vector));
        let n = self.norm();
        self.rotate_by(vector_align_3d(n, v));
    }

    /// Shift over X so the panel centre lines up with the body centre.
    pub fn center_x(&mut self) {
        let center_3d = self.point_to_3d(self.center_2d(3));
        self.translation[0] += -center_3d[0];
    }

    /// Flip the fabric side if the surface normal points towards the origin.
    ///
    /// Best called after the panel's translation is set.
    pub fn autonorm(&mut self) {
        let norm_dr = self.norm();
        if dot3(norm_dr, self.translation) < 0.0 {
            self.edges.reverse();
        }
    }

    /// Replace the panel with its mirror image about the Y axis.
    pub fn mirror(&mut self, axis: V2) {
        assert!(
            close_to_zero(axis[0]),
            "{}::ERROR::Mirroring over an arbitrary axis is not implemented",
            self.name
        );

        self.edges.reflect([0.0, 0.0], [0.0, 1.0]);
        self.translation[0] *= -1.0;

        let mut curr_euler = self.rotation.as_euler_xyz(false);
        curr_euler[1] *= -1.0;
        curr_euler[2] *= -1.0;
        self.rotate_to(Rotation::from_euler_xyz(curr_euler, false));

        self.autonorm();
    }

    // ----- assembly -----

    /// Convert the panel geometry to its serializable representation.
    ///
    /// The panel's edges are assumed to form a single closed loop.
    ///
    /// NOTE: this deliberately leaves `stitches` empty. A panel's own stitches
    /// (darts) name the panel itself, so assembling them needs a shared borrow
    /// while this method holds an exclusive one -- the caller adds them after
    /// this returns.
    pub fn assembly_geometry(&mut self) -> PatternSpec {
        // Always start the loop at the origin, for consistency between panels.
        let first_start = vget(&self.edges.first().borrow().start);
        self.set_pivot(first_start, true);

        let mut vertices: Vec<V2> = vec![vget(&self.edges.first().borrow().start)];
        let mut edges: Vec<EdgeSpec> = Vec::new();

        for i in 0..self.edges.len() {
            let (verts, mut edge) = assembly_edge(&self.edges[i]);

            let vert_shift = if *vertices.last().unwrap() == verts[0] {
                // Shares a location with the previous edge's end.
                vertices.extend_from_slice(&verts[1..]);
                vertices.len() - verts[1..].len() - 1
            } else {
                let shift = vertices.len();
                vertices.extend_from_slice(&verts);
                shift
            };

            edge.endpoints = [
                edge.endpoints[0] + vert_shift,
                edge.endpoints[1] + vert_shift,
            ];

            // Remember where this logical edge landed in the panel loop.
            self.edges[i].borrow_mut().geometric_id = edges.len();
            edges.push(edge);
        }

        // Close the loop.
        if vertices.len() > 1 && *vertices.last().unwrap() == vertices[0] {
            vertices.pop();
            let last = edges.len() - 1;
            edges[last].endpoints[1] = 0;
        }

        let panel = PanelSpec {
            translation: self.translation,
            rotation: self.rotation.as_euler_xyz(true),
            vertices,
            edges,
            label: if self.label.is_empty() {
                None
            } else {
                Some(self.label.clone())
            },
        };

        let mut spec = PatternSpec::empty();
        spec.name = self.name.clone();
        spec.panels.push((self.name.clone(), panel));
        spec
    }
}

/// `Edge.assembly` -- endpoints plus curvature/label properties.
fn assembly_edge(edge: &super::edge::EdgeRef) -> ([V2; 2], EdgeSpec) {
    let e = edge.borrow();
    let mut spec = EdgeSpec {
        endpoints: [0, 1],
        label: if e.label.is_empty() {
            None
        } else {
            Some(e.label.clone())
        },
        curvature: None,
    };

    match &e.kind {
        EdgeKind::Line => {}
        EdgeKind::Circle { .. } => {
            let (rad, large_arc, right) = e.as_radius_flag();
            spec.curvature = Some(crate::pattern::spec::Curvature::Circle {
                radius: rad,
                large_arc,
                right,
            });
        }
        EdgeKind::Curve { control_points } => {
            spec.curvature = Some(if control_points.len() == 1 {
                crate::pattern::spec::Curvature::Quadratic {
                    params: control_points.clone(),
                }
            } else {
                crate::pattern::spec::Curvature::Cubic {
                    params: control_points.clone(),
                }
            });
        }
    }

    ([e.start_p(), e.end_p()], spec)
}

/// Cache of panel names, used to keep generated names unique.
#[derive(Debug, Default)]
pub struct NameCounter {
    counts: HashMap<String, usize>,
}

impl NameCounter {
    pub fn next(&mut self, base: &str) -> String {
        let n = self.counts.entry(base.to_string()).or_insert(0);
        let name = if *n == 0 {
            base.to_string()
        } else {
            format!("{base}_{n}")
        };
        *n += 1;
        name
    }
}
