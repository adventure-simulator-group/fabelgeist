//! Builds a momentum character (skeleton, mesh, skin weights, blend shapes)
//! out of an MHR `lod*.fbx` rig.
//!
//! The traversal order mirrors `momentum::loadOpenFbxCharacter`, because the
//! joint indices it produces are what the `.model` parameter transform, the
//! skin clusters, and the pose-corrective network are all indexed by.

mod diagnostics;
mod error;
#[cfg(test)]
pub(crate) mod fixture;
mod geometry;
mod loading;
mod skin;

pub use diagnostics::{
    PolygonCornerCount, RigArrayCount, RigBlendShapeCount, RigMeshVertexOrdinal, RigObjectContext,
};
pub use error::CharacterDecodeError;

use crate::math::{Quat, Transform, quat_from_euler_degrees, quat_mul, rotation_order};
use fabelgeist_fbx::{
    FbxClassName, FbxObjectId, FbxObjectName, FbxPropertyName, FbxRecordName, ModelRole, Object,
    Scene,
};
use fabelgeist_rig::{RigJointName, RigJointOrdinal};

/// Momentum allows at most eight joint influences per vertex.
pub const MAX_SKIN_JOINTS: usize = 8;
/// `tx, ty, tz, rx, ry, rz, sc`.
pub const PARAMETERS_PER_JOINT: usize = 7;

/// The joint hierarchy, in momentum order (parents always precede children).
#[derive(Debug, Default, Clone)]
pub struct Skeleton {
    pub names: Vec<RigJointName>,
    /// Parent index, or `-1` for a root.
    pub parents: Vec<i32>,
    pub translation_offsets: Vec<[f32; 3]>,
    /// Rest rotation baked into the joint, `[x, y, z, w]`.
    pub prerotations: Vec<[f32; 4]>,
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
    pub fn bind_pose(&self) -> Vec<Transform> {
        let mut global: Vec<Transform> = Vec::with_capacity(self.len());
        for joint in 0..self.len() {
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
            global.push(if parent < 0 {
                local
            } else {
                global[parent as usize].compose(&local)
            });
        }
        global
    }
}

/// Triangulated mesh with the original polygon topology dropped.
#[derive(Debug, Default, Clone)]
pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
    pub texcoords: Vec<[f32; 2]>,
    pub texcoord_faces: Vec<[u32; 3]>,
}

/// Fixed-width sparse skinning, eight influences per vertex.
#[derive(Debug, Default, Clone)]
pub struct SkinWeights {
    pub index: Vec<[u32; MAX_SKIN_JOINTS]>,
    pub weight: Vec<[f32; MAX_SKIN_JOINTS]>,
}

/// Dense blend-shape basis, `shape-major`: `vectors[shape * n_verts * 3 + ..]`.
#[derive(Debug, Default, Clone)]
pub struct BlendShapes {
    pub names: Vec<FbxObjectName>,
    pub vectors: Vec<f32>,
    pub num_vertices: usize,
}

