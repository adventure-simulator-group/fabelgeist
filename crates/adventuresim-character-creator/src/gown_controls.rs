//! Construction controls for a continuous sewn gown.
use bevy_egui::egui;
use fabelgeist_armor::{DressBodice, GownDesign, SkirtDesign};

pub(super) fn show(ui: &mut egui::Ui, design: &mut GownDesign) -> bool {
    let mut changed = super::number(
        ui,
        &mut design.transition.0,
        DressBodice::TRANSITION_RANGE,
        "Waist transition",
    );
    changed |= super::number(
        ui,
        &mut design.waist_band.0,
        GownDesign::WAIST_BAND_RANGE,
        "Cloth below opening (mm)",
    );
    let upper = ui
        .collapsing("Bodice", |ui| {
            super::bodice::show(ui, &mut design.bodice, SkirtDesign::WAIST_RISE_RANGE)
        })
        .body_returned
        .unwrap_or(false);
    if upper {
        design.skirt.waist_rise = design.bodice.hem_rise;
        design.skirt.thickness = design.bodice.thickness;
    }
    let lower = ui
        .collapsing("Skirt", |ui| super::skirt::hanging(ui, &mut design.skirt))
        .body_returned
        .unwrap_or(false);
    if lower {
        design.bodice.hem_rise = design.skirt.waist_rise;
        design.bodice.thickness = design.skirt.thickness;
    }
    changed |= upper || lower;
    changed
}
