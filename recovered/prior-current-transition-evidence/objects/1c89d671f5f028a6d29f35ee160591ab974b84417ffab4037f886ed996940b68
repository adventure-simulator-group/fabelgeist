//! Resolve FBX cluster identities into the admitted rig's packed influences.

use super::MAX_SKIN_JOINTS;
use crate::math::{Mat4, affine_inverse, mat4_from_column_major};
use anyhow::{Result, bail};
use fabelgeist_fbx::{Object, Scene};
use fabelgeist_rig::{PackedRigSkin, RigJointOrdinal, RigJointTable};
use std::collections::HashMap;

/// Fixed-width sparse skinning, eight influences per vertex.
#[derive(Debug, Default, Clone)]
pub struct SkinWeights {
    pub index: PackedRigSkin<MAX_SKIN_JOINTS>,
    pub weight: Vec<[f32; MAX_SKIN_JOINTS]>,
}

#[derive(Clone, Copy)]
struct SkinInfluence {
    joint: RigJointOrdinal,
    weight: f64,
}

pub(super) fn parse_skin(
    scene: &Scene,
    geometry: &Object,
    joint_of_object: &HashMap<i64, RigJointOrdinal>,
    num_vertices: usize,
    inverse_bind_pose: &mut RigJointTable<Mat4>,
) -> Result<SkinWeights> {
    let mut per_vertex: Vec<Vec<SkinInfluence>> = vec![Vec::new(); num_vertices];

    let Some(skin) = scene
        .children(geometry.id)
        .find(|o| o.kind == "Deformer" && o.class == "Skin")
    else {
        bail!("geometry '{}' has no skin deformer", geometry.name);
    };

    for cluster in scene
        .children(skin.id)
        .filter(|o| o.kind == "Deformer" && o.class == "Cluster")
        .collect::<Vec<_>>()
    {
        let Some(bone) = scene
            .children(cluster.id)
            .find(|o| joint_of_object.contains_key(&o.id))
        else {
            bail!("cluster '{}' references an unknown bone", cluster.name);
        };
        let joint = joint_of_object[&bone.id];

        // The cluster's bind transform overrides the joint's own rest pose.
        if let Some(link) = cluster
            .node
            .child("TransformLink")
            .and_then(|n| n.f64_array())
            && link.len() == 16
        {
            inverse_bind_pose[joint] = affine_inverse(&mat4_from_column_major(&link));
        }

        let (Some(indices), Some(weights)) = (
            cluster.node.child("Indexes").and_then(|n| n.i64_array()),
            cluster.node.child("Weights").and_then(|n| n.f64_array()),
        ) else {
            continue;
        };
        if indices.len() != weights.len() {
            bail!(
                "cluster '{}' has mismatched indices and weights",
                cluster.name
            );
        }

        for (vertex, weight) in indices.iter().zip(&weights) {
            let vertex = *vertex as usize;
            if vertex >= num_vertices {
                bail!("cluster '{}' references vertex {vertex}", cluster.name);
            }
            if *weight <= 0.0 {
                continue;
            }
            per_vertex[vertex].push(SkinInfluence {
                joint,
                weight: weight.clamp(0.0, 1.0),
            });
        }
    }

    let mut indices = vec![[0; MAX_SKIN_JOINTS]; num_vertices];
    let mut weights = vec![[0.0; MAX_SKIN_JOINTS]; num_vertices];
    for (vertex, influences) in per_vertex.iter_mut().enumerate() {
        if influences.is_empty() {
            bail!("no skinning weights for vertex {vertex}");
        }
        // Keep the strongest influences, then renormalize what survived.
        influences.sort_by(|a, b| b.weight.total_cmp(&a.weight));
        influences.truncate(MAX_SKIN_JOINTS);
        let total: f64 = influences.iter().map(|influence| influence.weight).sum();
        if total <= 0.0 {
            bail!("empty weight sum for vertex {vertex}");
        }
        for (slot, influence) in influences.iter().enumerate() {
            indices[vertex][slot] = u32::try_from(influence.joint)?;
            weights[vertex][slot] = (influence.weight / total) as f32;
        }
    }

    Ok(SkinWeights {
        index: PackedRigSkin::from_native(indices, inverse_bind_pose.joint_count())?,
        weight: weights,
    })
}
