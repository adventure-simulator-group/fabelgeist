//! Pants.
//!
//! Ports `assets.garment_programs.pants`.

use super::bands;
use super::prelude::*;

/// Parameters that differ between the front and back of a pant leg.
pub struct PantPanelOpts {
    pub length: f64,
    pub waist: f64,
    pub hips: f64,
    pub hips_depth: f64,
    pub crotch_width: f64,
    pub dart_position: f64,
    pub match_top_int_to: Option<f64>,
    pub hipline_ext: f64,
    pub double_dart: bool,
}

impl Default for PantPanelOpts {
    fn default() -> Self {
        Self {
            length: 0.0,
            waist: 0.0,
            hips: 0.0,
            hips_depth: 0.0,
            crotch_width: 0.0,
            dart_position: 0.0,
            match_top_int_to: None,
            hipline_ext: 1.0,
            double_dart: false,
        }
    }
}

/// One pant panel -- half a leg, front or back.
pub fn pant_panel(name: &str, body: &Body, design: &Design, o: &PantPanelOpts) -> PanelRef {
    let panel = Panel::new(name);

    let flare = body.get("leg_circ") * (design.f("flare") - 1.0) / 4.0;
    let hips_depth = o.hips_depth * o.hipline_ext;

    let hip_side_incl = body.get("_hip_inclination").to_radians();
    let dart_depth = hips_depth * 0.8;

    let crotch_depth_diff = body.get("crotch_hip_diff");
    let crotch_extention = o.crotch_width;

    // Extra fabric at the waist, split between the side angle and a dart.
    // The waist is smaller than the hips, so this is positive.
    let w_diff = o.hips - o.waist;
    let mut hw_shift = hip_side_incl.tan() * hips_depth;
    if hw_shift > w_diff {
        hw_shift = w_diff;
    }

    // --- Edges ---
    let right_bottom = if close_enough(design.f("flare"), 1.0, TOL) {
        Edge::line([-flare, 0.0], [0.0, o.length])
    } else {
        curve_from_tangents(
            [-flare, 0.0],
            [0.0, o.length],
            None,
            Some([0.0, 1.0]),
            // Start with the control point closer to the hips.
            Some([0.75, 0.0]),
        )
    };
    let right_top = curve_from_tangents(
        right_bottom.borrow().end_p(),
        [hw_shift, o.length + hips_depth],
        Some([0.0, 1.0]),
        None,
        Some([0.5, 0.0]),
    );
    right_top.borrow_mut().start = right_bottom.borrow().end.clone();

    let top = Edge::line_v(
        right_top.borrow().end.clone(),
        vert_at([w_diff + o.waist, o.length + hips_depth]),
    );

    // A bit higher than the hip line. NOTE: this must stay below the minimum
    // rise value (0.5).
    let crotch_top = Edge::line_v(
        top.borrow().end.clone(),
        vert_at([o.hips, o.length + 0.45 * hips_depth]),
    );

    let crotch_bottom = curve_from_tangents(
        crotch_top.borrow().end_p(),
        [o.hips + crotch_extention, o.length - crotch_depth_diff],
        Some([0.0, -1.0]),
        Some([1.0, 0.0]),
        Some([0.5, -0.5]),
    );
    crotch_bottom.borrow_mut().start = crotch_top.borrow().end.clone();

    // NOTE: the "magic" -2 cm sets the default width, just behind the crotch
    // point; keeping the same distance front and back makes the curves match.
    // The inside edge either matches the outside length, or -- when the wanted
    // length is shorter than the crotch depth -- covers a little of the inside
    // leg below the crotch (panties-like shorts).
    let cb_end = crotch_bottom.borrow().end_p();
    let y = (o.length - crotch_depth_diff * 1.5).min(0.0);
    let left = curve_from_tangents(
        cb_end,
        [cb_end[0] - 2.0 + flare, y],
        None,
        Some([flare, y - cb_end[1]]),
        Some([0.3, 0.0]),
    );
    left.borrow_mut().start = crotch_bottom.borrow().end.clone();

    let mut edges = EdgeSequence::from_edges(vec![
        right_bottom.clone(),
        right_top.clone(),
        top.clone(),
        crotch_top.clone(),
        crotch_bottom.clone(),
        left.clone(),
    ]);
    edges.close_loop();
    let bottom = edges.last().clone();
    panel.borrow_mut().edges = edges;

    // Default placement.
    panel.borrow_mut().set_pivot(cb_end, false);
    panel.borrow_mut().translation = [-0.5, -hips_depth - crotch_depth_diff + 5.0, 0.0];

    // Interfaces are easier to define before the dart goes in.
    let outside = EdgeSequence::from_edges(vec![right_bottom, right_top]);
    let crotch = EdgeSequence::from_edges(vec![crotch_top, crotch_bottom]);

    let ifaces = [
        (
            "outside",
            Interface::with_ruffles(&panel, outside, &[1.0, o.hipline_ext]),
        ),
        ("crotch", Interface::plain(&panel, crotch)),
        ("inside", Interface::one(&panel, left)),
        ("bottom", Interface::one(&panel, bottom)),
    ];
    for (k, v) in ifaces {
        panel.borrow_mut().interfaces.set(k, v);
    }

    // Top dart. The ruffle indicator matches the waistline proportion, which
    // keeps the balance line correct.
    let dart_width = w_diff - hw_shift;
    let ruffle = o.match_top_int_to.map(|m| o.waist / m).unwrap_or(1.0);

    if w_diff > hw_shift {
        let (top_edges, int_edges) = add_pant_darts(
            &panel,
            &top,
            dart_width,
            dart_depth,
            o.dart_position,
            o.double_dart,
        );
        let iface = Interface::new(&panel, int_edges, ruffle, false);
        panel.borrow_mut().interfaces.set("top", iface);
        panel.borrow_mut().edges.substitute(&top, &top_edges);
    } else {
        let iface = Interface::new(&panel, EdgeSequence::one(top.clone()), ruffle, false);
        panel.borrow_mut().interfaces.set("top", iface);
    }

    panel
}

