//! Shared imports and helpers for the garment programs.

pub use crate::design::{Body, Design, Value};
pub use crate::garment::component::{Alignment, CompRef, Component, Element, LengthMode};
pub use crate::garment::connector::Stitches;
pub use crate::garment::edge::{
    Edge, EdgeKind, EdgeRef, EdgeSequence, Vert, linearize, subdivide_len, subdivide_param, vget,
    vset,
};
pub use crate::garment::edge_factory::*;
pub use crate::garment::interface::{Interface, InterfaceExt, InterfaceRef};
pub use crate::garment::operators as ops;
pub use crate::garment::panel::{Panel, PanelLength, PanelRef, WidthRule};
pub use crate::math::*;

/// Waist / hip measurements adjusted for a given rise.
///
/// Ports `BaseBottoms.eval_rise`: at rise 1 the garment sits on the waist, at
/// lower values it sits progressively closer to the hip line.
pub struct Rise {
    pub waist: f64,
    pub hips_depth: f64,
    pub back_waist: f64,
}

pub fn eval_rise(body: &Body, rise: f64) -> Rise {
    let (waist, hips) = (body.get("waist"), body.get("hips"));
    let hips_level = body.get("hips_line");

    Rise {
        waist: lin_interpolation(hips, waist, rise),
        hips_depth: rise * hips_level,
        back_waist: lin_interpolation(
            body.get("hip_back_width"),
            body.get("waist_back_width"),
            rise,
        ),
    }
}

/// Add a panel to a component's subcomponent list.
pub fn add_sub(comp: &CompRef, e: impl Into<Element>) -> usize {
    let mut c = comp.borrow_mut();
    c.subs.push(e.into());
    c.subs.len() - 1
}

/// Are these the same vertex object?
pub fn same_vert(a: &Vert, b: &Vert) -> bool {
    crate::garment::edge::vsame(a, b)
}

/// A fresh vertex at a location, so panel code reads like the reference's
/// vertex lists.
pub fn vert_at(p: V2) -> Vert {
    crate::garment::edge::vert(p)
}

/// Convenience: the element wrapper for a panel.
pub fn el(p: &PanelRef) -> Element {
    Element::Panel(p.clone())
}

/// Convenience: the element wrapper for a component.
pub fn ec(c: &CompRef) -> Element {
    Element::Comp(c.clone())
}
