//! Components -- garment elements built from panels and other components.
//!
//! Ports `pygarment.garmentcode.component` and the placement routines from
//! `pygarment.garmentcode.base`.

use std::cell::RefCell;
use std::rc::Rc;

use crate::math::*;
use crate::pattern::spec::PatternSpec;

use super::connector::Stitches;
use super::interface::InterfaceRef;
use super::panel::{InterfaceMap, PanelRef};

/// How a component reports its length.
///
/// The reference overrides `length()` on most garment components; these are
/// the shapes those overrides take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LengthMode {
    /// Sum over subcomponents -- the default.
    Sum,
    /// One subcomponent's length. Bands use `Sub(0)`: their two panels sit side
    /// by side, not stacked.
    Sub(usize),
    /// The sum of a chosen few, e.g. a sleeve plus its cuff.
    SubSum(Vec<usize>),
    /// The length of a named interface's edges.
    Interface(String),
    /// Contributes nothing (a projected-shape-only collar, a sleeveless
    /// sleeve).
    Zero,
}

#[derive(Debug)]
pub struct Component {
    pub name: String,
    pub subs: Vec<Element>,
    pub interfaces: InterfaceMap,
    pub stitching_rules: Stitches,
    pub length_mode: LengthMode,
    /// Rise value, for components that sit on the waist/hip line.
    pub rise: Option<f64>,
}

pub type CompRef = Rc<RefCell<Component>>;

impl Component {
    pub fn new(name: &str) -> CompRef {
        Rc::new(RefCell::new(Component {
            name: name.to_string(),
            subs: Vec::new(),
            interfaces: InterfaceMap::new(),
            stitching_rules: Stitches::new(),
            length_mode: LengthMode::Sum,
            rise: None,
        }))
    }
}

/// Either a panel or a nested component -- the reference's `BaseComponent`.
#[derive(Debug, Clone)]
pub enum Element {
    Panel(PanelRef),
    Comp(CompRef),
}

impl From<PanelRef> for Element {
    fn from(p: PanelRef) -> Self {
        Element::Panel(p)
    }
}

impl From<CompRef> for Element {
    fn from(c: CompRef) -> Self {
        Element::Comp(c)
    }
}

impl Element {
    pub fn name(&self) -> String {
        match self {
            Element::Panel(p) => p.borrow().name.clone(),
            Element::Comp(c) => c.borrow().name.clone(),
        }
    }

    pub fn set_name(&self, name: &str) {
        match self {
            Element::Panel(p) => p.borrow_mut().name = name.to_string(),
            Element::Comp(c) => c.borrow_mut().name = name.to_string(),
        }
    }

    pub fn interfaces(&self) -> InterfaceMap {
        match self {
            Element::Panel(p) => p.borrow().interfaces.clone(),
            Element::Comp(c) => c.borrow().interfaces.clone(),
        }
    }

    pub fn interface(&self, key: &str) -> InterfaceRef {
        self.interfaces().get(key)
    }

    // ----- info -----

    pub fn pivot_3d(&self) -> V3 {
        match self {
            Element::Panel(p) => p.borrow().pivot_3d(),
            Element::Comp(_) => {
                // For a block, the pivot is the top-centre of its 3D bounds.
                // Relative pivots of sub-blocks must be preserved by any
                // placement operation.
                let (lo, hi) = self.bbox3d();
                [(lo[0] + hi[0]) / 2.0, hi[1], (lo[2] + hi[2]) / 2.0]
            }
        }
    }

    pub fn bbox3d(&self) -> (V3, V3) {
        match self {
            Element::Panel(p) => p.borrow().bbox3d(),
            Element::Comp(c) => {
                let subs = &c.borrow().subs;
                if subs.is_empty() {
                    // A component without panel geometry has no bbox.
                    return ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
                }
                let mut lo = [f64::INFINITY; 3];
                let mut hi = [f64::NEG_INFINITY; 3];
                for s in subs {
                    let (l, h) = s.bbox3d();
                    lo = min3(lo, l);
                    hi = max3(hi, h);
                }
                (lo, hi)
            }
        }
    }

    pub fn length(&self) -> f64 {
        match self {
            Element::Panel(p) => p.borrow().length(false),
            Element::Comp(c) => {
                let c = c.borrow();
                match &c.length_mode {
                    LengthMode::Zero => 0.0,
                    LengthMode::Sub(i) => c.subs.get(*i).map(|s| s.length()).unwrap_or(0.0),
                    LengthMode::SubSum(ids) => ids
                        .iter()
                        .filter_map(|i| c.subs.get(*i))
                        .map(|s| s.length())
                        .sum(),
                    LengthMode::Interface(key) => c.interfaces.get(key).borrow().edges.length(),
                    LengthMode::Sum => c.subs.iter().map(|s| s.length()).sum(),
                }
            }
        }
    }

    pub fn is_self_intersecting(&self) -> bool {
        match self {
            Element::Panel(p) => p.borrow().is_self_intersecting(),
            Element::Comp(c) => c.borrow().subs.iter().any(|s| s.is_self_intersecting()),
        }
    }

    // ----- operations -----

