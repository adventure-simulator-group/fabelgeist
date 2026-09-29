//! Deep copies of panels and components.
//!
//! Stands in for Python's `copy.deepcopy`, which the garment programs use to
//! replicate panels (`distribute_Y`, `distribute_horisontally`) and to branch
//! design dictionaries.
//!
//! Like `deepcopy`'s memo dictionary, this preserves *sharing*: a vertex used
//! by two edges stays one vertex in the copy, and an interface still points at
//! the panel's own edge objects.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::component::{CompRef, Component, Element};
use super::connector::{Stitches, StitchingRule};
use super::edge::{Edge, EdgeRef, EdgeSequence, Vert, vert, vget};
use super::interface::{Interface, InterfaceRef};
use super::panel::{InterfaceMap, Panel, PanelRef};

#[derive(Default)]
pub struct CloneCtx {
    verts: HashMap<usize, Vert>,
    edges: HashMap<usize, EdgeRef>,
    panels: HashMap<usize, PanelRef>,
    interfaces: HashMap<usize, InterfaceRef>,
}

impl CloneCtx {
    pub fn new() -> Self {
        Self::default()
    }

    fn vert(&mut self, v: &Vert) -> Vert {
        let key = Rc::as_ptr(v) as usize;
        self.verts
            .entry(key)
            .or_insert_with(|| vert(vget(v)))
            .clone()
    }

    pub fn edge(&mut self, e: &EdgeRef) -> EdgeRef {
        let key = Rc::as_ptr(e) as usize;
        if let Some(existing) = self.edges.get(&key) {
            return existing.clone();
        }
        let src = e.borrow();
        let new = Rc::new(RefCell::new(Edge {
            start: self.vert(&src.start),
            end: self.vert(&src.end),
            label: src.label.clone(),
            geometric_id: src.geometric_id,
            kind: src.kind.clone(),
        }));
        self.edges.insert(key, new.clone());
        new
    }

    pub fn edge_seq(&mut self, seq: &EdgeSequence) -> EdgeSequence {
        EdgeSequence::from_edges(seq.edges.iter().map(|e| self.edge(e)).collect())
    }

    pub fn panel(&mut self, p: &PanelRef) -> PanelRef {
        let key = Rc::as_ptr(p) as usize;
        if let Some(existing) = self.panels.get(&key) {
            return existing.clone();
        }

        let (name, label, translation, rotation, verbose, length_source, width_rule) = {
            let src = p.borrow();
            (
                src.name.clone(),
                src.label.clone(),
                src.translation,
                src.rotation,
                src.verbose,
                src.length_source.clone(),
                src.width_rule,
            )
        };

        // Register before recursing: interfaces refer back to the panel.
        let new = Panel::new(&name);
        self.panels.insert(key, new.clone());
        {
            let mut n = new.borrow_mut();
            n.label = label;
            n.translation = translation;
            n.rotation = rotation;
            n.verbose = verbose;
            n.length_source = length_source;
            n.width_rule = width_rule;
        }

        let edges = {
            let src = p.borrow();
            self.edge_seq(&src.edges)
        };
        new.borrow_mut().edges = edges;

        let interfaces = {
            let src = p.borrow();
            src.interfaces.clone()
        };
        let new_interfaces = self.interface_map(&interfaces);
        new.borrow_mut().interfaces = new_interfaces;

        let rules = {
            let src = p.borrow();
            src.stitching_rules.clone()
        };
        let new_rules = self.stitches(&rules);
        new.borrow_mut().stitching_rules = new_rules;

        new
    }

    pub fn interface(&mut self, i: &InterfaceRef) -> InterfaceRef {
        let key = Rc::as_ptr(i) as usize;
        if let Some(existing) = self.interfaces.get(&key) {
            return existing.clone();
        }

        let src = i.borrow();
        let new = Rc::new(RefCell::new(Interface {
            edges: EdgeSequence::new(),
            panel: Vec::new(),
            right_wrong: src.right_wrong.clone(),
            edges_flipping: src.edges_flipping.clone(),
            ruffle: src.ruffle.clone(),
        }));
        self.interfaces.insert(key, new.clone());

        let edges = self.edge_seq(&src.edges);
        let panels: Vec<PanelRef> = src.panel.iter().map(|p| self.panel(p)).collect();
        drop(src);

        {
            let mut n = new.borrow_mut();
            n.edges = edges;
            n.panel = panels;
        }
        new
    }

