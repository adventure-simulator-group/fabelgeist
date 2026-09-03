//! Tiered skirts: a base skirt with gathered levels hanging from it.
//!
//! Ports `assets.garment_programs.skirt_levels`.

use super::circle_skirt::{SkirtOpts, skirt_circle};
use super::prelude::*;
use super::skirt_paneled::{pencil_skirt, skirt_2};

/// Which skirt to use for the base or for one level.
fn build_skirt(name: &str, body: &Body, design: &Design, opts: &SkirtOpts) -> CompRef {
    match name {
        "Skirt2" => skirt_2(body, design, opts),
        "PencilSkirt" => pencil_skirt(body, design, opts),
        "SkirtCircle" => skirt_circle(body, design, opts, false),
        "AsymmSkirtCircle" => skirt_circle(body, design, opts, true),
        other => panic!("skirt_levels::ERROR::unknown skirt style '{other}'"),
    }
}

/// A skirt made of several stitched skirts.
pub fn skirt_levels(body: &Body, design: &Design, rise: Option<f64>) -> CompRef {
    let comp = Component::new("SkirtLevels");

    let l = design.sub("levels-skirt");
    // The level measurements get overwritten, so work on a copy.
    let mut lbody = body.clone();
    let n_levels = l.i("num_levels") as usize;
    let ruffle = l.f("level_ruffle");

    // Bring the lengths to a common denominator.
    let total_length = l.f("length") * body.get("_leg_length");
    let base_len_frac = total_length * l.f("base_length_frac");
    let level_len = (total_length - base_len_frac) / n_levels as f64;
    // The hip line contributes zero length.
    let base_len = body.get("hips_line") * l.f("rise") + base_len_frac;

    let rise = rise.unwrap_or_else(|| l.f("rise"));
    comp.borrow_mut().rise = Some(rise);

    let base_style = l.s("base").expect("levels-skirt base style");
    let base = build_skirt(
        &base_style,
        body,
        design,
        &SkirtOpts {
            length: Some(base_len),
            rise: Some(rise),
            slit: false,
            ..Default::default()
        },
    );

    // Only the pencil skirt carries a hem angle.
    let angle = if base_style == "PencilSkirt" {
        design.f("pencil-skirt.low_angle")
    } else {
        0.0
    };

    add_sub(&comp, ec(&base));
    let mut prev = ec(&base);

    let level_style = l.s("level").expect("levels-skirt level style");
    for i in 0..n_levels {
        // Trick the level skirts into producing the right width by feeding them
        // the previous level's hem as a waist measurement.
        let prev_interfaces = prev.interfaces();
        lbody.set(
            "waist",
            ruffle * prev_interfaces.get("bottom").borrow().edges.length(),
        );
        lbody.set(
            "waist_back_width",
            ruffle * prev_interfaces.get("bottom_b").borrow().edges.length(),
        );

        let level = build_skirt(
            &level_style,
            &lbody,
            design,
            &SkirtOpts {
                tag: i.to_string(),
                length: Some(level_len),
                slit: false,
                top_ruffles: false,
                ..Default::default()
            },
        );
        let level_el = ec(&level);

        // Rotate if the base is asymmetric.
        level_el.rotate_by(Rotation::from_euler_xyz([0.0, 0.0, -angle], true));
        level_el.place_by_interface(
            &level.borrow().interfaces.get("top"),
            &prev_interfaces.get("bottom"),
            5.0,
            Alignment::Center,
            None,
        );

        comp.borrow_mut().stitching_rules.append(
            prev_interfaces.get("bottom"),
            level.borrow().interfaces.get("top"),
        );

        add_sub(&comp, level_el.clone());
        prev = level_el;
    }

    let top = base.borrow().interfaces.get("top");
    comp.borrow_mut().interfaces.set("top", top);

    comp
}
