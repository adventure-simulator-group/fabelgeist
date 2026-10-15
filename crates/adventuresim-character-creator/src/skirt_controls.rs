//! Placement and silhouette controls for a skirt enclosing both legs.

use bevy_egui::egui;
use fabelgeist_armor::{Permille, SkirtDesign, SkirtProfile};
use std::ops::RangeInclusive;

use super::number;

pub(super) fn show(ui: &mut egui::Ui, design: &mut SkirtDesign) -> bool {
    let mut changed = number(
        ui,
        &mut design.waist_rise.0,
        SkirtDesign::WAIST_RISE_RANGE,
        "Waist rise (mm)",
    );
    changed |= proportion(
        ui,
        &mut design.length,
        SkirtDesign::LENGTH_RANGE,
        "Length (hip to ankle)",
    );
    changed |= proportion(
        ui,
        &mut design.fitted_fraction,
        SkirtDesign::FITTED_FRACTION_RANGE,
        "Fitted upper portion",
    );
    ui.horizontal(|ui| {
        let straight = ui
            .selectable_label(matches!(design.profile, SkirtProfile::Straight), "Straight")
            .clicked();
        if straight && !matches!(design.profile, SkirtProfile::Straight) {
            design.profile = SkirtProfile::Straight;
            changed = true;
        }
        let flared = ui
            .selectable_label(
                matches!(design.profile, SkirtProfile::Flared { .. }),
                "Flared",
            )
            .clicked();
        if flared && !matches!(design.profile, SkirtProfile::Flared { .. }) {
            design.profile = SkirtProfile::Flared {
                width: Permille(1600),
                depth: Permille(1600),
            };
            changed = true;
        }
        let bell = ui
            .selectable_label(matches!(design.profile, SkirtProfile::Bell { .. }), "Bell")
            .clicked();
        if bell && !matches!(design.profile, SkirtProfile::Bell { .. }) {
            design.profile = SkirtProfile::Bell {
                width: Permille(2200),
                depth: Permille(2200),
                roundness: Permille(1000),
            };
            changed = true;
        }
    });
    match &mut design.profile {
        SkirtProfile::Straight => {}
        SkirtProfile::Flared { width, depth } | SkirtProfile::Bell { width, depth, .. } => {
            changed |= proportion(ui, width, SkirtProfile::HEM_GAIN_RANGE, "Hem width");
            changed |= proportion(ui, depth, SkirtProfile::HEM_GAIN_RANGE, "Hem depth");
        }
    }
    if let SkirtProfile::Bell { roundness, .. } = &mut design.profile {
        changed |= proportion(
            ui,
            roundness,
            SkirtProfile::BELL_ROUNDNESS_RANGE,
            "Bell roundness",
        );
    }
    changed |= number(
        ui,
        &mut design.clearance.0,
        SkirtDesign::CLEARANCE_RANGE,
        "Inner-layer clearance (mm)",
    );
    changed |= number(
        ui,
        &mut design.thickness.0,
        SkirtDesign::THICKNESS_RANGE,
        "Cloth thickness (mm)",
    );
    ui.horizontal(|ui| {
        ui.label("Fabric color");
        changed |= ui.color_edit_button_srgb(&mut design.color.0).changed();
    });
    changed
}

/// Convert once at the editor boundary; stored design values retain their unit.
fn proportion(
    ui: &mut egui::Ui,
    value: &mut Permille,
    range: RangeInclusive<u16>,
    label: &str,
) -> bool {
    let mut percent = value.unit() * 100.0;
    let changed = ui
        .add(
            egui::Slider::new(
                &mut percent,
                f32::from(*range.start()) / 10.0..=f32::from(*range.end()) / 10.0,
            )
            .text(label)
            .suffix("%"),
        )
        .changed();
    if changed {
        *value = Permille((percent * 10.0).round() as u16);
    }
    changed
}
