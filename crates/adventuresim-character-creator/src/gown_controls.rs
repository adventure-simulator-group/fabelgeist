//! Construction controls for a continuous sewn gown.
use bevy_egui::egui;
use fabelgeist_armor::{
    BodiceOpening, DressBodice, FittedSleeves, GownBodyFit, GownDesign, GownNecklineTrim,
    Millimeters, Permille, SkirtDesign,
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
        if loose {
            design.sleeves = None;
        }
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
    changed |= super::gown_dressing::show(ui, design);
    changed |= super::waist_support::show(ui, design);
    changed |= super::cloth_lacing::show(ui, design);
    changed |= sleeve_controls(ui, design);
    changed |= neckline_trim_controls(ui, design);
    changed
}

fn neckline_trim_controls(ui: &mut egui::Ui, gown: &mut GownDesign) -> bool {
    // An opened bodice has no closed neckline for the band to follow.
    let closed = matches!(gown.bodice.opening, BodiceOpening::Closed);
    let mut enabled = gown.neckline_trim.is_some() && closed;
    let response = ui.add_enabled(closed, egui::Checkbox::new(&mut enabled, "Neckline trim"));
    let mut changed = response.changed() || (gown.neckline_trim.is_some() && !closed);
    if changed {
        gown.neckline_trim = enabled.then(|| GownNecklineTrim {
            shoulder_width: Millimeters(20),
            front_width: Millimeters(40),
            relief: Millimeters(3),
            fabric: gown.bodice.fabric.clone(),
        });
    }
    if let Some(trim) = &mut gown.neckline_trim {
        changed |= super::number(
            ui,
            &mut trim.shoulder_width.0,
            GownNecklineTrim::WIDTH_RANGE,
            "Trim width over shoulders (mm)",
        );
        changed |= super::number(
            ui,
            &mut trim.front_width.0,
            GownNecklineTrim::WIDTH_RANGE,
            "Trim width along front (mm)",
        );
        trim.front_width.0 = trim.front_width.0.max(trim.shoulder_width.0);
        changed |= super::number(
            ui,
            &mut trim.relief.0,
            GownNecklineTrim::RELIEF_RANGE,
            "Trim relief (mm)",
        );
        ui.collapsing("Trim fabric", |ui| {
            changed |= super::textile::show(ui, &mut trim.fabric);
        });
    }
    changed
}

fn sleeve_controls(ui: &mut egui::Ui, gown: &mut GownDesign) -> bool {
    let mut enabled = gown.sleeves.is_some();
    let response = ui.add_enabled(
        matches!(gown.body_fit, GownBodyFit::Fitted),
        egui::Checkbox::new(&mut enabled, "Fitted sleeves"),
    );
    let mut changed = response.changed();
    if changed {
        gown.sleeves = enabled.then_some(FittedSleeves {
            length: Permille(980),
        });
    }
    if let Some(sleeves) = &mut gown.sleeves {
        changed |= super::number(
            ui,
            &mut sleeves.length.0,
            FittedSleeves::LENGTH_RANGE,
            "Sleeve reach",
        );
    }
    changed
}
