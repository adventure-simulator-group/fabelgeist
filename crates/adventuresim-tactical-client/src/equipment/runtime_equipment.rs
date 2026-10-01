use super::*;
mod body;
mod sockets;
use body::try_build_runtime_body;

use adventuresim_character_creator::{
    bracer::ForearmMorphSample,
    design_input::{load_bracer_design, load_breastplate_design},
    runtime_equipment::RuntimeBody,
};
use anyhow::{Context, bail};
use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode, GltfSkin},
    mesh::{VertexAttributeValues, morph::MorphAttributes, skinning::SkinnedMeshInverseBindposes},
};
use fabelgeist_armor::GeneratedArmor;

mod morphs;
use morphs::runtime_body_morphs;

#[derive(Resource, Default)]
pub(super) struct RuntimeEquipmentBodyCache {
    pub(super) body: Option<RuntimeBody>,
    pub(super) inverse_bindposes: Option<Handle<SkinnedMeshInverseBindposes>>,
    pub(super) bracer_design: Option<fabelgeist_armor::BracerDesign>,
    pub(super) breastplate_design: Option<fabelgeist_armor::BreastplateDesign>,
    pub(super) failed: bool,
    models: HashMap<(String, String), CachedEquipment>,
}

#[expect(clippy::too_many_arguments)]
pub(super) fn prepare_runtime_equipment_body(
    animation: Res<crate::animation::AnimationRuntime>,
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_skins: Res<Assets<GltfSkin>>,
    meshes: Res<Assets<Mesh>>,
    inverse_bindposes: Res<Assets<SkinnedMeshInverseBindposes>>,
    mut cache: ResMut<RuntimeEquipmentBodyCache>,
) {
    if cache.failed {
        return;
    }

    if cache.body.is_none() {
        let Some(base_handle) = animation.requested_base() else {
            return;
        };
        let Some(body) = try_build_runtime_body(
            base_handle,
            &gltfs,
            &gltf_meshes,
            &gltf_nodes,
            &gltf_skins,
            &meshes,
            &inverse_bindposes,
        ) else {
            return;
        };

        match body {
            Ok((body, inverse_bindposes_handle)) => {
                cache.body = Some(body);
                cache.inverse_bindposes = Some(inverse_bindposes_handle);
            }
            Err(error) => {
                error!("failed to prepare the runtime armor body: {error:#}");
                cache.failed = true;
                return;
            }
        }
    }

    if cache.bracer_design.is_none() {
        match load_bracer_design(None) {
            Ok(design) => cache.bracer_design = Some(design),
            Err(error) => {
                error!("failed to load the runtime bracer design: {error:#}");
                cache.failed = true;
                return;
            }
        }
    }
    if cache.breastplate_design.is_none() {
        match load_breastplate_design(None) {
            Ok(design) => cache.breastplate_design = Some(design),
            Err(error) => {
                error!("failed to load the runtime breastplate design: {error:#}");
                cache.failed = true;
            }
        }
    }
}

#[derive(Clone)]
struct CachedEquipment {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    sockets: BTreeMap<String, Transform>,
}

pub(super) fn generate_runtime_equipment_models(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cache: ResMut<RuntimeEquipmentBodyCache>,
    pending: Query<(Entity, &RuntimeEquipmentPresentation), Without<ProceduralEquipmentResolved>>,
) {
    let mut generated_this_frame = false;
    for (entity, presentation) in &pending {
        if cache.failed {
            commands
                .entity(entity)
                .insert((ProceduralEquipmentFailed, ProceduralEquipmentResolved));
            continue;
        }
        if cache.body.is_none()
            || cache.bracer_design.is_none()
            || cache.breastplate_design.is_none()
        {
            return;
        }
        let key = (
            presentation.item_id.clone(),
            presentation.placement_id.clone(),
        );
        if !cache.models.contains_key(&key) {
            // Share fitted base meshes across wearers; their morph weights and
            // skeleton bindings remain per instance. Bound cold generation work.
            if generated_this_frame {
                continue;
            }
            generated_this_frame = true;
            match cache.generate(presentation, &mut meshes, &mut materials) {
                Ok(model) => {
                    cache.models.insert(key.clone(), model);
                }
                Err(error) => {
                    error!(item = %presentation.item_id, "failed to generate runtime equipment: {error:#}");
                    commands
                        .entity(entity)
                        .insert((ProceduralEquipmentFailed, ProceduralEquipmentResolved));
                    continue;
                }
            }
        }
        let model = &cache.models[&key];
        if !model.sockets.is_empty() {
            commands
                .entity(presentation.item)
                .insert(EquipmentAttachmentSockets(model.sockets.clone()));
        }
        let part = ProceduralEquipmentPart::new(
            presentation.item,
            cache
                .inverse_bindposes
                .as_ref()
                .expect("prepared bind poses")
                .clone(),
            cache
                .body
                .as_ref()
                .expect("prepared body")
                .joint_names
                .clone(),
        );
        commands.entity(entity).with_child(part.render_bundle(
            format!("Runtime equipment {}", presentation.item_id),
            model.mesh.clone(),
            model.material.clone(),
        ));
        commands
            .entity(entity)
            .insert((ProceduralEquipmentResolved, Visibility::Inherited));
    }
}

