mod view_controls;
use crate::{
    app::Studio,
    document::{Document, Environment, View},
    inspector,
};
use adventuresim_procedural_textures::{PROCEDURAL_TEXTURE_CATALOGUE, TextureParameters};
use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{self, Color32, RichText},
};

pub(crate) fn draw(mut contexts: EguiContexts, mut studio: ResMut<Studio>, time: Res<Time>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(25, 30, 34);
    visuals.window_fill = visuals.panel_fill;
    visuals.selection.bg_fill = Color32::from_rgb(94, 111, 83);
    visuals.widgets.active.bg_fill = Color32::from_rgb(104, 123, 91);
    ctx.set_visuals(visuals);
    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        "studio-viewport".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );
    let mut history_navigation = false;
    let previous = studio.document.to_json();
    let now = time.elapsed_secs_f64();
    toolbar(
        &mut viewport_ui,
        &mut studio,
        &previous,
        now,
        &mut history_navigation,
    );
    egui::Panel::bottom("status").show_inside(&mut viewport_ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(&studio.status);
            if studio.failed_revision.is_some() && ui.small_button("Retry").clicked() {
                studio.edited(now);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.small("Drag to orbit · Scroll to zoom · Double-click to reset");
            });
        });
    });
    egui::Panel::left("environment")
        .resizable(true)
        .default_size(260.0)
        .size_range(220.0..=420.0)
        .show_inside(&mut viewport_ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("environment-scroll")
                .show(ui, |ui| {
                    environment(ui, &mut studio.document);
                });
        });
    material_panel(&mut viewport_ui, &mut studio, now);
    viewport(&mut viewport_ui, &mut studio, ctx);
    if !history_navigation && studio.document.to_json() != previous && studio.transaction.is_none()
    {
        studio.transaction = Some(previous);
    }
    if !ctx.input(|i| i.pointer.any_down())
        && let Some(before) = studio.transaction.take()
    {
        studio.remember(before);
        studio.save_due = true;
    }
}

fn environment(ui: &mut egui::Ui, document: &mut Document) {
    ui.add_space(10.0);
    ui.heading("Environment");
    ui.small("Light the material, then judge its detail.");
    ui.add_space(12.0);
    ui.horizontal_wrapped(|ui| {
        if ui
            .button("Reviewer")
            .on_hover_text("Exact reviewer light: 7,000 lux, ambient 120, Tony McMapface")
            .clicked()
        {
            document.environment = Environment::default();
        }
        if ui.button("Overcast").clicked() {
            document.environment = Environment {
                key_lux: 2500.0,
                ambient: 600.0,
                incidence_degrees: 65.0,
                fill_lux: 1800.0,
                ..Default::default()
            };
        }
        if ui.button("Warm").clicked() {
            document.environment = Environment {
                key_color: [1.0, 0.77, 0.50],
                fill_lux: 1200.0,
                ..Default::default()
            };
        }
        if ui.button("Raking").clicked() {
            document.environment = Environment {
                incidence_degrees: 8.0,
                ambient: 50.0,
                ..Default::default()
            };
        }
    });
    let e = &mut document.environment;
    ui.add_space(12.0);
    ui.strong("Key light");
    ui.color_edit_button_rgb(&mut e.key_color);
    ui.add(
        egui::Slider::new(&mut e.key_lux, 0.0..=100_000.0)
            .logarithmic(true)
            .text("lux"),
    );
    ui.add(egui::Slider::new(&mut e.azimuth_degrees, -180.0..=180.0).text("Azimuth °"));
    ui.add(egui::Slider::new(&mut e.incidence_degrees, 0.0..=90.0).text("Incidence °"));
    ui.add(egui::Slider::new(&mut e.ambient, 0.0..=1500.0).text("Ambient"));
    ui.add(egui::Slider::new(&mut e.reflection_strength, 0.0..=10000.0).text("Reflection studio"));
    ui.add(egui::Slider::new(&mut e.exposure_ev, -5.0..=5.0).text("Exposure EV"));
    egui::CollapsingHeader::new("Fill and background").show(ui, |ui| {
        ui.color_edit_button_rgb(&mut e.fill_color);
        ui.add(egui::Slider::new(&mut e.fill_lux, 0.0..=30_000.0).text("Fill lux"));
        ui.checkbox(&mut e.shadows, "Shadows");
        ui.label("Background");
        ui.color_edit_button_rgb(&mut e.background);
    });
    view_controls::draw(ui, document);
}

