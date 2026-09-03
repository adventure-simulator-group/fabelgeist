//! Panels for a straight upper garment (a T-shirt).
//!
//! Ports `assets.garment_programs.tee`. The code closely mirrors the fitted
//! bodice, minus the darts.

use super::prelude::*;

/// Which half of the torso a panel covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorsoSide {
    Front,
    Back,
}

/// Half of a non-fitted upper garment, fitted to the bust measurement.
pub fn torso_half_panel(name: &str, body: &Body, design: &Design, side: TorsoSide) -> PanelRef {
    let panel = Panel::new(name);
    let d = design.sub("shirt");

    // Account for ease in the basic measurements.
    let m_width = d.f("width") * body.get("bust");
    let mut b_width = d.f("flare") * m_width;

    let bust = body.get("bust");
    let body_width = match side {
        TorsoSide::Front => (bust - body.get("back_width")) / 2.0,
        TorsoSide::Back => body.get("back_width") / 2.0,
    };
    let frac = body_width / bust;
    let width = frac * m_width;
    b_width *= frac;

    let sh_tan = body.get("_shoulder_incl").to_radians().tan();
    let shoulder_incl = sh_tan * width;
    let mut length = d.f("length") * body.get("waist_line");

    if side == TorsoSide::Front {
        // The front panel is shortened by the shoulder inclination so the
        // sleeve still fits.
        let fb_diff = (frac - (0.5 - frac)) * bust;
        length -= sh_tan * fb_diff;
    }

    let edges = from_verts(
        &[
            [0.0, 0.0],
            [-b_width, 0.0],
            [-width, length],
            [0.0, length + shoulder_incl],
        ],
        true,
    );

    let n = edges.len();
    let (e0, e1, e_m3, e_m2, e_m1) = (
        edges[0].clone(),
        edges[1].clone(),
        edges[n - 3].clone(),
        edges[n - 2].clone(),
        edges[n - 1].clone(),
    );
    panel.borrow_mut().edges = edges;

    let bottom_match = match side {
        TorsoSide::Front => (body.get("waist") - body.get("waist_back_width")) / 2.0,
        TorsoSide::Back => body.get("waist_back_width") / 2.0,
    };
    let bottom_ruffle = e0.borrow().length() / bottom_match;

    let mut shoulder_corner = EdgeSequence::one(e_m3);
    shoulder_corner.push(e_m2.clone());
    let mut collar_corner = EdgeSequence::one(e_m2.clone());
    collar_corner.push(e_m1.clone());

    let ifaces = [
        ("outside", Interface::one(&panel, e1)),
        ("inside", Interface::one(&panel, e_m1)),
        ("shoulder", Interface::one(&panel, e_m2)),
        (
            "bottom",
            Interface::new(&panel, EdgeSequence::one(e0), bottom_ruffle, false),
        ),
        // Corners the sleeve and collar shapes are projected onto.
        ("shoulder_corner", Interface::plain(&panel, shoulder_corner)),
        ("collar_corner", Interface::plain(&panel, collar_corner)),
    ];
    for (k, v) in ifaces {
        panel.borrow_mut().interfaces.set(k, v);
    }

    panel.borrow_mut().width_rule = Some(WidthRule::SlopeOffset {
        shoulder_w: body.get("shoulder_w"),
        width,
    });

    // Default placement.
    panel.borrow_mut().translate_by([
        0.0,
        body.get("height") - body.get("head_l") - length - shoulder_incl,
        0.0,
    ]);

    panel
}