    pub fn interface_map(&mut self, map: &InterfaceMap) -> InterfaceMap {
        let mut out = InterfaceMap::new();
        let keys: Vec<String> = map.keys().map(|k| k.to_string()).collect();
        for k in keys {
            let v = map.get(&k);
            let cloned = self.interface(&v);
            out.set(&k, cloned);
        }
        out
    }

    pub fn stitches(&mut self, s: &Stitches) -> Stitches {
        Stitches {
            rules: s
                .rules
                .iter()
                .map(|r| StitchingRule {
                    int1: self.interface(&r.int1),
                    int2: self.interface(&r.int2),
                })
                .collect(),
        }
    }

    pub fn component(&mut self, c: &CompRef) -> CompRef {
        let (name, subs, length_mode, rise) = {
            let src = c.borrow();
            (
                src.name.clone(),
                src.subs.clone(),
                src.length_mode.clone(),
                src.rise,
            )
        };

        let new = Component::new(&name);
        {
            let mut n = new.borrow_mut();
            n.length_mode = length_mode;
            n.rise = rise;
        }
        new.borrow_mut().subs = subs.iter().map(|s| self.element(s)).collect();

        let interfaces = c.borrow().interfaces.clone();
        let new_interfaces = self.interface_map(&interfaces);
        new.borrow_mut().interfaces = new_interfaces;

        let rules = c.borrow().stitching_rules.clone();
        let new_rules = self.stitches(&rules);
        new.borrow_mut().stitching_rules = new_rules;

        new
    }

    pub fn element(&mut self, e: &Element) -> Element {
        match e {
            Element::Panel(p) => Element::Panel(self.panel(p)),
            Element::Comp(c) => Element::Comp(self.component(c)),
        }
    }
}

/// A standalone deep copy of an element.
pub fn deep_copy(e: &Element) -> Element {
    CloneCtx::new().element(e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garment::edge_factory::from_verts;
    use crate::garment::interface::Interface;

    fn square_panel() -> PanelRef {
        let p = Panel::new("sq");
        let edges = from_verts(&[[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]], true);
        let e0 = edges[0].clone();
        p.borrow_mut().edges = edges;
        p.borrow_mut().translation = [0.0, 0.0, 10.0];
        let i = Interface::one(&p, e0);
        p.borrow_mut().interfaces.set("bottom", i);
        p
    }

    #[test]
    fn copy_is_independent() {
        let p = square_panel();
        let copy = deep_copy(&Element::Panel(p.clone()));
        copy.translate_by([100.0, 0.0, 0.0]);

        assert_eq!(p.borrow().translation, [0.0, 0.0, 10.0]);
        let Element::Panel(cp) = &copy else { panic!() };
        assert_eq!(cp.borrow().translation, [100.0, 0.0, 10.0]);
    }

    #[test]
    fn copy_keeps_interface_wired_to_its_own_edges() {
        let p = square_panel();
        let Element::Panel(cp) = deep_copy(&Element::Panel(p.clone())) else {
            panic!()
        };

        let iface = cp.borrow().interfaces.get("bottom");
        let iface_edge = iface.borrow().edges[0].clone();
        let panel_edge = cp.borrow().edges[0].clone();

        assert!(
            super::super::edge::esame(&iface_edge, &panel_edge),
            "the copied interface must point at the copied panel's edge"
        );
        assert!(
            !super::super::edge::esame(
                &iface_edge,
                &p.borrow().interfaces.get("bottom").borrow().edges[0]
            ),
            "the copy must not alias the original"
        );
        assert!(Rc::ptr_eq(&iface.borrow().panel[0], &cp));
    }

    #[test]
    fn copy_preserves_vertex_sharing() {
        let p = square_panel();
        let Element::Panel(cp) = deep_copy(&Element::Panel(p)) else {
            panic!()
        };
        let edges = cp.borrow().edges.clone();
        assert!(edges.is_chained());
        assert!(edges.is_loop());
    }
}
