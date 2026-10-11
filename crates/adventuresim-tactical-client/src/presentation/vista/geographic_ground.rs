//! Preserve authored normals when the geographic frame reflects local north.
use bevy::{
    mesh::{Indices, MeshWindingInvertError, VertexAttributeValues},
    prelude::*,
};

#[derive(Debug, thiserror::Error)]
pub(in crate::presentation) enum GeographicGroundError {
    #[error("canonical city ground exceeds the mesh index range")]
    IndexCapacity,
    #[error(transparent)]
    Winding(#[from] MeshWindingInvertError),
}

/// Mesh upload adapter for the city's east/up/north source frame. The map root
/// reflects north into south; reversed indices preserve upward-facing ground.
/// Source normals remain authored normals, transformed by the root as usual.
pub(in crate::presentation) fn reflected(mut mesh: Mesh) -> Result<Mesh, GeographicGroundError> {
    if mesh.indices().is_none() {
        // Paving stores consecutive triangle vertices. Give that authored
        // topology indices so reflection can reverse winding without moving
        // vertices or changing any of their material attributes.
        let count = u32::try_from(mesh.count_vertices())
            .map_err(|_| GeographicGroundError::IndexCapacity)?;
        mesh.insert_indices(Indices::U32((0..count).collect()));
    }
    mesh.invert_winding()?;
    if let Some(VertexAttributeValues::Float32x4(tangents)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_TANGENT)
    {
        for tangent in tangents {
            tangent[3] = -tangent[3];
        }
    }
    Ok(mesh)
}
