//! Sleeves and the armhole shapes they attach to.
//!
//! Ports `assets.garment_programs.sleeves`.

use super::bands;
use super::prelude::*;
use crate::curve::Curve;

// ----- Armhole shapes -----

/// The cut-out projected onto the bodice, plus (optionally) the matching sleeve
/// opening.
pub struct Armhole {
    /// The shape to project onto the bodice corner.
    pub project: EdgeSequence,
    /// The inverse shape the sleeve attaches with. `None` for sleeveless
    /// designs, where no sleeve is built.
    pub opening: Option<EdgeSequence>,
}

/// Which armhole style to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmholeShape {
    Square,
    Angle,
    Curve,
}

impl ArmholeShape {
    pub fn from_name(name: &str) -> Self {
        match name {
            "ArmholeSquare" => ArmholeShape::Square,
            "ArmholeAngle" => ArmholeShape::Angle,
            "ArmholeCurve" => ArmholeShape::Curve,
            other => panic!("sleeves::ERROR::unknown armhole shape '{other}'"),
        }
    }
}

/// Parameters shared by the armhole builders.
pub struct ArmholeParams {
    pub incl: f64,
    pub width: f64,
    pub angle: f64,
    pub incl_coeff: f64,
    pub w_coeff: f64,
    pub invert: bool,
    pub bottom_angle_mix: f64,
}

pub fn armhole(shape: ArmholeShape, p: &ArmholeParams) -> Armhole {
    match shape {
        ArmholeShape::Square => armhole_square(p),
        ArmholeShape::Angle => armhole_angle(p),
        ArmholeShape::Curve => armhole_curve(p),
    }
}

/// A plain square cut-out.
///
/// Not recommended with sleeves -- stitching it in 3D can be hard.
fn armhole_square(p: &ArmholeParams) -> Armhole {
    let edges = from_verts(&[[0.0, 0.0], [p.incl, 0.0], [p.incl, p.width]], false);
    if !p.invert {
        return Armhole {
            project: edges,
            opening: None,
        };
    }

    let (sina, cosa) = p.angle.sin_cos();
    let l = edges[0].borrow().length();
    let sleeve_edges = from_verts(
        &[
            [p.incl + l * sina, -l * cosa],
            [p.incl, 0.0],
            [p.incl, p.width],
        ],
        false,
    );
    sleeve_edges.rotate(-p.angle);

    Armhole {
        project: edges,
        opening: Some(sleeve_edges),
    }
}

/// A piece-wise smooth armhole.
fn armhole_angle(p: &ArmholeParams) -> Armhole {
    let diff_incl = p.incl * (1.0 - p.incl_coeff);
    let edges = from_verts(
        &[
            [0.0, 0.0],
            [diff_incl, p.w_coeff * p.width],
            [p.incl, p.width],
        ],
        false,
    );
    if !p.invert {
        return Armhole {
            project: edges,
            opening: None,
        };
    }

    let (sina, cosa) = p.angle.sin_cos();
    let l = edges[0].borrow().length();
    let sleeve_edges = from_verts(
        &[
            [diff_incl + l * sina, p.w_coeff * p.width - l * cosa],
            [diff_incl, p.w_coeff * p.width],
            [p.incl, p.width],
        ],
        false,
    );
    sleeve_edges.rotate(-p.angle);

    Armhole {
        project: edges,
        opening: Some(sleeve_edges),
    }
}

