use super::*;

use adventuresim_armor_model::GeneratedArmor;
use adventuresim_character_creator::{
    design_input::{load_bracer_design, load_breastplate_design},
    runtime_equipment::{RuntimeBody, RuntimeBodyMorph},
};
use anyhow::{Context, bail};
use bevy::{
    gltf::{Gltf, GltfMesh, GltfNode, GltfSkin},
    mesh::{VertexAttributeValues, morph::MorphAttributes, skinning::SkinnedMeshInverseBindposes},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

#[derive(Component)]
pub(super) struct RuntimeArmorPresentation {
    pub(super) item: Entity,
    pub(super) item_id: String,
    pub(super) placement_id: String,
}

#[derive(Resource, Default)]
pub(super) struct RuntimeArmorBodyCache {
    pub(super) body: Option<RuntimeBody>,
    pub(super) inverse_bindposes: Option<Handle<SkinnedMeshInverseBindposes>>,
    pub(super) bracer_design: Option<adventuresim_armor_model::BracerDesign>,
    pub(super) breastplate_design: Option<adventuresim_armor_model::BreastplateDesign>,
    pub(super) failed: bool,
}

#[expect(clippy::too_many_arguments)]
pub(super) fn generate_runtime_armor_models(
    mut commands: Commands,
    animation: Res<crate::animation::AnimationRuntime>,
    gltfs: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_skins: Res<Assets<GltfSkin>>,
    mut meshes: ResMut<Assets<Mesh>>,
    inverse_bindposes: Res<Assets<SkinnedMeshInverseBindposes>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cache: ResMut<RuntimeArmorBodyCache>,
    pending: Query<(Entity, &RuntimeArmorPresentation), Without<ProceduralEquipmentResolved>>,
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
                return;
            }
        }
    }

    let Some(body) = cache.body.as_ref() else {
        return;
    };
    let Some(inverse_bindposes_handle) = cache.inverse_bindposes.as_ref() else {
        return;
    };
    let Some(bracer_design) = cache.bracer_design.as_ref() else {
        return;
    };
    let Some(breastplate_design) = cache.breastplate_design.as_ref() else {
        return;
    };

    for (entity, presentation) in &pending {
        let generated =
            match adventuresim_character_creator::runtime_equipment::generate_runtime_armor(
                body,
                &presentation.item_id,
                &presentation.placement_id,
                bracer_design,
                breastplate_design,
            ) {
                Ok(generated) => generated,
                Err(error) => {
                    error!(
                        item = %presentation.item_id,
                        placement = %presentation.placement_id,
                        "failed to generate runtime armor: {error:#}"
                    );
                    commands.entity(entity).insert(ProceduralEquipmentResolved);
                    continue;
                }
            };

        let mesh = runtime_armor_mesh(&generated, &mut meshes);
        let material = runtime_armor_material(
            &presentation.item_id,
            &generated,
            &mut images,
            &mut materials,
        );
        let part = ProceduralEquipmentPart::new(
            presentation.item,
            inverse_bindposes_handle.clone(),
            body.joint_names.clone(),
        );
        commands.entity(entity).with_children(|commands| {
            commands.spawn(part.render_bundle(
                format!("Runtime armor {}", presentation.item_id),
                mesh,
                material,
            ));
        });
        commands
            .entity(entity)
            .insert((ProceduralEquipmentResolved, Visibility::Inherited));
    }
}

