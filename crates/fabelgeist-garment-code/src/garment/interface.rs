//! Interfaces -- the parts of a panel or component that can take a stitch.
//!
//! Ports `pygarment.garmentcode.interface`.

use std::cell::RefCell;
use std::rc::Rc;

use crate::math::*;

use super::edge::{EdgeRef, EdgeSequence, Vert, esame, linearize_default, vget, vset};
use super::panel::PanelRef;

/// A run of interface edges sharing one ruffle coefficient.
#[derive(Debug, Clone)]
pub struct RuffleSection {
    pub coeff: f64,
    /// Half-open `[from, to)` range of edge indices.
    pub sec: [usize; 2],
}

/// A set of panel edges that connect to another interface as one unit.
#[derive(Debug, Clone)]
pub struct Interface {
    pub edges: EdgeSequence,
    /// The owning panel for each edge (an interface may span panels).
    pub panel: Vec<PanelRef>,
    /// Per-edge request to stitch this side's right face to the other's wrong
    /// face.
    pub right_wrong: Vec<bool>,
    /// Per-edge direction override, used when matching many-to-many stitches.
    pub edges_flipping: Vec<bool>,
    pub ruffle: Vec<RuffleSection>,
}

/// Interfaces are aliased freely -- a component's interface is often literally
/// a sub-panel's interface, and stitch matching mutates them in place.
pub type InterfaceRef = Rc<RefCell<Interface>>;

impl Interface {
    /// One ruffle coefficient over the whole interface.
    pub fn new(
        panel: &PanelRef,
        edges: EdgeSequence,
        ruffle: f64,
        right_wrong: bool,
    ) -> InterfaceRef {
        let n = edges.len();
        Rc::new(RefCell::new(Interface {
            panel: vec![panel.clone(); n],
            right_wrong: vec![right_wrong; n],
            edges_flipping: vec![false; n],
            ruffle: vec![RuffleSection {
                coeff: ruffle,
                sec: [0, n],
            }],
            edges,
        }))
    }

    /// Shorthand for a plain interface (no ruffles, right-to-right).
    pub fn plain(panel: &PanelRef, edges: EdgeSequence) -> InterfaceRef {
        Self::new(panel, edges, 1.0, false)
    }

    pub fn one(panel: &PanelRef, edge: EdgeRef) -> InterfaceRef {
        Self::new(panel, EdgeSequence::one(edge), 1.0, false)
    }

    /// An interface with no owning panel.
    ///
    /// Components use these to carry shapes that are only ever *projected* onto
    /// a panel -- sleeve armholes and collar necklines. They must not be used
    /// in a stitch, which needs a panel name per edge.
    pub fn detached(edges: EdgeSequence) -> InterfaceRef {
        let n = edges.len();
        Rc::new(RefCell::new(Interface {
            panel: Vec::new(),
            right_wrong: vec![false; n],
            edges_flipping: vec![false; n],
            ruffle: vec![RuffleSection {
                coeff: 1.0,
                sec: [0, n],
            }],
            edges,
        }))
    }

