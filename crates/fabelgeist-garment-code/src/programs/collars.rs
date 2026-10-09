//! Necklines and collars.
//!
//! Ports `assets.garment_programs.collars`. The neckline functions produce the
//! half-shape that is projected onto a bodice's collar corner; the collar
//! components add panels on top of that.

use super::bands::straight_band_panel;
use super::circle_skirt::circle_arc_panel;
use super::prelude::*;

// ----- Necklines without extra panels -----

/// Parameters shared by the neckline shapes.
#[derive(Debug, Clone, Copy)]
pub struct NeckParams {
    pub depth: f64,
    pub width: f64,
    pub angle: f64,
    pub flip: bool,
    pub bezier_x: f64,
    pub bezier_y: f64,
}

/// Build a named neckline half.
pub fn neckline(name: &str, p: &NeckParams) -> EdgeSequence {
    match name {
        "VNeckHalf" => v_neck_half(p.depth, p.width),
        "SquareNeckHalf" => square_neck_half(p.depth, p.width),
        "TrapezoidNeckHalf" => trapezoid_neck_half(p.depth, p.width, p.angle),
        "CurvyNeckHalf" => curvy_neck_half(p.depth, p.width, p.flip),
        "CircleArcNeckHalf" => circle_arc_neck_half(p.depth, p.width, p.angle, p.flip),
        "CircleNeckHalf" => circle_neck_half(p.depth, p.width),
        "Bezier2NeckHalf" => bezier2_neck_half(p.depth, p.width, p.flip, p.bezier_x, p.bezier_y),
        other => panic!("collars::ERROR::unknown neckline '{other}'"),
    }
}

/// A simple V.
pub fn v_neck_half(depth: f64, width: f64) -> EdgeSequence {
    EdgeSequence::one(Edge::line([0.0, 0.0], [width / 2.0, -depth]))
}

/// A square neckline.
pub fn square_neck_half(depth: f64, width: f64) -> EdgeSequence {
    from_verts(&[[0.0, 0.0], [0.0, -depth], [width / 2.0, -depth]], false)
}

/// A trapezoid. Degenerates to a V for angles near 0 or 180 degrees, or when
/// the parameters would make the shape self-overlap.
pub fn trapezoid_neck_half(depth: f64, width: f64, angle: f64) -> EdgeSequence {
    if close_enough(angle, 180.0, 1.0) || close_enough(angle, 0.0, 1.0) {
        return v_neck_half(depth, width);
    }

    let rad_angle = angle.to_radians();
    let bottom_x = -depth * rad_angle.cos() / rad_angle.sin();
    if bottom_x > width / 2.0 {
        // Invalid angle/depth/width combination -- it would create an overlap.
        return v_neck_half(depth, width);
    }

    from_verts(
        &[[0.0, 0.0], [bottom_x, -depth], [width / 2.0, -depth]],
        false,
    )
}

/// A curvy neckline, on a cubic Bezier.
pub fn curvy_neck_half(depth: f64, width: f64, flip: bool) -> EdgeSequence {
    let sign = if flip { -1.0 } else { 1.0 };
    EdgeSequence::one(Edge::curve(
        [0.0, 0.0],
        [width / 2.0, -depth],
        vec![[0.4, sign * 0.3], [0.8, sign * -0.3]],
        true,
    ))
}

/// A neckline whose side is a circular arc.
pub fn circle_arc_neck_half(depth: f64, width: f64, angle: f64, flip: bool) -> EdgeSequence {
    EdgeSequence::one(circle_from_points_angle(
        [0.0, 0.0],
        [width / 2.0, -depth],
        angle.to_radians(),
        !flip,
    ))
}

/// A neckline that forms a perfect circular arc once both halves are stitched.
pub fn circle_neck_half(depth: f64, width: f64) -> EdgeSequence {
    // Build the full desired arc, then take half of it.
    let circle = circle_from_three_points([0.0, 0.0], [width, 0.0], [width / 2.0, -depth], false);
    let subdiv = subdivide_len(&circle, &[0.5, 0.5], true);
    EdgeSequence::one(subdiv[0].clone())
}

