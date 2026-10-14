//! What a click on the world does: nothing, sow a species, or clear the
//! plants round the clicked spot.

use bevy::math::Ray3d;
use fabelgeist_botany::viewer::sim::{Command, SpeciesInfo};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    /// The mouse only moves the camera.
    Look,
    /// Sow the species at this index into the snapshot's species, the
    /// climbers after the plants.
    Sow(usize),
    /// Clear the plants within a gap's radius.
    Clear,
}

impl Tool {
    /// The line under the hotbar telling what the tool does.
    pub fn hint(self, species: &[SpeciesInfo]) -> String {
        match self {
            Tool::Look => "Drag to orbit  ·  right-drag to pan  ·  scroll to zoom".to_owned(),
            Tool::Sow(index) => match species.get(index) {
                Some(info) => format!("Click the ground to sow {}", info.name),
                None => String::new(),
            },
            Tool::Clear => "Click to clear every plant inside the ring".to_owned(),
        }
    }

    /// The species this tool sows, if it sows one that still grows here.
    pub fn sown(self, species: &[SpeciesInfo]) -> Option<&SpeciesInfo> {
        match self {
            Tool::Sow(index) => species.get(index),
            Tool::Look | Tool::Clear => None,
        }
    }
}

/// The seed a click with `species` throws along `ray`, from the eye through
/// the clicked pixel: botany decides where it lands, on open ground or at
/// the foot of the wall it hits.
pub fn sow(species: usize, ray: Ray3d) -> Command {
    Command::Sow {
        species,
        origin: ray.origin.to_array(),
        direction: ray.direction.to_array(),
    }
}
