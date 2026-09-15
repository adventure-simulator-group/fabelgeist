//! Paint choices expose the ingredients, evidence and estimated appearance.
use super::*;
use adventuresim_heraldry::paint::{Paint, PaintPalette, PaintRecipe};

pub(super) fn controls(ui: &mut egui::Ui, palette: &mut PaintPalette) -> Option<Tincture> {
    let mut open = None;
    egui::CollapsingHeader::new("Paint recipes")
        .default_open(true)
        .show(ui, |ui| {
            ui.weak("Sourced presets and measured reference mixes. Leaf is selected separately.");
            for tincture in Tincture::ALL {
                ui.push_id(tincture.index(), |ui| {
                    if paint(ui, tincture, &mut palette[tincture]) {
                        open = Some(tincture);
                    }
                });
            }
        });
    open
}

fn paint(ui: &mut egui::Ui, tincture: Tincture, paint: &mut Paint) -> bool {
    let label = match *paint {
        Paint::Recipe { recipe } => recipe.definition().label,
        Paint::Mixed { .. } => "Measured stock mixtures",
    };
    ui.horizontal(|ui| {
        ui.label(format!("{tincture:?}"));
        egui::ComboBox::from_id_salt("paint-recipe")
            .selected_text(label)
            .show_ui(ui, |ui| {
                for suggested in [true, false] {
                    ui.weak(if suggested {
                        "Suggested for this tincture"
                    } else {
                        "Other paint colors"
                    });
                    for recipe in PaintRecipe::ALL {
                        if (recipe.tincture() == tincture) != suggested {
                            continue;
                        }
                        ui.selectable_value(
                            paint,
                            Paint::Recipe { recipe },
                            recipe.definition().label,
                        );
                    }
                }
            });
    });
    match paint {
        Paint::Recipe { recipe } => {
            let definition = recipe.definition();
            ui.small(definition.binder.label());
            ui.collapsing("Recipe details", |ui| {
                for ingredient in definition.ingredients {
                    let parts = ingredient.historical_parts
                        .map(|p| format!(" · {p} historical parts"))
                        .unwrap_or_default();
                    ui.label(format!("{}{parts}", ingredient.pigment.label()));
                }
                ui.label(definition.evidence.label());
                ui.hyperlink_to(definition.source_location, definition.source_url);
                ui.weak("Painted shadow/highlight swatches are artistic estimates; their mixtures are not reconstructed.");
            });
        }
        Paint::Mixed { .. } => {
            ui.small("Gum Arabic · measured on parchment");
            ui.weak("Independent base, shadow and highlight recipes.");
        }
    }
    ui.button("Open measured paint mixer…").clicked()
}
