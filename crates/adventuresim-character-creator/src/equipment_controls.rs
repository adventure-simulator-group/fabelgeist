//! Egui controls for parametric equipment designs.

use super::{BracerDesign, BreastplateDesign, Studio, egui};

pub(super) fn bracer(ui: &mut egui::Ui, studio: &mut Studio) {
    ui.collapsing("Parametric bracers", |ui| {
        let mut changed = false;
        changed |= ui
            .add(
                egui::Slider::new(&mut studio.bracer_design.coverage.0, 50..=1_000)
                    .text("Forearm coverage")
                    .suffix(" ‰"),
            )
            .changed();
        let maximum_offset = 1_000_u16 - studio.bracer_design.coverage.0;
        if studio.bracer_design.wrist_offset.0 > maximum_offset {
            studio.bracer_design.wrist_offset.0 = maximum_offset;
            changed = true;
        }
        changed |= ui
            .add(
                egui::Slider::new(&mut studio.bracer_design.wrist_offset.0, 0..=maximum_offset)
                    .text("Wrist offset")
                    .suffix(" ‰"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut studio.bracer_design.wall_thickness.0, 1..=20)
                    .text("Wall thickness")
                    .suffix(" mm"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut studio.bracer_design.clearance.0, 1..=30)
                    .text("Body clearance")
                    .suffix(" mm"),
            )
            .changed();
        ui.horizontal(|ui| {
            if ui.button("Bracelet").clicked() {
                studio.bracer_design = BracerDesign::bracelet();
                changed = true;
            }
            if ui.button("Vambrace").clicked() {
                studio.bracer_design = BracerDesign::default();
                changed = true;
            }
            if ui.button("Full forearm").clicked() {
                studio.bracer_design = BracerDesign::full_forearm();
                changed = true;
            }
        });
        ui.small("Enable either Vambrace catalog placement above to preview it.");
        studio.dirty |= changed;
    });
}

pub(super) fn breastplate(ui: &mut egui::Ui, studio: &mut Studio) {
    ui.collapsing("Parametric breastplate", |ui| {
        let design = &mut studio.breastplate_design;
        let mut changed = false;
        changed |= ui
            .add(egui::Slider::new(&mut design.neck_width.0, 700..=1_300).text("Neck width"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut design.neck_depth.0, 600..=1_400).text("Neck depth"))
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut design.arm_opening_depth.0, 700..=1_300)
                    .text("Arm opening depth"),
            )
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut design.waist_width.0, 750..=1_200).text("Waist width"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut design.plate_length.0, 650..=1_150).text("Plate length"))
            .changed();
        changed |= ui
            .add(egui::Slider::new(&mut design.side_return.0, 850..=1_080).text("Side return"))
            .changed();
        changed |= crate::breastplate_controls::shape(ui, design);
        changed |= ui
            .add(
                egui::Slider::new(&mut design.skirt_length.0, 500..=1_600)
                    .text("Skirt length")
                    .suffix(" ‰"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut design.skirt_flare.0, 0..=70)
                    .text("Skirt flare")
                    .suffix(" mm"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut design.wall_thickness.0, 1..=20)
                    .text("Wall thickness")
                    .suffix(" mm"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut design.front_clearance.0, 4..=30)
                    .text("Front clearance")
                    .suffix(" mm"),
            )
            .changed();
        changed |= ui
            .add(
                egui::Slider::new(&mut design.back_clearance.0, 6..=35)
                    .text("Back clearance")
                    .suffix(" mm"),
            )
            .changed();
        if ui.button("Reset breastplate").clicked() {
            *design = BreastplateDesign::default();
            changed = true;
        }
        ui.small("Enable Breastplate · worn above to preview it.");
        studio.dirty |= changed;
    });
}
