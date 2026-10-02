//! Shared pierced helmet-plate opening controls.
use super::armor_controls::number;
use adventuresim_armor_model::{VentSides, VisorBreaths};
use bevy_egui::egui;

pub(super) fn show(
    ui: &mut egui::Ui,
    d: &mut VisorBreaths,
    heights: std::ops::RangeInclusive<u16>,
) -> bool {
    let mut changed = false;
    ui.label("Breath slots");
    changed |= ui
        .add(egui::Slider::new(&mut d.count_per_row, 0..=8).text("Slits per row"))
        .changed();
    changed |= ui
        .add(egui::Slider::new(&mut d.rows, 1..=4).text("Rows"))
        .changed();
    ui.horizontal(|ui| {
        ui.label("Breath sides");
        for (side, label) in [
            (VentSides::Both, "Both"),
            (VentSides::Left, "Left"),
            (VentSides::Right, "Right"),
        ] {
            changed |= ui.selectable_value(&mut d.sides, side, label).changed();
        }
    });
    for (value, range, label) in [
        (&mut d.width.0, 2..=6, "Slit width (mm)"),
        (&mut d.length.0, 8..=20, "Slit length (mm)"),
        (&mut d.span.0, 20..=60, "Pattern span (mm)"),
        (&mut d.row_spacing.0, 14..=25, "Row spacing (mm)"),
        (&mut d.center_offset.0, 10..=100, "Pattern offset (mm)"),
        (&mut d.height.0, heights, "Pattern height from top"),
        (&mut d.rounding.0, 0..=1000, "Slit roundness"),
    ] {
        changed |= number(ui, value, range, label);
    }
    changed |= ui
        .add(
            egui::Slider::new(&mut d.inclination.0, -90..=90)
                .text("Slit angle")
                .suffix("°"),
        )
        .changed();
    changed
}
