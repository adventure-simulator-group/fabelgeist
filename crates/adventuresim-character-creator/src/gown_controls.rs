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
    changed |= sleeve_controls(ui, &mut design.sleeves, &design.bodice.fabric);
    changed |= neck_binding_controls(ui, &mut design.neck_binding, &design.bodice.fabric);
    changed
}

fn sleeve_controls(
    ui: &mut egui::Ui,
    design: &mut Option<SewnSleeves>,
    fabric: &fabelgeist_armor::TextileAppearance,
) -> bool {
    let mut changed = false;
    let mut sleeves = design.is_some();
    let sleeve_response = ui.checkbox(&mut sleeves, "Sewn sleeves");
    if sleeve_response.changed() {
        *design = sleeves.then_some(SewnSleeves {
            fabric: fabric.clone(),
            length: Permille(1000),
            circumference: Millimeters(600),
            proximal_fullness: Permille(650),
            fullness_release: Permille(500),
            cuff_circumference: Millimeters(180),
            cuff_length: Millimeters(30),
            frill: None,
            cuff_slit: None,
            cuff_closures: None,
            cuff_binding: None,
            cuff_backing: None,
        });
        changed = true;
    }
    if let Some(sleeves) = design {
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
            &mut sleeves.proximal_fullness.0,
            SewnSleeves::PROXIMAL_FULLNESS_RANGE,
            "Shoulder fullness",
        );
        changed |= super::number(
            ui,
            &mut sleeves.fullness_release.0,
            SewnSleeves::FULLNESS_RELEASE_RANGE,
            "Full sleeve release distance",
        );
        changed |= super::number(
            ui,
            &mut sleeves.cuff_length.0,
            SewnSleeves::CUFF_LENGTH_RANGE,
            "Cuff length (mm)",
        );
        changed |= frill_controls(ui, &mut sleeves.frill);
        changed |= slit_controls(ui, sleeves);
        changed |= cuff_closure_controls(ui, sleeves);
        changed |= super::gown_cuffs::show(ui, sleeves);
    }
    changed
}

fn cuff_closure_controls(ui: &mut egui::Ui, sleeves: &mut SewnSleeves) -> bool {
    if sleeves.cuff_slit.is_none() {
        return sleeves.cuff_closures.take().is_some();
    }
    let mut fabric = sleeves.fabric.clone();
    fabric.relief = None;
    fabric.layers.0.clear();
    super::cloth_closures::show(
        ui,
        &mut sleeves.cuff_closures,
        fabelgeist_armor::ClothClosures {
            positions: vec![Permille(100), Permille(400)],
            fastener: fabelgeist_armor::ClothFastener::ButtonLoop {
                diameter: Millimeters(5),
                cord: fabelgeist_armor::Micrometers(500),
                fabric,
            },
        },
    )
}

fn neck_binding_controls(
    ui: &mut egui::Ui,
    tape: &mut Option<fabelgeist_armor::ClothTape>,
    fabric: &fabelgeist_armor::TextileAppearance,
) -> bool {
    let mut bound = tape.is_some();
    let response = ui.checkbox(&mut bound, "Bound neckline");
    let mut changed = response.changed();
    if changed {
        let mut fabric = fabric.clone();
        fabric.relief = None;
        *tape = bound.then_some(fabelgeist_armor::ClothTape {
            width: Millimeters(12),
            thickness: fabelgeist_armor::Micrometers(200),
            fabric,
        });
    }
    if let Some(tape) = tape {
        changed |= super::number(
            ui,
            &mut tape.width.0,
            fabelgeist_armor::ClothTape::WIDTH_RANGE,
            "Neck binding width (mm)",
        );
        changed |= super::number(
            ui,
            &mut tape.thickness.0,
            fabelgeist_armor::ClothTape::THICKNESS_RANGE,
            "Neck binding gauge (micrometres)",
        );
        changed |= super::textile::show(ui, &mut tape.fabric);
    }
    changed
}