fn try_build_runtime_body(
    base_handle: &Handle<Gltf>,
    gltfs: &Assets<Gltf>,
    gltf_meshes: &Assets<GltfMesh>,
    gltf_nodes: &Assets<GltfNode>,
    gltf_skins: &Assets<GltfSkin>,
    meshes: &Assets<Mesh>,
    inverse_bindposes: &Assets<SkinnedMeshInverseBindposes>,
) -> Option<anyhow::Result<(RuntimeBody, Handle<SkinnedMeshInverseBindposes>)>> {
    let gltf = gltfs.get(base_handle)?;

    for node_handle in &gltf.nodes {
        let Some(node) = gltf_nodes.get(node_handle) else {
            return None;
        };
        let (Some(mesh_handle), Some(skin_handle)) = (node.mesh.as_ref(), node.skin.as_ref())
        else {
            continue;
        };
        let Some(gltf_mesh) = gltf_meshes.get(mesh_handle) else {
            return None;
        };
        let Some(skin) = gltf_skins.get(skin_handle) else {
            return None;
        };
        let Some(inverse_bindposes_asset) = inverse_bindposes.get(&skin.inverse_bind_matrices)
        else {
            return None;
        };

        for primitive in &gltf_mesh.primitives {
            let Some(mesh) = meshes.get(&primitive.mesh) else {
                return None;
            };
            return Some(build_runtime_body(
                mesh,
                skin,
                inverse_bindposes_asset,
                gltf_nodes,
            ));
        }
    }

    Some(Err(anyhow::anyhow!(
        "the base rig contains no skinned mesh primitive"
    )))
}

