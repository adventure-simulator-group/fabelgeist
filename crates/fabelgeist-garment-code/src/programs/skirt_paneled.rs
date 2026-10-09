//! Panelled skirts: gathered, pencil and many-panel styles.
//!
//! Ports `assets.garment_programs.skirt_paneled`.

use super::circle_skirt::SkirtOpts;
use super::prelude::*;
use super::shapes;

/// One panel of a gathered skirt, with optional ruffles and hem flare.
#[allow(clippy::too_many_arguments)]
pub fn skirt_panel(
    name: &str,
    waist_length: f64,
    length: f64,
    ruffles: f64,
    match_top_int_to: Option<f64>,
    bottom_cut: f64,
    flare: f64,
) -> PanelRef {
    let panel = Panel::new(name);

    let base_width = waist_length;
    let top_width = base_width * ruffles;
    let low_width = top_width + 2.0 * flare;
    // Account for the flare at the hem.
    let x_shift_top = (low_width - top_width) / 2.0;

    let right = if bottom_cut != 0.0 {
        side_with_cut([0.0, 0.0], [x_shift_top, length], bottom_cut / length, 0.0)
    } else {
        EdgeSequence::one(Edge::line([0.0, 0.0], [x_shift_top, length]))
    };

    let waist = Edge::line_v(
        right.last().borrow().end.clone(),
        vert_at([x_shift_top + top_width, length]),
    );

    let left = if bottom_cut != 0.0 {
        let s = side_with_cut(
            waist.borrow().end_p(),
            [low_width, 0.0],
            0.0,
            bottom_cut / length,
        );
        s.first().borrow_mut().start = waist.borrow().end.clone();
        s
    } else {
        EdgeSequence::one(Edge::line_v(
            waist.borrow().end.clone(),
            vert_at([low_width, 0.0]),
        ))
    };

    let bottom = Edge::line_v(
        left.last().borrow().end.clone(),
        right.first().borrow().start.clone(),
    );

    let right_edge = right.last().clone();
    let left_edge = left.first().clone();

    let top_ruffle = match_top_int_to
        .map(|m| waist.borrow().length() / m)
        .unwrap_or(ruffles);

    let ifaces = [
        ("right", Interface::one(&panel, right_edge)),
        (
            "top",
            Interface::new(&panel, EdgeSequence::one(waist.clone()), top_ruffle, false)
                .reversed(true),
        ),
        ("left", Interface::one(&panel, left_edge)),
        ("bottom", Interface::one(&panel, bottom.clone())),
    ];
    for (k, v) in ifaces {
        panel.borrow_mut().interfaces.set(k, v);
    }

    // A single sequence, for correct assembly.
    let mut edges = right;
    edges.push(waist);
    edges.extend(&left);
    edges.push(bottom);
    panel.borrow_mut().edges = edges;

    panel.borrow_mut().top_center_pivot();
    // This panel is known to be centred over Y.
    panel.borrow_mut().center_x();

    panel
}

/// One narrow panel of a many-panel skirt.
pub fn thin_skirt_panel(
    name: &str,
    top_width: f64,
    bottom_width: f64,
    length: f64,
    b_curvature: f64,
) -> PanelRef {
    let panel = Panel::new(name);

    let flare = (bottom_width - top_width) / 2.0;
    let mut edges = from_verts(
        &[
            [0.0, 0.0],
            [flare, length],
            [flare + top_width, length],
            [flare * 2.0 + top_width, 0.0],
        ],
        false,
    );

    if close_to_zero(b_curvature) {
        edges.close_loop();
    } else {
        let arc = circle_from_three_points(
            edges.last().borrow().end_p(),
            edges.first().borrow().start_p(),
            [0.5, b_curvature],
            true,
        );
        arc.borrow_mut().start = edges.last().borrow().end.clone();
        arc.borrow_mut().end = edges.first().borrow().start.clone();
        edges.push(arc);
    }

    let (e0, e1, e2, elast) = (
        edges[0].clone(),
        edges[1].clone(),
        edges[2].clone(),
        edges.last().clone(),
    );
    panel.borrow_mut().edges = edges;

    // Pivot at the top-left point.
    let pivot = e0.borrow().end_p();
    panel.borrow_mut().set_pivot(pivot, false);

    let ifaces = [
        ("right", Interface::one(&panel, e0)),
        ("top", Interface::one(&panel, e1)),
        ("left", Interface::one(&panel, e2)),
        ("bottom", Interface::one(&panel, elast)),
    ];
    for (k, v) in ifaces {
        panel.borrow_mut().interfaces.set(k, v);
    }

    panel
}