fn response(ui: &mut egui::Ui, document: &mut Document) {
    let s = &mut document.surface;
    ui.add(egui::Slider::new(&mut s.normal_strength, 0.0..=3.0).text("Normal strength"));
    ui.add(egui::Slider::new(&mut s.ao_strength, 0.0..=1.0).text("AO strength"));
    ui.add(egui::Slider::new(&mut s.roughness, 0.0..=1.0).text("Roughness gain"));
    ui.add(egui::Slider::new(&mut s.metallic, 0.0..=1.0).text("Metallic gain"));
    use adventuresim_procedural_textures::TextureRecipeId::*;
    if matches!(document.recipe, OakBark | ForestSoil | ForestLitter) {
        ui.label("Surface pigment");
        ui.color_edit_button_rgb(&mut s.pigment);
    }
    if matches!(
        document.recipe,
        WhiteOakLeaf | DryWhiteOakLeaf | HazelLeaf | BlackthornLeaf | HawthornLeaf | BeechLeaf
    ) {
        ui.add(egui::Slider::new(&mut s.leaf_transmission, 0.0..=1.0).text("Leaf transmission"));
        ui.add(
            egui::Slider::new(&mut s.leaf_thickness_metres, 0.00005..=0.005)
                .logarithmic(true)
                .text("Leaf thickness (m)"),
        );
    }
    if document.recipe == OakBark {
        ui.add(
            egui::Slider::new(&mut s.bark_projection_sharpness, 1.0..=12.0)
                .text("Projection sharpness"),
        );
        ui.add(egui::Slider::new(&mut s.bark_branch_alignment, 0.0..=1.0).text("Branch alignment"));
        ui.add(egui::Slider::new(&mut s.bark_parallax, 0.0..=1.0).text("Parallax"));
        ui.add(egui::Slider::new(&mut s.bark_fade_metres, 1.0..=40.0).text("Relief fade (m)"));
    }
    if document.recipe == ForestLitter {
        for (name, linear) in ["Dark litter", "Mid litter", "Pale litter"]
            .into_iter()
            .zip(&mut s.litter_colors_linear)
        {
            let color = Color::linear_rgb(linear[0], linear[1], linear[2]).to_srgba();
            let mut rgb = [color.red, color.green, color.blue];
            ui.horizontal(|ui| {
                ui.label(name);
                if ui.color_edit_button_rgb(&mut rgb).changed() {
                    *linear = Color::srgb_from_array(rgb)
                        .to_linear()
                        .to_vec4()
                        .xyz()
                        .to_array();
                }
            });
        }
    }
}

