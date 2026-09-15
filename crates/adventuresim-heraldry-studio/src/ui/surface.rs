use super::*;
pub(super) fn controls(ui: &mut egui::Ui, s: &mut PaintedSurface, mixer: &mut super::mixer::Mixer) {
    ui.heading("Painted object");
    ui.collapsing("Support", |ui| {
        combo(
            ui,
            "Shape",
            &mut s.shape,
            &[DisplayShape::Shield, DisplayShape::Panel],
        );
        slider(ui, "Width · mm", &mut s.width.0, 50.0..=2000.0);
        slider(ui, "Height · mm", &mut s.height.0, 50.0..=2000.0);
        slider(ui, "Thickness · mm", &mut s.thickness.0, 3.0..=80.0);
        slider(
            ui,
            "Curvature · mm",
            &mut s.curvature.0,
            0.0..=s.width.0 * 0.25,
        );
        if s.shape == DisplayShape::Shield {
            slider(ui, "Shoulder", &mut s.shoulder.0, 0.0..=0.3);
            slider(ui, "Point", &mut s.point.0, 0.2..=0.8);
        }
        combo(
            ui,
            "Covering",
            &mut s.covering,
            &[Covering::None, Covering::Canvas, Covering::Hide],
        );
        slider(ui, "Ground · mm", &mut s.ground.0, 0.0..=3.0);
        slider(
            ui,
            "Substrate relief · mm",
            &mut s.substrate_relief.0,
            0.0..=2.0,
        );
    });
    ui.collapsing("Paint and metal leaf", |ui| {
        slider(ui, "Pigment layer · mm", &mut s.pigment.0, 0.0..=0.3);
        slider(ui, "Brush width · mm", &mut s.brush_width.0, 0.5..=40.0);
        slider(ui, "Brush angle", &mut s.brush_angle.0, -180.0..=180.0);
        slider(ui, "Brush relief · mm", &mut s.brush_relief.0, 0.0..=0.2);
        finish(ui, Tincture::Or, &mut s.gold);
        finish(ui, Tincture::Argent, &mut s.silver);
        slider(ui, "Overall clear coating", &mut s.glaze.0, 0.0..=1.0);
        if s.glaze.0 > 0.0 || matches!(s.gold, MetalFinish::YellowGlazedSilver { .. }) {
            slider(ui, "Coating roughness", &mut s.glaze_roughness.0, 0.0..=1.0);
        }
        ui.horizontal(|ui| {
            ui.label("Workmanship seed");
            ui.add(egui::DragValue::new(&mut s.seed.0));
        });
    });
    if let Some(tincture) = super::paint::controls(ui, &mut s.palette) {
        mixer.open(tincture, s.palette[tincture]);
    }
    ui.weak("Intact paint over a prepared support.");
}

fn finish(ui: &mut egui::Ui, tincture: Tincture, value: &mut MetalFinish) {
    ui.push_id(tincture.index(), |ui| {
        ui.label(if tincture == Tincture::Or {
            "Or · gold or yellow"
        } else {
            "Argent · silver or white"
        });
        egui::ComboBox::from_id_salt("application")
            .selected_text(finish_label(*value))
            .show_ui(ui, |ui| {
                for choice in MetalFinish::ALL {
                    if tincture == Tincture::Argent
                        && matches!(choice, MetalFinish::YellowGlazedSilver { .. })
                    {
                        continue;
                    }
                    let selected = std::mem::discriminant(value) == std::mem::discriminant(&choice);
                    if ui
                        .selectable_label(selected, finish_label(choice))
                        .clicked()
                        && !selected
                    {
                        *value = choice;
                    }
                }
            });
        match value {
            MetalFinish::Pigment => {
                ui.weak("Paint selected under Paint recipes.");
            }
            MetalFinish::WaterGilding { burnish } => {
                ui.weak("Leaf on a prepared bole ground, polished with a stone or tooth.");
                slider(ui, "Burnishing", &mut burnish.0, 0.0..=1.0);
            }
            MetalFinish::OilGilding => {
                ui.weak("Leaf on an oil adhesive: unburnished, with a softer reflection.");
            }
            MetalFinish::MordantGilding { relief } => {
                ui.weak("Leaf on raised wax/resin adhesive, following Behaim shield decoration.");
                slider(ui, "Adhesive relief · mm", &mut relief.0, 0.0..=0.1);
            }
            MetalFinish::YellowGlazedSilver { depth } => {
                ui.weak("Silver leaf under a yellow glaze, documented on the Behaim shields.");
                slider(ui, "Yellow glaze depth", &mut depth.0, 0.0..=1.0);
            }
        }
    });
}
fn finish_label(value: MetalFinish) -> &'static str {
    match value {
        MetalFinish::Pigment => "Pigment",
        MetalFinish::WaterGilding { .. } => "Water gilding · burnishable",
        MetalFinish::OilGilding => "Oil gilding · unburnished",
        MetalFinish::MordantGilding { .. } => "Raised mordant gilding",
        MetalFinish::YellowGlazedSilver { .. } => "Yellow-glazed silver",
    }
}
