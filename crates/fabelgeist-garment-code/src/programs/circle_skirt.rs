//! Circle skirts and the circular-arc panel they are built from.
//!
//! Ports `assets.garment_programs.circle_skirt`.

use super::prelude::*;

/// One wedge of a circle skirt: an annular sector with `angle` radians of arc.
pub fn circle_arc_panel(
    name: &str,
    top_rad: f64,
    length: f64,
    angle: f64,
    match_top_int_proportion: Option<f64>,
    match_bottom_int_proportion: Option<f64>,
) -> PanelRef {
    let panel = Panel::new(name);
    let halfarc = angle / 2.0;

    let dist_w = 2.0 * top_rad * halfarc.sin();
    let dist_out = 2.0 * (top_rad + length) * halfarc.sin();
    let vert_len = length * halfarc.cos();
    let large = halfarc > std::f64::consts::FRAC_PI_2;

    let mut edges = EdgeSequence::new();
    // Top arc.
    edges.push(circle_from_points_radius(
        [-dist_w / 2.0, 0.0],
        [dist_w / 2.0, 0.0],
        top_rad,
        large,
        true,
    ));
    // Right side.
    let prev_end = edges.last().borrow().end.clone();
    edges.push(Edge::line_v(prev_end, vert_at([dist_out / 2.0, -vert_len])));
    // Bottom arc.
    let bottom = circle_from_points_radius(
        edges.last().borrow().end_p(),
        [-dist_out / 2.0, -vert_len],
        top_rad + length,
        large,
        false,
    );
    bottom.borrow_mut().start = edges.last().borrow().end.clone();
    edges.push(bottom);
    edges.close_loop();

    let (e0, e1, e2, e3) = (
        edges[0].clone(),
        edges[1].clone(),
        edges[2].clone(),
        edges[3].clone(),
    );
    panel.borrow_mut().edges = edges;

    let top_ruffle = match_top_int_proportion
        .map(|m| e0.borrow().length() / m)
        .unwrap_or(1.0);
    let bottom_ruffle = match_bottom_int_proportion
        .map(|m| e2.borrow().length() / m)
        .unwrap_or(1.0);

    {
        let mut p = panel.borrow_mut();
        p.length_source = PanelLength::Interface("right".into());
    }
    let ifaces = [
        (
            "top",
            Interface::new(&panel, EdgeSequence::one(e0), top_ruffle, false).reversed(true),
        ),
        (
            "bottom",
            Interface::new(&panel, EdgeSequence::one(e2), bottom_ruffle, false),
        ),
        ("left", Interface::one(&panel, e1)),
        ("right", Interface::one(&panel, e3)),
    ];
    for (k, v) in ifaces {
        panel.borrow_mut().interfaces.set(k, v);
    }

    panel
}

/// A circle-arc panel described by its top width and a fraction of a full
/// circle ("suns").
pub fn circle_arc_from_w_length_suns(
    name: &str,
    length: f64,
    top_width: f64,
    sun_fraction: f64,
    match_top: Option<f64>,
    match_bottom: Option<f64>,
) -> PanelRef {
    let arc = sun_fraction * 2.0 * std::f64::consts::PI;
    let rad = top_width / arc;
    circle_arc_panel(name, rad, length, arc, match_top, match_bottom)
}

/// A circle-arc panel described by its top and bottom widths.
pub fn circle_arc_from_all_length(
    name: &str,
    length: f64,
    top_width: f64,
    bottom_width: f64,
    match_top: Option<f64>,
    match_bottom: Option<f64>,
) -> PanelRef {
    let diff = bottom_width - top_width;
    let arc = diff / length;
    let rad = top_width / arc;
    circle_arc_panel(name, rad, length, arc, match_top, match_bottom)
}

/// A circle-arc panel described by its top width and radius.
pub fn circle_arc_from_length_rad(
    name: &str,
    length: f64,
    top_width: f64,
    rad: f64,
    match_top: Option<f64>,
    match_bottom: Option<f64>,
) -> PanelRef {
    let arc = top_width / rad;
    circle_arc_panel(name, rad, length, arc, match_top, match_bottom)
}