/// A quadratic Bezier neckline.
pub fn bezier2_neck_half(depth: f64, width: f64, flip: bool, x: f64, y: f64) -> EdgeSequence {
    let sign = if flip { 1.0 } else { -1.0 };
    EdgeSequence::one(Edge::curve(
        [0.0, 0.0],
        [width / 2.0, -depth],
        vec![[x, sign * y]],
        true,
    ))
}

// ----- Collars with panels -----

/// Which collar component to build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollarStyle {
    /// Projected shapes only -- no extra panels.
    NoPanels,
    Turtle,
    SimpleLapel,
    Hood2Panels,
}

impl CollarStyle {
    pub fn from_name(name: Option<&str>) -> Self {
        match name {
            Some("Turtle") => CollarStyle::Turtle,
            Some("SimpleLapel") => CollarStyle::SimpleLapel,
            Some("Hood2Panels") => CollarStyle::Hood2Panels,
            // Anything unrecognised falls back to the shape-only collar, as the
            // reference's `getattr(collars, name, NoPanelsCollar)` does.
            _ => CollarStyle::NoPanels,
        }
    }
}

/// Build a collar component of the requested style.
pub fn collar(style: CollarStyle, name: &str, body: &Body, design: &Design) -> CompRef {
    match style {
        CollarStyle::NoPanels => no_panels_collar(name, design),
        CollarStyle::Turtle => turtle(name, body, design),
        CollarStyle::SimpleLapel => simple_lapel(name, body, design),
        CollarStyle::Hood2Panels => hood_2_panels(name, body, design),
    }
}

fn front_neck_params(design: &Design) -> NeckParams {
    let d = design.sub("collar");
    NeckParams {
        depth: d.f("fc_depth"),
        width: d.f("width"),
        angle: d.f("fc_angle"),
        flip: d.b("f_flip_curve"),
        bezier_x: d.f("f_bezier_x"),
        bezier_y: d.f("f_bezier_y"),
    }
}

fn back_neck_params(design: &Design) -> NeckParams {
    let d = design.sub("collar");
    NeckParams {
        depth: d.f("bc_depth"),
        width: d.f("width"),
        angle: d.f("bc_angle"),
        flip: d.b("b_flip_curve"),
        bezier_x: d.f("b_bezier_x"),
        bezier_y: d.f("b_bezier_y"),
    }
}

/// A collar that only contributes projected shapes.
pub fn no_panels_collar(name: &str, design: &Design) -> CompRef {
    let comp = Component::new(name);
    let d = design.sub("collar");

    let f_collar = neckline(
        &d.s("f_collar").expect("front collar style"),
        &front_neck_params(design),
    );
    let b_collar = neckline(
        &d.s("b_collar").expect("back collar style"),
        &back_neck_params(design),
    );

    {
        let mut c = comp.borrow_mut();
        c.interfaces
            .set("front_proj", Interface::detached(f_collar));
        c.interfaces.set("back_proj", Interface::detached(b_collar));
        c.length_mode = LengthMode::Zero;
    }
    comp
}

/// A turtleneck: two straight bands standing above the neckline.
pub fn turtle(tag: &str, body: &Body, design: &Design) -> CompRef {
    let comp = Component::new(&format!("Turtle_{tag}"));
    let d = design.sub("collar");
    let depth = d.f("component.depth");

    let f_collar = circle_neck_half(d.f("fc_depth"), d.f("width"));
    let b_collar = circle_neck_half(d.f("bc_depth"), d.f("width"));

    let length_f = f_collar.length();
    let length_b = b_collar.length();

    {
        let mut c = comp.borrow_mut();
        c.interfaces
            .set("front_proj", Interface::detached(f_collar));
        c.interfaces.set("back_proj", Interface::detached(b_collar));
    }

    let height_p = body.get("height") - body.get("head_l") + depth;

    let front = straight_band_panel(&format!("{tag}_collar_front"), length_f, depth, None);
    front
        .borrow_mut()
        .translate_by([-length_f / 2.0, height_p, 10.0]);
    let back = straight_band_panel(&format!("{tag}_collar_back"), length_b, depth, None);
    back.borrow_mut()
        .translate_by([-length_b / 2.0, height_p, -10.0]);

    add_sub(&comp, el(&front));
    add_sub(&comp, el(&back));

    let f = front.borrow().interfaces.clone();
    let b = back.borrow().interfaces.clone();
    comp.borrow_mut()
        .stitching_rules
        .append(f.get("right"), b.get("right"));

    {
        let mut c = comp.borrow_mut();
        c.interfaces.set("front", f.get("left"));
        c.interfaces.set("back", b.get("left"));
        c.interfaces.set(
            "bottom",
            Interface::from_multiple(&[f.get("bottom"), b.get("bottom")]),
        );
        c.length_mode = LengthMode::Interface("back".into());
    }

    comp
}

