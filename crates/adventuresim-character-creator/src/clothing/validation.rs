//! Reject unusable fitted vertices and missing triangle slots.

use super::{ClothingError, ClothingFaceContext, ClothingPieceName};

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

/// Grow an anatomical face mask by neighboring triangle rings.
pub(super) fn expand_selected_faces(
    faces: &[[u32; 3]],
    selected_faces: &mut [bool],
    vertices: usize,
    rings: usize,
) {
    for _ in 0..rings {
        let mut selected_vertices = vec![false; vertices];
        for (face, selected) in faces.iter().zip(selected_faces.iter()) {
            if *selected {
                for vertex in face {
                    selected_vertices[*vertex as usize] = true;
                }
            }
        }
        let previous = selected_faces.to_vec();
        for (index, face) in faces.iter().enumerate() {
            selected_faces[index] |= face
                .iter()
                .filter(|vertex| selected_vertices[**vertex as usize])
                .count()
                >= 2;
        }
        if *selected_faces == previous {
            break;
        }
    }
}
