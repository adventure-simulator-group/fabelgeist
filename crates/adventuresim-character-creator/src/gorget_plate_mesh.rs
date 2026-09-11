//! Meshing the measured collar and shoulder carrier with shared plate topology.
use super::CollarCage;
use adventuresim_armor_model::{GarmentArmorDesign, PartMesh, generate_gorget_plates};
use anyhow::Result;

impl CollarCage {
    pub(super) fn mesh(&self, design: &GarmentArmorDesign) -> Result<PartMesh> {
        Ok(generate_gorget_plates(
            design,
            self.center,
            |t, angle| self.collar_point(t, angle),
            |t, angle| self.bib_point(t, angle),
        )?)
    }
}
