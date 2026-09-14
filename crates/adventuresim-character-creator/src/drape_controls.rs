//! Mesh resolution and per-stage drape simulation settings for one garment.
use super::*;
use adventuresim_character_creator::garment::{ArmorFitSettings, DrapeSettings, StageSettings};

pub(super) fn show(ui: &mut egui::Ui, selection: &mut GarmentSelection) {
    if let Some(range) = selection.preset.length_range() {
        selection.length = selection.length.clamp(*range.start(), *range.end());
        ui.add(egui::Slider::new(&mut selection.length, range).text("Length (× neck to waist)"))
            .on_hover_text("Measured down from the shoulder: 1 reaches the waist, about 2.5 the knee.");
    }
    if selection.preset.is_fitted() {
        // The coif shares its shape controls with the catalog mail coif.
        super::armor_controls::coif(ui, &mut selection.coif);
    }
    ui.add(
        egui::Slider::new(
            &mut selection.resolution_cm,
            GarmentSelection::RESOLUTION_CM,
        )
        .text("Mesh spacing (cm)"),
    );
    let fabric = selection.fabric;
    let drape = &mut selection.drape;
    ui.collapsing("Drape stages", |ui| {
        ui.small("Changing a stage re-runs from that stage and reuses earlier ones.");
        ui.add(
            egui::Slider::new(&mut drape.preview_interval, DrapeSettings::PREVIEW_INTERVAL)
                .text("Preview every N steps"),
        );
        ui.collapsing("Sewing", |ui| stage(ui, &mut drape.sewing));
        ui.collapsing("Settling", |ui| stage(ui, &mut drape.settling));
        ui.collapsing("Armor fit", |ui| armor_fit(ui, &mut drape.armor_fit));
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
    .on_hover_text(
        "Swept contacts run on the GPU; with armor, each pass also reads the cloth back. Fewer passes are faster.",
    );
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
        ui.add(
            egui::Slider::new(&mut settings.armor_passes, StageSettings::ARMOR_PASSES)
                .text("Armor passes"),
        );
    });
}

fn armor_fit(ui: &mut egui::Ui, settings: &mut ArmorFitSettings) {
    ui.add(
        egui::Slider::new(&mut settings.passes, ArmorFitSettings::PASSES)
            .logarithmic(true)
            .text("Passes"),
    );
    ui.add(
        egui::Slider::new(
            &mut settings.contact_iterations,
            ArmorFitSettings::CONTACT_ITERATIONS,
        )
        .text("Contact iterations"),
    );
}