fn add_pant_darts(
    panel: &PanelRef,
    top: &EdgeRef,
    dart_width: f64,
    dart_depth: f64,
    dart_position: f64,
    double_dart: bool,
) -> (EdgeSequence, EdgeSequence) {
    let (offsets_mid, darts): (Vec<f64>, Vec<EdgeSequence>) = if double_dart {
        // Distance between darts -> distance between their centres.
        let dist = dart_position * 0.5;
        (
            vec![
                -(dart_position + dist / 2.0 + dart_width / 2.0 + dart_width / 4.0),
                -(dart_position - dist / 2.0) - dart_width / 4.0,
            ],
            vec![
                // NOTE: the reference passes the depth positionally, so it
                // lands on the dart's side length.
                dart_shape(dart_width / 2.0, Some(dart_depth * 0.9), None),
                dart_shape(dart_width / 2.0, Some(dart_depth), None),
            ],
        )
    } else {
        (
            vec![-dart_position - dart_width / 2.0],
            vec![dart_shape(dart_width, Some(dart_depth), None)],
        )
    };

    let mut top_edges = EdgeSequence::one(top.clone());
    let mut int_edges = EdgeSequence::one(top.clone());

    for (off, dart) in offsets_mid.iter().zip(darts) {
        let last = top_edges.last().clone();
        let left_edge_len = last.borrow().length();
        let (te, ie) = ops::add_dart(
            panel,
            &dart,
            &last,
            left_edge_len + off,
            true,
            Some(&mut top_edges),
            Some(&mut int_edges),
        );
        top_edges = te;
        int_edges = ie;
    }

    (top_edges, int_edges)
}

