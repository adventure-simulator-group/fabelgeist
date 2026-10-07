//! Sowing by hand: a seed of the chosen species, thrown from the eye
//! through the clicked pixel, takes root where botany says it lands -- on
//! open ground, or at the foot of the wall it hits.

use bevy::math::Ray3d;
use bevy_egui::egui::{self, Ui};
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

    /// The seed a click throws along `ray`, from the eye through the
    /// clicked pixel.
    pub fn throw(&self, ray: Ray3d) -> Command {
        Command::Sow {
            species: self.species,
            origin: ray.origin.to_array(),
            direction: ray.direction.to_array(),
        }
    }
}
