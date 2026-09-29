//! Fitted bodices, and the shirt components that assemble a whole top.
//!
//! Ports `assets.garment_programs.bodice`.

use super::collars::{self, CollarStyle};
use super::prelude::*;
use super::sleeves::{self, BodiceWidth};
use super::tee::{self, TorsoSide};

/// The front half of a fitted bodice block, with side and waist darts.
pub fn bodice_front_half(name: &str, body: &Body, _design: &Design) -> PanelRef {
    let panel = Panel::new(name);

    let m_bust = body.get("bust");
    let m_waist = body.get("waist");

    let bust_point = body.get("bust_points") / 2.0;
    let front_frac = (m_bust - body.get("back_width")) / 2.0 / m_bust;

    let width = front_frac * m_bust;
    let waist = (m_waist - body.get("waist_back_width")) / 2.0;
    let sh_tan = body.get("_shoulder_incl").to_radians().tan();
    let shoulder_incl = sh_tan * width;
    let bottom_d_width = (width - waist) * 2.0 / 3.0;

    let adjustment = sh_tan * (width - body.get("shoulder_w") / 2.0);
    let max_len = body.get("waist_over_bust_line") - adjustment;

    // The side length is adjusted for the shoulder inclination, so the sleeve
    // still fits.
    let fb_diff = (front_frac - (0.5 - front_frac)) * m_bust;
    let back_adjustment = sh_tan * (body.get("back_width") / 2.0 - body.get("shoulder_w") / 2.0);
    let side_len = body.get("waist_line") - back_adjustment - sh_tan * fb_diff;

    let mut edges = from_verts(
        &[
            [0.0, 0.0],
            [-width, 0.0],
            [-width, max_len],
            [0.0, max_len + shoulder_incl],
        ],
        false,
    );
    edges.close_loop();
    panel.borrow_mut().edges = edges;

    // Side dart.
    let bust_line = body.get("waist_line") - body.get("_bust_line");
    // NOTE: calculated value.
    let side_d_depth = 0.75 * (width - bust_point);
    let side_d_width = max_len - side_len;

    let side_edge = panel.borrow().edges[1].clone();
    let (s_edge, side_interface) = ops::add_dart(
        &panel,
        // NOTE: positionally the reference passes this as the dart's side
        // length, not its perpendicular depth.
        &dart_shape(side_d_width, Some(side_d_depth), None),
        &side_edge,
        bust_line + side_d_width / 2.0,
        true,
        None,
        None,
    );
    panel.borrow_mut().edges.substitute_at(1, &s_edge);

    // Take some fabric off the top to match the shoulder width.
    {
        let x_upd = width - body.get("shoulder_w") / 2.0;
        let last = s_edge.last().borrow().end.clone();
        let p = vget(&last);
        vset(&last, [p[0] + x_upd, p[1] + sh_tan * x_upd]);
    }

    // Bottom dart.
    let bottom_edge = panel.borrow().edges[0].clone();
    let (b_edge, b_interface) = ops::add_dart(
        &panel,
        &dart_shape(bottom_d_width, Some(0.9 * bust_line), None),
        &bottom_edge,
        bust_point + bottom_d_width / 2.0,
        true,
        None,
        None,
    );
    panel.borrow_mut().edges.substitute_at(0, &b_edge);

    // Take some fabric off the side at the bottom (after the side dart).
    {
        let last = b_edge.last().borrow().end.clone();
        let p = vget(&last);
        vset(&last, [-(waist + bottom_d_width), p[1]]);
    }

    let n = panel.borrow().edges.len();
    let (e_m3, e_m2, e_m1) = {
        let e = &panel.borrow().edges;
        (e[n - 3].clone(), e[n - 2].clone(), e[n - 1].clone())
    };

    let mut shoulder_corner = EdgeSequence::one(e_m3);
    shoulder_corner.push(e_m2.clone());
    let mut collar_corner = EdgeSequence::one(e_m2.clone());
    collar_corner.push(e_m1.clone());

    let ifaces = [
        ("outside", Interface::plain(&panel, side_interface)),
        ("inside", Interface::one(&panel, e_m1)),
        ("shoulder", Interface::one(&panel, e_m2)),
        ("bottom", Interface::plain(&panel, b_interface)),
        ("shoulder_corner", Interface::plain(&panel, shoulder_corner)),
        ("collar_corner", Interface::plain(&panel, collar_corner)),
    ];
    for (k, v) in ifaces {
        panel.borrow_mut().interfaces.set(k, v);
    }

    panel.borrow_mut().width_rule = Some(WidthRule::Slope {
        shoulder_w: body.get("shoulder_w"),
    });

    panel.borrow_mut().translate_by([
        0.0,
        body.get("height") - body.get("head_l") - max_len - shoulder_incl,
        0.0,
    ]);

    panel
}

