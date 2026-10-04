//! Turning a posed skeleton into skinning matrices.
//!
//! This is the CPU half of skinning: it produces the joint matrices a shader
//! multiplies vertices by, without knowing what a shader is. `fabelgeist-gpu` wraps
//! the result in a buffer-backed `Pose`.

use crate::skeleton::{Skeleton, SkeletonModelMatrices, SkinJointCount, SkinJointMatrices};
use fabelgeist_rig::RigJointCount;

/// Builds matrices that deform mesh-local vertices into mesh-local vertices.
///
/// Joint transforms are stored relative to the skeleton root, while imported
/// inverse bind matrices include that root transform. Applying the root here
/// keeps it out of `Mesh::transform`, which is reserved for rendering/placement.
pub fn build_skinning_matrices(
    skeleton: &Skeleton,
    world_transforms: &SkeletonModelMatrices,
) -> Result<SkinJointMatrices, SkinningError> {
    if world_transforms.count() != skeleton.joints.count() {
        return Err(SkinningError::JointCountMismatch {
            expected: skeleton.joints.count(),
            actual: world_transforms.count(),
        });
    }

    let skin_joint_count = skeleton
        .joints
        .iter()
        .filter_map(|joint| joint.joint_index)
        .max()
        .map_or_else(
            SkinJointCount::default,
            super::SkinJointOrdinal::required_count,
        );
    let skeleton_root = skeleton.transform.to_mat4();
    let mut joint_matrices = SkinJointMatrices::from(skin_joint_count);

    for (joint, world_transform) in skeleton.joints.iter().zip(world_transforms.iter()) {
        if let Some(index) = joint.joint_index {
            joint_matrices[index] = skeleton_root * *world_transform * joint.inverse_bind_matrix;
        }
    }

    Ok(joint_matrices)
}

/// A posed matrix collection must have one entry per ordered skeleton joint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkinningError {
    JointCountMismatch {
        expected: RigJointCount,
        actual: RigJointCount,
    },
}
impl std::fmt::Display for SkinningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Pose joint count mismatch with skeleton")
    }
}
impl std::error::Error for SkinningError {}
