//! Name, construction, pattern and layer controls for one draped garment.
use super::*;
use adventuresim_character_creator::garment::{
    ClothLayer, Construction,
    pattern::{Collar, Lower, Pattern, Sleeves, Upper, shapes},
};
use adventuresim_character_creator::inventory::InventoryItemId;
use std::ops::RangeInclusive;

pub(super) fn show(ui: &mut egui::Ui, id: InventoryItemId, selection: &mut GarmentSelection) {
    ui.horizontal(|ui| {
        ui.label("Name");
        ui.text_edit_singleline(&mut selection.name);
    });
    ui.horizontal(|ui| {
        let sewn = matches!(selection.construction, Construction::Sewn(_));
        if ui.selectable_label(sewn, "Sewn pattern").clicked() && !sewn {
            selection.construction = Construction::Sewn(Pattern::default());
        }
        if ui
            .selectable_label(!sewn, "Fitted coif")
            .on_hover_text("A mail coif's hood, neck and flaps, fitted around the head.")
            .clicked()
            && sewn
        {
            selection.construction = Construction::Coif(Default::default());
        }
    });
    match &mut selection.construction {
        Construction::Sewn(pattern) => {
            if let Some(shape) = start_from(ui, id) {
                *pattern = shape.pattern;
                selection.layer = shape.layer;
            }
            upper(ui, id, pattern);
            lower(ui, id, pattern);
        }
        // The coif shares its shape controls with the catalog mail coif.
        Construction::Coif(coif) => {
            super::armor_controls::coif(ui, coif);
        }
    }
}

/// The layer cloth is worn in; chainmail is always mail.
pub(super) fn layer(ui: &mut egui::Ui, id: InventoryItemId, selection: &mut GarmentSelection) {
    if selection.fabric == FabricPreset::Chainmail {
        ui.weak("Chainmail is worn in the mail layer.");
        return;
    }
    ui.horizontal(|ui| {
        ui.label("Layer");
        egui::ComboBox::from_id_salt(("garment_layer", id))
            .selected_text(selection.layer.label())
            .show_ui(ui, |ui| {
                for layer in ClothLayer::ALL {
                    ui.selectable_value(&mut selection.layer, layer, layer.label());
                }
            });
    })
    .response
    .on_hover_text("Padding goes under mail; outerwear goes over mail and plate.");
}

/// A named shape to fill the pattern from, when one is chosen.
fn start_from(ui: &mut egui::Ui, id: InventoryItemId) -> Option<shapes::Shape> {
    let mut chosen = None;
    egui::ComboBox::from_id_salt(("garment_shape", id))
        .selected_text("Start from a shape…")
        .show_ui(ui, |ui| {
            for shape in shapes::SHAPES {
                if ui.selectable_label(false, shape.name).clicked() {
                    chosen = Some(shape);
                }
            }
        })
        .response
        .on_hover_text("Fills in the pattern and layer; every setting stays editable.");
    chosen
}

