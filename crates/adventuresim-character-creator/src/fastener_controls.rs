//! Closure dimensions and leather color shared by preview and saved recipes.
use super::{EquipmentCatalog, Studio};
use adventuresim_character_creator::fasteners::{
    FULL_TURN_MILLIRADIANS, MIN_STRAP_ARC_MILLIRADIANS,
};
use bevy_egui::egui;

pub(super) fn show(ui: &mut egui::Ui, catalog: &mut EquipmentCatalog, studio: &mut Studio) {
    let ids = studio
        .recipe
        .clothing
        .iter()
        .map(|item| item.item_id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    for id in ids {
        let Some(recipe) = catalog.2.get_mut(&id) else {
            continue;
        };
        ui.collapsing(format!("{id} fastenings"), |ui| {
            match recipe {
                adventuresim_character_creator::fasteners::catalog::FastenerRecipe::Retention(d) => {
                    let strap = &mut d.closure;
                    let mut changed = false;
                    for (value, range, label) in [
                        (&mut strap.width.0, 8..=35, "Strap width (mm)"),
                        (&mut strap.thickness.0, 1..=4, "Leather thickness (mm)"),
                        (&mut strap.height.0, 100..=900, "Height on plate"),
                        (&mut strap.spacing.0, 0..=350, "Strap spacing"),
                        (&mut strap.buckle_position.0, 150..=850, "Buckle position"),
                        (
                            &mut strap.lining_clearance.0,
                            0..=15,
                            "Lining allowance (mm)",
                        ),
                        (&mut strap.underarm_drop.0, 0..=60, "Underarm drop (mm)"),
                    ] {
                        changed |= super::armor_controls::number(ui, value, range, label);
                    }
                    changed |= super::armor_controls::number(
                        ui,
                        &mut strap.start_angle.0,
                        0..=FULL_TURN_MILLIRADIANS,
                        "Arc start (mrad)",
                    );
                    let end_range = (strap.start_angle.0 + MIN_STRAP_ARC_MILLIRADIANS)
                        ..=(strap.start_angle.0 + FULL_TURN_MILLIRADIANS);
                    strap.end_angle.0 = strap.end_angle.0.clamp(*end_range.start(), *end_range.end());
                    changed |= super::armor_controls::number(
                        ui,
                        &mut strap.end_angle.0,
                        end_range,
                        "Arc end (mrad)",
                    );
                    changed |= ui
                        .add(egui::Slider::new(&mut strap.count, 1..=3).text("Strap count"))
                        .changed();
                    changed |= ui
                        .color_edit_button_srgb(&mut strap.leather_color)
                        .changed();
                    strap.thickness.0 = strap.thickness.0.min(strap.width.0 / 4);
                    let spread = u16::from(strap.count - 1) * strap.spacing.0 / 2;
                    strap.height.0 = strap.height.0.clamp(100 + spread, 900 - spread);
                    studio.dirty |= changed;
                }
                adventuresim_character_creator::fasteners::catalog::FastenerRecipe::TassetSuspension(d) => {
                    for (value, range, label) in [
                        (&mut d.width.0, 12..=24, "Hanger width (mm)"),
                        (&mut d.thickness.0, 1..=3, "Leather thickness (mm)"),
                        (&mut d.fauld_inset.0, 8..=100, "Inset from fauld edge (mm)"),
                        (&mut d.tasset_inset.0, 20..=60, "Inset from tasset edge (mm)"),
                    ] { studio.dirty |= super::armor_controls::number(ui, value, range, label); }
                    studio.dirty |= ui.add(egui::Slider::new(&mut d.count_per_panel, 1..=3).text("Hangers per tasset")).changed();
                    studio.dirty |= ui.color_edit_button_srgb(&mut d.leather_color).changed();
                }
            }
        });
    }
}
