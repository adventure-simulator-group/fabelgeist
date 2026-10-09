//! Evaluate proportion-dependent rest joints without reading an animated pose.
use super::*;
use adventuresim_core::character_proportions::{CharacterProportions, JointProportionBasis};

#[cfg_attr(test, derive(Default))]
pub(super) struct FittingRig {
    nodes: Vec<RigNode>,
    joints: Vec<usize>,
    pub reference: CharacterProportions,
}

struct RigNode {
    parent: Option<usize>,
    local: Transform,
    basis: Option<JointProportionBasis>,
}

#[cfg(test)]
#[path = "rig_tests.rs"]
mod tests;

#[derive(Deserialize)]
struct Extras {
    adventuresim_proportions: Option<JointProportionBasis>,
}

impl FittingRig {
    pub(super) fn new(
        gltf: &Gltf,
        skin: &GltfSkin,
        assets: &Assets<GltfNode>,
    ) -> anyhow::Result<Self> {
        let nodes = gltf
            .nodes
            .iter()
            .map(|handle| assets.get(handle).context("missing fitting rig node"))
            .collect::<anyhow::Result<Vec<_>>>()?;
        let mut parents = vec![None; nodes.len()];
        for (index, node) in nodes.iter().enumerate() {
            for child in &node.children {
                let child = gltf
                    .nodes
                    .iter()
                    .position(|handle| handle == child)
                    .context("missing rig child")?;
                anyhow::ensure!(
                    parents[child].replace(index).is_none(),
                    "rig node has multiple parents"
                );
            }
        }
        let mut reference = None;
        let nodes = nodes
            .iter()
            .zip(parents)
            .map(|(node, parent)| {
                let basis = node
                    .extras
                    .as_ref()
                    .map(|extras| serde_json::from_str::<Extras>(&extras.value))
                    .transpose()?
                    .and_then(|extras| extras.adventuresim_proportions);
                if let Some(basis) = &basis {
                    anyhow::ensure!(basis.is_finite(), "invalid skeletal basis");
                    anyhow::ensure!(
                        reference.is_none_or(|r| r == basis.reference),
                        "inconsistent skeletal references"
                    );
                    reference = Some(basis.reference);
                }
                Ok(RigNode {
                    parent,
                    local: node.transform,
                    basis,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let joints = skin
            .joints
            .iter()
            .map(|joint| {
                gltf.nodes
                    .iter()
                    .position(|handle| handle == joint)
                    .context("missing fitting joint")
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(Self {
            nodes,
            joints,
            reference: reference.unwrap_or_default(),
        })
    }

    fn global(
        &self,
        index: usize,
        proportions: Option<CharacterProportions>,
    ) -> anyhow::Result<Mat4> {
        let mut current = Some(index);
        let mut global = Mat4::IDENTITY;
        for _ in 0..self.nodes.len() {
            let Some(index) = current else {
                return Ok(global);
            };
            let node = &self.nodes[index];
            let mut local = node.local;
            if let (Some(proportions), Some(basis)) = (proportions, &node.basis) {
                local.translation += Vec3::from_array(basis.translation(proportions));
            }
            global = local.to_matrix() * global;
            current = node.parent;
        }
        anyhow::ensure!(current.is_none(), "cyclic fitting rig");
        Ok(global)
    }

    pub(super) fn fit(
        &self,
        body: &mut RuntimeBody,
        proportions: CharacterProportions,
    ) -> anyhow::Result<Vec<Mat4>> {
        let mut offsets = self
            .joints
            .iter()
            .map(|&joint| {
                Ok(self.global(joint, Some(proportions))?.w_axis.truncate()
                    - self.global(joint, None)?.w_axis.truncate())
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        // Match animation::skeletal_proportions: longer legs lift the root.
        let named = |role| {
            body.joint_names
                .iter()
                .position(|name| BoneRole::from_name(name) == Some(role))
        };
        if let (Some(left), Some(right), Some(root)) = (
            named(BoneRole::FootLeft),
            named(BoneRole::FootRight),
            named(BoneRole::Pelvis),
        ) {
            let correction = Vec3::Y * -(offsets[left].y + offsets[right].y) * 0.5;
            for (offset, &joint) in offsets.iter_mut().zip(&self.joints) {
                let mut ancestor = Some(joint);
                while let Some(index) = ancestor {
                    if index == self.joints[root] {
                        *offset += correction;
                        break;
                    }
                    ancestor = self.nodes[index].parent;
                }
            }
        }
        for (position, (joints, weights)) in body
            .positions
            .iter_mut()
            .zip(body.joint_indices.iter().zip(&body.joint_weights))
        {
            let mut offset = Vec3::ZERO;
            for (&joint, &weight) in joints.iter().zip(weights) {
                if weight != 0.0 {
                    offset += offsets
                        .get(joint as usize)
                        .context("invalid body skin joint")?
                        * weight;
                }
            }
            *position = (Vec3::from_array(*position) + offset).to_array();
        }
        Ok(body
            .global_joint_states
            .iter_mut()
            .zip(offsets)
            .map(|(state, offset)| {
                for axis in 0..3 {
                    state[axis] += offset[axis];
                }
                Mat4::from_scale_rotation_translation(
                    Vec3::splat(state[7]),
                    Quat::from_xyzw(state[3], state[4], state[5], state[6]),
                    Vec3::new(state[0], state[1], state[2]),
                )
                .inverse()
            })
            .collect())
    }
}
