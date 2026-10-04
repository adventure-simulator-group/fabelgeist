use super::{
    CharacterDecodeError, MAX_SKIN_JOINTS, RigArrayCount, RigMeshVertexOrdinal, RigObjectContext,
    SkinWeights,
};
use crate::MeshVertexCount;
use crate::math::{Mat4, affine_inverse, mat4_from_column_major};
use fabelgeist_fbx::{FbxClassName, FbxObjectId, FbxRecordName, Object, Scene};
use std::collections::HashMap;

pub(super) fn parse_skin(
    scene: &Scene,
    geometry: &Object,
    joint_of_object: &HashMap<FbxObjectId, usize>,
    num_vertices: usize,
    inverse_bind_pose: &mut [Mat4],
) -> Result<SkinWeights, CharacterDecodeError> {
    let mut per_vertex: Vec<Vec<(usize, f64)>> = vec![Vec::new(); num_vertices];

    let Some(skin) = scene.children(geometry.id).find(|o: &&Object| -> bool {
        o.kind == FbxRecordName::DEFORMER && o.class == FbxClassName::SKIN
    }) else {
        return Err(CharacterDecodeError::MissingSkin(RigObjectContext::from(
            geometry,
        )));
    };

    for cluster in scene
        .children(skin.id)
        .filter(|o: &&Object| -> bool {
            o.kind == FbxRecordName::DEFORMER && o.class == FbxClassName::CLUSTER
        })
        .collect::<Vec<_>>()
    {
        let Some(bone) = scene
            .children(cluster.id)
            .find(|o| joint_of_object.contains_key(&o.id))
        else {
            return Err(CharacterDecodeError::UnknownBone(RigObjectContext::from(
                cluster,
            )));
        };
        let joint = joint_of_object[&bone.id];

        // The cluster's bind transform overrides the joint's own rest pose.
        if let Some(link) = cluster
            .node
            .child(&FbxRecordName::TRANSFORM_LINK)
            .and_then(|n| n.f64_array())
            && link.len() == 16
        {
            inverse_bind_pose[joint] = affine_inverse(&mat4_from_column_major(&link));
        }

        let (Some(indices), Some(weights)) = (
            cluster
                .node
                .child(&FbxRecordName::INDEXES)
                .and_then(|n| n.i64_array()),
            cluster
                .node
                .child(&FbxRecordName::WEIGHTS)
                .and_then(|n| n.f64_array()),
        ) else {
            continue;
        };
        if indices.len() != weights.len() {
            return Err(CharacterDecodeError::SkinArrayMismatch {
                cluster: RigObjectContext::from(cluster),
                indices: RigArrayCount::from(indices.len()),
                weights: RigArrayCount::from(weights.len()),
            });
        }

        for (vertex, weight) in indices.iter().zip(&weights) {
            let vertex = *vertex as usize;
            if vertex >= num_vertices {
                return Err(CharacterDecodeError::SkinVertex {
                    cluster: RigObjectContext::from(cluster),
                    vertex: RigMeshVertexOrdinal::from(vertex),
                    vertices: MeshVertexCount::from(num_vertices),
                });
            }
            if *weight <= 0.0 {
                continue;
            }
            per_vertex[vertex].push((joint, weight.clamp(0.0, 1.0)));
        }
    }

    let mut skin_weights = SkinWeights {
        index: vec![[0; MAX_SKIN_JOINTS]; num_vertices],
        weight: vec![[0.0; MAX_SKIN_JOINTS]; num_vertices],
    };
    for (vertex, influences) in per_vertex.iter_mut().enumerate() {
        if influences.is_empty() {
            return Err(CharacterDecodeError::NoSkinWeights(
                RigMeshVertexOrdinal::from(vertex),
            ));
        }
        // Keep the strongest influences, then renormalize what survived.
        influences.sort_by(|a, b| b.1.total_cmp(&a.1));
        influences.truncate(MAX_SKIN_JOINTS);
        let total: f64 = influences.iter().map(|(_, w)| *w).sum();
        if total <= 0.0 {
            return Err(CharacterDecodeError::EmptySkinWeightSum(
                RigMeshVertexOrdinal::from(vertex),
            ));
        }
        for (slot, (joint, weight)) in influences.iter().enumerate() {
            skin_weights.index[vertex][slot] = *joint as u32;
            skin_weights.weight[vertex][slot] = (*weight / total) as f32;
        }
    }

    Ok(skin_weights)
}
