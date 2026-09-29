//! Godet skirts: a base skirt with triangular inserts let into its hem.
//!
//! Ports `assets.garment_programs.godet`.

use super::circle_skirt::SkirtOpts;
use super::prelude::*;
use super::skirt_paneled::{pencil_skirt, skirt_2};
use crate::garment::panel::PanelRef as PRef;

/// One triangular insert panel.
pub fn insert_panel(id: usize, width: f64, depth: f64) -> PanelRef {
    let panel = Panel::new(&format!("Insert_{id}"));

    let edges = from_verts(&[[0.0, 0.0], [width / 2.0, depth], [width, 0.0]], true);
    let first_two = edges.slice(0, 2);
    panel.borrow_mut().edges = edges;

    // The reference keeps this interface in a list; `int_0` is its only entry.
    panel
        .borrow_mut()
        .interfaces
        .set("int_0", Interface::plain(&panel, first_two));

    panel.borrow_mut().top_center_pivot();
    panel.borrow_mut().center_x();

    panel
}

/// A skirt with godet inserts.
pub fn godet_skirt(body: &Body, design: &Design, rise: Option<f64>) -> CompRef {
    let comp = Component::new("GodetSkirt");

    let g = design.sub("godet-skirt");
    let ins_w = g.f("insert_w");
    let ins_depth = g.f("insert_depth");

    // NOTE: godets do not get along with slits on the front/back of the base
    // skirt, so any slit is forced off.
    let opts = SkirtOpts {
        rise,
        slit: false,
        ..Default::default()
    };
    let base = match g.s("base").as_deref() {
        Some("Skirt2") => skirt_2(body, design, &opts),
        Some("PencilSkirt") | None => pencil_skirt(body, design, &opts),
        Some(other) => panic!("godet::ERROR::unknown base skirt '{other}'"),
    };
    comp.borrow_mut().rise = base.borrow().rise;

    add_sub(&comp, ec(&base));
    comp.borrow_mut().length_mode = LengthMode::Sub(0);

    let bintr = base.borrow().interfaces.get("bottom");
    let (edges, panels) = {
        let b = bintr.borrow();
        (b.edges.edges.clone(), b.panel.clone())
    };
    let n_sides = edges.len() as f64;

    for (edge, panel) in edges.iter().zip(&panels) {
        inserts(
            &comp,
            edge.clone(),
            panel,
            ins_w,
            ins_depth,
            g.f("num_inserts") / n_sides,
            g.f("cuts_distance"),
        );
    }

    let top = base.borrow().interfaces.get("top");
    comp.borrow_mut().interfaces.set("top", top);

    comp
}

/// Create insert panels for one skirt panel, cut matching notches into its hem,
/// and stitch the two together.
fn inserts(
    comp: &CompRef,
    bottom_edge: EdgeRef,
    panel: &PRef,
    ins_w: f64,
    ins_depth: f64,
    num_inserts: f64,
    cuts_dist: f64,
) {
    let num_inserts = num_inserts as usize;
    let bottom_len = bottom_edge.borrow().length();

    let pbbox = panel.borrow().bbox3d();
    let panel_z = panel.borrow().translation[2];
    let z_transl = panel_z + panel_z.signum() * 5.0;
    let y_base = pbbox.0[1];
    let x_shift = (pbbox.0[0] + pbbox.1[0]) / 2.0;

    let mut cut_width = (bottom_len - cuts_dist * num_inserts as f64) / num_inserts as f64;
    let mut cuts_dist = cuts_dist;
    if cut_width < 1.0 {
        // Cannot place that many cuts at the requested spacing; use the widest
        // spacing that fits.
        cut_width = 1.0;
        cuts_dist = (bottom_len - cut_width * num_inserts as f64) / num_inserts as f64;
    }

    // Insert panels.
    let insert = insert_panel(0, ins_w, ins_depth);
    insert.borrow_mut().translate_by([
        x_shift - num_inserts as f64 * ins_w / 2.0 + ins_w / 2.0,
        y_base + ins_depth,
        z_transl,
    ]);
    let new_inserts = ops::distribute_horisontally(
        &el(&insert),
        num_inserts,
        -ins_w,
        &format!("ins_{}", panel.borrow().name),
    );
    for e in &new_inserts {
        add_sub(comp, e.clone());
    }

    // The slanted side is the same on the skirt and on the insert.
    let side_len = ((ins_w / 2.0).powi(2) + ins_depth * ins_depth).sqrt();
    let cut_depth;
    if side_len > cut_width / 2.0 {
        cut_depth = (side_len * side_len - (cut_width / 2.0).powi(2)).sqrt();
    } else {
        // The requested width is too wide for the inserts; use the widest that
        // works.
        cut_depth = 1.0;
        cut_width = 2.0 * (side_len * side_len - cut_depth * cut_depth).sqrt();
    }

    let cut_shape = from_verts(
        &[[0.0, 0.0], [cut_width / 2.0, cut_depth], [cut_width, 0.0]],
        false,
    );

    // NOTE: heuristic matching the skirts in this collection.
    let right = z_transl < 0.0;

    let mut bottom_edge = bottom_edge;
    for i in 0..num_inserts {
        // start_offset + i * stride
        let offset = cut_width / 2.0 + if i == 0 { cuts_dist / 2.0 } else { cuts_dist };

        let res = ops::cut_into_edge_single(&cut_shape, &bottom_edge, offset, right, 1e-2);
        panel
            .borrow_mut()
            .edges
            .substitute(&bottom_edge, &res.new_edges);
        // The remaining edge to cut on the next step.
        bottom_edge = res.new_edges.last().clone();

        let cut_interface = Interface::plain(panel, res.inserted);
        if right {
            cut_interface.borrow_mut().reverse(false);
        }

        let insert_idx = if right { num_inserts - 1 - i } else { i };
        let insert_int = new_inserts[insert_idx].interface("int_0");
        comp.borrow_mut()
            .stitching_rules
            .append(insert_int, cut_interface);
    }
}
