//! Fitted model transforms install matching local transforms and inverse binds.
use super::{Skeleton, SkeletonModelMatrices};
use fabelgeist_math::{matrix::Mat4, transform::Transform};
use fabelgeist_rig::RigJointOrdinal;

impl Skeleton {
    pub(super) fn install_fitted_transforms(&mut self, fitted: &SkeletonModelMatrices) {
        let skeleton_root = self.transform.to_mat4();
        for joint in self.joints.ordinals() {
            let world = fitted[joint];
            let parent_world = self.joints[joint]
                .parent_index
                .map(|parent: RigJointOrdinal| -> Mat4 { fitted[parent] })
                .unwrap_or(Mat4::identity());
            let local = parent_world.inverse().unwrap_or(Mat4::identity()) * world;
            self.joints[joint].local_transform = Transform::from_mat4(local);
            self.joints[joint].inverse_bind_matrix = (skeleton_root * world)
                .inverse()
                .unwrap_or(Mat4::identity());
        }
    }
}
