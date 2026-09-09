//! Fit recipe meshes and transfer skinning with stable morph correspondence.

use super::*;
use adventuresim_armor_model::ArmorMorph;
use adventuresim_character_creator::{
    armor_frames::Wearer,
    armor_recipes::{self, ParametricDesign},
};

pub(super) fn fitted_design(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &ParametricDesign,
    placement: &str,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let character = &model.mhr.character;
    let wearer = |positions, normals, joints| Wearer {
        positions,
        normals,
        joints,
        joint_indices: &character.skin_weights.index,
        joint_weights: &character.skin_weights.weight,
        joint_names: &character.skeleton.names,
    };
    let mesh = armor_recipes::fitted_mesh(
        design,
        placement,
        &wearer(
            &generated.positions,
            &generated.normals,
            &generated.global_joint_states,
        ),
    )?;
    let normals = mesh.normals()?;
    let nearest = mesh
        .positions
        .iter()
        .map(|point| {
            generated
                .positions
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    squared_distance(*point, **a).total_cmp(&squared_distance(*point, **b))
                })
                .map(|(i, _)| i)
                .expect("validated wearer contains vertices")
        })
        .collect::<Vec<_>>();
    let mut uv = vec![[0.0; 2]; generated.positions.len()];
    for (face, uv_face) in character
        .mesh
        .faces
        .iter()
        .zip(&character.mesh.texcoord_faces)
    {
        for corner in 0..3 {
            uv[face[corner] as usize] = character.mesh.texcoords[uv_face[corner] as usize];
        }
    }
    let mut targets = Vec::new();
    for sample in morphs {
        let endpoint = armor_recipes::fitted_mesh(
            design,
            placement,
            &wearer(
                &sample.positions,
                &sample.normals,
                &sample.global_joint_states,
            ),
        )?;
        anyhow::ensure!(
            endpoint.indices == mesh.indices && endpoint.positions.len() == mesh.positions.len(),
            "armor fit changed morph topology"
        );
        let endpoint_normals = endpoint.normals()?;
        targets.push(ArmorMorph {
            name: sample.name.clone(),
            position_deltas: deltas(&mesh.positions, &endpoint.positions),
            normal_deltas: deltas(&normals, &endpoint_normals),
            direct_positions: endpoint.positions,
        });
    }
    let bytes = serde_json::to_vec(design)?;
    let armor = GeneratedArmor {
        design_hash: adventuresim_armor_model::parametric_design_hash(&bytes),
        surface_domain: MHR_ANATOMICAL_UV_DOMAIN.into(),
        positions: mesh.positions,
        normals,
        texcoords: nearest.iter().map(|i| uv[*i]).collect(),
        joint_indices: nearest
            .iter()
            .map(|i| character.skin_weights.index[*i])
            .collect(),
        joint_weights: nearest
            .iter()
            .map(|i| character.skin_weights.weight[*i])
            .collect(),
        indices: mesh.indices,
        morphs: targets,
    };
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
}

fn squared_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| (a[i] - b[i]).powi(2)).sum()
}
fn deltas(base: &[[f32; 3]], sample: &[[f32; 3]]) -> Vec<[f32; 3]> {
    base.iter()
        .zip(sample)
        .map(|(a, b)| std::array::from_fn(|i| b[i] - a[i]))
        .collect()
}

pub(super) fn selected(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
    bracer_design: &BracerDesign,
    breastplate_design: &BreastplateDesign,
    morphs: &[ForearmMorphSample],
) -> Result<Vec<(String, GeneratedArmor)>> {
    let mut pieces = Vec::new();
    for selection in &recipe.clothing {
        let id = &selection.item_id;
        let piece = match id.as_str() {
            "vambrace" => {
                let side = match selection.placement_id.as_str() {
                    "left" => ForearmSide::Left,
                    "right" => ForearmSide::Right,
                    _ => anyhow::bail!("invalid vambrace placement"),
                };
                fitted_bracer(model, generated, bracer_design, side, morphs)?
            }
            "breastplate" | "cuirass" => {
                fitted_breastplate(model, generated, breastplate_design, morphs)?
            }
            _ => {
                let Some(design) = catalog.design(id) else {
                    continue;
                };
                fitted_design(model, generated, &design, &selection.placement_id, morphs)?
            }
        };
        pieces.push((id.clone(), piece));
    }
    Ok(pieces)
}