/// The back half of a fitted bodice block.
pub fn bodice_back_half(name: &str, body: &Body, _design: &Design) -> PanelRef {
    let panel = Panel::new(name);

    let width = body.get("back_width") / 2.0;
    let waist = body.get("waist_back_width") / 2.0;
    // NOTE: no side inclination -- there is not much to begin with.
    let waist_width = if waist < width { width } else { waist };

    let sh_tan = body.get("_shoulder_incl").to_radians().tan();
    let shoulder_incl = sh_tan * width;

    // Measure the length from the shoulder rather than the garment's de-facto
    // side.
    let back_adjustment = sh_tan * (width - body.get("shoulder_w") / 2.0);
    let length = body.get("waist_line") - back_adjustment;

    // Base edge loop. The back is a little shorter at the centre.
    let edge_0 = curve_from_tangents(
        [0.0, shoulder_incl / 4.0],
        [-waist_width, 0.0],
        Some([-1.0, 0.0]),
        None,
        None,
    );

    let mut edges = EdgeSequence::one(edge_0.clone());
    let rest = from_verts(
        &[
            edge_0.borrow().end_p(),
            [-width, body.get("waist_line") - body.get("_bust_line")],
            [-width, length],
            // A little extra fabric at the neck, for the shoulder inclination.
            [0.0, length + shoulder_incl],
        ],
        false,
    );
    rest.first().borrow_mut().start = edge_0.borrow().end.clone();
    edges.extend(&rest);
    edges.close_loop();
    panel.borrow_mut().edges = edges;

    let n = panel.borrow().edges.len();
    let (e0, e1, e2, e_m3, e_m2, e_m1) = {
        let e = &panel.borrow().edges;
        (
            e[0].clone(),
            e[1].clone(),
            e[2].clone(),
            e[n - 3].clone(),
            e[n - 2].clone(),
            e[n - 1].clone(),
        )
    };

    let mut outside = EdgeSequence::one(e1);
    outside.push(e2.clone());
    let mut shoulder_corner = EdgeSequence::one(e_m3);
    shoulder_corner.push(e_m2.clone());
    let mut collar_corner = EdgeSequence::one(e_m2.clone());
    collar_corner.push(e_m1.clone());

    let ifaces = [
        ("outside", Interface::plain(&panel, outside)),
        ("inside", Interface::one(&panel, e_m1)),
        ("shoulder", Interface::one(&panel, e_m2)),
        ("bottom", Interface::one(&panel, e0.clone())),
        ("shoulder_corner", Interface::plain(&panel, shoulder_corner)),
        ("collar_corner", Interface::plain(&panel, collar_corner)),
    ];
    for (k, v) in ifaces {
        panel.borrow_mut().interfaces.set(k, v);
    }

    panel.borrow_mut().width_rule = Some(WidthRule::Constant(width));

    // Bottom darts, cut out of the straight waistline.
    let level = e2.borrow().end_p()[1] - e2.borrow().start_p()[1];
    if waist < panel.borrow().get_width(level) {
        let w_diff = waist_width - waist;
        // Don't take from the sides if the difference is too small.
        let side_adj = if w_diff < 4.0 { 0.0 } else { w_diff / 6.0 };
        // Double darts.
        let bottom_d_width = (w_diff - side_adj) / 2.0;
        // NOTE: calculated value.
        let bottom_d_depth = length - body.get("_bust_line");
        let bottom_d_position = body.get("bum_points") / 2.0;

        // Distance between darts -> distance between their centres.
        let dist = bottom_d_position * 0.5;
        let (mut b_edge, mut b_interface) = ops::add_dart(
            &panel,
            &dart_shape(bottom_d_width, Some(0.9 * bottom_d_depth), None),
            &e0,
            bottom_d_position + dist / 2.0 + bottom_d_width + bottom_d_width / 2.0,
            true,
            None,
            None,
        );

        let first = b_edge.first().clone();
        let (b2, i2) = ops::add_dart(
            &panel,
            &dart_shape(bottom_d_width, Some(bottom_d_depth), None),
            &first,
            bottom_d_position - dist / 2.0 + bottom_d_width / 2.0,
            true,
            Some(&mut b_edge),
            Some(&mut b_interface),
        );

        panel.borrow_mut().edges.substitute_at(0, &b2);
        let iface = Interface::plain(&panel, i2);
        panel.borrow_mut().interfaces.set("bottom", iface);

        // Remove fabric from the sides if the difference is big enough.
        let last = b2.last().borrow().end.clone();
        let p = vget(&last);
        vset(&last, [p[0] + side_adj, p[1]]);
    }

    panel.borrow_mut().translate_by([
        0.0,
        body.get("height") - body.get("head_l") - length - shoulder_incl,
        0.0,
    ]);

    panel
}

