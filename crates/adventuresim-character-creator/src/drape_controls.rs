//! Mesh resolution and per-stage drape simulation settings for one garment.
use super::*;
use adventuresim_character_creator::garment::{DrapeSettings, StageSettings};

pub(super) fn show(ui: &mut egui::Ui, selection: &mut GarmentSelection) {
    ui.add(
        egui::Slider::new(
            &mut selection.resolution_cm,
            GarmentSelection::RESOLUTION_CM,
        )
        .text("Mesh spacing (cm)"),
    );
    let fabric = selection.fabric;
    let drape = &mut selection.drape;
    ui.add(egui::Slider::new(&mut drape.body_fit, DrapeSettings::BODY_FIT).text("Body fit"))
        .on_hover_text(
            "How much of the gap between the settled cloth and the body it dresses to close, \
             without stretching it. Hems past the body keep hanging.",
        );
    ui.collapsing("Drape stages", |ui| {
        ui.small("Changing a stage re-runs from that stage and reuses earlier ones.");
        ui.add(
            egui::Slider::new(&mut drape.preview_interval, DrapeSettings::PREVIEW_INTERVAL)
                .text("Preview every N steps"),
        );
        ui.collapsing("Sewing", |ui| stage(ui, &mut drape.sewing));
        ui.collapsing("Settling", |ui| stage(ui, &mut drape.settling));
        if ui.button("Reset stage settings").clicked() {
            *drape = DrapeSettings::for_fabric(fabric.fabric());
        }
    });
}

fn stage(ui: &mut egui::Ui, settings: &mut StageSettings) {
    ui.add(egui::Slider::new(&mut settings.steps, StageSettings::STEPS).text("Steps"));
    ui.add(egui::Slider::new(&mut settings.substeps, StageSettings::SUBSTEPS).text("Substeps"));
    ui.add(
        egui::Slider::new(&mut settings.iterations, StageSettings::ITERATIONS)
            .text("Constraint iterations"),
    );
    ui.add(egui::Slider::new(&mut settings.gravity, StageSettings::GRAVITY).text("Gravity (m/s²)"));
    ui.add(
        egui::Slider::new(&mut settings.damping, StageSettings::DAMPING)
            .text("Damping (per second)"),
    );
    ui.checkbox(&mut settings.self_collision, "Self-collision");
    settings.host_contact_interval = settings.host_contact_interval.min(settings.substeps);
    ui.add(
        egui::Slider::new(&mut settings.host_contact_interval, 0..=settings.substeps)
            .text("Swept contacts every N substeps (0 = off)"),
    )
    .on_hover_text("Swept contacts run on the GPU. Fewer passes are faster.");
    ui.add_enabled_ui(settings.host_contact_interval > 0, |ui| {
        ui.add(
            egui::Slider::new(
                &mut settings.host_contact_iterations,
                StageSettings::CONTACT_ITERATIONS,
            )
            .text("Swept contact iterations"),
        );
        ui.checkbox(
            &mut settings.host_body_contacts,
            "Swept contacts against the body",
        );
    });
}
