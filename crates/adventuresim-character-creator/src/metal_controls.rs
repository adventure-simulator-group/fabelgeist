//! Finish controls shared by plate armor and catalog steel: the scratched metal
//! and the engraving cut into it.
use super::*;
use armor_preview::slider;
use fabelgeist_armor::{
    engraving::{Engraving, Relief},
    material::Metal,
};

pub(super) fn metal(ui: &mut egui::Ui, m: &mut Metal) {
    ui.color_edit_button_rgb(&mut m.color);
    slider(ui, "Roughness", &mut m.roughness, 0.08..=0.9);
    ui.add(egui::Slider::new(&mut m.scratch_density, 0..=3000).text("Scratch count"));
    slider(ui, "Scratch length", &mut m.scratch_length, 0.005..=0.4);
    slider(ui, "Scratch width", &mut m.scratch_width, 0.5..=3.0);
    slider(ui, "Scratch depth", &mut m.scratch_depth, 0.0..=1.0);
    slider(
        ui,
        "Scratch angle",
        &mut m.scratch_angle,
        -std::f32::consts::PI..=std::f32::consts::PI,
    );
    slider(
        ui,
        "Angle spread",
        &mut m.scratch_spread,
        0.0..=std::f32::consts::PI,
    );
    ui.add(egui::DragValue::new(&mut m.seed).prefix("Seed "));
}

/// Engraving controls. Returns whether the engraving changed.
pub(super) fn engraving(ui: &mut egui::Ui, engraving: &mut Option<Engraving>) -> bool {
    let before = engraving.clone();
    match engraving {
        None => {
            if ui.button("Add engraving").clicked() {
                *engraving = Some(Engraving::new(""));
            }
        }
        Some(cut) => {
            relief(ui, cut);
            if ui.button("Remove engraving").clicked() {
                *engraving = None;
            }
        }
    }
    *engraving != before
}

fn relief(ui: &mut egui::Ui, cut: &mut Engraving) {
    let mut path = cut.image.to_string_lossy().into_owned();
    if ui
        .add(egui::TextEdit::singleline(&mut path).hint_text("ornament.png"))
        .changed()
    {
        cut.image = path.into();
    }
    if !cut.image.as_os_str().is_empty() && !cut.image.is_file() {
        ui.colored_label(egui::Color32::LIGHT_RED, "Image not found");
    }
    ui.horizontal(|ui| {
        ui.label("Relief");
        let height = matches!(cut.relief, Relief::Height { .. });
        if ui.selectable_label(height, "Height map").clicked() && !height {
            cut.relief = Relief::Height {
                depth: Relief::ETCH_DEPTH,
            };
        }
        if ui.selectable_label(!height, "Normal map").clicked() && height {
            cut.relief = Relief::Normal { strength: 1.0 };
        }
    });
    match &mut cut.relief {
        Relief::Height { depth } => {
            let mut millimetres = *depth * 1000.0;
            slider(
                ui,
                "Depth (mm)",
                &mut millimetres,
                0.0..=Relief::MAX_DEPTH * 1000.0,
            );
            *depth = millimetres / 1000.0;
        }
        Relief::Normal { strength } => {
            slider(ui, "Strength", strength, 0.0..=Relief::MAX_STRENGTH);
        }
    }
    slider(
        ui,
        "Repeats per tile",
        &mut cut.tiles,
        Engraving::MIN_TILES..=Engraving::MAX_TILES,
    );
    slider(
        ui,
        "Rotation",
        &mut cut.rotation,
        -std::f32::consts::PI..=std::f32::consts::PI,
    );
    slider(ui, "Recess roughness", &mut cut.recess_roughness, 0.0..=1.0);
}