/// The classic sleeve opening, on cubic Bezier curves.
fn armhole_curve(p: &ArmholeParams) -> Armhole {
    let cps = [[0.5, 0.2], [0.8, 0.35]];
    let edge = Edge::curve([p.incl, p.width], [0.0, 0.0], cps.to_vec(), true);
    edge.borrow_mut().reverse();
    let edge_as_seq = EdgeSequence::one(edge.clone());

    if !p.invert {
        return Armhole {
            project: edge_as_seq,
            opening: None,
        };
    }

    // Initial guess for the inverse, at angle 0: a full opening is vertical.
    let down_direction: V2 = [0.0, -1.0];
    let mut inv_cps = cps;
    inv_cps[1][1] *= -1.0;

    let straight_len = dist2(edge.borrow().end_p(), edge.borrow().start_p());
    let inv_edge = Edge::curve(
        [p.incl, p.width],
        add2([p.incl, p.width], scale2(down_direction, straight_len)),
        inv_cps.to_vec(),
        true,
    );
    // Rotate by the sleeve's rest angle.
    inv_edge.borrow().rotate(-p.angle);

    let shortcut = inv_edge.borrow().shortcut();
    let rotated_direction = normalize2(sub2(shortcut[1], shortcut[0]));
    let left_direction: V2 = [-1.0, 0.0];
    let mix = p.bottom_angle_mix;

    let dir = add2(
        scale2(rotated_direction, 1.0 - mix),
        if mix > 0.0 {
            scale2(down_direction, mix)
        } else {
            scale2(left_direction, -mix)
        },
    );

    let target_len = edge.borrow().length();
    let fin_inv_edge = ops::curve_match_tangents(
        &inv_edge.borrow().as_curve(),
        down_direction,
        dir,
        Some(target_len),
    );
    fin_inv_edge.borrow_mut().reverse();

    Armhole {
        project: edge_as_seq,
        opening: Some(EdgeSequence::one(fin_inv_edge)),
    }
}

// ----- Sleeve panels -----

const MIN_SLEEVE_LENGTH: f64 = 5.0;
const STANDING_MARGIN: f64 = 5.0;

/// Half a sleeve.
///
/// `length_shift` forces the sleeve length by this amount, which is how the
/// cuff makes room for itself.
pub fn sleeve_panel(
    name: &str,
    body: &Body,
    design: &Design,
    open_shape: &mut EdgeSequence,
    length_shift: f64,
) -> PanelRef {
    let panel = Panel::new(name);

    let shoulder_angle = body.get("_shoulder_incl").to_radians();
    let rest_angle = design.f("sleeve_angle").to_radians().max(shoulder_angle);
    let mut standing = design.b("standing_shoulder");

    // Evaluate the extension and end sizes before any ruffles: ruffles add to
    // the pattern's length and width, but not to the sleeve's size in 3D.
    let opening_span =
        (open_shape.first().borrow().start_p()[1] - open_shape.last().borrow().end_p()[1]).abs();
    let mut end_width = design.f("end_width") * opening_span;
    // Make sure it fits whatever the parameters say.
    end_width = end_width.max(body.get("wrist") / 2.0);

    let connect_ruffle = design.f("connect_ruffle");
    if !close_enough(connect_ruffle, 1.0, TOL) {
        open_shape.extend_by(connect_ruffle);
    }

    // --- Main body of the sleeve ---
    let opening_length =
        (open_shape.first().borrow().start_p()[0] - open_shape.last().borrow().end_p()[0]).abs();
    let arm_width =
        (open_shape.first().borrow().start_p()[1] - open_shape.last().borrow().end_p()[1]).abs();

    // From the border of the opening to the end of the sleeve.
    let length = design.f("length") * (body.get("arm_length") - opening_length);
    // If asked to reduce by too much, reduce as much as possible.
    let length = (length + length_shift).max(MIN_SLEEVE_LENGTH);

    let mut edges = from_verts(
        &[[0.0, 0.0], [0.0, -end_width], [length, -arm_width]],
        false,
    );

    // Align the opening and chain it on.
    let anchor = edges.last().borrow().end.clone();
    open_shape.snap_to(vget(&anchor));
    open_shape.first().borrow_mut().start = anchor;
    edges.extend(open_shape);
    edges.close_loop();

    if standing {
        if rest_angle > shoulder_angle + STANDING_MARGIN.to_radians() {
            // A "shelf" that gives the shoulder a square appearance.
            let top_edge = edges.last().clone();
            let start = top_edge.borrow().start.clone();
            let len = design.f("standing_shoulder_len");

            let x_shift = len * (rest_angle - shoulder_angle).cos();
            let y_shift = len * (rest_angle - shoulder_angle).sin();

            let start_p = vget(&start);
            let standing_edge =
                Edge::line_v(start, vert_at([start_p[0] - x_shift, start_p[1] + y_shift]));
            top_edge.borrow_mut().start = standing_edge.borrow().end.clone();

            let mut replacement = EdgeSequence::one(standing_edge);
            replacement.push(top_edge.clone());
            edges.substitute(&top_edge, &replacement);
        } else {
            // The rest angle has to exceed the shoulder angle by the margin for
            // a standing shoulder to make sense.
            standing = false;
        }
    }

    let n = edges.len();
    let (e0, e1) = (edges[0].clone(), edges[1].clone());
    let top_edges = if standing {
        edges.slice(n - 2, n)
    } else {
        EdgeSequence::one(edges[n - 1].clone())
    };
    let last_start = edges.last().borrow().start_p();
    panel.borrow_mut().edges = edges;

    let ifaces = [
        // NOTE: the opening was reversed during construction, hence the
        // interface direction.
        (
            "in",
            Interface::new(&panel, open_shape.clone(), connect_ruffle, false),
        ),
        (
            "out",
            Interface::new(
                &panel,
                EdgeSequence::one(e0),
                design.f("cuff.top_ruffle"),
                false,
            ),
        ),
        ("top", Interface::plain(&panel, top_edges)),
        ("bottom", Interface::one(&panel, e1)),
    ];
    for (k, v) in ifaces {
        panel.borrow_mut().interfaces.set(k, v);
    }

    panel.borrow_mut().length_source = PanelLength::Interface("bottom".into());

    // Default placement.
    panel.borrow_mut().set_pivot(last_start, false);
    panel.borrow_mut().translate_to([
        -body.get("shoulder_w") / 2.0,
        body.get("height") - body.get("head_l"),
        0.0,
    ]);

    panel
}

