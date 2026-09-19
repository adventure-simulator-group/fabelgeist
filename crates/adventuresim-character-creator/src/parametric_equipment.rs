//! Fit recipe meshes and transfer skinning with stable morph correspondence.

use super::*;
use adventuresim_armor_model::ArmorMorph;
use adventuresim_character_creator::{
    armor_frames::Wearer,
    armor_recipes::{self, ParametricDesign},
    nearest_vertex::NearestVertices,
};
use rayon::prelude::*;

#[path = "cop_skin.rs"]
mod cop_skin;
#[path = "helmet_skin.rs"]
mod helmet_skin;
#[path = "limb_plate_skin.rs"]
mod limb_plate_skin;

#[derive(Clone, Copy)]
pub(super) struct LayerSupport<'a> {
    current: &'a crate::equipment_layering::PlannedSelection,
    fitted: &'a [crate::equipment_layering::FittedLayer],
}

#[derive(Clone, Copy)]
pub(super) enum EquipmentFit<'a> {
    CharacterInstance,
    ReusableAsset(&'a [ForearmMorphSample]),
}

pub(super) fn fitted_design(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &ParametricDesign,
    placement: &str,
    morphs: &[ForearmMorphSample],
    underlayer_envelope: Option<&adventuresim_character_creator::underlayer::UnderlayerEnvelope>,
    layer_support: Option<LayerSupport<'_>>,
) -> Result<GeneratedArmor> {
    if let ParametricDesign::Underlayer(d) = design {
        return crate::underlayer_equipment::fitted(
            model,
            generated,
            d,
            placement,
            morphs,
            underlayer_envelope.context("underlayer fit envelope was not prepared")?,
        );
    }
    if let ParametricDesign::TrunkHose(d) = design {
        return crate::underlayer_equipment::fitted_trunk_hose(
            model,
            generated,
            d,
            placement,
            morphs,
            underlayer_envelope.context("trunk-hose fit envelope was not prepared")?,
        );
    }
    fitted_armor_design(model, generated, design, placement, morphs, layer_support)
}