/// The presentation of an item generated on the armor device, if it is one.
pub(super) fn presentation(
    item: Entity,
    properties: Option<&ItemProperties>,
    topology: Option<&EquipmentTopology>,
) -> Option<RuntimeEquipmentPresentation> {
    let properties = properties.filter(|properties| {
        adventuresim_character_creator::runtime_equipment::is_runtime_equipment(&properties.id)
    })?;
    Some(RuntimeEquipmentPresentation {
        item,
        item_id: properties.id.clone(),
        placement_id: topology
            .and_then(|topology| topology.placement_id.clone())
            .unwrap_or_else(|| "worn".into()),
    })
}

impl RuntimeEquipmentBodyCache {
    fn generate(
        &self,
        presentation: &RuntimeEquipmentPresentation,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> anyhow::Result<CachedEquipment> {
        use adventuresim_character_creator::runtime_equipment as generator;
        let body = self.body.as_ref().context("runtime body not prepared")?;
        let generated = if generator::is_runtime_armor(&presentation.item_id) {
            generator::generate_runtime_armor(
                body,
                &presentation.item_id,
                &presentation.placement_id,
                self.bracer_design
                    .as_ref()
                    .context("bracer design missing")?,
                self.breastplate_design
                    .as_ref()
                    .context("breastplate design missing")?,
            )?
        } else {
            generator::generate_runtime_clothing(
                body,
                &presentation.item_id,
                &presentation.placement_id,
            )?
        };
        Ok(CachedEquipment {
            sockets: sockets::garment_sockets(body, &generated, &presentation.item_id),
            mesh: runtime_equipment_mesh(&generated, meshes),
            material: runtime_equipment_material(&presentation.item_id, materials),
        })
    }
}

fn runtime_equipment_mesh(armor: &GeneratedArmor, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, armor.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, armor.normals.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, armor.texcoords.clone())
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(
            armor
                .joint_indices
                .iter()
                .zip(armor.joint_weights.iter())
                .map(|(joint, weight)| strongest_four(*joint, *weight).0)
                .collect::<Vec<[u16; 4]>>(),
        ),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_WEIGHT,
        armor
            .joint_weights
            .iter()
            .zip(armor.joint_indices.iter())
            .map(|(weights, joints)| strongest_four(*joints, *weights).1)
            .collect::<Vec<[f32; 4]>>(),
    )
    .with_inserted_indices(Indices::U32(armor.indices.clone()));

    let morphs = armor
        .morphs
        .iter()
        .flat_map(|morph| {
            morph
                .position_deltas
                .iter()
                .zip(&morph.normal_deltas)
                .map(|(position, normal)| {
                    MorphAttributes::new(
                        Vec3::from_array(*position),
                        Vec3::from_array(*normal),
                        Vec3::ZERO,
                    )
                })
        })
        .collect::<Vec<_>>();
    let names = armor
        .morphs
        .iter()
        .map(|morph| morph.name.clone())
        .collect::<Vec<_>>();
    if !morphs.is_empty() {
        mesh.set_morph_targets(morphs);
        mesh.set_morph_target_names(names);
    }
    meshes.add(mesh)
}

fn runtime_equipment_material(
    item_id: &str,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    let material =
        adventuresim_character_creator::runtime_equipment::runtime_equipment_material(item_id)
            .unwrap_or_else(|| panic!("runtime armor material missing for {item_id}"));
    let (base_color, metallic, roughness) = adventuresim_character_creator::equipment_pbr(material);
    materials.add(StandardMaterial {
        base_color: Color::srgba(base_color[0], base_color[1], base_color[2], base_color[3]),
        metallic,
        perceptual_roughness: roughness,
        ..default()
    })
}

fn strongest_four(joints: [u32; 8], weights: [f32; 8]) -> ([u16; 4], [f32; 4]) {
    let mut combined = std::collections::BTreeMap::<u32, f32>::new();
    for (joint, weight) in joints.into_iter().zip(weights) {
        *combined.entry(joint).or_default() += weight;
    }
    let mut influences = combined.into_iter().collect::<Vec<_>>();
    influences.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    influences.truncate(4);
    let sum = influences.iter().map(|(_, weight)| *weight).sum::<f32>();
    let mut result_joints = [0; 4];
    let mut result_weights = [0.0; 4];
    for (index, (joint, weight)) in influences.into_iter().enumerate() {
        result_joints[index] = joint as u16;
        result_weights[index] = weight / sum;
    }
    (result_joints, result_weights)
}