    pub fn set_panel_label(&self, label: &str, overwrite: bool) {
        match self {
            Element::Panel(p) => p.borrow_mut().set_panel_label(label, overwrite),
            Element::Comp(c) => {
                for s in &c.borrow().subs {
                    s.set_panel_label(label, overwrite);
                }
            }
        }
    }

    pub fn translate_by(&self, delta: V3) {
        match self {
            Element::Panel(p) => p.borrow_mut().translate_by(delta),
            Element::Comp(c) => {
                let subs = c.borrow().subs.clone();
                for s in subs {
                    s.translate_by(delta);
                }
            }
        }
    }

    pub fn translate_to(&self, new_translation: V3) {
        match self {
            Element::Panel(p) => p.borrow_mut().translate_to(new_translation),
            Element::Comp(c) => {
                let pivot = self.pivot_3d();
                let subs = c.borrow().subs.clone();
                for s in subs {
                    let sub_pivot = s.pivot_3d();
                    s.translate_to(add3(new_translation, sub3(sub_pivot, pivot)));
                }
            }
        }
    }

    pub fn rotate_by(&self, delta: Rotation) {
        match self {
            Element::Panel(p) => p.borrow_mut().rotate_by(delta),
            Element::Comp(c) => {
                let pivot = self.pivot_3d();
                let subs = c.borrow().subs.clone();
                for s in subs {
                    // Preserve the relationships between subcomponents.
                    let rel = sub3(s.pivot_3d(), pivot);
                    let rel_rotated = delta.apply(rel);
                    s.rotate_by(delta);
                    s.translate_by(sub3(rel_rotated, rel));
                }
            }
        }
    }

    pub fn mirror(&self, axis: V2) {
        match self {
            Element::Panel(p) => p.borrow_mut().mirror(axis),
            Element::Comp(c) => {
                let subs = c.borrow().subs.clone();
                for s in subs {
                    s.mirror(axis);
                }
            }
        }
    }

    /// Place this element directly below `other`, leaving `gap` between them.
    pub fn place_below(&self, other: &Element, gap: f64) {
        let other_bbox = other.bbox3d();
        let curr_bbox = self.bbox3d();
        self.translate_by([0.0, other_bbox.0[1] - curr_bbox.1[1] - gap, 0.0]);
    }

    /// Move this element so that `self_interface` meets `out_interface`.
    ///
    /// `alignment` is one of `center`, `top`, `bottom`, `left`, `right`.
    pub fn place_by_interface(
        &self,
        self_interface: &InterfaceRef,
        out_interface: &InterfaceRef,
        gap: f64,
        alignment: Alignment,
        gap_dir: Option<V3>,
    ) {
        let self_bbox = self_interface.borrow().bbox_3d();
        let out_bbox = out_interface.borrow().bbox_3d();

        let mut point_out = scale3(add3(out_bbox.1, out_bbox.0), 0.5);
        let mut point_self = scale3(add3(self_bbox.1, self_bbox.0), 0.5);

        match alignment {
            Alignment::Center => {}
            Alignment::Top => {
                point_out[1] = out_bbox.1[1];
                point_self[1] = self_bbox.1[1];
            }
            Alignment::Bottom => {
                point_out[1] = out_bbox.0[1];
                point_self[1] = self_bbox.0[1];
            }
            Alignment::Right => {
                point_out[0] = out_bbox.0[0];
                point_self[0] = self_bbox.0[0];
            }
            Alignment::Left => {
                point_out[0] = out_bbox.1[0];
                point_self[0] = self_bbox.1[0];
            }
        }

        // Leave a gap outside the current element.
        let gap_dir = gap_dir.unwrap_or_else(|| {
            let full_bbox = self.bbox3d();
            let center = scale3(add3(full_bbox.0, full_bbox.1), 0.5);
            let mid_self = scale3(add3(self_bbox.1, self_bbox.0), 0.5);
            sub3(mid_self, center)
        });
        let gap_dir = scale3(gap_dir, gap / norm3(gap_dir));

        self.translate_by(sub3(point_out, add3(point_self, gap_dir)));
    }

    // ----- assembly -----

    /// Build the serializable pattern for this element and everything under it.
    pub fn assembly(&self) -> PatternSpec {
        match self {
            Element::Panel(p) => {
                let mut spec = p.borrow_mut().assembly_geometry();
                // A panel can carry inner stitches (darts); they are assembled
                // once the exclusive borrow above has ended.
                let rules = p.borrow().stitching_rules.clone();
                spec.stitches = rules.assembly();
                spec
            }
            Element::Comp(c) => {
                let (name, subs) = {
                    let c = c.borrow();
                    (c.name.clone(), c.subs.clone())
                };

                let mut spec = PatternSpec::empty();
                spec.name = name;
                if subs.is_empty() {
                    return spec;
                }

                for sub in &subs {
                    spec.merge(sub.assembly());
                }

                // Own rules come last, once every panel has a geometric id.
                let rules = c.borrow().stitching_rules.assembly();
                spec.stitches.extend(rules);
                spec
            }
        }
    }
}

/// Interface alignment options for [`Element::place_by_interface`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Center,
    Top,
    Bottom,
    Left,
    Right,
}
