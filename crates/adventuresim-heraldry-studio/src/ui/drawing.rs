use super::*;
pub(super) fn controls(ui: &mut egui::Ui, s: &mut DrawingStyle) {
    ui.heading("Drawing");
    slider(
        ui,
        "Contour character",
        &mut s.contour_character.0,
        0.0..=1.0,
    );
    slider(ui, "Asymmetry", &mut s.asymmetry.0, 0.0..=1.0);
    slider(ui, "Line width", &mut s.stroke_width.0, 0.001..=0.012);
    egui::CollapsingHeader::new("Eagle anatomy")
        .default_open(true)
        .show(ui, |ui| {
            slider(ui, "Feather line detail", &mut s.detail.0, 0.0..=1.0);
            let e = &mut s.eagle;
            for (label, v) in [
                ("Body width", &mut e.body_width),
                ("Wing span", &mut e.wing_span),
                ("Wing lift", &mut e.wing_lift),
                ("Feather length", &mut e.feather_length),
                ("Neck length", &mut e.neck_length),
                ("Head size", &mut e.head_size),
                ("Leg spread", &mut e.leg_spread),
                ("Tail fan", &mut e.tail_spread),
            ] {
                slider(ui, label, &mut v.0, 0.5..=1.5);
            }
            ui.add(egui::Slider::new(&mut e.feather_count, 6..=16).text("Flight feathers"));
        });
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
    ui.collapsing("Lion paint", |ui| {
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