/// Parameters that differ between the front and back of a fitted skirt panel.
pub struct FittedSkirtOpts {
    pub waist: f64,
    pub hips: f64,
    pub hips_depth: f64,
    pub length: f64,
    pub hipline_ext: f64,
    pub dart_position: Option<f64>,
    pub dart_frac: f64,
    pub double_dart: bool,
    pub match_top_int_to: Option<f64>,
    pub slit: f64,
    pub left_slit: f64,
    pub right_slit: f64,
    pub side_cut: Option<shapes::SideCut>,
    pub flip_side_cut: bool,
}

impl Default for FittedSkirtOpts {
    fn default() -> Self {
        Self {
            waist: 0.0,
            hips: 0.0,
            hips_depth: 0.0,
            length: 0.0,
            hipline_ext: 1.0,
            dart_position: None,
            dart_frac: 0.5,
            double_dart: false,
            match_top_int_to: None,
            slit: 0.0,
            left_slit: 0.0,
            right_slit: 0.0,
            side_cut: None,
            flip_side_cut: false,
        }
    }
}

/// A fitted (pencil-skirt) panel.
pub fn fitted_skirt_panel(
    name: &str,
    body: &Body,
    design: &Design,
    o: &FittedSkirtOpts,
) -> PanelRef {
    let panel = Panel::new(name);

    let low_angle = design.f("low_angle");
    let hip_side_incl = body.get("_hip_inclination").to_radians();
    let flare = design.f("flare");
    // Distribute the flare difference equally between front and back.
    let low_width = body.get("hips") * (flare - 1.0) / 4.0 + o.hips;

    // Adjust for the rise.
    let adj_hips_depth = o.hips_depth * o.hipline_ext;
    let dart_depth = o.hips_depth * o.dart_frac;
    let dart_depth = (dart_depth - (o.hips_depth - adj_hips_depth)).max(0.0);

    // Extra fabric, distributed between the side angle and a dart.
    // The waist is smaller than the hips, so this is positive.
    let w_diff = o.hips - o.waist;
    let mut hw_shift = hip_side_incl.tan() * adj_hips_depth;
    if hw_shift > w_diff {
        hw_shift = w_diff;
    }

    // Tilt the hem to the requested angle.
    let angle_shift = low_angle.to_radians().tan() * low_width;

    // --- Edges ---
    let right_bottom = if close_enough(flare, 1.0, TOL) {
        // Straight skirt -- skip the optimisation.
        Edge::line([o.hips - low_width, angle_shift], [0.0, o.length])
    } else {
        curve_from_tangents(
            [o.hips - low_width, angle_shift],
            [0.0, o.length],
            None,
            Some([0.0, 1.0]),
            // Start with the control point closer to the hips.
            Some([0.75, 0.0]),
        )
    };
    let right_top = curve_from_tangents(
        right_bottom.borrow().end_p(),
        [hw_shift, o.length + adj_hips_depth],
        Some([0.0, 1.0]),
        None,
        Some([0.5, 0.0]),
    );
    right_top.borrow_mut().start = right_bottom.borrow().end.clone();
    let mut right = EdgeSequence::from_edges(vec![right_bottom.clone(), right_top.clone()]);

    let top = Edge::line_v(
        right.last().borrow().end.clone(),
        vert_at([o.hips * 2.0 - hw_shift, o.length + adj_hips_depth]),
    );

    let left_top = curve_from_tangents(
        top.borrow().end_p(),
        [o.hips * 2.0, o.length],
        None,
        Some([0.0, -1.0]),
        Some([0.5, 0.0]),
    );
    left_top.borrow_mut().start = top.borrow().end.clone();

    let left_bottom = if close_enough(flare, 1.0, TOL) {
        Edge::line(
            left_top.borrow().end_p(),
            [o.hips + low_width, -angle_shift],
        )
    } else {
        curve_from_tangents(
            left_top.borrow().end_p(),
            [o.hips + low_width, -angle_shift],
            Some([0.0, -1.0]),
            None,
            Some([0.25, 0.0]),
        )
    };
    left_bottom.borrow_mut().start = left_top.borrow().end.clone();
    let mut left = EdgeSequence::from_edges(vec![left_top.clone(), left_bottom.clone()]);

    let mut edges = EdgeSequence::new();
    edges.extend(&right);
    edges.push(top.clone());
    edges.extend(&left);
    edges.close_loop();

    let mut bottom_seq = EdgeSequence::one(edges.last().clone());

    if o.slit != 0.0 {
        // A long, thin, disconnected dart makes the cutout.
        let bottom = edges.last().clone();
        let offset = bottom.borrow().length() / 2.0;
        let res = ops::cut_into_edge_single(
            &dart_shape(2.0, None, Some(o.slit * o.length)),
            &bottom,
            offset,
            true,
            1e-2,
        );
        edges.substitute(&bottom, &res.new_edges);
        bottom_seq = res.leftovers;
    }

    let mut left_bottom = left_bottom;
    let mut right_bottom = right_bottom;

    if o.left_slit != 0.0 {
        let frac = o.left_slit;
        let new_left_bottom = subdivide_len(&left_bottom, &[1.0 - frac, frac], true);
        left.substitute(&left_bottom, &EdgeSequence::one(new_left_bottom[0].clone()));
        edges.substitute(&left_bottom, &new_left_bottom);
        left_bottom = new_left_bottom[0].clone();
    }

    if o.right_slit != 0.0 {
        let frac = o.right_slit;
        let new_rbottom = subdivide_len(&right_bottom, &[frac, 1.0 - frac], true);
        right.substitute(&right_bottom, &EdgeSequence::one(new_rbottom[1].clone()));
        edges.substitute(&right_bottom, &new_rbottom);
        right_bottom = new_rbottom[1].clone();
    }
    let _ = &right_bottom;

    if let Some(side_cut) = &o.side_cut {
        // A stylistic cutout. Skipped if it does not fit -- e.g. because a slit
        // already took that part of the edge.
        let offset = left_bottom.borrow().length() / 2.0;
        let fits = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match side_cut {
            shapes::SideCut::Single(seq) => {
                ops::cut_into_edge_single(seq, &left_bottom, offset, true, 1e-2)
            }
            shapes::SideCut::Multi(seqs) => {
                ops::cut_into_edge_multi(seqs, &left_bottom, offset, true, o.flip_side_cut, 1e-2)
            }
        }));
        if let Ok(res) = fits {
            edges.substitute(&left_bottom, &res.new_edges);
            left.substitute(&left_bottom, &res.new_edges);
        }
    }

    panel.borrow_mut().edges = edges.clone();

    // Default placement.
    panel.borrow_mut().top_center_pivot();
    // NOTE: assigned directly rather than through `translate_to`, so the panel
    // normal is not recomputed yet.
    panel.borrow_mut().translation = [-o.hips / 2.0, 5.0, 0.0];

    // Interfaces are easier to define before the darts go in.
    // The hipline extension carries a ruffle factor (used by the back panel).
    let right_ruffles: Vec<f64> = (0..right.len())
        .map(|i| {
            if i == right.len() - 1 {
                o.hipline_ext
            } else {
                1.0
            }
        })
        .collect();
    let left_ruffles: Vec<f64> = (0..left.len())
        .map(|i| if i == 0 { o.hipline_ext } else { 1.0 })
        .collect();

    let right_int = Interface::with_ruffles(&panel, right.clone(), &right_ruffles);
    let left_int = Interface::with_ruffles(&panel, left.clone(), &left_ruffles);
    left_int.borrow_mut().edges_flipping[0] = true;
    let n = right_int.borrow().edges_flipping.len();
    right_int.borrow_mut().edges_flipping[n - 1] = true;

    {
        let mut p = panel.borrow_mut();
        p.interfaces
            .set("bottom", Interface::plain(&panel, bottom_seq));
        p.interfaces.set("right", right_int);
        p.interfaces.set("left", left_int);
    }

    // Top darts.
    if w_diff > hw_shift {
        let dart_width = w_diff - hw_shift;
        let (top_edges, int_edges) = add_skirt_darts(
            &panel,
            &top,
            dart_width,
            dart_depth,
            o.dart_position.unwrap_or(0.0),
            o.double_dart,
        );

        let ruffle = o
            .match_top_int_to
            .map(|m| int_edges.length() / m)
            .unwrap_or(1.0);
        let iface = Interface::new(&panel, int_edges, ruffle, false);
        panel.borrow_mut().interfaces.set("top", iface);
        panel.borrow_mut().edges.substitute(&top, &top_edges);
    } else {
        let ruffle = o
            .match_top_int_to
            .map(|m| top.borrow().length() / m)
            .unwrap_or(1.0);
        let iface = Interface::new(&panel, EdgeSequence::one(top.clone()), ruffle, false);
        panel.borrow_mut().interfaces.set("top", iface);
    }

    panel
}

