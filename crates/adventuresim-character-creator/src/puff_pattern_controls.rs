//! Shared treatment controls for limb and skirt textiles.
use super::number;
use bevy_egui::egui;

pub(super) fn show(ui: &mut egui::Ui, design: &mut fabelgeist_armor::PuffAndSlashPattern) -> bool {
    let mut changed = ui
        .add(egui::Slider::new(&mut design.puff_count, 1..=8).text("Puff courses"))
        .changed();
    changed |= number(
        ui,
        &mut design.puff_fullness.0,
        5..=90,
        "Puff fullness (mm)",
    );
    changed |= number(
        ui,
        &mut design.puff_roundness.0,
        400..=2500,
        "Puff roundness",
    );
    let mut slashed = design.slash_count != 0;
    let slash_toggle = ui.checkbox(&mut slashed, "Open slashes").changed();
    if slash_toggle {
        design.slash_count = if slashed { 8 } else { 0 };
        changed = true;
    }
    if slashed {
        changed |= ui
            .add(egui::Slider::new(&mut design.slash_count, 3..=16).text("Slashes per course"))
            .changed();
        changed |= number(ui, &mut design.slash_width.0, 50..=650, "Slash width");
        changed |= number(ui, &mut design.slash_length.0, 300..=900, "Slash length");
        changed |= number(ui, &mut design.rotation.0, 0..=1000, "Pattern rotation");
    }
    changed |= number(
        ui,
        &mut design.distal_fullness.0,
        250..=1500,
        "Distal fullness",
    );
    changed |= number(
        ui,
        &mut design.constriction_width.0,
        40..=350,
        "Constricted band width",
    );
    ui.collapsing("Outer fabric", |ui| {
        changed |= super::textile::show(ui, &mut design.outer_fabric);
    });
    ui.collapsing("Undercloth", |ui| {
        changed |= super::textile::show(ui, &mut design.undercloth_fabric);
    });
    changed
}
