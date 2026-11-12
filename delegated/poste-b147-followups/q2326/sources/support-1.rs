//! Model detail and pose-corrective selection in the character studio.
use super::*;

pub(super) fn show(ui: &mut egui::Ui, studio: &mut Studio) {
    let lod_changed = ui
        .add(
            egui::Slider::new(
                &mut studio.selected_lod,
                MIN_CHARACTER_LOD..=MAX_CHARACTER_LOD,
            )
            .text("Mesh LOD")
            .custom_formatter(|value, _| {
                let lod = value.round() as u8;
                let vertices = CharacterLod::try_from(lod)
                    .expect("slider bounds")
                    .vertices();
                format!("{lod} · {vertices} vertices")
            }),
        )
        .changed();
    if lod_changed {
        studio.status = format!("Loading MHR LOD {}…", studio.selected_lod);
    }
    ui.small("LOD 4 is highest fidelity; LOD 6 is lowest.");
    if ui
        .checkbox(&mut studio.selected_correctives, "Pose-corrective model")
        .changed()
    {
        studio.status = format!(
            "Loading MHR LOD {} with correctives {}…",
            studio.selected_lod,
            if studio.selected_correctives {
                "enabled"
            } else {
                "disabled"
            }
        );
    }
    ui.small("Correctives improve posed deformation but require substantially more memory.");
}
