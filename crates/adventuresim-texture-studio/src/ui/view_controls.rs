use crate::document::{Document, Shape};
use bevy_egui::egui;
pub(super) fn draw(ui: &mut egui::Ui, document: &mut Document) {
    ui.add_space(18.0);
    ui.separator();
    ui.heading("View");
    let v = &mut document.view;
    egui::ComboBox::from_id_salt("shape")
        .selected_text(format!("{:?}", v.shape))
        .show_ui(ui, |ui| {
            for shape in Shape::ALL {
                ui.selectable_value(&mut v.shape, shape, format!("{shape:?}"));
            }
        });
    ui.checkbox(&mut v.orthographic, "Orthographic");
    ui.add(
        egui::Slider::new(&mut v.backdrop_distance, 0.1..=5.0).text("Glass background distance"),
    );
    if v.shape != Shape::Plane {
        v.displacement = 0.0;
    }
    ui.checkbox(&mut v.turntable, "Turntable");
    ui.add(
        egui::Slider::new(&mut v.repeats, 0.05..=8.0)
            .logarithmic(true)
            .text("UV repeats"),
    );
    ui.add(egui::Slider::new(&mut v.offset[0], 0.0..=1.0).text("U offset"));
    ui.add(egui::Slider::new(&mut v.offset[1], 0.0..=1.0).text("V offset"));
    ui.horizontal(|ui| {
        if ui.button("Overview").clicked() {
            v.repeats = 1.0;
            v.offset = [0.0; 2];
        }
        if ui.button("Detail").clicked() {
            use adventuresim_procedural_textures::TextureRecipeId;
            (v.repeats, v.offset) = match document.recipe {
                TextureRecipeId::DressedStone => (0.25, [0.30, 0.36]),
                TextureRecipeId::HandmadeBrick => (0.65, [0.17, 0.21]),
                _ => (0.38, [0.523, 0.177]),
            };
        }
        if ui.button("Distance").clicked() {
            v.repeats = 4.0;
            v.offset = [0.0; 2];
        }
    });
    egui::CollapsingHeader::new("Diagnostic displacement").show(ui, |ui| {
        ui.small("Physical height on a subdivided plane. Normal relief is disabled to avoid applying it twice.");
        ui.add_enabled(v.shape == Shape::Plane, egui::Slider::new(&mut v.displacement, 0.0..=4.0).text("Height gain"));
    });
}
