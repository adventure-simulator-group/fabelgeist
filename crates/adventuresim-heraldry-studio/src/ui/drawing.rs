use super::*;
pub(super) fn controls(ui: &mut egui::Ui, s: &mut DrawingStyle) {
    ui.heading("Drawing");
    slider(ui, "Asymmetry", &mut s.asymmetry.0, 0.0..=1.0);
    slider(ui, "Line width", &mut s.stroke_width.0, 0.001..=0.012);
    ui.collapsing("Lion anatomy", |ui| {
        let l = &mut s.lion;
        for (label, v) in [
            ("Body width", &mut l.body_width),
            ("Spine arch", &mut l.spine_arch),
            ("Head size", &mut l.head_size),
            ("Foreleg reach", &mut l.foreleg_reach),
            ("Hindleg spread", &mut l.hindleg_spread),
            ("Mane fullness", &mut l.mane_fullness),
            ("Tail curl", &mut l.tail_curl),
            ("Paw size", &mut l.paw_size),
        ] {
            slider(ui, label, &mut v.0, 0.5..=1.5);
        }
        ui.weak("German drawing, c. 1530. The mane and interior lines follow the anatomy.");
    });
    ui.collapsing("Charge paint", |ui| {
        ui.horizontal(|ui| {
            if ui.button("Flat").clicked() {
                s.painted_modeling = PaintedModeling::FLAT;
            }
            if ui.button("Modeled").clicked() {
                s.painted_modeling = PaintedModeling::MODELED;
            }
        });
        slider(ui, "Shadows", &mut s.painted_modeling.shadows.0, 0.0..=1.0);
        slider(
            ui,
            "Highlights",
            &mut s.painted_modeling.highlights.0,
            0.0..=1.0,
        );
        ui.weak("Paint coverage. Flat retains the interior lines. Painted tones stay fixed as the light moves.");
    });
}
