use adventuresim_character_creator::{bracer::ForearmMorphSample, runtime_equipment::RuntimeBody};
use anyhow::{Context, Result};
use bevy::math::Mat4;
pub fn load(bytes: &[u8]) -> Result<(RuntimeBody, Vec<ForearmMorphSample>)> {
    let gltf = gltf::Gltf::from_slice(bytes)?;
    let blob = gltf.blob.as_deref().context("expected GLB buffer")?;
    let node = gltf
        .nodes()
        .find(|node| node.skin().is_some() && node.mesh().is_some())
        .context("missing body")?;
    let skin = node.skin().unwrap();
    let mesh = node.mesh().unwrap();
    let primitive = mesh.primitives().next().unwrap();
    let reader = primitive.reader(|_| Some(blob));
    let positions = reader
        .read_positions()
        .context("positions")?
        .collect::<Vec<_>>();
    let normals = reader
        .read_normals()
        .context("normals")?
        .collect::<Vec<_>>();
    let indices = reader
        .read_indices()
        .context("triangles")?
        .into_u32()
        .collect::<Vec<_>>();
    let faces = indices.as_chunks::<3>().0.to_vec();
    let texcoords = surface_chart(&positions);
    let joint_names = skin
        .joints()
        .map(|joint| fabelgeist_rig::RigJointName::from(joint.name().unwrap_or("joint")))
        .collect();
    let global_joint_states = joint_states(&skin, blob)?;
    let morphs = reader
        .read_morph_targets()
        .enumerate()
        .map(|(index, (p, n, _))| ForearmMorphSample {
            name: format!("target_{index}"),
            positions: positions
                .iter()
                .zip(p.unwrap())
                .map(|(p, d)| std::array::from_fn(|i| p[i] + d[i]))
                .collect(),
            normals: n
                .map(|d| {
                    normals
                        .iter()
                        .zip(d)
                        .map(|(p, d)| std::array::from_fn(|i| p[i] + d[i]))
                        .collect()
                })
                .unwrap_or_else(|| normals.clone()),
            global_joint_states: global_joint_states.clone(),
            device: Default::default(),
        })
        .collect();
    Ok((
        RuntimeBody {
            domain: "runtime_body_cylindrical_v1".into(),
            positions,
            normals,
            texcoords,
            texcoord_faces: faces.clone(),
            faces,
            joint_names,
            global_joint_states,
            joint_indices: reader
                .read_joints(0)
                .context("joints")?
                .into_u16()
                .map(|j| {
                    [
                        j[0] as u32,
                        j[1] as u32,
                        j[2] as u32,
                        j[3] as u32,
                        0,
                        0,
                        0,
                        0,
                    ]
                })
                .collect(),
            joint_weights: reader
                .read_weights(0)
                .context("weights")?
                .into_f32()
                .map(|w| [w[0], w[1], w[2], w[3], 0.0, 0.0, 0.0, 0.0])
                .collect(),
            device: Default::default(),
        },
        morphs,
    ))
}

fn joint_states(skin: &gltf::Skin<'_>, blob: &[u8]) -> Result<Vec<[f32; 8]>> {
    Ok(skin
        .reader(|_| Some(blob))
        .read_inverse_bind_matrices()
        .context("bind poses")?
        .map(|matrix| {
            let (scale, rotation, p) = Mat4::from_cols_array_2d(&matrix)
                .inverse()
                .to_scale_rotation_translation();
            [
                p.x, p.y, p.z, rotation.x, rotation.y, rotation.z, rotation.w, scale.x,
            ]
        })
        .collect::<Vec<_>>())
}

fn surface_chart(positions: &[[f32; 3]]) -> Vec<[f32; 2]> {
    let low = positions.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
    let high = positions
        .iter()
        .map(|p| p[1])
        .fold(f32::NEG_INFINITY, f32::max);
    positions
        .iter()
        .map(|[x, y, z]| {
            [
                x.atan2(*z) / std::f32::consts::TAU + 0.5,
                (y - low) / (high - low),
            ]
        })
        .collect()
}
