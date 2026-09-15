use super::*;
use bevy_egui::{EguiContexts, egui};

pub(super) fn draw(
    mut contexts: EguiContexts,
    mut editor: ResMut<Editor>,
    options: Res<Options>,
) -> Result {
    if options.output.is_some() {
        return Ok(());
    }
    egui::Window::new("Parametric plants and fungi").show(contexts.ctx_mut()?, |ui| {
        ui.label("Drag to orbit · scroll to zoom. Dimensions are metres.");
        let mut family = editor.parameters.family();
        egui::ComboBox::from_id_salt("family")
            .selected_text(format!("{family:?}"))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut family, Family::Flowers, "Flowers");
                ui.selectable_value(&mut family, Family::Fungi, "Fungi");
            });
        if family != editor.parameters.family() {
            editor.parameters = family.recipe(0);
            editor.origin = RecipeOrigin::Preset;
            editor.preset = 0;
            editor.dirty = true;
            editor.reframe = true;
        }
        let mut selected = editor.preset;
        let mut load_preset = false;
        egui::ComboBox::from_id_salt("preset")
            .selected_text(match editor.origin {
                RecipeOrigin::Preset => family.name(selected),
                RecipeOrigin::Custom => "Custom recipe",
            })
            .show_ui(ui, |ui| {
                for index in 0..family.count() {
                    load_preset |= ui
                        .selectable_value(&mut selected, index, family.name(index))
                        .clicked();
                }
            });
        if selected != editor.preset || load_preset {
            editor.preset = selected;
            editor.reframe = true;
            editor.parameters = family.recipe(selected);
            editor.origin = RecipeOrigin::Preset;
            editor.dirty = true;
        }
        if ui.button("Frame specimen").clicked() {
            editor.reframe = true;
        }
        let mut value = editor.parameters.controls();
        egui::ScrollArea::vertical()
            .max_height(760.0)
            .show(ui, |ui| {
                if edit_value(ui, "", &mut value) {
                    match editor.parameters.edited(value) {
                        Ok(p) => {
                            editor.parameters = p;
                            editor.origin = RecipeOrigin::Custom;
                            editor.dirty = true;
                        }
                        Err(error) => editor.error = error.to_string(),
                    }
                }
            });
        if ui
            .button("Save recipe to target/plant-recipe.json")
            .clicked()
        {
            let result = (|| -> std::io::Result<()> {
                std::fs::create_dir_all("target")?;
                std::fs::write(
                    "target/plant-recipe.json",
                    serde_json::to_vec_pretty(&editor.parameters)?,
                )
            })();
            editor.error = result.err().map(|e| e.to_string()).unwrap_or_default();
        }
        ui.label(&editor.error);
    });
    Ok(())
}

fn edit_value(ui: &mut egui::Ui, label: &str, value: &mut serde_json::Value) -> bool {
    let mut changed = false;
    match value {
        serde_json::Value::Object(map) => {
            for (key, v) in map {
                changed |= edit_value(ui, key, v);
            }
        }
        serde_json::Value::Number(number) => {
            let integer = number.is_u64();
            let mut v = number.as_f64().unwrap_or_default();
            ui.horizontal(|ui| {
                ui.label(label);
                changed = ui
                    .add(egui::DragValue::new(&mut v).speed(if integer { 1.0 } else { 0.001 }))
                    .changed();
            });
            if changed {
                *value = if integer {
                    serde_json::json!(v.max(0.0) as u64)
                } else {
                    serde_json::json!(v)
                };
            }
        }
        serde_json::Value::Array(items) => {
            let mut rgb = [0_u8; 3];
            for (i, v) in items.iter().take(3).enumerate() {
                rgb[i] = v.as_u64().unwrap_or_default() as u8;
            }
            ui.horizontal(|ui| {
                ui.label(label);
                changed = ui.color_edit_button_srgb(&mut rgb).changed();
            });
            if changed {
                *value = serde_json::json!(rgb);
            }
        }
        serde_json::Value::String(selected) => {
            let choices: &[&str] = match label {
                "corolla" => &["FreePetals", "RayAndDisk", "FusedBell"],
                "leaf_arrangement" => &["BasalRosette", "Alternate", "Whorl"],
                "fertile_surface" => &["Gills", "Pores", "Ridges", "Enclosed"],
                _ => &[],
            };
            egui::ComboBox::from_id_salt(label)
                .selected_text(selected.as_str())
                .show_ui(ui, |ui| {
                    for choice in choices {
                        changed |= ui
                            .selectable_value(selected, (*choice).to_owned(), *choice)
                            .changed();
                    }
                });
        }
        _ => {}
    }
    changed
}