/// Half of an upper garment: torso panels plus sleeve and collar cut-outs.
pub fn bodice_half(name: &str, body: &Body, design: &Design, fitted: bool) -> CompRef {
    let comp = Component::new(name);
    // Recalculate freely.
    let design = design.deep_copy();

    let (ftorso, btorso) = if fitted {
        (
            bodice_front_half(&format!("{name}_ftorso"), body, &design),
            bodice_back_half(&format!("{name}_btorso"), body, &design),
        )
    } else {
        (
            tee::torso_half_panel(&format!("{name}_ftorso"), body, &design, TorsoSide::Front),
            tee::torso_half_panel(&format!("{name}_btorso"), body, &design, TorsoSide::Back),
        )
    };
    ftorso.borrow_mut().translate_by([0.0, 0.0, 30.0]);
    btorso.borrow_mut().translate_by([0.0, 0.0, -25.0]);

    add_sub(&comp, el(&ftorso));
    let btorso_idx = add_sub(&comp, el(&btorso));
    comp.borrow_mut().length_mode = LengthMode::Sub(btorso_idx);

    {
        let f = ftorso.borrow().interfaces.clone();
        let b = btorso.borrow().interfaces.clone();
        let mut c = comp.borrow_mut();
        c.interfaces.set("f_bottom", f.get("bottom"));
        c.interfaces.set("b_bottom", b.get("bottom"));
        c.interfaces.set("front_in", f.get("inside"));
        c.interfaces.set("back_in", b.get("inside"));
    }

    eval_dep_params(&comp, body, &design, &ftorso, &btorso);

    // NOTE: strapless is only defined for fitted tops.
    if design.b("shirt.strapless") && fitted {
        make_strapless(body, &design, &ftorso, &btorso);
    } else {
        add_sleeves(&comp, name, body, &design, &ftorso, &btorso);
        add_collars(&comp, name, body, &design, &ftorso, &btorso);

        let f = ftorso.borrow().interfaces.get("shoulder");
        let b = btorso.borrow().interfaces.get("shoulder");
        comp.borrow_mut().stitching_rules.append(f, b);
    }

    // Sides.
    let f = ftorso.borrow().interfaces.get("outside");
    let b = btorso.borrow().interfaces.get("outside");
    comp.borrow_mut().stitching_rules.append(f, b);

    comp
}

