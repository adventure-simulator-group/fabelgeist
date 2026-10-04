use super::skin::parse_skin;
use super::{
    Character, CharacterDecodeError, Mesh, RigObjectContext, parse_blend_shapes, parse_skeleton,
};
use crate::math::{Mat4, Transform};
use fabelgeist_fbx::{FbxClassName, FbxObjectId, FbxRecordName, Scene};
use std::collections::HashMap;

impl Character {
    /// Loads a character from the bytes of a binary FBX rig.
    pub fn from_fbx_bytes(
        data: &fabelgeist_fs::FileContents,
    ) -> Result<Self, CharacterDecodeError> {
        let scene = Scene::parse(fabelgeist_storage::StorageView::from(data.as_ref()))
            .map_err(CharacterDecodeError::Fbx)?;
        Self::from_scene(scene)
    }

    pub(super) fn from_scene(scene: Scene) -> Result<Self, CharacterDecodeError> {
        let (skeleton, joint_ids) = parse_skeleton(&scene);
        if skeleton.is_empty() {
            return Err(CharacterDecodeError::NoJoints);
        }

        let joint_of_object: HashMap<FbxObjectId, usize> = joint_ids
            .iter()
            .enumerate()
            .map(|(index, id)| (*id, index))
            .collect();

        // Default to each joint's bind-pose global transform; clusters override
        // the joints they actually bind, which for MHR is all of them.
        let bind_pose = skeleton.bind_pose();
        let mut inverse_bind_pose: Vec<Mat4> = bind_pose
            .iter()
            .map(|transform| {
                let inverse = transform.inverse();
                let rotation = crate::math::quat_to_matrix(inverse.rotation);
                let mut m = [[0.0; 4]; 4];
                for row in 0..3 {
                    for col in 0..3 {
                        m[row][col] = rotation[row][col] * inverse.scale;
                    }
                    m[row][3] = inverse.translation[row];
                }
                m[3][3] = 1.0;
                m
            })
            .collect();

        let mesh_model = scene
            .objects_of_kind(&FbxRecordName::MODEL, &FbxClassName::MESH)
            .next()
            .ok_or(CharacterDecodeError::MissingMesh)?;
        let geometry = scene
            .child_of_kind(mesh_model.id, &FbxRecordName::GEOMETRY, &FbxClassName::MESH)
            .ok_or_else(|| -> CharacterDecodeError {
                CharacterDecodeError::MissingGeometry(RigObjectContext::from(mesh_model))
            })?;

        let mesh = Mesh::from_geometry(geometry)?;
        let vertices = mesh.vertices.len();

        let blend_shapes = parse_blend_shapes(&scene, geometry, vertices);

        let skin_weights = parse_skin(
            &scene,
            geometry,
            &joint_of_object,
            vertices,
            &mut inverse_bind_pose,
        )?;

        Ok(Self {
            skeleton,
            mesh,
            skin_weights,
            inverse_bind_pose: inverse_bind_pose
                .iter()
                .map(|m| Transform::from_matrix(m).to_skel_state())
                .collect(),
            blend_shapes,
        })
    }
}

#[cfg(test)]
mod tests;