// ----- Sleeve component -----

/// The bodice width at a given level, for the front and back halves.
///
/// Widths may be fixed numbers or functions of the vertical level -- fitted
/// bodices need the latter.
pub enum BodiceWidth {
    Fixed(f64),
    OfLevel(PanelRef),
}

impl BodiceWidth {
    fn at(&self, level: f64) -> f64 {
        match self {
            BodiceWidth::Fixed(v) => *v,
            BodiceWidth::OfLevel(p) => p.borrow().get_width(level),
        }
    }
}

/// A sleeve: two panels, the armhole shapes they attach with, and an optional
/// cuff.
pub fn sleeve(
    tag: &str,
    body: &Body,
    design: &Design,
    front_w: &BodiceWidth,
    back_w: &BodiceWidth,
) -> CompRef {
    let comp = Component::new(&format!("Sleeve_{tag}"));
    let d = design.sub("sleeve");

    let sleeve_balance = body.get("_base_sleeve_balance") / 2.0;
    let rest_angle = d
        .f("sleeve_angle")
        .to_radians()
        .max(body.get("_shoulder_incl").to_radians());

    let connecting_width = d.f("connecting_width");
    let smoothing_coeff = d.f("smoothing_coeff");
    let sleeveless = d.b("sleeveless");

    let front_w = front_w.at(connecting_width);
    let back_w = back_w.at(connecting_width);

    // NOTE: non-traditional armholes are only used for sleeveless styles --
    // inverting them is ambiguous and leads to stitching errors.
    let shape = if sleeveless {
        ArmholeShape::from_name(&d.s("armhole_shape").unwrap_or_default())
    } else {
        ArmholeShape::Curve
    };

    let params = |incl: f64| ArmholeParams {
        incl,
        width: connecting_width,
        angle: rest_angle,
        incl_coeff: smoothing_coeff,
        w_coeff: smoothing_coeff,
        invert: !sleeveless,
        bottom_angle_mix: d.f("opening_dir_mix"),
    };

    let front = armhole(shape, &params(front_w - sleeve_balance));
    let back = armhole(shape, &params(back_w - sleeve_balance));

    {
        let mut c = comp.borrow_mut();
        // These shapes belong to the component, not to a panel; they are only
        // ever projected onto the bodice.
        c.interfaces
            .set("in_front_shape", Interface::detached(front.project.clone()));
        c.interfaces
            .set("in_back_shape", Interface::detached(back.project.clone()));
    }

    if sleeveless {
        // Nothing else is needed.
        comp.borrow_mut().length_mode = LengthMode::Zero;
        return comp;
    }

    let mut front_opening = front.opening.expect("armhole opening for a sleeve");
    let mut back_opening = back.opening.expect("armhole opening for a sleeve");

    if front_w != back_w {
        // ~2 mm tolerance, as a fraction of the opening length.
        let tol = 0.2 / front_opening.length();
        ops::even_armhole_openings(&mut front_opening, &mut back_opening, tol);
    }

    let cuff_len_adj = cuff_len_adj(body, &d);

    let f_sleeve = sleeve_panel(
        &format!("{tag}_sleeve_f"),
        body,
        &d,
        &mut front_opening,
        -cuff_len_adj,
    );
    f_sleeve.borrow_mut().translate_by([0.0, 0.0, 15.0]);
    let b_sleeve = sleeve_panel(
        &format!("{tag}_sleeve_b"),
        body,
        &d,
        &mut back_opening,
        -cuff_len_adj,
    );
    b_sleeve.borrow_mut().translate_by([0.0, 0.0, -15.0]);

    let f_idx = add_sub(&comp, el(&f_sleeve));
    add_sub(&comp, el(&b_sleeve));

    let fi = f_sleeve.borrow().interfaces.clone();
    let bi = b_sleeve.borrow().interfaces.clone();
    comp.borrow_mut().stitching_rules = Stitches::from_pairs(vec![
        (fi.get("top"), bi.get("top")),
        (fi.get("bottom"), bi.get("bottom")),
    ]);

    {
        let mut c = comp.borrow_mut();
        c.interfaces.set(
            "in",
            Interface::from_multiple(&[fi.get("in"), bi.get("in").reversed(true)]),
        );
        c.interfaces.set(
            "out",
            Interface::from_multiple(&[fi.get("out"), bi.get("out")]),
        );
    }
    comp.borrow_mut().length_mode = LengthMode::Sub(f_idx);

    // Cuff.
    if let Some(cuff_name) = d.s("cuff.type") {
        let kind = bands::CuffKind::from_name(&cuff_name)
            .unwrap_or_else(|| panic!("sleeves::ERROR::unknown cuff type '{cuff_name}'"));

        // Copy, so the original design tree is left alone.
        let cdesign = d.deep_copy();
        let out_len = comp.borrow().interfaces.get("out").borrow().edges.length();
        let mut cuff_circ = out_len / d.f("cuff.top_ruffle");
        // Make sure it fits whatever the parameters say.
        cuff_circ = cuff_circ.max(body.get("wrist"));
        cdesign.set_f("cuff.b_width", cuff_circ);
        cdesign.set_f("cuff.cuff_len", cuff_len_adj);

        let cuff = bands::cuff(kind, &format!("sl_{tag}"), &cdesign);

        // Position: the cuff comes from the -Ox direction.
        ec(&cuff).rotate_by(Rotation::from_euler_xyz([0.0, 0.0, -90.0], true));
        let out_int = comp.borrow().interfaces.get("out");
        ec(&cuff).place_by_interface(
            &cuff.borrow().interfaces.get("top"),
            &out_int,
            2.0,
            Alignment::Top,
            None,
        );

        let cuff_top = cuff.borrow().interfaces.get("top");
        comp.borrow_mut().stitching_rules.append(cuff_top, out_int);

        let cuff_idx = add_sub(&comp, ec(&cuff));
        let cuff_bottom = cuff.borrow().interfaces.get("bottom");
        comp.borrow_mut().interfaces.set("out", cuff_bottom);
        comp.borrow_mut().length_mode = LengthMode::SubSum(vec![f_idx, cuff_idx]);
    }

    // Final rotation of the whole sleeve.
    ec(&comp).rotate_by(Rotation::from_euler_xyz(
        [0.0, 0.0, body.get("arm_pose_angle")],
        true,
    ));

    ec(&comp).set_panel_label("arm", true);

    comp
}

/// How much shorter the sleeve gets to make room for its cuff.
fn cuff_len_adj(body: &Body, d: &Design) -> f64 {
    if d.s("cuff.type").is_none() {
        return 0.0;
    }
    let mut adj = d.f("cuff.cuff_len") * body.get("arm_length");
    let max_len = d.f("length") * body.get("arm_length");
    if adj > max_len * 0.7 {
        adj = max_len * 0.7;
    }
    adj
}

/// The curve underlying an armhole edge, for callers that want to inspect it.
pub fn armhole_curve_of(seq: &EdgeSequence) -> Curve {
    seq.first().borrow().as_curve()
}
