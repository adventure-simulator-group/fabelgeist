//! Reject unusable fitted vertices and missing triangle slots.
use super::{ClothingError, ClothingFaceContext, ClothingPieceName};

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
