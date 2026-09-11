//! Breastplate silhouette presets and independent relief controls.
use adventuresim_armor_model::{BreastplateDesign, BreastplateProfile};
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
    changed | crate::fluting_controls::show(ui, &mut design.fluting)
}