/// Half of a shifted arc section -- the panel of an asymmetric circle skirt.
pub fn asym_half_circle_panel(
    name: &str,
    top_rad: f64,
    length_f: f64,
    length_s: f64,
    match_top_int_proportion: Option<f64>,
    match_bottom_int_proportion: Option<f64>,
) -> PanelRef {
    let panel = Panel::new(name);

    let dist_w = 2.0 * top_rad;
    let dist_out = 2.0 * (top_rad + length_s);

    let mut edges = EdgeSequence::new();
    edges.push(circle_from_points_radius(
        [-dist_w / 2.0, 0.0],
        [dist_w / 2.0, 0.0],
        top_rad,
        false,
        true,
    ));
    let prev_end = edges.last().borrow().end.clone();
    edges.push(Edge::line_v(prev_end, vert_at([dist_out / 2.0, 0.0])));

    let bottom = circle_from_three_points(
        edges.last().borrow().end_p(),
        [-dist_out / 2.0, 0.0],
        [0.0, -(top_rad + length_f)],
        false,
    );
    bottom.borrow_mut().start = edges.last().borrow().end.clone();
    edges.push(bottom);
    edges.close_loop();

    let (e0, e1, e2, e3) = (
        edges[0].clone(),
        edges[1].clone(),
        edges[2].clone(),
        edges[3].clone(),
    );
    panel.borrow_mut().edges = edges;
    panel.borrow_mut().length_source = PanelLength::Interface("right".into());

    let top_ruffle = match_top_int_proportion
        .map(|m| e0.borrow().length() / m)
        .unwrap_or(1.0);
    let bottom_ruffle = match_bottom_int_proportion
        .map(|m| e2.borrow().length() / m)
        .unwrap_or(1.0);

    let ifaces = [
        (
            "top",
            Interface::new(&panel, EdgeSequence::one(e0), top_ruffle, false).reversed(true),
        ),
        (
            "bottom",
            Interface::new(&panel, EdgeSequence::one(e2), bottom_ruffle, false),
        ),
        ("left", Interface::one(&panel, e1)),
        ("right", Interface::one(&panel, e3)),
    ];
    for (k, v) in ifaces {
        panel.borrow_mut().interfaces.set(k, v);
    }

    panel
}

/// Options shared by the skirt components.
#[derive(Debug, Clone)]
pub struct SkirtOpts {
    pub tag: String,
    pub length: Option<f64>,
    pub rise: Option<f64>,
    pub slit: bool,
    pub top_ruffles: bool,
}

impl Default for SkirtOpts {
    fn default() -> Self {
        Self {
            tag: String::new(),
            length: None,
            rise: None,
            slit: true,
            top_ruffles: true,
        }
    }
}

const MIN_SKIRT_LEN: f64 = 5.0;