fn frill_controls(ui: &mut egui::Ui, frill: &mut Option<fabelgeist_armor::ClothFrill>) -> bool {
    let mut enabled = frill.is_some();
    let response = ui.checkbox(&mut enabled, "Frilled sleeve hem");
    let mut changed = response.changed();
    if changed {
        *frill = enabled.then_some(fabelgeist_armor::ClothFrill {
            length: Millimeters(25),
            flare: Millimeters(14),
            folds: None,
        });
    }
    if let Some(frill) = frill {
        changed |= super::number(
            ui,
            &mut frill.length.0,
            fabelgeist_armor::ClothFrill::LENGTH_RANGE,
            "Frill length (mm)",
        );
        changed |= super::number(
            ui,
            &mut frill.flare.0,
            fabelgeist_armor::ClothFrill::FLARE_RANGE,
            "Frill flare (mm)",
        );
        let mut folded = frill.folds.is_some();
        let response = ui.checkbox(&mut folded, "Even free-edge folds");
        if response.changed() {
            changed = true;
            frill.folds = folded.then_some(fabelgeist_armor::ClothPleats {
                count: fabelgeist_armor::TextileRepeatCount(8),
                depth: Millimeters(frill.flare.0.clamp(1, 6)),
                profile: fabelgeist_armor::ClothPleatProfile::Fluted,
            });
            frill.flare.0 = frill.flare.0.max(1);
        }
        if let Some(folds) = &mut frill.folds {
            let depth = folds.depth.0.min(frill.flare.0.max(1));
            changed |= folds.depth.0 != depth;
            folds.depth.0 = depth;
            frill.flare.0 = frill.flare.0.max(depth);
            changed |= ui
                .add(
                    egui::Slider::new(
                        &mut folds.count.0,
                        fabelgeist_armor::ClothFrill::FOLD_COUNT_RANGE,
                    )
                    .text("Free-edge fold count"),
                )
                .changed();
            changed |= super::number(
                ui,
                &mut folds.depth.0,
                1..=frill
                    .flare
                    .0
                    .min(*fabelgeist_armor::ClothFrill::FOLD_DEPTH_RANGE.end()),
                "Free-edge fold depth (mm)",
            );
            changed |= ui
                .selectable_value(
                    &mut folds.profile,
                    fabelgeist_armor::ClothPleatProfile::Fluted,
                    "Fluted free edge",
                )
                .changed();
            changed |= ui
                .selectable_value(
                    &mut folds.profile,
                    fabelgeist_armor::ClothPleatProfile::RoundedGathers,
                    "Rounded free gathers",
                )
                .changed();
        }
    }
    changed
}

fn slit_controls(ui: &mut egui::Ui, sleeves: &mut SewnSleeves) -> bool {
    use fabelgeist_armor::ClothSlit;
    let frill = sleeves.frill.as_ref().map_or(0, |f| f.length.0);
    let minimum = (frill + 1).max(*ClothSlit::LENGTH_RANGE.start());
    let maximum = (frill + sleeves.cuff_length.0 - 1).min(*ClothSlit::LENGTH_RANGE.end());
    let available = minimum <= maximum;
    let mut enabled = sleeves.cuff_slit.is_some();
    let response = ui.add_enabled(
        available,
        egui::Checkbox::new(&mut enabled, "Open cuff slit"),
    );
    let mut changed = response.changed();
    if changed {
        sleeves.cuff_slit = enabled.then_some(ClothSlit {
            length: Millimeters((frill + sleeves.cuff_length.0 / 2).clamp(minimum, maximum)),
            gap: Millimeters(2),
            rotation: Permille(500),
        });
    }
    if !available {
        changed |= sleeves.cuff_slit.take().is_some();
        return changed;
    }
    if let Some(slit) = &mut sleeves.cuff_slit {
        let length = slit.length.0.clamp(minimum, maximum);
        changed |= slit.length.0 != length;
        slit.length.0 = length;
        changed |= super::number(
            ui,
            &mut slit.length.0,
            minimum..=maximum,
            "Cuff slit length (mm)",
        );
        changed |= super::number(
            ui,
            &mut slit.gap.0,
            ClothSlit::GAP_RANGE,
            "Slit gap at hem (mm)",
        );
        changed |= super::number(
            ui,
            &mut slit.rotation.0,
            ClothSlit::ROTATION_RANGE,
            "Slit position around cuff",
        );
    }
    changed
}
