//! Construction controls for a continuous sewn gown.
use bevy_egui::egui;
use fabelgeist_armor::{
    DressBodice, GownBodyFit, GownDesign, Millimeters, Permille, SewnSleeves, SkirtDesign,
};

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
    let mut loose = matches!(design.body_fit, GownBodyFit::Loose { .. });
    let body_fit_response = ui.checkbox(&mut loose, "Loose body");
    if body_fit_response.changed() {
        design.body_fit = if loose {
            GownBodyFit::Loose {
                ease: Millimeters(30),
                release: Permille(600),
            }
        } else {
            GownBodyFit::Fitted
        };
        changed = true;
    }
    if let GownBodyFit::Loose { ease, release } = &mut design.body_fit {
        changed |= super::number(ui, &mut ease.0, GownDesign::EASE_RANGE, "Body ease (mm)");
        changed |= super::number(
            ui,
            &mut release.0,
            GownDesign::RELEASE_RANGE,
            "Fullness release",
        );
    }
    let mut sleeves = design.sleeves.is_some();
    let sleeve_response = ui.checkbox(&mut sleeves, "Sewn sleeves");
    if sleeve_response.changed() {
        design.sleeves = sleeves.then_some(SewnSleeves {
            fabric: design.bodice.fabric.clone(),
            length: Permille(1000),
            circumference: Millimeters(600),
            cuff_circumference: Millimeters(180),
            cuff_length: Millimeters(30),
        });
        changed = true;
    }
    if let Some(sleeves) = &mut design.sleeves {
        changed |= super::textile::show(ui, &mut sleeves.fabric);
        changed |= super::number(
            ui,
            &mut sleeves.length.0,
            SewnSleeves::LENGTH_RANGE,
            "Sleeve reach",
        );
        changed |= super::number(
            ui,
            &mut sleeves.circumference.0,
            SewnSleeves::CIRCUMFERENCE_RANGE,
            "Sleeve circumference (mm)",
        );
        changed |= super::number(
            ui,
            &mut sleeves.cuff_circumference.0,
            SewnSleeves::CUFF_CIRCUMFERENCE_RANGE,
            "Cuff circumference (mm)",
        );
        changed |= super::number(
            ui,
            &mut sleeves.cuff_length.0,
            SewnSleeves::CUFF_LENGTH_RANGE,
            "Cuff length (mm)",
        );
    }
    changed
}