/// A circle skirt, optionally with a front/back asymmetry.
pub fn skirt_circle(body: &Body, design: &Design, opts: &SkirtOpts, asymm: bool) -> CompRef {
    let class_name = if asymm {
        "AsymmSkirtCircle"
    } else {
        "SkirtCircle"
    };
    let name = if opts.tag.is_empty() {
        class_name.to_string()
    } else {
        format!("{class_name}_{}", opts.tag)
    };
    let comp = Component::new(&name);

    let d = design.sub("flare-skirt");
    let suns = d.f("suns");
    let rise = opts.rise.unwrap_or_else(|| d.f("rise"));
    comp.borrow_mut().rise = Some(rise);
    let r = eval_rise(body, rise);

    let length = opts
        .length
        .unwrap_or_else(|| r.hips_depth + d.f("length") * body.get("_leg_length"));
    // Some rise/length combinations drive this negative.
    let length = length.max(MIN_SKIRT_LEN);

    let front_name = if opts.tag.is_empty() {
        "skirt_front".to_string()
    } else {
        format!("skirt_front_{}", opts.tag)
    };
    let back_name = if opts.tag.is_empty() {
        "skirt_back".to_string()
    } else {
        format!("skirt_back_{}", opts.tag)
    };

    let match_front = body.get("waist") - body.get("waist_back_width");
    let match_back = body.get("waist_back_width");

    let (front, back) = if !asymm {
        (
            circle_arc_from_w_length_suns(
                &front_name,
                length,
                r.waist / 2.0,
                suns / 2.0,
                Some(match_front),
                None,
            ),
            circle_arc_from_w_length_suns(
                &back_name,
                length,
                r.waist / 2.0,
                suns / 2.0,
                Some(match_back),
                None,
            ),
        )
    } else {
        // Asymmetric front/back is only defined on a full (one sun) skirt.
        let w_rad = r.waist / 2.0 / std::f64::consts::PI;
        let f_length = d.f("asymm.front_length") * length;
        let tot_len = w_rad * 2.0 + length + f_length;
        let del_r = tot_len / 2.0 - f_length - w_rad;
        let s_length = ((tot_len / 2.0).powi(2) - del_r * del_r).sqrt() - w_rad;

        (
            asym_half_circle_panel(
                &front_name,
                w_rad,
                f_length,
                s_length,
                Some(match_front),
                None,
            ),
            asym_half_circle_panel(&back_name, w_rad, length, s_length, Some(match_back), None),
        )
    };

    front
        .borrow_mut()
        .translate_by([0.0, body.get("_waist_level"), 15.0]);
    back.borrow_mut()
        .translate_by([0.0, body.get("_waist_level"), -15.0]);

    if d.b("cut.add") && opts.slit {
        let target = if d.f("cut.place") > 0.0 {
            front.clone()
        } else {
            back.clone()
        };
        add_skirt_cut(&target, &d, length);
    }

    add_sub(&comp, el(&front));
    add_sub(&comp, el(&back));
    comp.borrow_mut().length_mode = LengthMode::Sub(0);

    let f = front.borrow().interfaces.clone();
    let b = back.borrow().interfaces.clone();
    comp.borrow_mut().stitching_rules = Stitches::from_pairs(vec![
        (f.get("right"), b.get("right")),
        (f.get("left"), b.get("left")),
    ]);

    {
        let mut c = comp.borrow_mut();
        c.interfaces.set(
            "top",
            Interface::from_multiple(&[f.get("top"), b.get("top")]),
        );
        c.interfaces.set("bottom_f", f.get("bottom"));
        c.interfaces.set("bottom_b", b.get("bottom"));
        c.interfaces.set(
            "bottom",
            Interface::from_multiple(&[f.get("bottom"), b.get("bottom")]),
        );
    }

    comp
}

/// Cut a decorative slit into a circle-skirt panel's hem.
fn add_skirt_cut(panel: &PanelRef, design: &Design, sk_length: f64) {
    let width = design.f("cut.width") * sk_length;
    let depth = design.f("cut.depth") * sk_length;

    let bottom_int = panel.borrow().interfaces.get("bottom");
    let target_edge = bottom_int.borrow().edges[0].clone();
    let t_len = target_edge.borrow().length();

    let mut offset = (design.f("cut.place") * t_len).abs();
    offset = offset.max(width / 2.0);
    offset = offset.min(t_len - width / 2.0);

    // NOTE: heuristic, specific to the panels used here.
    let right = target_edge.borrow().start_p()[0] > target_edge.borrow().end_p()[0];

    let cut_shape = dart_shape(width, None, Some(depth));
    let res = ops::cut_into_edge_single(&cut_shape, &target_edge, offset, right, 1e-2);

    panel
        .borrow_mut()
        .edges
        .substitute(&target_edge, &res.new_edges);

    let panels = vec![panel.clone(); res.leftovers.len()];
    bottom_int
        .borrow_mut()
        .substitute(&target_edge, &res.leftovers, &panels);
}