/// The front panel of a simple lapel.
pub fn simple_lapel_panel(name: &str, length: f64, max_depth: f64) -> PanelRef {
    let panel = Panel::new(name);

    let mut edges = from_verts(&[[0.0, 0.0], [max_depth, 0.0], [max_depth, -length]], false);
    let curve = Edge::curve(
        edges.last().borrow().end_p(),
        edges.first().borrow().start_p(),
        vec![[0.7, 0.2]],
        true,
    );
    curve.borrow_mut().start = edges.last().borrow().end.clone();
    curve.borrow_mut().end = edges.first().borrow().start.clone();
    edges.push(curve);

    let (e0, e1) = (edges[0].clone(), edges[1].clone());
    panel.borrow_mut().edges = edges;

    panel
        .borrow_mut()
        .interfaces
        .set("to_collar", Interface::one(&panel, e0));
    panel
        .borrow_mut()
        .interfaces
        .set("to_bodice", Interface::one(&panel, e1));

    panel
}

/// A simple lapel collar.
pub fn simple_lapel(tag: &str, body: &Body, design: &Design) -> CompRef {
    // NOTE: the reference names this component "Turtle_<tag>" too.
    let comp = Component::new(&format!("Turtle_{tag}"));
    let d = design.sub("collar");

    let depth = d.f("component.depth");
    let standing = d.b("component.lapel_standing");

    // Any front neckline will do here.
    let f_collar = neckline(
        &d.s("f_collar").expect("front collar style"),
        &front_neck_params(design),
    );
    let b_collar = circle_neck_half(d.f("bc_depth"), d.f("width"));

    let length_f = f_collar.length();
    let length_b = b_collar.length();
    let b_first = b_collar.first().clone();

    {
        let mut c = comp.borrow_mut();
        c.interfaces
            .set("front_proj", Interface::detached(f_collar));
        c.interfaces.set("back_proj", Interface::detached(b_collar));
    }

    let height_p = body.get("height") - body.get("head_l") + depth * 2.0;

    let front = simple_lapel_panel(&format!("{tag}_collar_front"), length_f, depth);
    // TODOLOW (reference): this placement should follow the bodice panels.
    front
        .borrow_mut()
        .translate_by([-depth * 2.0, height_p, 35.0]);

    let back = if standing {
        let p = straight_band_panel(&format!("{tag}_collar_back"), length_b, depth, None);
        p.borrow_mut()
            .translate_by([-length_b / 2.0, height_p, -10.0]);
        p
    } else {
        // A curved back panel that follows the collar opening.
        let (rad, angle, _) = b_first.borrow().as_radius_angle();
        let p = circle_arc_panel(&format!("{tag}_collar_back"), rad, depth, angle, None, None);
        p.borrow_mut().translate_by([-length_b, height_p, -10.0]);
        p.borrow_mut()
            .rotate_by(Rotation::from_euler_xyz([90.0, 45.0, 0.0], true));
        p
    };

    add_sub(&comp, el(&front));
    add_sub(&comp, el(&back));

    let f = front.borrow().interfaces.clone();
    let b = back.borrow().interfaces.clone();

    if standing {
        b.get("right").borrow_mut().set_right_wrong(true);
    }

    comp.borrow_mut()
        .stitching_rules
        .append(f.get("to_collar"), b.get("right"));

    {
        let mut c = comp.borrow_mut();
        // NOTE: no front interface here.
        c.interfaces.set("back", b.get("left"));
        let bottom_second = if standing {
            b.get("bottom")
        } else {
            b.get("top").right_to_wrong(true)
        };
        c.interfaces.set(
            "bottom",
            Interface::from_multiple(&[f.get("to_bodice").right_to_wrong(true), bottom_second]),
        );
        c.length_mode = LengthMode::Interface("back".into());
    }

    comp
}

