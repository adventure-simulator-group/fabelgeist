//! Fit recipe meshes and transfer skinning with stable morph correspondence.

use super::*;
use adventuresim_armor_model::ArmorMorph;
use adventuresim_character_creator::{
    armor_frames::Wearer,
    armor_recipes::{self, ParametricDesign},
    nearest_vertex::NearestVertices,
};

#[path = "cop_skin.rs"]
mod cop_skin;
#[path = "helmet_skin.rs"]
mod helmet_skin;
#[path = "limb_plate_skin.rs"]
mod limb_plate_skin;

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
    let needs_support = matches!(
        design,
        ParametricDesign::Limb(adventuresim_armor_model::LimbArmorDesign::Pauldron(_))
            | ParametricDesign::WaistAssembly(_)
            | ParametricDesign::Helmet(adventuresim_armor_model::HelmetDesign::CloseHelmet(_))
    ) || matches!(design, ParametricDesign::Limb(adventuresim_armor_model::LimbArmorDesign::Spaulder(d)) if d.besagew.is_some())
        || matches!(design, ParametricDesign::Garment(d) if d.kind == adventuresim_armor_model::GarmentArmorKind::Fauld);
    let support = needs_support
        .then(|| {
            crate::torso_support::TorsoSupport::new(model, generated, catalog, breastplate, morphs)
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
    let normals = mesh.normals().context("reference armor plate normals")?;
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
        let endpoint_normals = endpoint.normals().with_context(|| {
            format!("armor plate normals at morph {} ({placement})", sample.name)
        })?;
        targets.push(ArmorMorph {
            name: sample.name.clone(),
            position_deltas: deltas(&mesh.positions, &endpoint.positions),
            normal_deltas: deltas(&normals, &endpoint_normals),
            direct_positions: endpoint.positions,
        });
    }
    let sheets = mesh.shell_vertex_ranges().collect::<Vec<_>>();
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
    attach_plates(model, generated, design, placement, &sheets, &mut armor)?;
    attach_besagews(model, &mut armor)?;
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
}

fn attach_besagews(model: &BodyModel, armor: &mut GeneratedArmor) -> Result<()> {
    let joint = model
        .mhr
        .character
        .skeleton
        .names
        .iter()
        .position(|n| n == "c_spine3")
        .context("missing besagew suspension joint")? as u32;
    for component in &armor.components {
        if component.role == adventuresim_armor_model::ArmorComponentRole::Besagew {
            armor.joint_indices[component.vertices.clone()].fill([joint; 8]);
            armor.joint_weights[component.vertices.clone()]
                .fill([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        }
    }
    Ok(())
}

/// Plate attachments exclude unrelated elbow and skin-twist translations.
fn attach_plates(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &ParametricDesign,
    placement: &str,
    sheets: &[std::ops::Range<usize>],
    armor: &mut GeneratedArmor,
) -> Result<()> {
    match design {
        ParametricDesign::WaistAssembly(_) => {
            crate::waist_skin::attach(model, generated, sheets, armor)
        }
        ParametricDesign::Garment(garment)
            if garment.kind == adventuresim_armor_model::GarmentArmorKind::Fauld =>
        {
            crate::waist_skin::attach(model, generated, sheets, armor)
        }
        ParametricDesign::Helmet(helmet) => {
            helmet_skin::attach(helmet, &model.mhr.character.skeleton.names, armor)
        }
        ParametricDesign::Limb(adventuresim_armor_model::LimbArmorDesign::Couter(_)) => {
            cop_skin::attach(
                cop_skin::CopJoint::Elbow,
                placement,
                &model.mhr.character.skeleton.names,
                armor,
            )
        }
        ParametricDesign::Limb(adventuresim_armor_model::LimbArmorDesign::Poleyn(_)) => {
            cop_skin::attach(
                cop_skin::CopJoint::Knee,
                placement,
                &model.mhr.character.skeleton.names,
                armor,
            )
        }
        ParametricDesign::Limb(adventuresim_armor_model::LimbArmorDesign::Rerebrace(_)) => {
            limb_plate_skin::attach(
                limb_plate_skin::LimbPlate::UpperArm,
                placement,
                &model.mhr.character.skeleton.names,
                armor,
            )
        }
        ParametricDesign::Limb(adventuresim_armor_model::LimbArmorDesign::Cuisse(_)) => {
            limb_plate_skin::attach(
                limb_plate_skin::LimbPlate::Thigh,
                placement,
                &model.mhr.character.skeleton.names,
                armor,
            )
        }
        ParametricDesign::Limb(
            adventuresim_armor_model::LimbArmorDesign::Pauldron(_)
            | adventuresim_armor_model::LimbArmorDesign::Spaulder(_),
        ) => crate::shoulder_skin::attach(model, generated, placement, armor),
        _ => Ok(()),
    }
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
        mesh.shell_vertex_ranges()
            .eq(endpoint.shell_vertex_ranges()),
        "armor fit changed physical sheet correspondence"
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
    let source = NearestVertices::new(&generated.positions);
    let nearest = positions
        .iter()
        .map(|point| source.nearest(*point))
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

fn deltas(base: &[[f32; 3]], sample: &[[f32; 3]]) -> Vec<[f32; 3]> {
    base.iter()
        .zip(sample)
        .map(|(a, b)| std::array::from_fn(|i| b[i] - a[i]))
        .collect()
}

pub(super) struct SelectedArmor {
    pub item_id: String,
    pub placement_id: String,
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
                fitted_bracer(model, generated, bracer_design, side, morphs)
                    .with_context(|| format!("fitting {id} {}", selection.placement_id))?
            }
            "breastplate" | "cuirass" => {
                fitted_breastplate(model, generated, breastplate_design, morphs)
                    .with_context(|| format!("fitting {id} {}", selection.placement_id))?
            }
            _ => {
                let Some(design) = catalog.design(id, &selection.placement_id) else {
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
                )
                .with_context(|| format!("fitting {id} {}", selection.placement_id))?
            }
        };
        let piece = crate::fastener_equipment::attach(
            model,
            generated,
            morphs,
            catalog,
            id,
            &selection.placement_id,
            piece,
        )
        .with_context(|| format!("attaching {id} {} fasteners", selection.placement_id))?;
        pieces.push(SelectedArmor {
            item_id: id.clone(),
            placement_id: selection.placement_id.clone(),
            name: format!("{id}--{}", selection.placement_id),
            generated: piece,
        });
    }
    Ok(pieces)
}