    /// Per-edge ruffle coefficients, collapsed into contiguous sections the
    /// same way the reference does.
    pub fn with_ruffles(panel: &PanelRef, edges: EdgeSequence, ruffle: &[f64]) -> InterfaceRef {
        assert_eq!(
            ruffle.len(),
            edges.len(),
            "Interface::ERROR::Ruffles and Edges don't match"
        );

        let mut sections = Vec::new();
        let mut last_coeff: Option<f64> = None;
        let mut last_start = 0usize;
        for (i, coef) in ruffle.iter().enumerate() {
            match last_coeff {
                None => last_coeff = Some(*coef),
                Some(prev) if prev == *coef => {}
                Some(prev) => {
                    sections.push(RuffleSection {
                        coeff: prev,
                        sec: [last_start, i],
                    });
                    last_start = i;
                    last_coeff = Some(*coef);
                }
            }
        }
        sections.push(RuffleSection {
            coeff: last_coeff.unwrap_or(1.0),
            sec: [last_start, ruffle.len()],
        });

        let n = edges.len();
        Rc::new(RefCell::new(Interface {
            panel: vec![panel.clone(); n],
            right_wrong: vec![false; n],
            edges_flipping: vec![false; n],
            ruffle: sections,
            edges,
        }))
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    pub fn panel_names(&self) -> Vec<String> {
        self.panel.iter().map(|p| p.borrow().name.clone()).collect()
    }

    pub fn needs_flipping(&self, i: usize) -> bool {
        self.edges_flipping[i]
    }

    /// The edge shape to project onto the other side of a stitch, with ruffles
    /// applied.
    pub fn projecting_edges(&self, on_oriented: bool) -> EdgeSequence {
        let projected = if on_oriented {
            self.oriented_edges()
        } else {
            self.edges.copy()
        };

        for r in &self.ruffle {
            if close_enough(r.coeff, 1.0, 1e-3) {
                continue;
            }
            let (sec0, sec1) = (r.sec[0], r.sec[1]);

            let separable =
                sec1 < projected.len() && projected.slice(sec1 - 1, sec1 + 1).is_chained();
            if !separable {
                projected.slice(sec0, sec1).extend_by(1.0 / r.coeff);
                continue;
            }

            // Keep the stretch from dragging the rest of the sequence along:
            // detach the shared vertex, extend, then move the tail back.
            let prev_edge = projected[sec1 - 1].clone();
            let next_edge = projected[sec1].clone();

            let common_v: Vert = {
                let p = prev_edge.borrow();
                let n = next_edge.borrow();
                if super::edge::vsame(&p.end, &n.end) || super::edge::vsame(&p.end, &n.start) {
                    p.end.clone()
                } else {
                    p.start.clone()
                }
            };
            let common_v_copy = super::edge::vert(vget(&common_v));

            let copy_to_end = super::edge::vsame(&common_v, &next_edge.borrow().end);
            if copy_to_end {
                next_edge.borrow_mut().end = common_v_copy.clone();
            } else {
                next_edge.borrow_mut().start = common_v_copy.clone();
            }

            projected.slice(sec0, sec1).extend_by(1.0 / r.coeff);

            let shift = sub2(vget(&common_v), vget(&common_v_copy));
            projected.slice(sec1, projected.len()).translate_by(shift);

            if copy_to_end {
                next_edge.borrow_mut().end = common_v.clone();
            } else {
                next_edge.borrow_mut().start = common_v.clone();
            }
            // Keep the detached vertex in step, in case anything still reads it.
            vset(&common_v_copy, vget(&common_v));
        }

        projected
    }

    /// Per-edge lengths after ruffle coefficients are applied.
    pub fn projecting_lengths(&self) -> Vec<f64> {
        let mut out = Vec::with_capacity(self.edges.len());
        for r in &self.ruffle {
            for i in r.sec[0]..r.sec[1] {
                let l = self.edges[i].borrow().length();
                out.push(if close_enough(r.coeff, 1.0, 1e-3) {
                    l
                } else {
                    l / r.coeff
                });
            }
        }
        out
    }

    pub fn projecting_fractions(&self) -> Vec<f64> {
        let lengths = self.projecting_lengths();
        let total: f64 = lengths.iter().sum();
        lengths.into_iter().map(|l| l / total).collect()
    }

    /// A copy of the interface edges, oriented along the interface direction.
    pub fn oriented_edges(&self) -> EdgeSequence {
        let oriented = self.edges.copy();
        for i in 0..self.edges.len() {
            if self.needs_flipping(i) {
                oriented[i].borrow_mut().reverse();
            }
        }
        oriented
    }

    /// 3D positions of the (unique) vertices in the interface.
    pub fn verts_3d(&self) -> Vec<V3> {
        let mut verts_2d: Vec<Vert> = Vec::new();
        let mut matching: Vec<PanelRef> = Vec::new();

        for (e, panel) in self.edges.edges.iter().zip(&self.panel) {
            let e = e.borrow();
            if !verts_2d.iter().any(|v| super::edge::vsame(&e.start, v)) {
                verts_2d.push(e.start.clone());
                matching.push(panel.clone());
            }
            if !verts_2d.iter().any(|v| super::edge::vsame(&e.end, v)) {
                verts_2d.push(e.end.clone());
                matching.push(panel.clone());
            }
        }

        verts_2d
            .iter()
            .zip(&matching)
            .map(|(v, p)| p.borrow().point_to_3d(vget(v)))
            .collect()
    }

    /// 3D bounding box, using linearized curves for accuracy.
    pub fn bbox_3d(&self) -> (V3, V3) {
        let mut verts_3d: Vec<V3> = Vec::new();
        for (e, panel) in self.edges.edges.iter().zip(&self.panel) {
            let lin = linearize_default(e);
            for v in lin.verts() {
                verts_3d.push(panel.borrow().point_to_3d(vget(&v)));
            }
        }

        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        for v in verts_3d {
            lo = min3(lo, v);
            hi = max3(hi, v);
        }
        (lo, hi)
    }

    // ----- updates -----

    /// Reverse the order of edges (optionally also flipping their directions).
    pub fn reverse(&mut self, with_edge_dir_reverse: bool) {
        self.edges.reverse_order();
        self.panel.reverse();
        self.edges_flipping.reverse();
        // NOTE: `right_wrong` is deliberately *not* reversed -- the reference
        // leaves it alone here.
        if with_edge_dir_reverse {
            for f in self.edges_flipping.iter_mut() {
                *f = !*f;
            }
        }

        let enum_len = self.edges.len();
        for r in self.ruffle.iter_mut() {
            let a = enum_len - r.sec[0];
            let b = enum_len - r.sec[1];
            r.sec = [b, a];
        }
    }

    /// Flip every edge's direction flag.
    pub fn flip_edges(&mut self) {
        for f in self.edges_flipping.iter_mut() {
            *f = !*f;
        }
    }

    pub fn set_right_wrong(&mut self, right_wrong: bool) {
        self.right_wrong = vec![right_wrong; self.edges.len()];
    }

    /// Move edges from `curr_ids` to `projected_ids`.
    pub fn reorder(&mut self, curr_ids: &[usize], projected_ids: &[usize]) {
        for (i, j) in curr_ids.iter().zip(projected_ids) {
            for r in &self.ruffle {
                if *i >= r.sec[0] && *i < r.sec[1] && (*j < r.sec[0] || *j >= r.sec[1]) {
                    panic!(
                        "Interface::ERROR::reordering between panel-related \
                         sub-segments is not supported"
                    );
                }
            }
        }

        let mut new_edges = EdgeSequence::new();
        let mut new_panel = Vec::new();
        let mut new_flip = Vec::new();
        let mut new_rw = Vec::new();

        for i in 0..self.panel.len() {
            let id = match curr_ids.iter().position(|c| *c == i) {
                Some(k) => projected_ids[k],
                None => i,
            };
            new_edges.push(self.edges[id].clone());
            new_flip.push(self.edges_flipping[id]);
            new_panel.push(self.panel[id].clone());
            new_rw.push(self.right_wrong[id]);
        }

        self.edges = new_edges;
        self.panel = new_panel;
        self.edges_flipping = new_flip;
        self.right_wrong = new_rw;
    }

    /// Replace the edge at index `orig` with `new_edges`, one panel per edge.
    pub fn substitute_at(
        &mut self,
        orig: usize,
        new_edges: &EdgeSequence,
        new_panels: &[PanelRef],
    ) {
        self.edges.substitute_at(orig, new_edges);

        self.panel.remove(orig);
        let curr_flip = self.edges_flipping.remove(orig);
        let curr_rw = self.right_wrong.remove(orig);

        for (j, p) in new_panels.iter().enumerate() {
            self.panel.insert(orig + j, p.clone());
            self.edges_flipping.insert(orig + j, curr_flip);
            self.right_wrong.insert(orig + j, curr_rw);
        }

        let ins_len = new_edges.len();
        if ins_len > 1 {
            for r in self.ruffle.iter_mut() {
                if r.sec[0] > orig {
                    r.sec[0] += ins_len - 1;
                }
                if r.sec[1] > orig {
                    r.sec[1] += ins_len - 1;
                }
            }
        }
    }

    /// Replace `orig` (matched by identity) with `new_edges`.
    pub fn substitute(
        &mut self,
        orig: &EdgeRef,
        new_edges: &EdgeSequence,
        new_panels: &[PanelRef],
    ) {
        let idx = self
            .edges
            .edges
            .iter()
            .position(|e| esame(e, orig))
            .expect("Interface::substitute::ERROR::edge is not in this interface");
        self.substitute_at(idx, new_edges, new_panels);
    }

    /// Concatenate interfaces, preserving each one's ruffle sections.
    ///
    /// Allows a single interface to span panels and mix ruffle rates.
    pub fn from_multiple(ints: &[InterfaceRef]) -> InterfaceRef {
        let mut edges = EdgeSequence::new();
        let mut panel = Vec::new();
        let mut right_wrong = Vec::new();
        let mut edges_flipping = Vec::new();
        let mut ruffle = Vec::new();

        for elem in ints {
            let elem = elem.borrow();
            let shift = edges.len();
            for r in &elem.ruffle {
                ruffle.push(RuffleSection {
                    coeff: r.coeff,
                    sec: [r.sec[0] + shift, r.sec[1] + shift],
                });
            }
            edges.extend(&elem.edges);
            panel.extend(elem.panel.iter().cloned());
            right_wrong.extend(elem.right_wrong.iter().cloned());
            edges_flipping.extend(elem.edges_flipping.iter().cloned());
        }

        Rc::new(RefCell::new(Interface {
            edges,
            panel,
            right_wrong,
            edges_flipping,
            ruffle,
        }))
    }
}

/// Chaining helpers, so that construction reads like the reference's
/// `Interface(...).reverse(True)`.
pub trait InterfaceExt {
    fn reversed(self, with_edge_dir_reverse: bool) -> Self;
    fn flipped(self) -> Self;
    fn right_to_wrong(self, value: bool) -> Self;
}

impl InterfaceExt for InterfaceRef {
    fn reversed(self, with_edge_dir_reverse: bool) -> Self {
        self.borrow_mut().reverse(with_edge_dir_reverse);
        self
    }

    fn flipped(self) -> Self {
        self.borrow_mut().flip_edges();
        self
    }

    fn right_to_wrong(self, value: bool) -> Self {
        self.borrow_mut().set_right_wrong(value);
        self
    }
}
