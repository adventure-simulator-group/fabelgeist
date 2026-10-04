use fabelgeist_math::matrix::Mat4;
use fabelgeist_math::transform::Transform;
use fabelgeist_rig::{RigJointName, RigJointOrdinal};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Skeleton {
    pub joints: SkeletonJoints,
    #[serde(default = "Transform::identity")]
    pub transform: Transform,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JointInfo {
    pub name: RigJointName,
    pub index: RigJointOrdinal,
    pub joint_index: Option<SkinJointOrdinal>,
}

impl Skeleton {
    pub fn new(joints: Vec<Joint>) -> Self {
        Self {
            joints: SkeletonJoints::from(joints),
            transform: Transform::identity(),
        }
    }

    pub fn joints_info(&self) -> Vec<JointInfo> {
        self.joints
            .iter()
            .map(|j| JointInfo {
                name: j.name.clone(),
                index: j.index,
                joint_index: j.joint_index,
            })
            .collect()
    }

    pub fn find_joint_by_name(&self, name: &RigJointName) -> Option<RigJointOrdinal> {
        self.joints
            .iter()
            .position(|j| &j.name == name)
            .map(RigJointOrdinal::from)
    }

    /// The exact label at an ordered skeleton slot; the caller owns range validity.
    pub fn joint_name(&self, index: RigJointOrdinal) -> &RigJointName {
        &self.joints[index].name
    }

    pub fn world_positions(&self) -> Vec<fabelgeist_math::vector::Vec3> {
        let mut world_matrices = vec![Mat4::identity(); usize::from(self.joints.count())];
        let mut world_positions = vec![
            fabelgeist_math::vector::Vec3::new(0.0, 0.0, 0.0);
            usize::from(self.joints.count())
        ];

        for i in self.joints.ordinals() {
            let joint = &self.joints[i];
            let local = joint.local_transform.to_mat4();
            let world = if let Some(parent_idx) = joint.parent_index {
                world_matrices[usize::from(parent_idx)] * local
            } else {
                local
            };
            world_matrices[usize::from(i)] = world;
            world_positions[usize::from(i)] = fabelgeist_math::vector::Vec3::new(
                world.columns[3][0],
                world.columns[3][1],
                world.columns[3][2],
            );
        }
        world_positions
    }

    pub fn world_transforms(&self) -> SkeletonModelMatrices {
        let mut world_matrices = vec![Mat4::identity(); usize::from(self.joints.count())];

        for i in self.joints.ordinals() {
            let joint = &self.joints[i];
            let local = joint.local_transform.to_mat4();
            let world = if let Some(parent_idx) = joint.parent_index {
                world_matrices[usize::from(parent_idx)] * local
            } else {
                local
            };
            world_matrices[usize::from(i)] = world;
        }
        SkeletonModelMatrices::from(world_matrices)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeType {
    Sphere,
    #[default]
    Capsule,
}

fn default_radius() -> f32 {
    0.15
}

fn default_smoothstep_start() -> f32 {
    0.0
}

fn default_smoothstep_end() -> f32 {
    0.2
}

fn default_enabled() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Joint {
    pub name: RigJointName,
    pub index: RigJointOrdinal,
    pub parent_index: Option<RigJointOrdinal>,
    pub inverse_bind_matrix: Mat4,
    pub local_transform: Transform,
    pub joint_index: Option<SkinJointOrdinal>, // Index in the GPU skinning buffer
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_radius")]
    pub radius: f32,
    #[serde(default)]
    pub shape_type: ShapeType,
    #[serde(default = "default_smoothstep_start")]
    pub smoothstep_start: f32,
    #[serde(default = "default_smoothstep_end")]
    pub smoothstep_end: f32,
}

impl Joint {
    pub fn new(
        name: RigJointName,
        index: RigJointOrdinal,
        parent_index: Option<RigJointOrdinal>,
        inverse_bind_matrix: Mat4,
        local_transform: Transform,
        joint_index: Option<SkinJointOrdinal>,
    ) -> Self {
        Self {
            name,
            index,
            parent_index,
            inverse_bind_matrix,
            local_transform,
            joint_index,
            enabled: true,
            radius: 0.15,
            shape_type: ShapeType::Capsule,
            smoothstep_start: 0.0,
            smoothstep_end: 0.2,
        }
    }
}

pub mod auto_rigger;
pub mod mixamo;
pub mod skinning;

pub use skinning::{SkinningError, build_skinning_matrices};

mod fitting;
mod joints;
mod matrices;
mod slots;
pub use joints::SkeletonJoints;
pub use matrices::{SkeletonModelMatrices, SkinJointMatrices};
pub use slots::{SkinJointCount, SkinJointOrdinal};

mod weights;
pub use weights::{
    SkinBlendWeight, SkinnedVertexCount, SkinnedVertexOrdinal, VertexSkinBinding, VertexSkinWeights,
};
