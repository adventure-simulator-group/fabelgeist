//! Waistbands and cuffs.
//!
//! Ports `assets.garment_programs.bands`.

use super::circle_skirt::circle_arc_from_all_length;
use super::prelude::*;
use super::skirt_paneled::skirt_panel;

/// A plain rectangular band panel.
pub fn straight_band_panel(
    name: &str,
    width: f64,
    depth: f64,
    match_int_proportion: Option<f64>,
) -> PanelRef {
    let panel = Panel::new(name);

    let edges = from_verts(
        &[[0.0, 0.0], [0.0, depth], [width, depth], [width, 0.0]],
        true,
    );
    let (e0, e1, e2, e3) = (
        edges[0].clone(),
        edges[1].clone(),
        edges[2].clone(),
        edges[3].clone(),
    );
    panel.borrow_mut().edges = edges;

    let ruffle = match_int_proportion.map(|m| width / m).unwrap_or(1.0);

    let ifaces = [
        ("right", Interface::one(&panel, e0)),
        (
            "top",
            Interface::new(&panel, EdgeSequence::one(e1), ruffle, false).reversed(true),
        ),
        ("left", Interface::one(&panel, e2)),
        (
            "bottom",
            Interface::new(&panel, EdgeSequence::one(e3), ruffle, false),
        ),
    ];
    for (k, v) in ifaces {
        panel.borrow_mut().interfaces.set(k, v);
    }

    panel.borrow_mut().top_center_pivot();
    panel.borrow_mut().center_x();

    panel
}

/// Shape of a waistband: straight (two rectangles) or fitted (two arcs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaistbandKind {
    /// A simple two-panel band.
    Straight,
    /// A yoke: two circular-arc panels that follow the body's curvature and so
    /// sit tight.
    Fitted,
}

/// A waistband.
///
/// `rise` is the rise of the bottoms the band attaches to; the band adapts its
/// shape to sit tight on top of that level. A rise of 1 (or anything smaller
/// than the band's own width) is ignored and the band is built to sit on the
/// waist.
pub fn waistband(body: &Body, design: &Design, rise: f64, kind: WaistbandKind) -> CompRef {
    let class_name = match kind {
        WaistbandKind::Straight => "StraightWB",
        WaistbandKind::Fitted => "FittedWB",
    };
    let comp = Component::new(class_name);

    let waist_scale = design.f("waistband.waist");
    let waist = waist_scale * body.get("waist");
    let waist_back_frac = body.get("waist_back_width") / body.get("waist");
    let hips = body.get("hips") * waist_scale;
    let hips_back_frac = body.get("hip_back_width") / body.get("hips");

    let mut width = design.f("waistband.width");
    let mut rise = rise;
    if rise + width > 1.0 {
        rise = 1.0 - width;
    }
    comp.borrow_mut().rise = Some(rise);

    let top_width = lin_interpolation(hips, waist, rise + width);
    let top_back_fraction = lin_interpolation(hips_back_frac, waist_back_frac, rise + width);
    width *= body.get("hips_line");

    let match_front = body.get("waist") - body.get("waist_back_width");
    let match_back = body.get("waist_back_width");

    let (front, back) = match kind {
        WaistbandKind::Straight => {
            let back_width = top_width * top_back_fraction;
            (
                straight_band_panel("wb_front", top_width - back_width, width, Some(match_front)),
                straight_band_panel("wb_back", back_width, width, Some(match_back)),
            )
        }
        WaistbandKind::Fitted => {
            let bottom_width = lin_interpolation(hips, waist, rise);
            let bottom_back_fraction = lin_interpolation(hips_back_frac, waist_back_frac, rise);
            (
                circle_arc_from_all_length(
                    "wb_front",
                    width,
                    top_width * (1.0 - top_back_fraction),
                    bottom_width * (1.0 - bottom_back_fraction),
                    Some(match_front),
                    Some(match_front),
                ),
                circle_arc_from_all_length(
                    "wb_back",
                    width,
                    top_width * top_back_fraction,
                    bottom_width * bottom_back_fraction,
                    Some(match_back),
                    Some(match_back),
                ),
            )
        }
    };

    front
        .borrow_mut()
        .translate_by([0.0, body.get("_waist_level"), 20.0]);
    back.borrow_mut()
        .translate_by([0.0, body.get("_waist_level"), -15.0]);

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
        c.interfaces.set("bottom_f", f.get("bottom"));
        c.interfaces.set("bottom_b", b.get("bottom"));
        c.interfaces.set("top_f", f.get("top"));
        c.interfaces.set("top_b", b.get("top"));
        c.interfaces.set(
            "bottom",
            Interface::from_multiple(&[f.get("bottom"), b.get("bottom")]),
        );
        c.interfaces.set(
            "top",
            Interface::from_multiple(&[f.get("top"), b.get("top")]),
        );
    }

    comp
}