/// Insert one or two pairs of waist darts into the top edge.
fn add_skirt_darts(
    panel: &PanelRef,
    top: &EdgeRef,
    dart_width: f64,
    dart_depth: f64,
    dart_position: f64,
    double_dart: bool,
) -> (EdgeSequence, EdgeSequence) {
    let top_edge_len = top.borrow().length();

    let (offsets_mid, darts): (Vec<f64>, Vec<EdgeSequence>) = if double_dart {
        // Distance between darts -> distance between their centres.
        let dist = dart_position * 0.5;
        let offsets = vec![
            -(dart_position + dist / 2.0 + dart_width / 2.0) - dart_width / 4.0,
            -(dart_position - dist / 2.0) - dart_width / 4.0,
            dart_position - dist / 2.0 + dart_width / 4.0,
            dart_position + dist / 2.0 + dart_width / 2.0 + dart_width / 4.0,
        ];
        // NOTE: the reference passes the dart depth positionally, which lands on
        // `side_len` -- the dart's slanted side, not its perpendicular depth.
        let full = || dart_shape(dart_width / 2.0, Some(dart_depth), None);
        let small = || dart_shape(dart_width / 2.0, Some(dart_depth * 0.9), None);
        (offsets, vec![small(), full(), full(), small()])
    } else {
        let offsets = vec![
            -dart_position - dart_width / 2.0,
            dart_position + dart_width / 2.0,
        ];
        let shape = || dart_shape(dart_width, Some(dart_depth), None);
        (offsets, vec![shape(), shape()])
    };

    let mut top_edges = EdgeSequence::one(top.clone());
    let mut int_edges = EdgeSequence::one(top.clone());

    for (off, dart) in offsets_mid.iter().zip(darts) {
        let last = top_edges.last().clone();
        let left_edge_len = last.borrow().length();
        let offset = (left_edge_len - top_edge_len / 2.0) + off;

        let (te, ie) = ops::add_dart(
            panel,
            &dart,
            &last,
            offset,
            true,
            Some(&mut top_edges),
            Some(&mut int_edges),
        );
        top_edges = te;
        int_edges = ie;
    }

    (top_edges, int_edges)
}

