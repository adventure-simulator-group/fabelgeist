//! Adapt the loaded glTF body mesh to the runtime armor generator's input.

use super::*;
use adventuresim_character_creator::runtime_equipment::{RuntimeBody, RuntimeBodyMorph};
use anyhow::{Context, bail};

pub(super) fn try_build_runtime_body(
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
    let faces = runtime_body_faces(mesh)?;
    let joint_names = runtime_body_joint_names(skin, gltf_nodes);
    let global_joint_states = runtime_body_joint_states(inverse_bindposes);
    let morphs = runtime_body_morphs(mesh, &positions, &normals, &global_joint_states);
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
    Ok((body, skin.inverse_bind_matrices.clone()))
}

fn runtime_body_faces(mesh: &Mesh) -> anyhow::Result<Vec<[u32; 3]>> {
    let indices = mesh
        .indices()
        .context("runtime armor body has no index buffer")?;
    let indices = match indices {
        Indices::U16(indices) => indices.iter().map(|index| u32::from(*index)).collect(),
        Indices::U32(indices) => indices.clone(),
    };
    let (faces, remainder) = indices.as_chunks::<3>();
    if !remainder.is_empty() {
        bail!("runtime armor body index buffer is not triangle-aligned");
    }
    Ok(faces.to_vec())
}

fn runtime_body_joint_names(skin: &GltfSkin, gltf_nodes: &Assets<GltfNode>) -> Vec<String> {
    skin.joints
        .iter()
        .map(|joint| {
            gltf_nodes
                .get(joint)
                .map(|node| node.name.clone())
                .unwrap_or_else(|| "joint".into())
        })
        .collect()
}

fn runtime_body_joint_states(inverse_bindposes: &SkinnedMeshInverseBindposes) -> Vec<[f32; 8]> {
    inverse_bindposes
        .iter()
        .map(|inverse_bindpose| {
            let (scale, rotation, position) =
                inverse_bindpose.inverse().to_scale_rotation_translation();
            [
                position.x, position.y, position.z, rotation.x, rotation.y, rotation.z, rotation.w,
                scale.x,
            ]
        })
        .collect()
}

fn runtime_body_morphs(
    mesh: &Mesh,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    global_joint_states: &[[f32; 8]],
) -> Vec<RuntimeBodyMorph> {
    mesh.get_morph_targets()
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
                        global_joint_states: global_joint_states.to_vec(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
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
