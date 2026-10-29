//! Hair placed again on every sampled head, for the body's morph channels.
use super::*;
use adventuresim_character_creator::hair::{GroomedHair, HairGeometry, HeadPlacement, HeadSkin};
use fabelgeist_mhr::character::MeshVertexIndex;

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
                    joint_indices: character.skin_weights.indices(),
                    joint_weights: character.skin_weights.weights().as_native_slice(),
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
                let skin = fabelgeist_rig::RigInfluences::<8>::whole_joint(placement.head_joint);
                let joints = fabelgeist_rig::PackedRigSkin::repeated(
                    skin.joints,
                    count,
                    self.reference_joints.joint_count(),
                )?;
                remove_skeletal_translation(
                    &mut delta.positions,
                    sample,
                    &self.reference_joints,
                    &joints,
                    (0..count).map(|vertex| {
                        (
                            MeshVertexIndex::from_native_vertex_index(vertex),
                            skin.weights,
                        )
                    }),
                );
                Ok(delta)
            })
            .collect()
    }
}