/// A fitted pencil skirt.
pub fn pencil_skirt(body: &Body, design: &Design, opts: &SkirtOpts) -> CompRef {
    let name = if opts.tag.is_empty() {
        "PencilSkirt".to_string()
    } else {
        format!("PencilSkirt_{}", opts.tag)
    };
    let comp = Component::new(&name);

    let d = design.sub("pencil-skirt");

    let (style_l, style_r) = match d.s("style_side_cut") {
        Some(style) => {
            let depth = 0.7 * (body.get("hips") / 4.0 - body.get("bust_points") / 2.0);
            let (l, r) = shapes::build(
                &style,
                depth * 1.5,
                depth,
                6,
                depth * 0.2,
                d.node("style_side_file")
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
            );
            (Some(l), Some(r))
        }
        None => (None, None),
    };

    let rise = opts.rise.unwrap_or_else(|| d.f("rise"));
    comp.borrow_mut().rise = Some(rise);
    let r = eval_rise(body, rise);

    let length = match opts.length {
        // Depends on leg length.
        None => d.f("length") * body.get("_leg_length"),
        Some(l) => l - r.hips_depth,
    };

    let slit = |key: &str| if opts.slit { d.f(key) } else { 0.0 };

    let front = fitted_skirt_panel(
        "skirt_front",
        body,
        &d,
        &FittedSkirtOpts {
            waist: (r.waist - r.back_waist) / 2.0,
            hips: (body.get("hips") - body.get("hip_back_width")) / 2.0,
            hips_depth: r.hips_depth,
            length,
            dart_position: Some(body.get("bust_points") / 2.0),
            // Differs between front and back.
            dart_frac: 0.8,
            match_top_int_to: Some(body.get("waist") - body.get("waist_back_width")),
            slit: slit("front_slit"),
            left_slit: slit("left_slit"),
            right_slit: slit("right_slit"),
            side_cut: style_l,
            ..Default::default()
        },
    );
    front
        .borrow_mut()
        .translate_to([0.0, body.get("_waist_level"), 25.0]);

    let back = fitted_skirt_panel(
        "skirt_back",
        body,
        &d,
        &FittedSkirtOpts {
            waist: r.back_waist / 2.0,
            hips: body.get("hip_back_width") / 2.0,
            hips_depth: r.hips_depth,
            length,
            hipline_ext: 1.05,
            dart_position: Some(body.get("bum_points") / 2.0),
            dart_frac: 0.85,
            double_dart: true,
            match_top_int_to: Some(body.get("waist_back_width")),
            slit: slit("back_slit"),
            left_slit: slit("left_slit"),
            right_slit: slit("right_slit"),
            side_cut: style_r,
            flip_side_cut: false,
        },
    );
    back.borrow_mut()
        .translate_to([0.0, body.get("_waist_level"), -20.0]);

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
        c.interfaces.set("top_f", f.get("top"));
        c.interfaces.set("top_b", b.get("top"));
        c.interfaces.set(
            "top",
            Interface::from_multiple(&[f.get("top").flipped(), b.get("top").reversed(true)]),
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

/// A simple two-panel gathered skirt.
pub fn skirt_2(body: &Body, design: &Design, opts: &SkirtOpts) -> CompRef {
    let name = if opts.tag.is_empty() {
        "Skirt2".to_string()
    } else {
        format!("Skirt2_{}", opts.tag)
    };
    let comp = Component::new(&name);

    let d = design.sub("skirt");
    let rise = opts.rise.unwrap_or_else(|| d.f("rise"));
    comp.borrow_mut().rise = Some(rise);
    let r = eval_rise(body, rise);

    let length = opts
        .length
        .unwrap_or_else(|| r.hips_depth + d.f("length") * body.get("_leg_length"));
    let length = length.max(5.0);

    // Ruffles only make sense on a waistband.
    let ruffles = if opts.top_ruffles { d.f("ruffle") } else { 1.0 };
    let bottom_cut = if opts.slit {
        d.f("bottom_cut") * d.f("length")
    } else {
        0.0
    };

    let (front_name, back_name) = if opts.tag.is_empty() {
        ("skirt_front".to_string(), "skirt_back".to_string())
    } else {
        (
            format!("skirt_front_{}", opts.tag),
            format!("skirt_back_{}", opts.tag),
        )
    };

    let front = skirt_panel(
        &front_name,
        r.waist - r.back_waist,
        length,
        ruffles,
        Some(body.get("waist") - body.get("waist_back_width")),
        bottom_cut,
        d.f("flare"),
    );
    front
        .borrow_mut()
        .translate_to([0.0, body.get("_waist_level"), 25.0]);

    let back = skirt_panel(
        &back_name,
        r.back_waist,
        length,
        ruffles,
        Some(body.get("waist_back_width")),
        bottom_cut,
        d.f("flare"),
    );
    back.borrow_mut()
        .translate_to([0.0, body.get("_waist_level"), -20.0]);

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
        c.interfaces.set("top_f", f.get("top"));
        c.interfaces.set("top_b", b.get("top"));
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

/// A round skirt made of many narrow panels.
pub fn skirt_many_panels(body: &Body, design: &Design, opts: &SkirtOpts) -> CompRef {
    let n_panels = design.i("flare-skirt.skirt-many-panels.n_panels") as usize;
    let tag_extra = n_panels.to_string();
    let tag = if opts.tag.is_empty() {
        tag_extra.clone()
    } else {
        format!("{}_{}", opts.tag, tag_extra)
    };
    let comp = Component::new(&format!("SkirtManyPanels_{tag}"));

    let d = design.sub("flare-skirt");
    let rise = opts.rise.unwrap_or_else(|| d.f("rise"));
    comp.borrow_mut().rise = Some(rise);
    let r = eval_rise(body, rise);

    // Length depends on leg length.
    let length = r.hips_depth + d.f("length") * body.get("_leg_length");
    let length = length.max(5.0);

    let flare_coeff_pi = 1.0 + d.f("suns") * length * 2.0 * std::f64::consts::PI / r.waist;

    let panel_w = r.waist / n_panels as f64;
    let front = thin_skirt_panel(
        "front",
        panel_w,
        panel_w * flare_coeff_pi,
        length,
        d.f("skirt-many-panels.panel_curve"),
    );

    // Move far enough out that the widest part of the panels fits on the circle.
    let dist = front
        .borrow()
        .interfaces
        .get("bottom")
        .borrow()
        .edges
        .length()
        / (2.0 * (std::f64::consts::PI / n_panels as f64).tan());

    front
        .borrow_mut()
        .translate_to([-dist, body.get("_waist_level"), 0.0]);
    // Align the orientation with the body.
    front
        .borrow_mut()
        .rotate_by(Rotation::from_euler_xyz([0.0, -90.0, 0.0], true));
    front.borrow_mut().rotate_align([-dist, 0.0, panel_w / 2.0]);

    front
        .borrow()
        .interfaces
        .get("top")
        .borrow_mut()
        .reverse(true);

    let subs = ops::distribute_y(&el(&front), n_panels, 0.0, "skirt_panel");
    comp.borrow_mut().subs = subs.clone();
    comp.borrow_mut().length_mode = LengthMode::Sub(0);

    for i in 1..n_panels {
        let a = subs[i - 1].interface("left");
        let b = subs[i].interface("right");
        comp.borrow_mut().stitching_rules.append(a, b);
    }
    let a = subs[n_panels - 1].interface("left");
    let b = subs[0].interface("right");
    comp.borrow_mut().stitching_rules.append(a, b);

    let tops: Vec<InterfaceRef> = subs.iter().map(|s| s.interface("top")).collect();
    comp.borrow_mut()
        .interfaces
        .set("top", Interface::from_multiple(&tops));

    comp
}
