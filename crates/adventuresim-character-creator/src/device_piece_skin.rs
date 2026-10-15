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
        if matches!(design, ParametricDesign::Skirt(_))
            || matches!(design, ParametricDesign::Garment(garment) if matches!(
                garment.kind,
                fabelgeist_armor::GarmentArmorKind::MailSkirt
                    | fabelgeist_armor::GarmentArmorKind::PaddedSkirt
            ))
        {
            // A plain skirt is one smooth shell: its outer carrier vertices
            // precede their inner-wall copies. Both walls represent the same
            // material points and must receive identical animation ownership.
            let carrier_count = if matches!(design, ParametricDesign::Skirt(_)) {
                let count = self.base.positions.len() / 2;
                anyhow::ensure!(
                    self.base.positions.len() == 2 * count,
                    "invalid skirt shell layout"
                );
                count
            } else {
                self.base.positions.len()
            };
            let skin = crate::skin_transfer::SkinSource {
                positions: wearer.positions,
                faces: body.faces,
                indices: body.joint_indices,
                weights: body.joint_weights,
                names: body.joint_names,
                joints: wearer.joints,
            }
            .skirt(&self.base.positions[..carrier_count])?;
            self.skin.joint_indices = skin.indices;
            self.skin.joint_weights = skin.weights;
            if matches!(design, ParametricDesign::Skirt(_)) {
                self.skin.joint_indices.extend_from_within(..);
                self.skin.joint_weights.extend_from_within(..);
            }
        }
        Ok(())
    }
}