/// Derive sleeve and collar parameters that depend on the torso geometry.
///
/// Writes back into `design`, which is why the caller works on a copy.
fn eval_dep_params(
    _comp: &CompRef,
    body: &Body,
    design: &Design,
    ftorso: &PanelRef,
    btorso: &PanelRef,
) {
    // --- Sleeves ---
    // NOTE: the vertical side is assumed to be the first edge of the corner.
    let max_cwidth = ftorso
        .borrow()
        .interfaces
        .get("shoulder_corner")
        .borrow()
        .edges[0]
        .borrow()
        .length()
        - 1.0;
    let min_cwidth = body.get("_armscye_depth");
    let v = design.f("sleeve.connecting_width");
    design.set_f(
        "sleeve.connecting_width",
        (min_cwidth + min_cwidth * v).min(max_cwidth),
    );

    // --- Collars ---
    // NOTE: the first edge is assumed to be the top one. The back panel is
    // narrower, so it sets the limit.
    // 1 cm in from the default sleeve.
    let max_w = body.get("_base_sleeve_balance") - 2.0;
    let min_w = body.get("neck_w");

    let cw = design.f("collar.width");
    let width = if cw >= 0.0 {
        lin_interpolation(min_w, max_w, cw)
    } else {
        lin_interpolation(0.0, min_w, 1.0 + cw)
    };
    design.set_f("collar.width", width);

    // Collar depth is given w.r.t. length; adjust for the shoulder inclination.
    let tg = body.get("_shoulder_incl").to_radians().tan();
    let f_depth_adj = tg * (ftorso.borrow().get_width(0.0) - width / 2.0);
    let b_depth_adj = tg * (btorso.borrow().get_width(0.0) - width / 2.0);

    let corner_len = |p: &PanelRef| {
        p.borrow().interfaces.get("collar_corner").borrow().edges[1]
            .borrow()
            .length()
    };
    let max_f_len = corner_len(ftorso) - tg * ftorso.borrow().get_width(0.0) - 1.0;
    let max_b_len = corner_len(btorso) - tg * btorso.borrow().get_width(0.0) - 1.0;

    let f_strapless = (design.f("collar.fc_depth") * body.get("_bust_line")).min(max_f_len);
    design.set_f("collar.f_strapless_depth", f_strapless);
    design.set_f("collar.fc_depth", f_strapless + f_depth_adj);

    let b_strapless = (design.f("collar.bc_depth") * body.get("_bust_line")).min(max_b_len);
    design.set_f("collar.b_strapless_depth", b_strapless);
    design.set_f("collar.bc_depth", b_strapless + b_depth_adj);
}

fn add_sleeves(
    comp: &CompRef,
    name: &str,
    body: &Body,
    design: &Design,
    ftorso: &PanelRef,
    btorso: &PanelRef,
) {
    let sleeve = sleeves::sleeve(
        name,
        body,
        design,
        &BodiceWidth::OfLevel(ftorso.clone()),
        &BodiceWidth::OfLevel(btorso.clone()),
    );

    let f_shape = sleeve.borrow().interfaces.get("in_front_shape");
    let b_shape = sleeve.borrow().interfaces.get("in_back_shape");
    let f_edges = f_shape.borrow().edges.clone();
    let b_edges = b_shape.borrow().edges.clone();

    // NOTE: bind the interfaces first -- `cut_corner` mutates the panel, and an
    // inline `panel.borrow()` would still be live during the call.
    let f_corner = ftorso.borrow().interfaces.get("shoulder_corner");
    let b_corner = btorso.borrow().interfaces.get("shoulder_corner");

    let (_, f_sleeve_int) = ops::cut_corner(&f_edges, &f_corner);
    let (_, b_sleeve_int) = ops::cut_corner(&b_edges, &b_corner);

    add_sub(comp, ec(&sleeve));

    if !design.b("sleeve.sleeveless") {
        let bodice_sleeve_int = Interface::from_multiple(&[
            f_sleeve_int.clone().reversed(true),
            b_sleeve_int.clone().reversed(false),
        ]);

        let sleeve_in = sleeve.borrow().interfaces.get("in");
        comp.borrow_mut()
            .stitching_rules
            .append(sleeve_in.clone(), bodice_sleeve_int.clone());

        // NOTE: a heuristic tuned for the 30-60 degree arm poses used in the
        // dataset.
        let gap = -1.0 - body.get("arm_pose_angle") / 10.0;
        ec(&sleeve).place_by_interface(&sleeve_in, &bodice_sleeve_int, gap, Alignment::Top, None);
    }

    let label = format!("{}_armhole", comp.borrow().name);
    f_sleeve_int.borrow().edges.propagate_label(&label);
    b_sleeve_int.borrow().edges.propagate_label(&label);
}