/// One leg of a pair of pants, with an optional cuff.
pub fn pants_half(tag: &str, body: &Body, design: &Design, rise: Option<f64>) -> CompRef {
    let comp = Component::new(&format!("PantsHalf_{tag}"));
    let d = design.sub("pants");

    let rise = rise.unwrap_or_else(|| d.f("rise"));
    comp.borrow_mut().rise = Some(rise);
    let r = eval_rise(body, rise);

    // The minimum is a full sum greater than the leg circumference; the maximum
    // has the pant leg falling flat from the back, mostly from the back side.
    // This controls the foundation width of the pant.
    // (5 cm == the 2 inch ease from the pattern-making book.)
    let min_ext = body.get("leg_circ") - body.get("hips") / 2.0 + 5.0;
    let front_hip = (body.get("hips") - body.get("hip_back_width")) / 2.0;
    let crotch_extention = min_ext * d.f("width");
    // From the pattern-making book.
    let front_extention = front_hip / 4.0;
    let back_extention = crotch_extention - front_extention;

    let mut length = d.f("length");
    let mut cuff_len = d.f("cuff.cuff_len");
    let has_cuff = d.s("cuff.type").is_some();

    if has_cuff {
        let min_length = match d.node("length.range") {
            Some(Value::List(vs)) => vs[0].as_f64().unwrap_or(0.0),
            _ => 0.0,
        };
        if length - cuff_len < min_length {
            // The cuff cannot be longer than the pant.
            cuff_len = length - min_length;
        }
        // Fold the cuff into the overall length, unless that would make the
        // length negative.
        length -= cuff_len;
    }
    length *= body.get("_leg_length");
    cuff_len *= body.get("_leg_length");

    let front = pant_panel(
        &format!("pant_f_{tag}"),
        body,
        &d,
        &PantPanelOpts {
            length,
            waist: (r.waist - r.back_waist) / 2.0,
            hips: (body.get("hips") - body.get("hip_back_width")) / 2.0,
            hips_depth: r.hips_depth,
            dart_position: body.get("bust_points") / 2.0,
            crotch_width: front_extention,
            match_top_int_to: Some((body.get("waist") - body.get("waist_back_width")) / 2.0),
            ..Default::default()
        },
    );
    front
        .borrow_mut()
        .translate_by([0.0, body.get("_waist_level") - 5.0, 25.0]);

    let back = pant_panel(
        &format!("pant_b_{tag}"),
        body,
        &d,
        &PantPanelOpts {
            length,
            waist: r.back_waist / 2.0,
            hips: body.get("hip_back_width") / 2.0,
            hips_depth: r.hips_depth,
            hipline_ext: 1.1,
            dart_position: body.get("bum_points") / 2.0,
            crotch_width: back_extention,
            match_top_int_to: Some(body.get("waist_back_width") / 2.0),
            double_dart: true,
        },
    );
    back.borrow_mut()
        .translate_by([0.0, body.get("_waist_level") - 5.0, -20.0]);

    let front_idx = add_sub(&comp, el(&front));
    add_sub(&comp, el(&back));
    comp.borrow_mut().length_mode = LengthMode::Sub(front_idx);

    let f = front.borrow().interfaces.clone();
    let b = back.borrow().interfaces.clone();
    comp.borrow_mut().stitching_rules = Stitches::from_pairs(vec![
        (f.get("outside"), b.get("outside")),
        (f.get("inside"), b.get("inside")),
    ]);

    if let Some(cuff_name) = d.s("cuff.type") {
        let kind = bands::CuffKind::from_name(&cuff_name)
            .unwrap_or_else(|| panic!("pants::ERROR::unknown cuff type '{cuff_name}'"));

        let pant_bottom = Interface::from_multiple(&[f.get("bottom"), b.get("bottom")]);

        // Copy, so the original design tree is left alone.
        let cdesign = d.deep_copy();
        cdesign.set_f(
            "cuff.b_width",
            pant_bottom.borrow().edges.length() / d.f("cuff.top_ruffle"),
        );
        cdesign.set_f("cuff.cuff_len", cuff_len);

        let cuff = bands::cuff(kind, &format!("pant_{tag}"), &cdesign);

        ec(&cuff).place_by_interface(
            &cuff.borrow().interfaces.get("top"),
            &pant_bottom,
            5.0,
            Alignment::Left,
            None,
        );

        let cuff_top = cuff.borrow().interfaces.get("top");
        comp.borrow_mut()
            .stitching_rules
            .append(pant_bottom, cuff_top);

        let cuff_idx = add_sub(&comp, ec(&cuff));
        comp.borrow_mut().length_mode = LengthMode::SubSum(vec![front_idx, cuff_idx]);
    }

    {
        let mut c = comp.borrow_mut();
        c.interfaces.set("crotch_f", f.get("crotch"));
        c.interfaces.set("crotch_b", b.get("crotch"));
        c.interfaces.set("top_f", f.get("top"));
        c.interfaces.set("top_b", b.get("top"));
    }

    comp
}

/// A pair of pants.
pub fn pants(body: &Body, design: &Design, rise: Option<f64>) -> CompRef {
    let comp = Component::new("Pants");

    let right = pants_half("r", body, design, rise);
    let left = pants_half("l", body, design, rise);
    ec(&left).mirror([0.0, 1.0]);

    comp.borrow_mut().rise = right.borrow().rise;

    add_sub(&comp, ec(&right));
    add_sub(&comp, ec(&left));
    comp.borrow_mut().length_mode = LengthMode::Sub(0);

    let r = right.borrow().interfaces.clone();
    let l = left.borrow().interfaces.clone();

    comp.borrow_mut().stitching_rules = Stitches::from_pairs(vec![
        (r.get("crotch_f"), l.get("crotch_f")),
        (r.get("crotch_b"), l.get("crotch_b")),
    ]);

    {
        let mut c = comp.borrow_mut();
        c.interfaces.set(
            "top_f",
            Interface::from_multiple(&[r.get("top_f"), l.get("top_f")]),
        );
        c.interfaces.set(
            "top_b",
            Interface::from_multiple(&[r.get("top_b"), l.get("top_b")]),
        );
        // Around the body, starting from the front right. Some sides are
        // reversed so the seams connect in the right order.
        c.interfaces.set(
            "top",
            Interface::from_multiple(&[
                r.get("top_f").flipped(),
                l.get("top_f").reversed(true),
                l.get("top_b").flipped(),
                // Flips the edges and restores the direction.
                r.get("top_b").reversed(true),
            ]),
        );
    }

    comp
}
