//! Sample MHR identity and refit clothing with fixed vertex correspondence.

use super::*;
use adventuresim_character_creator::clothing::ClothingShell;
use adventuresim_core::character_morph::{IDENTITY_MORPH_STEP, IdentityMorph};
use adventuresim_core::character_proportions::BodyProportion;
use adventuresim_core::skeletal_fit::SkeletalFitMorph;

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
    reference_joints: Vec<[f32; 8]>,
    joint_indices: Vec<[u32; 8]>,
    joint_weights: Vec<[f32; 8]>,
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
        for target in SkeletalFitMorph::ALL {
            let mut sample_recipe = recipe.clone();
            sample_recipe
                .proportions
                .set(BodyProportion::SpineLength, target.endpoint())
                .map_err(anyhow::Error::msg)?;
            let sample = generate_character(model, &sample_recipe)?;
            body.push(MorphDelta {
                name: target.name().into(),
                positions: vec![[0.0; 3]; base.positions.len()],
                normals: vec![[0.0; 3]; base.normals.len()],
            });
            samples.push(ForearmMorphSample {
                name: target.name().into(),
                positions: sample.positions,
                normals: sample.normals,
                global_joint_states: sample.global_joint_states,
            });
        }
        Ok(Self {
            samples,
            body,
            reference_joints: base.global_joint_states.clone(),
            joint_indices: model.mhr.character.skin_weights.index.clone(),
            joint_weights: model.mhr.character.skin_weights.weight.clone(),
        })
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
                        let mut delta = MorphDelta::between(
                            sample.name.clone(),
                            &base.positions,
                            &base.normals,
                            &fitted.positions,
                            &fitted.normals,
                        )?;
                        remove_skeletal_translation(
                            &mut delta.positions,
                            sample,
                            &self.reference_joints,
                            &self.joint_indices,
                            &self.joint_weights,
                        );
                        Ok(delta)
                    })
                    .collect()
            })
            .collect()
    }
}

/// Fit targets contain only the displacement not already produced by the bones.
fn remove_skeletal_translation(
    deltas: &mut [[f32; 3]],
    sample: &ForearmMorphSample,
    reference: &[[f32; 8]],
    indices: &[[u32; 8]],
    weights: &[[f32; 8]],
) {
    if SkeletalFitMorph::from_name(&sample.name).is_none() {
        return;
    }
    for ((delta, indices), weights) in deltas.iter_mut().zip(indices).zip(weights) {
        for (&joint, &weight) in indices.iter().zip(weights) {
            if weight == 0.0 {
                continue;
            }
            for (axis, value) in delta.iter_mut().enumerate() {
                *value -= weight
                    * (sample.global_joint_states[joint as usize][axis]
                        - reference[joint as usize][axis]);
            }
        }
    }
}

pub(super) fn correct_armor_fit(
    mut armor: GeneratedArmor,
    reference: &GeneratedCharacter,
    samples: &[ForearmMorphSample],
) -> GeneratedArmor {
    for target in &mut armor.morphs {
        if let Some(sample) = samples.iter().find(|sample| sample.name == target.name) {
            remove_skeletal_translation(
                &mut target.position_deltas,
                sample,
                &reference.global_joint_states,
                &armor.joint_indices,
                &armor.joint_weights,
            );
        }
    }
    armor
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

/// Hardware has its own surface, regardless of the catalog's plate material.
pub(super) fn component_materials(armor: &GeneratedArmor, shells: &mut [RiggedShell<'_>]) {
    for (component, shell) in armor.components.iter().zip(shells) {
        if let Some(material) = component.material {
            shell.base_color = material.base_color;
            shell.metallic = material.metallic;
            shell.roughness = material.roughness;
            shell.textures = None;
            shell.plate_edges = &[];
        }
    }
}

pub(super) fn rigged_clothing<'a>(
    shell: &'a ClothingShell,
    targets: &'a [RiggedMorphTarget<'a>],
) -> RiggedShell<'a> {
    let specification = &shell.specification;
    RiggedShell {
        plate_edges: &[],
        textures: None,
        texcoords: None,
        hinge: None,
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
) -> Vec<RiggedShell<'a>> {
    let shell = |name, faces, hinge| RiggedShell {
        plate_edges: &armor.plate_edges,
        textures: None,
        texcoords: Some(&armor.texcoords),
        name,
        hinge,
        positions: &armor.positions,
        normals: &armor.normals,
        faces,
        joint_indices: Some(&armor.joint_indices),
        joint_weights: Some(&armor.joint_weights),
        morph_targets: targets,
        base_color: [0.769, 0.776, 0.776, 1.0],
        metallic: 1.0,
        roughness: 0.20,
    };
    if armor.components.is_empty() {
        vec![shell(name, faces, None)]
    } else {
        armor
            .components
            .iter()
            .map(|component| {
                shell(
                    component.role.name(),
                    &faces[component.indices.start / 3..component.indices.end / 3],
                    component.hinge,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeletal_fit_corrects_surface_without_counting_bone_translation_twice() {
        let sample = ForearmMorphSample {
            name: SkeletalFitMorph::LongSpine.name().into(),
            positions: vec![],
            normals: vec![],
            global_joint_states: vec![[0.0, 0.1, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0]],
        };
        let mut deltas = [[0.0, 0.12, 0.0]];
        remove_skeletal_translation(
            &mut deltas,
            &sample,
            &[[0.0; 8]],
            &[[0; 8]],
            &[[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]],
        );
        assert!((deltas[0][1] - 0.02).abs() < 1e-6);
        assert!((deltas[0][1] + 0.1 - 0.12).abs() < 1e-6);
    }
}
