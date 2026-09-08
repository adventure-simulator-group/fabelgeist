//! Sample MHR identity and refit clothing with fixed vertex correspondence.

use super::*;
use adventuresim_character_creator::clothing::ClothingShell;
use adventuresim_core::character_morph::{IDENTITY_MORPH_STEP, IdentityMorph};

pub(super) struct MorphDelta {
    name: String,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
}

impl MorphDelta {
    fn between(
        name: String,
        base_positions: &[[f32; 3]],
        base_normals: &[[f32; 3]],
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
    ) -> Result<Self> {
        anyhow::ensure!(
            base_positions.len() == positions.len() && base_normals.len() == normals.len(),
            "morph sample changed vertex correspondence"
        );
        let subtract = |base: &[[f32; 3]], sample: &[[f32; 3]]| {
            base.iter()
                .zip(sample)
                .map(|(base, sample)| std::array::from_fn(|axis| sample[axis] - base[axis]))
                .collect()
        };
        Ok(Self {
            name,
            positions: subtract(base_positions, positions),
            normals: subtract(base_normals, normals),
        })
    }

    pub(super) fn rigged(&self) -> RiggedMorphTarget<'_> {
        RiggedMorphTarget {
            name: &self.name,
            position_deltas: &self.positions,
            normal_deltas: &self.normals,
        }
    }
}

pub(super) struct CharacterMorphs {
    pub samples: Vec<ForearmMorphSample>,
    pub body: Vec<MorphDelta>,
}

impl CharacterMorphs {
    pub(super) fn generate(
        model: &BodyModel,
        recipe: &CharacterRecipe,
        base: &GeneratedCharacter,
    ) -> Result<Self> {
        let mut samples = Vec::new();
        let mut body = Vec::new();
        for target in IdentityMorph::all() {
            let mut sample_recipe = recipe.clone();
            sample_recipe.identity[target.index()] += IDENTITY_MORPH_STEP;
            let sample = generate_character(model, &sample_recipe)?;
            body.push(MorphDelta::between(
                target.name(),
                &base.positions,
                &base.normals,
                &sample.positions,
                &sample.normals,
            )?);
            samples.push(ForearmMorphSample {
                name: target.name(),
                positions: sample.positions,
                normals: sample.normals,
                global_joint_states: sample.global_joint_states,
            });
        }
        Ok(Self { samples, body })
    }

    pub(super) fn clothing(&self, shells: &[ClothingShell]) -> Result<Vec<Vec<MorphDelta>>> {
        shells
            .iter()
            .map(|base| {
                self.samples
                    .iter()
                    .map(|sample| {
                        let fitted = base
                            .refit(&sample.positions, &sample.normals)
                            .map_err(anyhow::Error::msg)?;
                        MorphDelta::between(
                            sample.name.clone(),
                            &base.positions,
                            &base.normals,
                            &fitted.positions,
                            &fitted.normals,
                        )
                    })
                    .collect()
            })
            .collect()
    }
}

pub(super) fn armor_targets(armor: &GeneratedArmor) -> Vec<RiggedMorphTarget<'_>> {
    armor
        .morphs
        .iter()
        .map(|target| RiggedMorphTarget {
            name: &target.name,
            position_deltas: &target.position_deltas,
            normal_deltas: &target.normal_deltas,
        })
        .collect()
}

pub(super) fn rigged_clothing<'a>(
    shell: &'a ClothingShell,
    targets: &'a [RiggedMorphTarget<'a>],
) -> RiggedShell<'a> {
    let specification = &shell.specification;
    RiggedShell {
        name: &specification.name,
        positions: &shell.positions,
        normals: &shell.normals,
        faces: &shell.faces,
        joint_indices: None,
        joint_weights: None,
        morph_targets: targets,
        base_color: specification.base_color,
        metallic: specification.metallic,
        roughness: specification.roughness,
    }
}

pub(super) fn rigged_armor<'a>(
    name: &'a str,
    armor: &'a GeneratedArmor,
    faces: &'a [[u32; 3]],
    targets: &'a [RiggedMorphTarget<'a>],
) -> RiggedShell<'a> {
    RiggedShell {
        name,
        positions: &armor.positions,
        normals: &armor.normals,
        faces,
        joint_indices: Some(&armor.joint_indices),
        joint_weights: Some(&armor.joint_weights),
        morph_targets: targets,
        base_color: [0.769, 0.776, 0.776, 1.0],
        metallic: 1.0,
        roughness: 0.20,
    }
}
