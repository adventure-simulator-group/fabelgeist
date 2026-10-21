//! Admit aligned body inputs and reject unusable fitted garment geometry.

use super::{ClothingError, ClothingFaceContext, ClothingPieceName};
use fabelgeist_rig::{PackedRigSkin, RigJointName, RigJointTable};

type Result<T> = std::result::Result<T, ClothingError>;

/// The fitting kernels consume rest-frame XYZ points/normals and native vertex
/// slots alongside admitted rig slots. Check their shared surface and joint
/// extents before any garment selection or rest-frame numerical operation.
pub(super) fn validate_body_inputs(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    faces: &[[u32; 3]],
    joint_indices: &PackedRigSkin<8>,
    joint_weights: &[[f32; 8]],
    joint_names: &RigJointTable<RigJointName>,
    global_joint_states: &RigJointTable<[f32; 8]>,
) -> Result<()> {
    let vertices = positions.len();
    if normals.len() != vertices
        || joint_indices.len() != vertices
        || joint_weights.len() != vertices
        || global_joint_states.len() != joint_names.len()
        || joint_indices.joint_count() != joint_names.joint_count()
    {
        return Err(ClothingError::InconsistentBodyInputs);
    }
    if faces
        .iter()
        .flatten()
        .any(|vertex| *vertex as usize >= vertices)
    {
        return Err(ClothingError::MissingBodyVertex);
    }
    Ok(())
}

/// `positions` are fitted XYZ points in metres in the body's rest frame;
/// `faces` hold three encoded vertex slots per triangle. Rejected triangles
/// retain those source slots in `ClothingFaceContext` for diagnosis.
pub(super) fn validated_placeholder_faces(
    name: &ClothingPieceName,
    faces: &[[u32; 3]],
    positions: &[[f32; 3]],
) -> Result<Vec<[u32; 3]>> {
    if positions.iter().flatten().any(|value| !value.is_finite()) {
        return Err(ClothingError::NonFiniteFittedVertex {
            garment: name.clone(),
        });
    }
    if let Some(face) = faces.iter().find(|face| {
        face.iter()
            .any(|vertex| *vertex as usize >= positions.len())
    }) {
        return Err(ClothingError::OutOfRangeFace {
            garment: name.clone(),
            face: ClothingFaceContext::from(*face),
        });
    }
    Ok(faces.to_vec())
}