fn add_collars(
    comp: &CompRef,
    name: &str,
    body: &Body,
    design: &Design,
    ftorso: &PanelRef,
    btorso: &PanelRef,
) {
    let style = CollarStyle::from_name(design.s("collar.component.style").as_deref());
    let collar_comp = collars::collar(style, name, body, design);

    let f_edges = collar_comp
        .borrow()
        .interfaces
        .get("front_proj")
        .borrow()
        .edges
        .clone();
    let b_edges = collar_comp
        .borrow()
        .interfaces
        .get("back_proj")
        .borrow()
        .edges
        .clone();

    let f_corner = ftorso.borrow().interfaces.get("collar_corner");
    let b_corner = btorso.borrow().interfaces.get("collar_corner");

    let (_, fc_interface) = ops::cut_corner(&f_edges, &f_corner);
    let (_, bc_interface) = ops::cut_corner(&b_edges, &b_corner);

    add_sub(comp, ec(&collar_comp));

    let ci = collar_comp.borrow().interfaces.clone();
    if let Some(bottom) = ci.try_get("bottom") {
        let joined = Interface::from_multiple(&[fc_interface.clone(), bc_interface.clone()]);
        comp.borrow_mut().stitching_rules.append(joined, bottom);
    }

    if let Some(front) = ci.try_get("front") {
        let mut c = comp.borrow_mut();
        c.interfaces.set("front_collar", front.clone());
        let torso_in = ftorso.borrow().interfaces.get("inside");
        c.interfaces
            .set("front_in", Interface::from_multiple(&[torso_in, front]));
    }
    if let Some(back) = ci.try_get("back") {
        let mut c = comp.borrow_mut();
        c.interfaces.set("back_collar", back.clone());
        let torso_in = btorso.borrow().interfaces.get("inside");
        c.interfaces
            .set("back_in", Interface::from_multiple(&[torso_in, back]));
    }

    let label = format!("{}_collar", comp.borrow().name);
    fc_interface.borrow().edges.propagate_label(&label);
    bc_interface.borrow().edges.propagate_label(&label);
}

/// Crop the top of both torso panels for a strapless style.
fn make_strapless(body: &Body, design: &Design, ftorso: &PanelRef, btorso: &PanelRef) {
    let mut out_depth = design.f("sleeve.connecting_width");
    let f_in_depth = design.f("collar.f_strapless_depth");
    let b_in_depth = design.f("collar.b_strapless_depth");

    // Shoulder adjustment for the back.
    let shoulder_angle = body.get("_shoulder_incl").to_radians();
    let sleeve_balance = body.get("_base_sleeve_balance") / 2.0;
    let back_w = btorso.borrow().get_width(0.0);
    out_depth -= shoulder_angle.tan() * (back_w - sleeve_balance);

    adjust_top_level(btorso, out_depth, b_in_depth, None);

    // The front depth compensates for the length difference.
    let len_back = btorso
        .borrow()
        .interfaces
        .get("outside")
        .borrow()
        .edges
        .length();
    let len_front = ftorso
        .borrow()
        .interfaces
        .get("outside")
        .borrow()
        .edges
        .length();
    adjust_top_level(ftorso, out_depth, f_in_depth, Some(len_front - len_back));

    ftorso
        .borrow()
        .interfaces
        .get("shoulder")
        .borrow()
        .edges
        .propagate_label("strapless_top");
    btorso
        .borrow()
        .interfaces
        .get("shoulder")
        .borrow()
        .edges
        .propagate_label("strapless_top");
}

