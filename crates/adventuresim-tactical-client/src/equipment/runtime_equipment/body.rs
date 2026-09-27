//! Extract the canonical skinned body used to generate fitted equipment.
use super::*;

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
        let node = gltf_nodes.get(node_handle)?;
        let (Some(mesh_handle), Some(skin_handle)) = (node.mesh.as_ref(), node.skin.as_ref())
        else {
            continue;
        };
        let gltf_mesh = gltf_meshes.get(mesh_handle)?;
        let skin = gltf_skins.get(skin_handle)?;
        let inverse_bindposes_asset = inverse_bindposes.get(&skin.inverse_bind_matrices)?;

        if let Some(primitive) = gltf_mesh.primitives.first() {
            let mesh = meshes.get(&primitive.mesh)?;
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
    let texcoords = body_surface_coordinates(&positions)?;
    let joint_indices = attribute_u16x4(mesh, Mesh::ATTRIBUTE_JOINT_INDEX, "joint indices")?;
    let joint_weights = attribute_f32x4(mesh, Mesh::ATTRIBUTE_JOINT_WEIGHT, "joint weights")?;
    let indices = mesh
        .indices()
        .context("runtime armor body has no index buffer")?;
    let indices = match indices {
        Indices::U16(indices) => indices.iter().map(|index| u32::from(*index)).collect(),
        Indices::U32(indices) => indices.clone(),
    };
    let (triangles, remainder) = indices.as_chunks::<3>();
    if !remainder.is_empty() {
        bail!("runtime armor body index buffer is not triangle-aligned");
    }
    let faces = triangles.to_vec();

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

    let morphs = runtime_body_morphs(mesh, &positions, &normals, &global_joint_states);

    let body = RuntimeBody {
        domain: "runtime_body_cylindrical_v1".into(),
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
        device: Default::default(),
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

/// The canonical animation body contains geometry and skin without a UV atlas.
/// Runtime surfaces own a geometric UV domain; they do not claim correspondence
/// with the offline MHR atlas used by authored equipment attachment sockets.
fn body_surface_coordinates(positions: &[[f32; 3]]) -> anyhow::Result<Vec<[f32; 2]>> {
    let mut minimum = f32::INFINITY;
    let mut maximum = f32::NEG_INFINITY;
    for position in positions {
        if !position.iter().all(|value| value.is_finite()) {
            bail!("non-finite runtime body vertex");
        }
        minimum = minimum.min(position[1]);
        maximum = maximum.max(position[1]);
    }
    let height = maximum - minimum;
    if !height.is_finite() || height <= f32::EPSILON {
        bail!("runtime body has no height");
    }
    Ok(positions
        .iter()
        .map(|[x, y, z]| {
            [
                x.atan2(*z) / std::f32::consts::TAU + 0.5,
                (y - minimum) / height,
            ]
        })
        .collect())
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn geometric_surface_chart_handles_uv_free_body_and_rejects_invalid_geometry() {
        let positions = [[0.0, -1.0, 0.2], [0.2, 0.0, 0.0], [-0.2, 1.0, 0.0]];
        let chart = body_surface_coordinates(&positions).unwrap();
        assert!(
            chart
                .iter()
                .flatten()
                .all(|value| (0.0..=1.0).contains(value))
        );
        assert_ne!(chart[1][0], chart[2][0]);
        assert!(body_surface_coordinates(&[]).is_err());
        assert!(body_surface_coordinates(&[[0.0; 3]; 2]).is_err());
        assert!(body_surface_coordinates(&[[0.0, f32::NAN, 0.0]]).is_err());
    }
}