fn upper(ui: &mut egui::Ui, id: InventoryItemId, pattern: &mut Pattern) {
    let straight = matches!(pattern.upper, Some(Upper::Straight { .. }));
    let fitted = pattern.upper == Some(Upper::Fitted);
    // Something must remain to be sewn.
    let may_clear = pattern.lower.is_some();
    ui.horizontal(|ui| {
        ui.label("Body");
        egui::ComboBox::from_id_salt(("garment_upper", id))
            .selected_text(match pattern.upper {
                None => "None",
                Some(Upper::Straight { .. }) => "Straight tunic",
                Some(Upper::Fitted) => "Fitted bodice",
            })
            .show_ui(ui, |ui| {
                let none = ui.add_enabled_ui(may_clear, |ui| {
                    ui.selectable_label(pattern.upper.is_none(), "None")
                });
                if none.inner.clicked() {
                    pattern.upper = None;
                }
                if ui.selectable_label(straight, "Straight tunic").clicked() && !straight {
                    pattern.upper = Some(Upper::STRAIGHT);
                }
                if ui.selectable_label(fitted, "Fitted bodice").clicked() {
                    pattern.upper = Some(Upper::Fitted);
                }
            });
    });
    if pattern.upper.is_none() {
        pattern.sleeves = None;
        pattern.collar = None;
        return;
    }
    ui.indent(("garment_upper_settings", id), |ui| {
        match &mut pattern.upper {
            Some(Upper::Straight {
                length,
                width,
                flare,
            }) => {
                slider(ui, length, Upper::LENGTH, "Length (× neck to waist)").on_hover_text(
                    "Measured down from the shoulder: 1 reaches the waist, about 2.5 the knee.",
                );
                slider(ui, width, Upper::WIDTH, "Ease");
                slider(ui, flare, Upper::FLARE, "Hem flare");
            }
            _ => {
                ui.weak("Cut at the waist.");
            }
        }
        toggle(ui, &mut pattern.sleeves, Sleeves::SHORT, "Sleeves");
        if let Some(sleeves) = &mut pattern.sleeves {
            slider(
                ui,
                &mut sleeves.length,
                Sleeves::LENGTH,
                "Sleeve length (× arm)",
            );
            slider(
                ui,
                &mut sleeves.cuff_width,
                Sleeves::CUFF_WIDTH,
                "Cuff width (× armhole)",
            );
        }
        toggle(ui, &mut pattern.collar, Collar::STANDING, "Standing collar");
        if let Some(collar) = &mut pattern.collar {
            slider(
                ui,
                &mut collar.height_cm,
                Collar::HEIGHT_CM,
                "Collar height (cm)",
            );
        }
    });
}

fn lower(ui: &mut egui::Ui, id: InventoryItemId, pattern: &mut Pattern) {
    let trousers = matches!(pattern.lower, Some(Lower::Trousers { .. }));
    let skirt = matches!(pattern.lower, Some(Lower::Skirt { .. }));
    let may_clear = pattern.upper.is_some();
    ui.horizontal(|ui| {
        ui.label("Legs");
        egui::ComboBox::from_id_salt(("garment_lower", id))
            .selected_text(match pattern.lower {
                None => "None",
                Some(Lower::Trousers { .. }) => "Trousers",
                Some(Lower::Skirt { .. }) => "Skirt",
            })
            .show_ui(ui, |ui| {
                let none = ui.add_enabled_ui(may_clear, |ui| {
                    ui.selectable_label(pattern.lower.is_none(), "None")
                });
                if none.inner.clicked() {
                    pattern.lower = None;
                }
                if ui.selectable_label(trousers, "Trousers").clicked() && !trousers {
                    pattern.lower = Some(Lower::TROUSERS);
                }
                if ui.selectable_label(skirt, "Skirt").clicked() && !skirt {
                    pattern.lower = Some(Lower::SKIRT);
                }
            });
    });
    ui.indent(("garment_lower_settings", id), |ui| {
        match &mut pattern.lower {
            Some(Lower::Trousers {
                length,
                width,
                flare,
            }) => {
                slider(ui, length, Lower::TROUSERS_LENGTH, "Leg length (× leg)");
                slider(ui, width, Lower::TROUSERS_WIDTH, "Ease");
                slider(ui, flare, Lower::TROUSERS_FLARE, "Hem flare")
                    .on_hover_text("Narrower hems slide down the wearer while settling.");
            }
            Some(Lower::Skirt { length, flare_cm }) => {
                slider(ui, length, Lower::SKIRT_LENGTH, "Skirt length (× leg)");
                slider(ui, flare_cm, Lower::SKIRT_FLARE_CM, "Hem flare (cm)");
            }
            None => {}
        }
    });
}

/// A checkbox that adds a part at its default or removes it.
fn toggle<T: Copy>(ui: &mut egui::Ui, part: &mut Option<T>, default: T, text: &str) {
    let mut on = part.is_some();
    if ui.checkbox(&mut on, text).changed() {
        *part = on.then_some(default);
    }
}

fn slider<T: egui::emath::Numeric>(
    ui: &mut egui::Ui,
    value: &mut T,
    range: RangeInclusive<T>,
    text: &str,
) -> egui::Response {
    ui.add(egui::Slider::new(value, range).text(text))
}
