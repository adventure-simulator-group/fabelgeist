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
    catalog: &EquipmentCatalog,
    breastplate: &BreastplateDesign,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    if let ParametricDesign::Underlayer(d) = design {
        return crate::underlayer_equipment::fitted(model, generated, d, placement, morphs);
    }
    let character = &model.mhr.character;
    let wearer = |positions, normals, joints| Wearer {
        faces: &character.mesh.faces,
        positions,
        normals,
        joints,
        joint_indices: &character.skin_weights.index,
        joint_weights: &character.skin_weights.weight,
        joint_names: &character.skeleton.names,
    };
    let support = matches!(
        design,
        ParametricDesign::Limb(adventuresim_armor_model::LimbArmorDesign::Pauldron(_))
    )
    .then(|| {
        crate::pauldron_support::PauldronSupport::new(
            model,
            generated,
            catalog,
            breastplate,
            morphs,
        )
    })
    .transpose()?;
    let fitted = |body: &Wearer<'_>, target: Option<&str>| {
        if let Some(support) = &support {
            support.mesh(design, placement, body, target)
        } else {
            armor_recipes::fitted_mesh(design, placement, body, &[])
        }
    };
    let mesh = fitted(
        &wearer(
            &generated.positions,
            &generated.normals,
            &generated.global_joint_states,
        ),
        None,
    )?;
    let normals = mesh.normals()?;
    let (nearest, uv) = source_correspondence(model, generated, &mesh.positions);
    let mut targets = Vec::new();
    for sample in morphs {
        let endpoint = fitted(
            &wearer(
                &sample.positions,
                &sample.normals,
                &sample.global_joint_states,
            ),
            Some(&sample.name),
        )
        .with_context(|| format!("fitting armor morph {} ({placement})", sample.name))?;
        validate_correspondence(&mesh, &endpoint)?;
        let endpoint_normals = endpoint.normals()?;
        targets.push(ArmorMorph {
            name: sample.name.clone(),
            position_deltas: deltas(&mesh.positions, &endpoint.positions),
            normal_deltas: deltas(&normals, &endpoint_normals),
            direct_positions: endpoint.positions,
        });
    }
    let bytes = serde_json::to_vec(design)?;
    let mut armor = GeneratedArmor {
        plate_edges: mesh.plate_edges(),
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
        components: mesh.components,
    };
    attach_plates(model, generated, design, placement, &mut armor)?;
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
}

/// Plate attachments exclude unrelated elbow and skin-twist translations.
fn attach_plates(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &ParametricDesign,
    placement: &str,
    armor: &mut GeneratedArmor,
) -> Result<()> {
    let anchor = match design {
        ParametricDesign::Helmet(adventuresim_armor_model::HelmetDesign::CloseHelmet(_)) => {
            "c_head"
        }
        ParametricDesign::Limb(adventuresim_armor_model::LimbArmorDesign::Pauldron(_)) => {
            return crate::pauldron_skin::attach(model, generated, placement, armor);
        }
        _ => return Ok(()),
    };
    let joint = model
        .mhr
        .character
        .skeleton
        .names
        .iter()
        .position(|name| name == anchor)
        .context("missing rigid armor attachment joint")? as u32;
    armor.joint_indices.fill([joint; 8]);
    armor
        .joint_weights
        .fill([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    Ok(())
}

fn validate_correspondence(
    mesh: &adventuresim_armor_model::PartMesh,
    endpoint: &adventuresim_armor_model::PartMesh,
) -> Result<()> {
    anyhow::ensure!(
        endpoint.indices == mesh.indices && endpoint.positions.len() == mesh.positions.len(),
        "armor fit changed morph topology"
    );
    anyhow::ensure!(
        endpoint
            .components
            .iter()
            .map(|part| (&part.role, &part.vertices, &part.indices))
            .eq(mesh
                .components
                .iter()
                .map(|part| (&part.role, &part.vertices, &part.indices))),
        "armor fit changed component correspondence"
    );
    Ok(())
}

fn source_correspondence(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    positions: &[[f32; 3]],
) -> (Vec<usize>, Vec<[f32; 2]>) {
    let character = &model.mhr.character;
    let nearest = positions
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
    (nearest, uv)
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

pub(super) struct SelectedArmor {
    pub item_id: String,
    pub name: String,
    pub generated: GeneratedArmor,
}

pub(super) fn selected(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
    bracer_design: &BracerDesign,
    breastplate_design: &BreastplateDesign,
    morphs: &[ForearmMorphSample],
) -> Result<Vec<SelectedArmor>> {
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
                fitted_design(
                    model,
                    generated,
                    &design,
                    &selection.placement_id,
                    catalog,
                    breastplate_design,
                    morphs,
                )?
            }
        };
        pieces.push(SelectedArmor {
            item_id: id.clone(),
            name: format!("{id}--{}", selection.placement_id),
            generated: piece,
        });
    }
    Ok(pieces)
}