/// Cuff style, for sleeves and pant legs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CuffKind {
    /// A plain band.
    Band,
    /// A flared, skirt-like cuff.
    Skirt,
    /// A band with a skirt hanging from it.
    BandSkirt,
}

impl CuffKind {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "CuffBand" => Some(CuffKind::Band),
            "CuffSkirt" => Some(CuffKind::Skirt),
            "CuffBandSkirt" => Some(CuffKind::BandSkirt),
            _ => None,
        }
    }
}

/// Build a cuff of the requested kind.
///
/// `design` is the sleeve's or pants' design subtree -- the cuff parameters are
/// read from its `cuff` entry.
pub fn cuff(kind: CuffKind, tag: &str, design: &Design) -> CompRef {
    match kind {
        CuffKind::Band => cuff_band(tag, design, None),
        CuffKind::Skirt => cuff_skirt(tag, design, None),
        CuffKind::BandSkirt => cuff_band_skirt(tag, design),
    }
}

/// A band-like cuff.
pub fn cuff_band(tag: &str, design: &Design, length: Option<f64>) -> CompRef {
    let comp = Component::new(&format!("CuffBand_{tag}"));
    let d = design.sub("cuff");

    let length = length.unwrap_or_else(|| d.f("cuff_len"));
    let half_width = d.f("b_width") / 2.0;

    let front = straight_band_panel(&format!("{tag}_cuff_f"), half_width, length, None);
    front.borrow_mut().translate_by([0.0, 0.0, 15.0]);
    let back = straight_band_panel(&format!("{tag}_cuff_b"), half_width, length, None);
    back.borrow_mut().translate_by([0.0, 0.0, -15.0]);

    finish_cuff(&comp, &front, &back);
    comp
}

/// A skirt-like flared cuff.
pub fn cuff_skirt(tag: &str, design: &Design, length: Option<f64>) -> CompRef {
    let comp = Component::new(&format!("CuffSkirt_{tag}"));
    let d = design.sub("cuff");

    let width = d.f("b_width");
    let flare_diff = (d.f("skirt_flare") - 1.0) * width / 2.0;
    let length = length.unwrap_or_else(|| d.f("cuff_len"));
    let ruffles = d.f("skirt_ruffle");

    let front = skirt_panel(
        &format!("{tag}_cuff_skirt_f"),
        width / 2.0,
        length,
        ruffles,
        None,
        0.0,
        flare_diff,
    );
    front.borrow_mut().translate_by([0.0, 0.0, 15.0]);
    let back = skirt_panel(
        &format!("{tag}_cuff_skirt_b"),
        width / 2.0,
        length,
        ruffles,
        None,
        0.0,
        flare_diff,
    );
    back.borrow_mut().translate_by([0.0, 0.0, -15.0]);

    finish_cuff(&comp, &front, &back);
    comp
}

/// Shared wiring for the two-panel cuffs.
fn finish_cuff(comp: &CompRef, front: &PanelRef, back: &PanelRef) {
    add_sub(comp, el(front));
    add_sub(comp, el(back));
    comp.borrow_mut().length_mode = LengthMode::Sub(0);

    let f = front.borrow().interfaces.clone();
    let b = back.borrow().interfaces.clone();
    comp.borrow_mut().stitching_rules = Stitches::from_pairs(vec![
        (f.get("right"), b.get("right")),
        (f.get("left"), b.get("left")),
    ]);

    let mut c = comp.borrow_mut();
    c.interfaces.set(
        "bottom",
        Interface::from_multiple(&[f.get("bottom"), b.get("bottom")]),
    );
    c.interfaces.set("top_front", f.get("top"));
    c.interfaces.set("top_back", b.get("top"));
    c.interfaces.set(
        "top",
        Interface::from_multiple(&[f.get("top"), b.get("top")]),
    );
}

/// A band with a flared skirt hanging from it.
pub fn cuff_band_skirt(tag: &str, design: &Design) -> CompRef {
    let comp = Component::new("CuffBandSkirt");
    let d = design.sub("cuff");

    let total = d.f("cuff_len");
    let skirt_fraction = d.f("skirt_fraction");

    let band = cuff_band(tag, design, Some(total * (1.0 - skirt_fraction)));
    let skirt = cuff_skirt(tag, design, Some(total * skirt_fraction));

    ec(&skirt).place_below(&ec(&band), 2.0);

    add_sub(&comp, ec(&band));
    add_sub(&comp, ec(&skirt));

    let bi = band.borrow().interfaces.clone();
    let si = skirt.borrow().interfaces.clone();
    comp.borrow_mut()
        .stitching_rules
        .append(bi.get("bottom"), si.get("top"));

    {
        let mut c = comp.borrow_mut();
        c.interfaces.set("top", bi.get("top"));
        c.interfaces.set("top_front", bi.get("top_front"));
        c.interfaces.set("top_back", bi.get("top_back"));
        c.interfaces.set("bottom", si.get("bottom"));
    }

    comp
}
