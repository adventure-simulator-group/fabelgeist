//! Independent sleeve-edge finishing and inner cuff backing.
use bevy_egui::egui;
use fabelgeist_armor::{ClothTape, Micrometers, Millimeters, SewnSleeves};

pub(super) fn show(ui: &mut egui::Ui, sleeves: &mut SewnSleeves) -> bool {
    let mut fabric = sleeves.fabric.clone();
    fabric.relief = None;
    fabric.layers.0.clear();
    let mut changed = tape_controls(
        ui,
        &mut sleeves.cuff_binding,
        ClothTape {
            width: Millimeters(2),
            thickness: Micrometers(200),
            fabric: fabric.clone(),
        },
        "Finished sleeve and slit edges",
        Millimeters(*ClothTape::WIDTH_RANGE.end()),
    );
    changed |= tape_controls(
        ui,
        &mut sleeves.cuff_backing,
        ClothTape {
            width: Millimeters(sleeves.cuff_length.0.min(32)),
            thickness: Micrometers(200),
            fabric,
        },
        "Inner cuff backing",
        sleeves.cuff_length,
    );
    changed
}
fn tape_controls(
    ui: &mut egui::Ui,
    tape: &mut Option<ClothTape>,
    initial: ClothTape,
    label: &str,
    limit: Millimeters,
) -> bool {
    let mut enabled = tape.is_some();
    let response = ui.checkbox(&mut enabled, label);
    let mut changed = response.changed();
    if changed {
        *tape = enabled.then_some(initial);
    }
    if let Some(tape) = tape {
        let maximum = limit.0.min(*ClothTape::WIDTH_RANGE.end());
        let width = tape.width.0.min(maximum);
        changed |= width != tape.width.0;
        tape.width.0 = width;
        changed |= super::number(
            ui,
            &mut tape.width.0,
            *ClothTape::WIDTH_RANGE.start()..=maximum,
            "Finished width (mm)",
        );
        changed |= super::number(
            ui,
            &mut tape.thickness.0,
            ClothTape::THICKNESS_RANGE,
            "Cloth gauge (micrometres)",
        );
        changed |= super::textile::show(ui, &mut tape.fabric);
    }
    changed
}
