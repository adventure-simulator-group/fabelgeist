//! Independent controls for fabric folds and optional support members.

use bevy_egui::egui;
use fabelgeist_armor::{
    PleatStyle, SkirtDesign, SkirtHoops, SkirtPleats, SkirtStays, SkirtSupport,
};

use super::{super::number, proportion};

pub(super) fn show(ui: &mut egui::Ui, design: &mut SkirtDesign) -> bool {
    let mut changed = toggle(ui, &mut design.pleats, "Pleated fabric");
    if let Some(pleats) = &mut design.pleats {
        changed |= ui
            .add(egui::Slider::new(&mut pleats.count.0, SkirtPleats::COUNT_RANGE).text("Pleats"))
            .changed();
        changed |= number(
            ui,
            &mut pleats.depth.0,
            SkirtPleats::DEPTH_RANGE,
            "Pleat depth (mm)",
        );
        changed |= ui
            .selectable_value(&mut pleats.style, PleatStyle::Rounded, "Rounded folds")
            .changed();
        changed |= ui
            .selectable_value(&mut pleats.style, PleatStyle::Accordion, "Accordion folds")
            .changed();
        changed |= proportion(ui, &mut pleats.rotation, 0..=1000, "Pleat rotation");
        changed |= proportion(
            ui,
            &mut pleats.front_depth,
            SkirtPleats::FRONT_DEPTH_RANGE,
            "Front fold depth",
        );
        changed |= number(
            ui,
            &mut pleats.waist_depth.0,
            SkirtPleats::WAIST_DEPTH_RANGE,
            "Waist gather depth (mm)",
        );
        changed |= proportion(
            ui,
            &mut pleats.front_panel,
            SkirtPleats::FRONT_PANEL_RANGE,
            "Quiet front panel width",
        );
        changed |= gather_controls(ui, pleats);
    }
    changed |= toggle(ui, &mut design.support, "Structural support");
    if let Some(support) = &mut design.support {
        changed |= toggle(ui, &mut support.stays, "Longitudinal stays (boning)");
        if let Some(stays) = &mut support.stays {
            changed |= ui
                .add(egui::Slider::new(&mut stays.count.0, SkirtStays::COUNT_RANGE).text("Stays"))
                .changed();
            changed |= gauge(ui, &mut stays.width, &mut stays.thickness);
            changed |= proportion(
                ui,
                &mut stays.length,
                SkirtStays::LENGTH_RANGE,
                "Stay length",
            );
        }
        changed |= toggle(ui, &mut support.hoops, "Circumferential hoops");
        if let Some(hoops) = &mut support.hoops {
            changed |= ui
                .add(egui::Slider::new(&mut hoops.count.0, SkirtHoops::COUNT_RANGE).text("Hoops"))
                .changed();
            changed |= gauge(ui, &mut hoops.width, &mut hoops.thickness);
            changed |= proportion(
                ui,
                &mut hoops.start,
                *SkirtHoops::DESCENT_RANGE.start()..=hoops.end.0 - 1,
                "First hoop descent",
            );
            changed |= proportion(
                ui,
                &mut hoops.end,
                hoops.start.0 + 1..=*SkirtHoops::DESCENT_RANGE.end(),
                "Last hoop descent",
            );
        }
        ui.horizontal(|ui| {
            ui.label("Support color");
            changed |= ui.color_edit_button_srgb(&mut support.color.0).changed();
        });
        if support.stays.is_none() && support.hoops.is_none() {
            design.support = None;
        }
    }
    changed
}

fn gauge(
    ui: &mut egui::Ui,
    width: &mut fabelgeist_armor::Millimeters,
    thickness: &mut fabelgeist_armor::Millimeters,
) -> bool {
    let mut changed = number(
        ui,
        &mut width.0,
        SkirtSupport::WIDTH_RANGE,
        "Member width (mm)",
    );
    changed |= number(
        ui,
        &mut thickness.0,
        SkirtSupport::THICKNESS_RANGE,
        "Member thickness (mm)",
    );
    changed
}

fn gather_controls(ui: &mut egui::Ui, pleats: &mut SkirtPleats) -> bool {
    let mut changed = ui
        .add(
            egui::Slider::new(&mut pleats.waist_count.0, SkirtPleats::WAIST_COUNT_RANGE)
                .text("Fine waist gathers"),
        )
        .changed();
    changed |= proportion(
        ui,
        &mut pleats.gather_falloff,
        SkirtPleats::GATHER_FALLOFF_RANGE,
        "Waist gather reach",
    );
    changed |= proportion(
        ui,
        &mut pleats.gather_retention,
        SkirtPleats::GATHER_RETENTION_RANGE,
        "Fine folds retained toward hem",
    );
    changed
}

/// The editor's checkbox is a native Boolean; authoring retains the construction.
fn toggle<T: Default>(ui: &mut egui::Ui, value: &mut Option<T>, label: &str) -> bool {
    let mut enabled = value.is_some();
    let changed = ui.checkbox(&mut enabled, label).changed();
    if changed {
        *value = if enabled { Some(T::default()) } else { None };
    }
    changed
}
