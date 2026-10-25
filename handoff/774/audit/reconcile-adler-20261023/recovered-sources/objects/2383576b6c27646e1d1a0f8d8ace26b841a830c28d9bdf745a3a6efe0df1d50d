//! The authored FBX traversal establishes every ordered joint column together.

use crate::math::{Quat, Transform, quat_from_euler_degrees, quat_mul, rotation_order};
use fabelgeist_fbx::{Object, Scene};
use fabelgeist_rig::{RigJointName, RigJointOrdinal, RigJointTable};

/// The joint hierarchy, in momentum order (parents always precede children).
#[derive(Debug, Default, Clone)]
pub struct Skeleton {
    pub names: RigJointTable<RigJointName>,
    /// Roots have no parent; every child follows an earlier admitted joint.
    pub parents: RigJointTable<Option<RigJointOrdinal>>,
    pub translation_offsets: RigJointTable<[f32; 3]>,
    /// Rest rotation baked into the joint, `[x, y, z, w]`.
    pub prerotations: RigJointTable<[f32; 4]>,
}

struct SkeletonBuilder<'a> {
    scene: &'a Scene,
    skeleton: Skeleton,
    /// FBX object id per joint index, used to resolve skin clusters back to joints.
    joint_ids: RigJointTable<i64>,
}

pub(super) struct ParsedSkeleton {
    pub(super) skeleton: Skeleton,
    pub(super) joint_ids: RigJointTable<i64>,
}

impl Skeleton {
    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn joint_index(&self, name: &RigJointName) -> Option<RigJointOrdinal> {
        name.index_in(&self.names)
    }

    /// Bind-pose global transform per joint (all joint parameters zero).
    pub fn bind_pose(&self) -> RigJointTable<Transform> {
        let mut global: RigJointTable<Transform> = RigJointTable::default();
        for joint in self.names.ordinals() {
            let offset = self.translation_offsets[joint];
            let prerotation = self.prerotations[joint];
            let local = Transform {
                translation: [offset[0] as f64, offset[1] as f64, offset[2] as f64],
                rotation: [
                    prerotation[0] as f64,
                    prerotation[1] as f64,
                    prerotation[2] as f64,
                    prerotation[3] as f64,
                ],
                scale: 1.0,
            };
            let parent = self.parents[joint];
            global.push(match parent {
                None => local,
                Some(parent) => global[parent].compose(&local),
            });
        }
        global
    }
}

impl<'a> SkeletonBuilder<'a> {
    fn visit(&mut self, object: &Object, parent: Option<RigJointOrdinal>) {
        if !object.is_node() {
            return;
        }

        if object.is_null_node() {
            // Nulls are collision primitives or locators; neither is a joint.
            // A null at the top of the file is just a grouping node, so keep descending.
            if parent.is_none() && object.node.property70("col_type").is_none() {
                for child in self.scene.children(object.id).collect::<Vec<_>>() {
                    self.visit(child, None);
                }
            }
            return;
        }

        if !object.is_limb() {
            return;
        }

        let order = rotation_order(object.node.property70_i64("RotationOrder", 0));
        let local_rotation =
            quat_from_euler_degrees(object.node.property70_vec3("Lcl Rotation", [0.0; 3]), order);
        let pre_rotation = quat_from_euler_degrees(
            object.node.property70_vec3("PreRotation", [0.0; 3]),
            [0, 1, 2],
        );
        // momentum bakes any rest rotation into the joint's pre-rotation.
        let prerotation: Quat = quat_mul(pre_rotation, local_rotation);
        let offset = object.node.property70_vec3("Lcl Translation", [0.0; 3]);

        let index = RigJointOrdinal::from(self.skeleton.len());
        self.skeleton.names.push(RigJointName::from(&object.name));
        self.skeleton.parents.push(parent);
        self.skeleton.translation_offsets.push([
            offset[0] as f32,
            offset[1] as f32,
            offset[2] as f32,
        ]);
        self.skeleton.prerotations.push([
            prerotation[0] as f32,
            prerotation[1] as f32,
            prerotation[2] as f32,
            prerotation[3] as f32,
        ]);
        self.joint_ids.push(object.id);

        for child in self.scene.children(object.id).collect::<Vec<_>>() {
            self.visit(child, Some(index));
        }
    }
}

pub(super) fn parse_skeleton(scene: &Scene) -> ParsedSkeleton {
    let mut builder = SkeletonBuilder {
        scene,
        skeleton: Skeleton::default(),
        joint_ids: RigJointTable::default(),
    };
    for root in scene.children(0).collect::<Vec<_>>() {
        builder.visit(root, None);
    }
    ParsedSkeleton {
        skeleton: builder.skeleton,
        joint_ids: builder.joint_ids,
    }
}
