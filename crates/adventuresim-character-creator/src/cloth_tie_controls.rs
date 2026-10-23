//! Reusable closure-tape dimensions and cloth appearance.
use bevy_egui::egui;
use fabelgeist_armor::{
    ClothTies, Micrometers, Millimeters, TextileAppearance, TextileColor, TextileFinish,
};

pub(super) fn show(ui: &mut egui::Ui, ties: &mut Option<ClothTies>) -> bool {
    let mut enabled = ties.is_some();
    let mut changed = ui.checkbox(&mut enabled, "Cloth ties").changed();
    if changed {
        *ties = enabled.then_some(ClothTies {
            width: Millimeters(13),
            tail_length: Millimeters(180),
            bow_span: Millimeters(45),
            thickness: Micrometers(200),
            fabric: TextileAppearance::solid(TextileColor([242, 240, 230]), TextileFinish::Linen),
        });
    }
    if let Some(ties) = ties {
        changed |= super::number(
            ui,
            &mut ties.width.0,
            ClothTies::WIDTH_RANGE,
            "Tape width (mm)",
        );
        changed |= super::number(
            ui,
            &mut ties.tail_length.0,
            ClothTies::TAIL_LENGTH_RANGE,
            "Tail length (mm)",
        );
        changed |= super::number(
            ui,
            &mut ties.bow_span.0,
            ClothTies::BOW_SPAN_RANGE,
            "Bow loop span (mm)",
        );
        changed |= super::number(
            ui,
            &mut ties.thickness.0,
            ClothTies::THICKNESS_RANGE,
            "Tie gauge (micrometres)",
        );
        ui.label("Bow span must be at least three tape widths.");
        ui.collapsing("Tie fabric", |ui| {
            changed |= super::textile::show(ui, &mut ties.fabric)
        });
    }
    changed
}
