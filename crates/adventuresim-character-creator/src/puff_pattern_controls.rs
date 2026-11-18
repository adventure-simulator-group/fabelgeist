//! Shared treatment controls for limb and skirt textiles.
use super::number;
use bevy_egui::egui;
use fabelgeist_armor::{PuffAndSlashPattern, PuffCourse, PuffEdge, PuffSlashes};

pub(super) fn show(ui: &mut egui::Ui, design: &mut PuffAndSlashPattern) -> bool {
    let mut changed = false;
    let mut count = design.courses.len();
    let resized = ui
        .add(
            egui::Slider::new(&mut count, 1..=PuffAndSlashPattern::MAX_COURSES)
                .text("Puff courses"),
        )
        .changed();
    if resized {
        // New courses repeat the proximal one, next to the existing cloth.
        let proximal = design.courses.last().cloned().unwrap_or_default();
        design.courses.resize(count, proximal);
        changed = true;
    }
    for (index, course) in design.courses.iter_mut().enumerate() {
        ui.collapsing(format!("Course {} (distal first)", index + 1), |ui| {
            changed |= course_controls(ui, course);
        });
    }
    changed |= number(
        ui,
        &mut design.puff_roundness.0,
        PuffAndSlashPattern::ROUNDNESS_RANGE,
        "Puff roundness",
    );
    changed |= number(ui, &mut design.rotation.0, 0..=1000, "Pattern rotation");
    changed |= number(
        ui,
        &mut design.lining_fullness.0,
        PuffAndSlashPattern::LINING_FULLNESS_RANGE,
        "Lining fullness",
    );
    ui.horizontal(|ui| {
        ui.label("Proximal edge");
        let band = ui.selectable_value(&mut design.proximal_edge, PuffEdge::Band, "Band");
        let seam = ui.selectable_value(&mut design.proximal_edge, PuffEdge::Seam, "Seam");
        changed |= band.changed() || seam.changed();
    });
    ui.collapsing("Outer fabric", |ui| {
        changed |= super::textile::show(ui, &mut design.outer_fabric);
    });
    ui.collapsing("Band fabric", |ui| {
        changed |= super::textile::show(ui, &mut design.band_fabric);
    });
    ui.collapsing("Undercloth", |ui| {
        changed |= super::textile::show(ui, &mut design.undercloth_fabric);
    });
    changed
}

fn course_controls(ui: &mut egui::Ui, course: &mut PuffCourse) -> bool {
    let mut changed = number(
        ui,
        &mut course.length.0,
        PuffCourse::LENGTH_RANGE,
        "Length share",
    );
    changed |= number(
        ui,
        &mut course.fullness.0,
        PuffCourse::FULLNESS_RANGE,
        "Puff fullness (mm)",
    );
    changed |= number(
        ui,
        &mut course.constriction_width.0,
        PuffCourse::CONSTRICTION_RANGE,
        "Constricted band width",
    );
    let mut slashed = course.slashes.is_some();
    let slash_toggle = ui.checkbox(&mut slashed, "Open slashes").changed();
    if slash_toggle {
        course.slashes = slashed.then(PuffSlashes::default);
        changed = true;
    }
    if let Some(slashes) = &mut course.slashes {
        changed |= ui
            .add(
                egui::Slider::new(&mut slashes.count, PuffSlashes::COUNT_RANGE)
                    .text("Slashes per course"),
            )
            .changed();
        changed |= number(
            ui,
            &mut slashes.width.0,
            PuffSlashes::WIDTH_RANGE,
            "Slash width",
        );
        changed |= number(
            ui,
            &mut slashes.length.0,
            PuffSlashes::LENGTH_RANGE,
            "Slash length",
        );
        changed |= number(
            ui,
            &mut slashes.widest.0,
            PuffSlashes::WIDEST_RANGE,
            "Widest point",
        );
        changed |= number(ui, &mut slashes.slant.0, 0..=1000, "Slash slant");
    }
    changed
}