fn toolbar(
    ui: &mut egui::Ui,
    studio: &mut Studio,
    previous: &str,
    now: f64,
    history_navigation: &mut bool,
) {
    egui::Panel::top("title").show_inside(ui, |ui| {
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("FABELGEIST")
                    .strong()
                    .size(15.0)
                    .color(Color32::from_rgb(195, 207, 167)),
            );
            ui.label(RichText::new(" /  TEXTURE STUDIO").size(15.0));
            ui.separator();
            let old = studio.document.recipe;
            egui::ComboBox::from_id_salt("recipe")
                .selected_text(inspector::label(old.slug()))
                .width(175.0)
                .show_ui(ui, |ui| {
                    for recipe in PROCEDURAL_TEXTURE_CATALOGUE {
                        ui.selectable_value(
                            &mut studio.document.recipe,
                            recipe.id,
                            inspector::label(recipe.id.slug()),
                        );
                    }
                });
            if old != studio.document.recipe {
                studio.channel = None;
                studio.edited(now);
            }
            ui.separator();
            if ui
                .add_enabled(!studio.undo.is_empty(), egui::Button::new("Undo"))
                .clicked()
                && let Some(source) = studio.undo.pop()
            {
                *history_navigation = true;
                if let Ok(document) = Document::from_json(&source) {
                    studio.redo.push(previous.to_owned());
                    studio.replace(document, now);
                }
            }
            if ui
                .add_enabled(!studio.redo.is_empty(), egui::Button::new("Redo"))
                .clicked()
                && let Some(source) = studio.redo.pop()
            {
                *history_navigation = true;
                if let Ok(document) = Document::from_json(&source) {
                    studio.undo.push(previous.to_owned());
                    studio.replace(document, now);
                }
            }
            if ui.button("Import").clicked() {
                crate::exchange::choose_import();
            }
            if ui.button("Save preset").clicked() {
                crate::exchange::save_preset(&studio.document);
            }
            if ui
                .add_enabled(studio.current.is_some(), egui::Button::new("Export maps"))
                .clicked()
            {
                studio.status = crate::exchange::export_maps(studio).unwrap_or_else(|e| e);
            }
            if ui.button("Screenshot").clicked() {
                studio.screenshot = true;
            }
        });
        ui.add_space(8.0);
    });
}

fn material_panel(ui: &mut egui::Ui, studio: &mut Studio, now: f64) {
    egui::Panel::right("parameters").resizable(true).default_size(370.0).size_range(300.0..=650.0).show_inside(ui, |ui| {
        ui.add_space(10.0);
        ui.heading("Material");
        ui.add(egui::TextEdit::singleline(&mut studio.document.name).hint_text("Preset name").desired_width(f32::INFINITY));
            preset_library(ui, studio, now);
        ui.add_space(8.0);
        ui.add(egui::TextEdit::singleline(&mut studio.search).hint_text("Search parameters…").desired_width(f32::INFINITY));
        ui.horizontal(|ui| {
            if ui.button("Reset recipe").clicked() {
                let mut controls = serde_json::to_value(&studio.document.texture).unwrap();
                for path in studio.document.recipe.control_paths() {
                    if let (Some(value),Some(default)) = (controls.pointer_mut(path.as_str()),studio.defaults.pointer(path.as_str())) { *value = default.clone(); }
                }
                studio.document.texture = serde_json::from_value(controls.clone()).unwrap();
                studio.controls = controls;
                studio.parameter_error = None;
                studio.edited(now);
            }
            let ready = matches!(studio.applied,Some((revision, quality)) if revision == studio.revision && quality == studio.document.texture.resolution);
            if ui.add_enabled(ready,egui::Button::new("Pin comparison")).on_hover_text("Pin the completed final-quality material").clicked() {
                studio.pinned = studio.current.clone().map(|bake| (bake, studio.document.clone()));
                studio.scene_dirty = true;
            }
            if studio.pinned.is_some() && ui.small_button("Clear").clicked() { studio.pinned = None; studio.scene_dirty = true; }
        });
        if let Some(error) = &studio.parameter_error { ui.colored_label(Color32::from_rgb(232,164,124),error); }
        ui.horizontal(|ui| {
            use adventuresim_procedural_textures::BakeResolution;
            ui.label("Output resolution");
            let old = studio.document.texture.resolution;
            let mut resolution = old;
            egui::ComboBox::from_id_salt("output-resolution").selected_text(match old {BakeResolution::Full=>"Native",BakeResolution::Medium=>"256 px",BakeResolution::Draft=>"128 px"}).show_ui(ui,|ui| {
                for (label,value) in [("Native",BakeResolution::Full),("256 px",BakeResolution::Medium),("128 px",BakeResolution::Draft)] { ui.selectable_value(&mut resolution,value,label); }
            });
            if resolution != old { studio.document.texture.resolution = resolution; studio.controls["resolution"] = serde_json::to_value(resolution).unwrap(); studio.edited(now); }
        });
        ui.separator();
        egui::ScrollArea::vertical().id_salt("texture-scroll").show(ui, |ui| {
            egui::CollapsingHeader::new("Material response").show(ui, |ui| response(ui, &mut studio.document));
            let Studio { document, controls, defaults, search, .. } = studio;
            if inspector::draw(ui, document.recipe, controls, defaults, &search.to_lowercase()) {
                match serde_json::from_value::<TextureParameters>(controls.clone()) {
                    Ok(params) => match params.validate() {
                        Ok(()) => { studio.document.texture = params; studio.parameter_error = None; studio.edited(now); }
                        Err(error) => studio.parameter_error = Some(error.to_string()),
                    },
                    Err(error) => studio.parameter_error = Some(error.to_string()),
                }
            }
        });
    });
}

