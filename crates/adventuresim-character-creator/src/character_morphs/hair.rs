//! Hair placed again on every sampled head, for the body's morph channels.
use super::*;
use adventuresim_character_creator::hair::{GroomedHair, HairGeometry, HeadPlacement, HeadSkin};

impl CharacterMorphs {
    /// One target per body channel: `base`, the hair as `placement` puts it
    /// on the recipe's own head, carried to where the same placement puts it
    /// on the sampled head. Like clothing, a fit target keeps only what the
    /// head joint does not carry.
    pub(crate) fn hair(
        &self,
        model: &BodyModel,
        hair: &GroomedHair,
        base: &HairGeometry,
        placement: &HeadPlacement,
    ) -> Result<Vec<MorphDelta>> {
        let character = &model.mhr.character;
        self.samples
            .iter()
            .map(|sample| {
                let skin = HeadSkin {
                    positions: &sample.positions,
                    joint_indices: &character.skin_weights.index,
                    joint_weights: &character.skin_weights.weight,
                    joint_names: &character.skeleton.names,
                    global_joint_states: &sample.global_joint_states,
                };
                let sampled = HeadPlacement::fit(&skin, hair.head())?;
                let placed = base.carried(placement, &sampled);
                let mut delta = MorphDelta::between(
                    sample.name.clone(),
                    &base.positions,
                    &base.normals,
                    &placed.positions,
                    &placed.normals,
                )?;
                let count = base.positions.len();
                let skin = WholeJoint::new(placement.head_joint);
                remove_skeletal_translation(
                    &mut delta.positions,
                    sample,
                    &self.reference_joints,
                    &vec![skin.indices; count],
                    &vec![skin.weights; count],
                );
                Ok(delta)
            })
            .collect()
    }
}

/// Eight skin influences carrying a vertex wholly with one joint.
pub(crate) struct WholeJoint {
    pub indices: [u32; 8],
    pub weights: [f32; 8],
}

impl WholeJoint {
    pub(crate) fn new(joint: u32) -> Self {
        let mut indices = [0; 8];
        indices[0] = joint;
        let mut weights = [0.0; 8];
        weights[0] = 1.0;
        Self { indices, weights }
    }
}
