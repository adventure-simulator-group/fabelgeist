//! Resolve FBX cluster identities into the admitted rig's packed influences.

use super::{MAX_SKIN_JOINTS, MeshVertexCount, MeshVertexIndex, MeshVertexTable};
use crate::math::{Mat4, affine_inverse, mat4_from_column_major};
use anyhow::{Result, bail};
use fabelgeist_fbx::{Object, Scene};
use fabelgeist_rig::{PackedRigSkin, RigInfluences, RigJointOrdinal, RigJointTable};
use std::collections::HashMap;

/// Fixed-width sparse skinning, eight influences per vertex.
#[derive(Debug, Default, Clone)]
pub struct SkinWeights {
    index: PackedRigSkin<MAX_SKIN_JOINTS>,
    weight: MeshVertexTable<[f32; MAX_SKIN_JOINTS]>,
}

#[derive(Clone, Copy)]
struct SkinInfluence {
    joint: RigJointOrdinal,
    weight: f64,
}

impl SkinWeights {
    pub fn indices(&self) -> &PackedRigSkin<MAX_SKIN_JOINTS> {
        &self.index
    }

    pub fn weights(&self) -> &MeshVertexTable<[f32; MAX_SKIN_JOINTS]> {
        &self.weight
    }

    /// Checked selection from the actual paired vertex rows. The native ordinal
    /// is used only to decode the shared packed skin's word storage.
    pub fn vertex(&self, vertex: MeshVertexIndex) -> Option<RigInfluences<MAX_SKIN_JOINTS>> {
        let weights = *self.weight.get(vertex)?;
        let joints = self.index.get_vertex(vertex.native_vertex_index())?;
        Some(RigInfluences { joints, weights })
    }

    pub fn indexed(
        &self,
    ) -> impl Iterator<Item = (MeshVertexIndex, RigInfluences<MAX_SKIN_JOINTS>)> {
        self.weight
            .indexed()
            .zip(self.index.iter())
            .map(|((vertex, weights), joints)| {
                (
                    vertex,
                    RigInfluences {
                        joints,
                        weights: *weights,
                    },
                )
            })
    }
}

pub(super) fn parse_skin(
    scene: &Scene,
    geometry: &Object,
    joint_of_object: &HashMap<i64, RigJointOrdinal>,
    num_vertices: MeshVertexCount,
    inverse_bind_pose: &mut RigJointTable<Mat4>,
) -> Result<SkinWeights> {
    let mut per_vertex = MeshVertexTable::filled(num_vertices, Vec::<SkinInfluence>::new());

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
            let vertex = MeshVertexIndex::from_native_vertex_index(*vertex as usize);
            let Some(influences) = per_vertex.get_mut(vertex) else {
                bail!("cluster '{}' references vertex {vertex}", cluster.name);
            };
            if *weight <= 0.0 {
                continue;
            }
            influences.push(SkinInfluence {
                joint,
                weight: weight.clamp(0.0, 1.0),
            });
        }
    }

    let mut indices = Vec::new();
    let mut weights = Vec::new();
    for (vertex, influences) in per_vertex.indexed_mut() {
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
        let mut joint_row = [0; MAX_SKIN_JOINTS];
        let mut weight_row = [0.0; MAX_SKIN_JOINTS];
        for (slot, influence) in influences.iter().enumerate() {
            joint_row[slot] = u32::try_from(influence.joint)?;
            weight_row[slot] = (influence.weight / total) as f32;
        }
        indices.push(joint_row);
        weights.push(weight_row);
    }

    Ok(SkinWeights {
        index: PackedRigSkin::from_native(indices, inverse_bind_pose.joint_count())?,
        weight: weights.into(),
    })
}
