//! Reject unusable fitted vertices and missing triangle slots.

use super::{ClothingError, ClothingFaceContext, ClothingPieceName};
use fabelgeist_rig::RigJointOrdinal;

/// `positions` are fitted XYZ points in metres in the body's rest frame;
/// `faces` hold three encoded vertex slots per triangle. Rejected triangles
/// retain those source slots in `ClothingFaceContext` for diagnosis.
pub(super) fn validated_placeholder_faces(
    name: &ClothingPieceName,
    faces: &[[u32; 3]],
    positions: &[[f32; 3]],
) -> Result<Vec<[u32; 3]>, ClothingError> {
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

/// Decode MHR's packed eight-u32 skin rows, also used by device upload and
/// glTF export. All clothing selection after admission retains rig ordinals.
pub(super) fn decoded_skin_joints(rows: &[[u32; 8]]) -> Vec<[RigJointOrdinal; 8]> {
    rows.iter()
        .map(|joints| joints.map(RigJointOrdinal::from))
        .collect()
}
