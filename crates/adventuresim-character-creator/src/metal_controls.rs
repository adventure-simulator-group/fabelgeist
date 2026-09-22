//! Finish controls shared by plate armor and catalog steel: the scratched metal,
//! the engraving cut into it, and the trim along a plate's edges.
use super::*;
use armor_preview::slider;
use fabelgeist_armor::{
    engraving::{Engraving, Relief, ReliefSource},
    material::Metal,
    ornament::Ornament,
    trim::Trim,
};

pub(super) fn metal(ui: &mut egui::Ui, m: &mut Metal) {
    ui.color_edit_button_rgb(&mut m.color);
    slider(ui, "Roughness", &mut m.roughness, 0.08..=0.9);
    let mut millimetres = m.waviness * 1000.0;
    slider(
        ui,
        "Hammered waviness (mm)",
        &mut millimetres,
        0.0..=Metal::MAX_WAVINESS * 1000.0,
    );
    m.waviness = millimetres / 1000.0;
    slider(ui, "Polish grain", &mut m.grain, 0.0..=1.0);
    slider(ui, "Uneven gloss", &mut m.smudge, 0.0..=0.4);
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
                *engraving = Some(Engraving::ornament(Ornament::default()));
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

/// Trim controls: the band's width, its own metal, and the ornament running
/// along it. Returns whether the trim changed.
pub(super) fn trim(ui: &mut egui::Ui, trim: &mut Option<Trim>) -> bool {
    let before = trim.clone();
    match trim {
        None => {
            let add = ui.button("Add trim");
            if add.clicked() {
                *trim = Some(Trim::default());
            }
        }
        Some(band) => {
            let mut millimetres = band.width * 1000.0;
            slider(
                ui,
                "Width (mm)",
                &mut millimetres,
                Trim::MIN_WIDTH * 1000.0..=Trim::MAX_WIDTH * 1000.0,
            );
            band.width = millimetres / 1000.0;
            ui.collapsing("Finish", |ui| metal(ui, &mut band.metal));
            ui.collapsing("Ornament", |ui| {
                let added = band.metal.engraving.is_none();
                engraving(ui, &mut band.metal.engraving);
                let fitted = band.cell_tiles();
                let period = band.period();
                let Some(cut) = &mut band.metal.engraving else {
                    return;
                };
                if added {
                    cut.tiles = fitted;
                }
                ui.small(format!(
                    "One cell runs {:.0} mm along the edge, from the edge inward.",
                    period * 1000.0
                ));
                let fit = ui.button("Fit cell to band width");
                if fit.clicked() {
                    cut.tiles = fitted;
                }
            });
            let remove = ui.button("Remove trim");
            if remove.clicked() {
                *trim = None;
            }
        }
    }
    *trim != before
}

fn relief(ui: &mut egui::Ui, cut: &mut Engraving) {
    ui.horizontal(|ui| {
        let drawn = matches!(cut.source, ReliefSource::Ornament(_));
        let pattern = ui.selectable_label(drawn, "Pattern");
        if pattern.clicked() && !drawn {
            cut.source = ReliefSource::Ornament(Ornament::default());
            cut.relief = Relief::Height {
                depth: Relief::ETCH_DEPTH,
            };
        }
        let image = ui.selectable_label(!drawn, "Image");
        if image.clicked() && drawn {
            cut.source = ReliefSource::Image(Default::default());
        }
    });
    match &mut cut.source {
        ReliefSource::Ornament(ornament) => ornament_controls::ornament(ui, ornament),
        ReliefSource::Image(image) => image_source(ui, image, &mut cut.relief),
    }
    depth(ui, &mut cut.relief);
    tiling(ui, cut);
}

fn image_source(ui: &mut egui::Ui, image: &mut std::path::PathBuf, relief: &mut Relief) {
    let mut path = image.to_string_lossy().into_owned();
    if ui
        .add(egui::TextEdit::singleline(&mut path).hint_text("ornament.png"))
        .changed()
    {
        *image = path.into();
    }
    if !image.as_os_str().is_empty() && !image.is_file() {
        ui.colored_label(egui::Color32::LIGHT_RED, "Image not found");
    }
    ui.horizontal(|ui| {
        ui.label("Relief");
        let height = matches!(relief, Relief::Height { .. });
        if ui.selectable_label(height, "Height map").clicked() && !height {
            *relief = Relief::Height {
                depth: Relief::ETCH_DEPTH,
            };
        }
        if ui.selectable_label(!height, "Normal map").clicked() && height {
            *relief = Relief::Normal { strength: 1.0 };
        }
    });
}

fn depth(ui: &mut egui::Ui, relief: &mut Relief) {
    match relief {
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
}

fn tiling(ui: &mut egui::Ui, cut: &mut Engraving) {
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