/// One side of a hood.
#[allow(clippy::too_many_arguments)]
pub fn hood_panel(
    name: &str,
    f_depth: f64,
    b_depth: f64,
    f_length: f64,
    b_length: f64,
    width: f64,
    in_length: f64,
    depth: f64,
) -> PanelRef {
    let panel = Panel::new(name);

    // The panel covers one half only.
    let width = width / 2.0;
    let length = in_length + width / 2.0;

    let mut edges = EdgeSequence::new();

    // Bottom-back.
    let bottom_back_in = Edge::curve(
        [-width, -b_depth],
        [0.0, 0.0],
        vec![[0.3, -0.2], [0.6, 0.2]],
        true,
    );
    let bottom_back = ops::curve_match_tangents(
        &bottom_back_in.borrow().as_curve(),
        [1.0, 0.0],
        [1.0, 0.0],
        Some(b_length),
    );
    edges.push(bottom_back);

    // Bottom-front.
    let bottom_front_in = Edge::curve(
        edges.last().borrow().end_p(),
        [width, -f_depth],
        vec![[0.3, 0.2], [0.6, -0.2]],
        true,
    );
    let bottom_front = ops::curve_match_tangents(
        &bottom_front_in.borrow().as_curve(),
        [1.0, 0.0],
        [1.0, 0.0],
        Some(f_length),
    );
    bottom_front.borrow_mut().start = edges.last().borrow().end.clone();
    edges.push(bottom_front);

    // Front-top straight section.
    let straight = from_verts(
        &[
            edges.last().borrow().end_p(),
            [width * 1.2, length],
            [width * 1.2 - depth, length],
        ],
        false,
    );
    straight.first().borrow_mut().start = edges.last().borrow().end.clone();
    edges.extend(&straight);

    // Back of the hood.
    let back = Edge::curve(
        edges.last().borrow().end_p(),
        edges.first().borrow().start_p(),
        vec![[0.2, -0.5]],
        true,
    );
    back.borrow_mut().start = edges.last().borrow().end.clone();
    back.borrow_mut().end = edges.first().borrow().start.clone();
    edges.push(back);

    let n = edges.len();
    let to_other_side = edges.slice(n - 2, n);
    let to_bodice = edges.slice(0, 2);
    panel.borrow_mut().edges = edges;

    let to_bodice_int = Interface::plain(&panel, to_bodice).reversed(false);
    {
        let mut p = panel.borrow_mut();
        p.interfaces
            .set("to_other_side", Interface::plain(&panel, to_other_side));
        p.interfaces.set("to_bodice", to_bodice_int);
    }

    panel
        .borrow_mut()
        .rotate_by(Rotation::from_euler_xyz([0.0, -90.0, 0.0], true));
    panel.borrow_mut().translate_by([-width, 0.0, 0.0]);

    panel
}

/// A two-panel hood.
pub fn hood_2_panels(tag: &str, body: &Body, design: &Design) -> CompRef {
    let comp = Component::new(&format!("Hood_{tag}"));
    let d = design.sub("collar");

    let width = d.f("width");
    let f_collar = circle_neck_half(d.f("fc_depth"), width);
    let b_collar = circle_neck_half(d.f("bc_depth"), width);
    let (f_length, b_length) = (f_collar.length(), b_collar.length());

    {
        let mut c = comp.borrow_mut();
        c.interfaces
            .set("front_proj", Interface::detached(f_collar));
        c.interfaces.set("back_proj", Interface::detached(b_collar));
    }

    let panel = hood_panel(
        &format!("{tag}_hood"),
        d.f("fc_depth"),
        d.f("bc_depth"),
        f_length,
        b_length,
        width,
        body.get("head_l") * d.f("component.hood_length"),
        width / 2.0 * d.f("component.hood_depth"),
    );
    panel
        .borrow_mut()
        .translate_by([0.0, body.get("height") - body.get("head_l") + 10.0, 0.0]);

    add_sub(&comp, el(&panel));

    let p = panel.borrow().interfaces.clone();
    {
        let mut c = comp.borrow_mut();
        // NOTE: no front interface here.
        c.interfaces.set("back", p.get("to_other_side"));
        c.interfaces.set("bottom", p.get("to_bodice"));
        c.length_mode = LengthMode::Sub(0);
    }

    comp
}
