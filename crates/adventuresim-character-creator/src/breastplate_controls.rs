//! Breastplate silhouette presets and independent relief controls.
use adventuresim_armor_model::{BreastplateDesign, BreastplateFluting, BreastplateProfile};
use bevy_egui::egui;

pub(super) fn shape(ui: &mut egui::Ui, design: &mut BreastplateDesign) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        for (label, preset) in [
            ("Rounded", BreastplateDesign::globose()),
            ("Central ridge", BreastplateDesign::tapul()),
            ("Peascod", BreastplateDesign::peascod()),
            ("Fluted", BreastplateDesign::fluted()),
        ] {
            if ui.button(label).clicked() {
                *design = preset;
                changed = true;
            }
        }
    });
    for (value, range, label, unit) in [
        (
            &mut design.profile.projection.0,
            BreastplateProfile::PROJECTION_RANGE,
            "Chest projection",
            " mm",
        ),
        (
            &mut design.profile.upper_chest_recession.0,
            BreastplateProfile::UPPER_RECESSION_RANGE,
            "Upper chest recession",
            " mm",
        ),
        (
            &mut design.profile.projection_height.0,
            BreastplateProfile::PROJECTION_HEIGHT_RANGE,
            "Projection height above waist",
            " ‰",
        ),
        (
            &mut design.profile.fullness.0,
            BreastplateProfile::FULLNESS_RANGE,
            "Chest fullness",
            " ‰",
        ),
        (
            &mut design.profile.medial_ridge.0,
            BreastplateProfile::RIDGE_RANGE,
            "Central ridge",
            " mm",
        ),
        (
            &mut design.profile.waist_point.0,
            BreastplateProfile::WAIST_POINT_RANGE,
            "Waist point",
            " mm",
        ),
    ] {
        changed |= ui
            .add(egui::Slider::new(value, range).text(label).suffix(unit))
            .changed();
    }
    changed |= ui
        .add(
            egui::Slider::new(
                &mut design.profile.waist_point_width.0,
                BreastplateProfile::WAIST_POINT_WIDTH_RANGE,
            )
            .text("Waist point width")
            .suffix(" ‰"),
        )
        .changed();
    changed |= ui
        .add(
            egui::Slider::new(
                &mut design.profile.waist_projection.0,
                BreastplateProfile::PROJECTION_RANGE,
            )
            .text("Waist projection")
            .suffix(" mm"),
        )
        .changed();
    changed |= ui
        .add(
            egui::Slider::new(&mut design.back_depth.0, 700..=1100)
                .text("Back depth")
                .suffix(" ‰"),
        )
        .changed();
    changed | fluting(ui, design)
}

fn fluting(ui: &mut egui::Ui, design: &mut BreastplateDesign) -> bool {
    let mut changed = false;
    let mut enabled = design.fluting.is_some();
    let fluting_toggle = ui.checkbox(&mut enabled, "Fluting");
    if fluting_toggle.changed() {
        design.fluting = enabled.then(BreastplateFluting::default);
        changed = true;
    }
    if let Some(pattern) = &mut design.fluting {
        let max_start = pattern.end.0 - pattern.fade.0 * 2;
        let min_end = pattern.start.0 + pattern.fade.0 * 2;
        changed |= ui
            .add(
                egui::Slider::new(&mut pattern.count.0, BreastplateFluting::COUNT_RANGE)
                    .text("Flute count"),
            )
            .changed();
        for (value, range, label, unit) in [
            (
                &mut pattern.width.0,
                BreastplateFluting::WIDTH_RANGE,
                "Flute width / spacing",
                " ‰",
            ),
            (
                &mut pattern.depth.0,
                BreastplateFluting::DEPTH_RANGE,
                "Flute depth",
                " mm",
            ),
            (
                &mut pattern.spread.0,
                BreastplateFluting::SPREAD_RANGE,
                "Pattern spread",
                " ‰",
            ),
            (
                &mut pattern.lower_spread.0,
                BreastplateFluting::LOWER_SPREAD_RANGE,
                "Lower spread / upper spread",
                " ‰",
            ),
            (
                &mut pattern.start.0,
                BreastplateFluting::MIN_START..=max_start,
                "Start above waist",
                " ‰",
            ),
            (
                &mut pattern.end.0,
                min_end..=BreastplateFluting::MAX_END,
                "End above waist",
                " ‰",
            ),
        ] {
            changed |= ui
                .add(egui::Slider::new(value, range).text(label).suffix(unit))
                .changed();
        }
        let max_fade =
            ((pattern.end.0 - pattern.start.0) / 2).min(*BreastplateFluting::FADE_RANGE.end());
        changed |= ui
            .add(
                egui::Slider::new(
                    &mut pattern.fade.0,
                    *BreastplateFluting::FADE_RANGE.start()..=max_fade,
                )
                .text("Flute end taper")
                .suffix(" ‰"),
            )
            .changed();
    }
    changed
}
