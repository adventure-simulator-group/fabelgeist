//! Model detail and pose-corrective selection in the character studio.
use super::*;

pub(super) fn show(ui: &mut egui::Ui, studio: &mut Studio) {
    let previous = studio.selected_config;
    egui::ComboBox::from_label("Mesh LOD")
        .selected_text(format!(
            "{} · {} vertices",
            previous.lod,
            previous.lod.vertices()
        ))
        .show_ui(ui, |ui: &mut egui::Ui| -> () {
            for lod in CharacterLod::ALL {
                ui.selectable_value(
                    &mut studio.selected_config.lod,
                    lod,
                    format!("{lod} · {} vertices", lod.vertices()),
                );
            }
        });
    ui.small("LOD 4 is highest fidelity; LOD 6 is lowest.");
    let mut enabled = studio.selected_config.pose_correctives == PoseCorrectivePolicy::Enabled;
    ui.checkbox(&mut enabled, "Pose-corrective model");
    studio.selected_config.pose_correctives = PoseCorrectivePolicy::from(enabled);
    if previous != studio.selected_config {
        studio.status = format!(
            "Loading MHR LOD {} with correctives {}…",
            studio.selected_config.lod, studio.selected_config.pose_correctives,
        );
    }
    ui.small("Correctives improve posed deformation but require substantially more memory.");
}