fn fitted_armor_design(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &ParametricDesign,
    placement: &str,
    morphs: &[ForearmMorphSample],
    layer_support: Option<LayerSupport<'_>>,
) -> Result<GeneratedArmor> {
    let character = &model.mhr.character;
    let wearer = |positions, normals, joints| Wearer {
        detail: model.armor_detail,
        faces: &character.mesh.faces,
        positions,
        normals,
        joints,
        joint_indices: &character.skin_weights.index,
        joint_weights: &character.skin_weights.weight,
        joint_names: &character.skeleton.names,
    };
    let fitted = |body: &Wearer<'_>, target: Option<&str>| {
        let layers = layer_support
            .map(|support| {
                support
                    .fitted
                    .iter()
                    .filter_map(|layer| layer.supports(support.current, target).transpose())
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();
        armor_recipes::fitted_mesh(design, placement, body, &layers)
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
    let body_texcoords = nearest.iter().map(|i| uv[*i]).collect::<Vec<_>>();
    let (normal_map, texcoords) = mesh
        .runtime_fluting(&body_texcoords)?
        .map_or((None, body_texcoords), |(map, texcoords)| {
            (Some(map), texcoords)
        });
    let mut armor = GeneratedArmor {
        construction_faces: mesh.construction_face_ranges(),
        plate_edges: mesh.plate_edges(),
        design_hash: adventuresim_armor_model::parametric_design_hash(&bytes),
        surface_domain: MHR_ANATOMICAL_UV_DOMAIN.into(),
        positions: mesh.positions,
        normals,
        texcoords,
        normal_map,
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
        ParametricDesign::Limb(
            adventuresim_armor_model::LimbArmorDesign::Rerebrace(_)
            | adventuresim_armor_model::LimbArmorDesign::Spaulder(_),
        ) => limb_plate_skin::attach(
            limb_plate_skin::LimbPlate::UpperArm,
            placement,
            &model.mhr.character.skeleton.names,
            armor,
        ),
        ParametricDesign::Limb(adventuresim_armor_model::LimbArmorDesign::Cuisse(_)) => {
            limb_plate_skin::attach(
                limb_plate_skin::LimbPlate::Thigh,
                placement,
                &model.mhr.character.skeleton.names,
                armor,
            )
        }
        ParametricDesign::Limb(adventuresim_armor_model::LimbArmorDesign::Pauldron(_)) => {
            crate::shoulder_skin::attach(model, generated, placement, armor)
        }
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
    fit: EquipmentFit<'_>,
) -> Result<Vec<SelectedArmor>> {
    let morphs = match fit {
        EquipmentFit::CharacterInstance => &[],
        EquipmentFit::ReusableAsset(morphs) => morphs,
    };
    let selections = recipe
        .clothing
        .iter()
        .filter(|selection| {
            matches!(
                selection.item_id.as_str(),
                "vambrace" | "breastplate" | "cuirass"
            ) || catalog
                .design(&selection.item_id, &selection.placement_id)
                .is_some()
        })
        .collect::<Vec<_>>();
    let batches = crate::profiling::measure("equipment_layer_plan", || {
        crate::equipment_layering::plan_batches(&selections, catalog)
    })?;
    let needs_underlayer_envelope = batches.iter().flatten().any(|plan| {
        matches!(
            catalog.design(&plan.item_id, &plan.placement_id),
            Some(ParametricDesign::Underlayer(_) | ParametricDesign::TrunkHose(_))
        )
    });
    let underlayer_envelope = needs_underlayer_envelope.then(|| {
        crate::profiling::measure("underlayer_envelope", || match fit {
            EquipmentFit::CharacterInstance => {
                crate::underlayer_equipment::fit_instance_envelope(model, generated)
            }
            EquipmentFit::ReusableAsset(_) => {
                crate::underlayer_equipment::fit_envelope(model, generated, morphs)
            }
        })
    });
    let fitter = OutfitFitter {
        model,
        generated,
        catalog,
        bracer_design,
        breastplate_design,
        morphs,
        underlayer_envelope: underlayer_envelope.as_ref(),
    };
    let mut fitted_layers = Vec::new();
    for batch in batches {
        let fitted = batch
            .into_par_iter()
            .map(|plan| {
                let generated = fitter.fit(&plan, &fitted_layers)?;
                Ok(crate::equipment_layering::FittedLayer { plan, generated })
            })
            .collect::<Result<Vec<_>>>()?;
        fitted_layers.extend(fitted);
    }
    fitted_layers
        .into_par_iter()
        .map(|layer| finish_selected(model, generated, catalog, morphs, layer))
        .collect()
}

struct OutfitFitter<'a> {
    model: &'a BodyModel,
    generated: &'a GeneratedCharacter,
    catalog: &'a EquipmentCatalog,
    bracer_design: &'a BracerDesign,
    breastplate_design: &'a BreastplateDesign,
    morphs: &'a [ForearmMorphSample],
    underlayer_envelope: Option<&'a adventuresim_character_creator::underlayer::UnderlayerEnvelope>,
}

impl OutfitFitter<'_> {
    fn fit(
        &self,
        plan: &crate::equipment_layering::PlannedSelection,
        fitted: &[crate::equipment_layering::FittedLayer],
    ) -> Result<GeneratedArmor> {
        let id = &plan.item_id;
        let fit_label = format!("item_fit:{id}--{}", plan.placement_id);
        crate::profiling::measure(&fit_label, || {
            Ok(match id.as_str() {
                "vambrace" => {
                    let side = match plan.placement_id.as_str() {
                        "left" => ForearmSide::Left,
                        "right" => ForearmSide::Right,
                        _ => anyhow::bail!("invalid vambrace placement"),
                    };
                    fitted_bracer(
                        self.model,
                        self.generated,
                        self.bracer_design,
                        side,
                        self.morphs,
                    )?
                }
                "breastplate" | "cuirass" => fitted_breastplate(
                    self.model,
                    self.generated,
                    self.breastplate_design,
                    self.morphs,
                )?,
                _ => {
                    let design = self
                        .catalog
                        .design(id, &plan.placement_id)
                        .context("planned equipment lost its parametric design")?;
                    fitted_design(
                        self.model,
                        self.generated,
                        &design,
                        &plan.placement_id,
                        self.morphs,
                        self.underlayer_envelope,
                        Some(LayerSupport {
                            current: plan,
                            fitted,
                        }),
                    )?
                }
            })
        })
        .with_context(|| format!("fitting {id} {}", plan.placement_id))
    }
}

fn finish_selected(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    catalog: &EquipmentCatalog,
    morphs: &[ForearmMorphSample],
    layer: crate::equipment_layering::FittedLayer,
) -> Result<SelectedArmor> {
    let id = layer.plan.item_id;
    let placement_id = layer.plan.placement_id;
    let fastener_label = format!("fasteners:{id}--{placement_id}");
    let piece = crate::profiling::measure(&fastener_label, || {
        crate::fastener_equipment::attach(
            model,
            generated,
            morphs,
            catalog,
            &id,
            &placement_id,
            layer.generated,
        )
        .with_context(|| format!("attaching {id} {placement_id} fasteners"))
    })?;
    let topology_label = format!("render_topology:{id}--{placement_id}");
    let generated =
        crate::profiling::measure(&topology_label, || piece.for_rendering(model.armor_detail));
    Ok(SelectedArmor {
        name: format!("{id}--{placement_id}"),
        item_id: id,
        placement_id,
        generated,
    })
}
