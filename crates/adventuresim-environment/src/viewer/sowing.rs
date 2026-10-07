//! Sowing by hand: a seed of the chosen species, thrown from the eye
//! through the clicked pixel, takes root where botany says it lands -- on
//! open ground, or at the foot of the wall it hits.

use eframe::egui::{self, Pos2, Rect, Ui};
use fabelgeist_botany::viewer::camera::{self, OrbitCamera};
use fabelgeist_botany::viewer::sim::{Command, SpeciesInfo};

#[derive(Default)]
pub struct Sowing {
    /// The species a click sows, an index into the snapshot's species, the
    /// climbers after the plants.
    species: usize,
    /// Whether a click on the view sows at all.
    active: bool,
}

impl Sowing {
    pub fn active(&self) -> bool {
        self.active
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    /// A new scene has its own species; the choice starts from its first.
    pub fn forget_species(&mut self) {
        self.species = 0;
    }

    /// The species to sow, and the switch that makes a click sow it.
    pub fn show(&mut self, ui: &mut Ui, species: &[SpeciesInfo]) {
        let Some(last) = species.len().checked_sub(1) else {
            return;
        };
        self.species = self.species.min(last);
        ui.strong("Sowing");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("sow-species")
                .selected_text(species[self.species].name.as_str())
                .show_ui(ui, |ui| {
                    for (index, info) in species.iter().enumerate() {
                        ui.selectable_value(&mut self.species, index, info.name.as_str());
                    }
                });
            let label = if self.active { "Sowing" } else { "Sow" };
            ui.toggle_value(&mut self.active, label)
                .on_hover_text("While on, a click on the view sows the species picked here (S).");
        });
    }

    /// The seed a click at `at` throws, the view showing `camera` in `view`.
    pub fn throw(&self, camera: &OrbitCamera, view: Rect, at: Pos2) -> Command {
        let x = (at.x - view.left()) / view.width().max(1.0) * 2.0 - 1.0;
        let y = 1.0 - (at.y - view.top()) / view.height().max(1.0) * 2.0;
        let (forward, right, up) = camera.basis();
        let tan = (camera.fov_y * 0.5).tan();
        let aspect = view.width() / view.height().max(1.0);
        let direction = camera::add(
            forward,
            camera::add(
                camera::scale(right, x * tan * aspect),
                camera::scale(up, y * tan),
            ),
        );
        Command::Sow {
            species: self.species,
            origin: camera.eye(),
            direction,
        }
    }
}