fn viewport(ui: &mut egui::Ui, studio: &mut Studio, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show_inside(ui, |ui| {
            studio.viewport = ui.max_rect();
            ui.horizontal(|ui| {
                let old = studio.channel;
                egui::ComboBox::from_id_salt("channel")
                    .selected_text(
                        old.map(|v| inspector::label(v.slug()))
                            .unwrap_or("Shaded material".into()),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut studio.channel, None, "Shaded material");
                        let channels = studio
                            .current
                            .as_ref()
                            .map(|b| b.maps.iter().map(|m| m.channel).collect::<Vec<_>>())
                            .unwrap_or_default();
                        for channel in channels {
                            ui.selectable_value(
                                &mut studio.channel,
                                Some(channel),
                                inspector::label(channel.slug()),
                            );
                        }
                    });
                if old != studio.channel {
                    studio.scene_dirty = true;
                }
                if studio.pinned.is_some() {
                    ui.label("PINNED  /  CURRENT");
                }
            });
            if studio.document.recipe
                == adventuresim_procedural_textures::TextureRecipeId::WindowGlass
                && studio.channel.is_none()
            {
                egui::Frame::default()
                    .fill(ui.visuals().panel_fill)
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.label("Shading uses nominal thickness. Inspect the R channel of Thickness roughness for local thickness.");
                    });
            }
            let response = ui.interact(
                ui.available_rect_before_wrap(),
                ui.id().with("orbit"),
                egui::Sense::drag(),
            );
            if response.dragged() {
                let delta = ctx.input(|i| i.pointer.delta());
                studio.document.view.yaw -= delta.x * 0.008;
                studio.document.view.pitch =
                    (studio.document.view.pitch + delta.y * 0.008).clamp(-1.45, 1.45);
            }
            if response.hovered() {
                let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
                studio.document.view.distance =
                    (studio.document.view.distance * (-scroll * 0.002).exp()).clamp(0.2, 30.0);
            }
            if response.double_clicked() {
                studio.document.view = View::default();
                studio.scene_dirty = true;
            }
        });
}

fn preset_library(ui: &mut egui::Ui, studio: &mut Studio, now: f64) {
    ui.horizontal(|ui| {
        if ui.button("Save locally").clicked() {
            let name = studio.document.name.trim().to_owned();
            if !name.is_empty() {
                let source = studio.document.to_json();
                studio.library.insert(name, source);
                studio.status = crate::exchange::save_library(&studio.library)
                    .map(|()| "Saved to this browser’s preset library".into())
                    .unwrap_or_else(|error| error);
            }
        }
        egui::ComboBox::from_id_salt("preset-library")
            .selected_text("Saved presets…")
            .show_ui(ui, |ui| {
                let presets = studio
                    .library
                    .iter()
                    .map(|(name, source)| (name.clone(), source.clone()))
                    .collect::<Vec<_>>();
                for (name, source) in presets {
                    if ui.selectable_label(false, name).clicked() {
                        match Document::from_json(&source) {
                            Ok(document) => studio.replace(document, now),
                            Err(error) => studio.status = error,
                        }
                    }
                }
            });
    });
}