fn build_runtime_body(
    mesh: &Mesh,
    skin: &GltfSkin,
    inverse_bindposes: &SkinnedMeshInverseBindposes,
    gltf_nodes: &Assets<GltfNode>,
) -> anyhow::Result<(RuntimeBody, Handle<SkinnedMeshInverseBindposes>)> {
    let positions = attribute_vec3(mesh, Mesh::ATTRIBUTE_POSITION, "positions")?;
    let normals = attribute_vec3(mesh, Mesh::ATTRIBUTE_NORMAL, "normals")?;
    let texcoords = attribute_vec2(mesh, Mesh::ATTRIBUTE_UV_0, "UVs")?;
    let joint_indices = attribute_u16x4(mesh, Mesh::ATTRIBUTE_JOINT_INDEX, "joint indices")?;
    let joint_weights = attribute_f32x4(mesh, Mesh::ATTRIBUTE_JOINT_WEIGHT, "joint weights")?;
    let indices = mesh
        .indices()
        .context("runtime armor body has no index buffer")?;
    let indices = match indices {
        Indices::U16(indices) => indices.iter().map(|index| u32::from(*index)).collect(),
        Indices::U32(indices) => indices.clone(),
    };
    let faces = indices
        .chunks_exact(3)
        .map(|face| [face[0], face[1], face[2]])
        .collect::<Vec<_>>();
    if faces.len() * 3 != indices.len() {
        bail!("runtime armor body index buffer is not triangle-aligned");
    }

    let joint_names = skin
        .joints
        .iter()
        .map(|joint| {
            gltf_nodes
                .get(joint)
                .map(|node| node.name.clone())
                .unwrap_or_else(|| "joint".into())
        })
        .collect::<Vec<_>>();
    let global_joint_states = inverse_bindposes
        .iter()
        .map(|inverse_bindpose| {
            let (scale, rotation, position) =
                inverse_bindpose.inverse().to_scale_rotation_translation();
            [
                position.x, position.y, position.z, rotation.x, rotation.y, rotation.z, rotation.w,
                scale.x,
            ]
        })
        .collect::<Vec<_>>();

    let morphs = mesh
        .get_morph_targets()
        .map(|targets| {
            let names = mesh.morph_target_names().unwrap_or_default();
            let vertex_count = positions.len();
            names
                .iter()
                .enumerate()
                .filter_map(|(index, name)| {
                    let start = index.checked_mul(vertex_count)?;
                    let target = targets.get(start..start + vertex_count)?;
                    Some(RuntimeBodyMorph {
                        name: name.clone(),
                        positions: positions
                            .iter()
                            .zip(target)
                            .map(|(base, delta)| {
                                let delta = delta.position.to_array();
                                [base[0] + delta[0], base[1] + delta[1], base[2] + delta[2]]
                            })
                            .collect(),
                        normals: normals
                            .iter()
                            .zip(target)
                            .map(|(base, delta)| {
                                let delta = delta.normal.to_array();
                                [base[0] + delta[0], base[1] + delta[1], base[2] + delta[2]]
                            })
                            .collect(),
                        global_joint_states: global_joint_states.clone(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let body = RuntimeBody {
        detail: adventuresim_armor_model::ArmorDetail::Runtime(
            adventuresim_armor_model::ArmorLod::Lod5,
        ),
        domain: "mhr_body_v1".into(),
        faces: faces.clone(),
        positions,
        normals,
        texcoords,
        texcoord_faces: faces,
        joint_indices: joint_indices
            .into_iter()
            .map(|joint| {
                [
                    u32::from(joint[0]),
                    u32::from(joint[1]),
                    u32::from(joint[2]),
                    u32::from(joint[3]),
                    0,
                    0,
                    0,
                    0,
                ]
            })
            .collect(),
        joint_weights: joint_weights
            .into_iter()
            .map(|weight| {
                [
                    weight[0], weight[1], weight[2], weight[3], 0.0, 0.0, 0.0, 0.0,
                ]
            })
            .collect(),
        joint_names,
        global_joint_states,
        morphs,
    };
    let inverse_bindposes_handle = skin.inverse_bind_matrices.clone();
    Ok((body, inverse_bindposes_handle))
}

fn attribute_vec3(
    mesh: &Mesh,
    attribute: bevy::mesh::MeshVertexAttribute,
    label: &str,
) -> anyhow::Result<Vec<[f32; 3]>> {
    match mesh.attribute(attribute).context(label.to_owned())? {
        VertexAttributeValues::Float32x3(values) => Ok(values.clone()),
        _ => bail!("runtime armor body {label} have an unsupported vertex format"),
    }
}

fn attribute_vec2(
    mesh: &Mesh,
    attribute: bevy::mesh::MeshVertexAttribute,
    label: &str,
) -> anyhow::Result<Vec<[f32; 2]>> {
    match mesh.attribute(attribute).context(label.to_owned())? {
        VertexAttributeValues::Float32x2(values) => Ok(values.clone()),
        _ => bail!("runtime armor body {label} have an unsupported vertex format"),
    }
}

fn attribute_u16x4(
    mesh: &Mesh,
    attribute: bevy::mesh::MeshVertexAttribute,
    label: &str,
) -> anyhow::Result<Vec<[u16; 4]>> {
    match mesh.attribute(attribute).context(label.to_owned())? {
        VertexAttributeValues::Uint16x4(values) => Ok(values.clone()),
        _ => bail!("runtime armor body {label} have an unsupported vertex format"),
    }
}

fn attribute_f32x4(
    mesh: &Mesh,
    attribute: bevy::mesh::MeshVertexAttribute,
    label: &str,
) -> anyhow::Result<Vec<[f32; 4]>> {
    match mesh.attribute(attribute).context(label.to_owned())? {
        VertexAttributeValues::Float32x4(values) => Ok(values.clone()),
        _ => bail!("runtime armor body {label} have an unsupported vertex format"),
    }
}

fn runtime_armor_mesh(armor: &GeneratedArmor, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
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

fn runtime_armor_material(
    item_id: &str,
    armor: &GeneratedArmor,
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    let material =
        adventuresim_character_creator::runtime_equipment::runtime_armor_material(item_id)
            .unwrap_or_else(|| panic!("runtime armor material missing for {item_id}"));
    let (base_color, metallic, roughness) = adventuresim_character_creator::equipment_pbr(material);
    let normal_map = armor.normal_map.as_ref().map(|normal_map| {
        images.add(Image::new(
            Extent3d {
                width: normal_map.width,
                height: normal_map.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            normal_map.rgba8.clone(),
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        ))
    });
    materials.add(StandardMaterial {
        base_color: Color::srgba(base_color[0], base_color[1], base_color[2], base_color[3]),
        metallic,
        perceptual_roughness: roughness,
        normal_map_texture: normal_map,
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
