//! Separate circular armpit plates and their full-circle radial flutes.
use adventuresim_armor_model::{BesagewDesign, PlateFluting, RadialFluting};
use bevy_egui::egui;

pub(super) fn show(ui: &mut egui::Ui, besagew: &mut Option<BesagewDesign>) -> bool {
    let original = besagew.clone();
    let mut enabled = besagew.is_some();
    let toggle = ui.checkbox(&mut enabled, "Separate besagew armpit plate");
    if toggle.changed() {
        *besagew = enabled.then(BesagewDesign::default);
    }
    if let Some(design) = besagew {
        for (value, range, label) in [
            (&mut design.radius.0, 35..=85, "Disc radius (mm)"),
            (
                &mut design.boss_height.0,
                0..=30,
                "Central boss height (mm)",
            ),
            (
                &mut design.shoulder_drop.0,
                35..=120,
                "Drop below shoulder (mm)",
            ),
            (&mut design.medial_offset.0, 0..=70, "Medial offset (mm)"),
            (
                &mut design.plate_clearance.0,
                3..=15,
                "Supporting plate clearance (mm)",
            ),
            (&mut design.outward_tilt.0, 0..=350, "Outward tilt (mrad)"),
        ] {
            super::armor_controls::number(ui, value, range, label);
        }
        radial_fluting(ui, &mut design.fluting);
    }
    *besagew != original
}

fn radial_fluting(ui: &mut egui::Ui, fluting: &mut Option<RadialFluting>) {
    let mut enabled = fluting.is_some();
    let toggle = ui.checkbox(&mut enabled, "Radial fluting around entire disc");
    if toggle.changed() {
        *fluting = enabled.then(RadialFluting::default);
    }
    if let Some(pattern) = fluting {
        ui.add(egui::Slider::new(&mut pattern.count.0, 4..=48).text("Radial spoke count"));
        for (value, range, label) in [
            (
                &mut pattern.width.0,
                PlateFluting::WIDTH_RANGE,
                "Spoke width / spacing",
            ),
            (
                &mut pattern.depth.0,
                PlateFluting::DEPTH_RANGE,
                "Spoke depth (mm)",
            ),
        ] {
            super::armor_controls::number(ui, value, range, label);
        }
        let maximum_start = 700.min(pattern.end.0 - pattern.fade.0 * 2);
        super::armor_controls::number(
            ui,
            &mut pattern.start.0,
            200..=maximum_start,
            "Pattern inner radius",
        );
        let minimum_end = 800.max(pattern.start.0 + pattern.fade.0 * 2);
        super::armor_controls::number(
            ui,
            &mut pattern.end.0,
            minimum_end..=1000,
            "Pattern outer radius",
        );
        let maximum_fade = 250.min((pattern.end.0 - pattern.start.0) / 2);
        super::armor_controls::number(
            ui,
            &mut pattern.fade.0,
            50..=maximum_fade,
            "Radial end taper",
        );
    }
}