/// Lower a panel's top edge by the given depths.
///
/// `target_remove` asks for the outside depth to be tuned so that exactly that
/// much length is removed from the side seam.
fn adjust_top_level(panel: &PanelRef, out_level: f64, in_level: f64, target_remove: Option<f64>) {
    let panel_top = panel.borrow().interfaces.get("shoulder").borrow().edges[0].clone();
    let (ts, te) = {
        let t = panel_top.borrow();
        (t.start.clone(), t.end.clone())
    };
    let min_y = vget(&ts)[1].min(vget(&te)[1]);

    // Order the vertices: `ins` is the higher one.
    let (ins, out) = if vget(&ts)[1] < vget(&te)[1] {
        (te, ts)
    } else {
        (ts, te)
    };

    // The inside is a simple vertical line -- just change its Y.
    let p = vget(&ins);
    vset(&ins, [p[0], min_y - in_level]);

    // The outside may be inclined, so it needs more work.
    let outside_edge = panel
        .borrow()
        .interfaces
        .get("outside")
        .borrow()
        .edges
        .last()
        .clone();
    let (bs, be) = {
        let e = outside_edge.borrow();
        (e.start.clone(), e.end.clone())
    };
    let bot = if super::prelude::same_vert(&bs, &out) {
        be
    } else {
        bs
    };

    let mut out_level = out_level;
    if let Some(target_remove) = target_remove {
        let out_p = vget(&out);
        let bot_p = vget(&bot);
        let angle_sin = (out_p[1] - bot_p[1]).abs() / outside_edge.borrow().length();
        let curr_remove = out_level / angle_sin;
        let length_diff = target_remove - curr_remove;
        out_level += length_diff * angle_sin;
    }

    let out_p = vget(&out);
    let bot_p = vget(&bot);
    let angle_cotan = (out_p[0] - bot_p[0]).abs() / (out_p[1] - bot_p[1]).abs();
    vset(
        &out,
        [out_p[0] - out_level * angle_cotan, min_y - out_level],
    );
}

/// A whole upper garment: two mirrored bodice halves.
pub fn shirt(body: &Body, design: &Design, fitted: bool) -> CompRef {
    let class_name = if fitted { "FittedShirt" } else { "Shirt" };
    let comp = Component::new(class_name);

    let design = shirt_eval_dep_params(design);
    let asym = design.b("left.enable_asym");

    let right = bodice_half("right", body, &design, fitted);
    let left_design = if asym {
        design.sub("left")
    } else {
        design.clone()
    };
    let left = bodice_half("left", body, &left_design, fitted);
    ec(&left).mirror([0.0, 1.0]);

    add_sub(&comp, ec(&right));
    add_sub(&comp, ec(&left));
    comp.borrow_mut().length_mode = LengthMode::Sub(0);

    let r = right.borrow().interfaces.clone();
    let l = left.borrow().interfaces.clone();

    comp.borrow_mut()
        .stitching_rules
        .append(r.get("front_in"), l.get("front_in"));
    comp.borrow_mut()
        .stitching_rules
        .append(r.get("back_in"), l.get("back_in"));

    // Ordered for correct connectivity around the body.
    comp.borrow_mut().interfaces.set(
        "bottom",
        Interface::from_multiple(&[
            r.get("f_bottom").reversed(false),
            l.get("f_bottom"),
            l.get("b_bottom").reversed(false),
            r.get("b_bottom"),
        ]),
    );

    comp
}

/// Reconcile the left/right design subtrees when asymmetry is enabled.
fn shirt_eval_dep_params(design: &Design) -> Design {
    if !design.b("left.enable_asym") {
        return design.clone();
    }

    // NOTE: full collars combined with a partially strapless top, or mixed
    // panelled collar styles, are not supported yet.
    let design = design.deep_copy();

    // Force no collars -- the two sides' collars are not compatible.
    design.set_v("collar.component.style", Value::Null);
    design.set_v("left.collar.component.style", Value::Null);

    // Left/right design compatibility.
    design.copy_v("shirt.length", "left.shirt.length");
    design.copy_v("collar.fc_depth", "left.collar.fc_depth");
    design.copy_v("collar.bc_depth", "left.collar.bc_depth");

    design
}