impl BlendShapes {
    pub fn len(&self) -> RigBlendShapeCount {
        RigBlendShapeCount::from(self.names.len())
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

/// Everything the MHR forward pass needs from the rig file.
pub struct Character {
    pub skeleton: Skeleton,
    pub mesh: Mesh,
    pub skin_weights: SkinWeights,
    /// Inverse bind pose per joint as a skeleton state `[t, q, s]`.
    pub inverse_bind_pose: Vec<[f32; 8]>,
    pub blend_shapes: BlendShapes,
}

struct SkeletonBuilder<'a> {
    scene: &'a Scene,
    skeleton: Skeleton,
    /// FBX object id per joint index, used to resolve skin clusters back to joints.
    joint_ids: Vec<FbxObjectId>,
}

impl<'a> SkeletonBuilder<'a> {
    fn visit(&mut self, object: &Object, parent: Option<usize>) {
        match object.model_role() {
            Some(ModelRole::Joint) => {}
            Some(ModelRole::Null) => {
                // Root grouping nulls remain traversable; collision nulls and
                // nested locators do not contribute joints.
                if parent.is_none()
                    && object
                        .node
                        .property70(&FbxPropertyName::COLLISION_TYPE)
                        .is_none()
                {
                    for child in self.scene.children(object.id).collect::<Vec<_>>() {
                        self.visit(child, None);
                    }
                }
                return;
            }
            None | Some(ModelRole::Uninterpreted) => return,
        }

        let order = rotation_order(
            object
                .node
                .property70_i64(&FbxPropertyName::ROTATION_ORDER, 0),
        );
        let local_rotation = quat_from_euler_degrees(
            object
                .node
                .property70_vec3(&FbxPropertyName::LOCAL_ROTATION, [0.0; 3]),
            order,
        );
        let pre_rotation = quat_from_euler_degrees(
            object
                .node
                .property70_vec3(&FbxPropertyName::PRE_ROTATION, [0.0; 3]),
            [0, 1, 2],
        );
        // momentum bakes any rest rotation into the joint's pre-rotation.
        let prerotation: Quat = quat_mul(pre_rotation, local_rotation);
        let offset = object
            .node
            .property70_vec3(&FbxPropertyName::LOCAL_TRANSLATION, [0.0; 3]);

        let index = self.skeleton.len();
        self.skeleton.names.push(RigJointName::from(&object.name));
        self.skeleton
            .parents
            .push(parent.map(|p| p as i32).unwrap_or(-1));
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

fn parse_skeleton(scene: &Scene) -> (Skeleton, Vec<FbxObjectId>) {
    let mut builder = SkeletonBuilder {
        scene,
        skeleton: Skeleton::default(),
        joint_ids: Vec::new(),
    };
    for root in scene.children(FbxObjectId::SCENE_ROOT).collect::<Vec<_>>() {
        builder.visit(root, None);
    }
    (builder.skeleton, builder.joint_ids)
}

fn parse_blend_shapes(scene: &Scene, geometry: &Object, num_vertices: usize) -> BlendShapes {
    let mut names = Vec::new();
    let mut vectors: Vec<f32> = Vec::new();

    let Some(blend_shape) = scene.children(geometry.id).find(|o: &&Object| -> bool {
        o.kind == FbxRecordName::DEFORMER && o.class == FbxClassName::BLEND_SHAPE
    }) else {
        return BlendShapes::default();
    };

    for channel in scene
        .children(blend_shape.id)
        .filter(|o: &&Object| -> bool {
            o.kind == FbxRecordName::DEFORMER && o.class == FbxClassName::BLEND_SHAPE_CHANNEL
        })
        .collect::<Vec<_>>()
    {
        for shape in scene
            .children(channel.id)
            .filter(|o: &&Object| -> bool {
                o.kind == FbxRecordName::GEOMETRY && o.class == FbxClassName::SHAPE
            })
            .collect::<Vec<_>>()
        {
            let offsets = shape
                .node
                .child(&FbxRecordName::VERTICES)
                .and_then(|n| n.f64_array());
            let indices = shape
                .node
                .child(&FbxRecordName::INDEXES)
                .and_then(|n| n.i64_array());
            let base = vectors.len();
            vectors.resize(base + num_vertices * 3, 0.0);
            if let (Some(offsets), Some(indices)) = (offsets, indices) {
                for (slot, vertex) in indices.iter().enumerate() {
                    let vertex = *vertex as usize;
                    if vertex >= num_vertices || slot * 3 + 2 >= offsets.len() {
                        continue;
                    }
                    for axis in 0..3 {
                        vectors[base + vertex * 3 + axis] = offsets[slot * 3 + axis] as f32;
                    }
                }
            }
            names.push(shape.name.clone());
        }
    }

    BlendShapes {
        names,
        vectors,
        num_vertices,
    }
}
