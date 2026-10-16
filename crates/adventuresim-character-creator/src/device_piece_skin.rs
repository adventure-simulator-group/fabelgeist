//! Animation ownership for fitted flexible textile pieces.

use anyhow::Result;

use crate::armor_recipes::ParametricDesign;
use crate::device_fit::{DevicePiece, FitBody, Realization};

impl DevicePiece {
    pub(crate) fn bind_flexible_textile(
        &mut self,
        body: &FitBody<'_>,
        wearer: &Realization<'_>,
        design: &ParametricDesign,
    ) -> Result<()> {
        if matches!(
            design,
            ParametricDesign::Skirt(_) | ParametricDesign::Gown(_)
        ) || matches!(design, ParametricDesign::Garment(garment) if matches!(
            garment.kind,
            fabelgeist_armor::GarmentArmorKind::MailSkirt
                | fabelgeist_armor::GarmentArmorKind::PaddedSkirt
        )) {
            let skin = crate::skin_transfer::SkinSource {
                positions: wearer.positions,
                faces: body.faces,
                indices: body.joint_indices,
                weights: body.joint_weights,
                names: body.joint_names,
                joints: wearer.joints,
            }
            .skirt(&self.base.positions)?;
            self.skin.joint_indices = skin.indices;
            self.skin.joint_weights = skin.weights;
            if matches!(
                design,
                ParametricDesign::Skirt(_) | ParametricDesign::Gown(_)
            ) {
                anyhow::ensure!(
                    self.skin_aliases.len() == self.base.positions.len()
                        && self
                            .skin_aliases
                            .iter()
                            .all(|&i| i < self.base.positions.len()),
                    "invalid shell skin aliases"
                );
                let indices = self.skin.joint_indices.clone();
                let weights = self.skin.joint_weights.clone();
                self.skin.joint_indices = self.skin_aliases.iter().map(|&i| indices[i]).collect();
                self.skin.joint_weights = self.skin_aliases.iter().map(|&i| weights[i]).collect();
            }
        }
        Ok(())
    }
}
