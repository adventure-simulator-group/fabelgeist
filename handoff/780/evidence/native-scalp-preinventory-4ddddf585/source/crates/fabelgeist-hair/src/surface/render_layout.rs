//! Native topology and attribute admission for drawing a surface.

use super::Surface;
use fabelgeist_mesh::{MeshCornerCount, MeshTriangleCount, MeshVertexCount};
use std::num::TryFromIntError;

pub type SurfaceRenderResult<T> = Result<T, SurfaceRenderError>;

/// Checked native mesh counts for a surface with paired positions and normals.
/// This does not validate the surface's sampling tables or physical geometry.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceRenderLayout {
    vertices: MeshVertexCount,
    triangles: MeshTriangleCount,
    corners: MeshCornerCount,
}

#[derive(Debug, thiserror::Error)]
pub enum SurfaceRenderError {
    #[error("surface vertex count exceeds native mesh addresses")]
    VertexCount(#[source] TryFromIntError),
    #[error("surface triangle count exceeds native mesh addresses")]
    TriangleCount(#[source] TryFromIntError),
    #[error("surface {triangles} triangles exceed native corner addresses")]
    CornerCount { triangles: MeshTriangleCount },
    #[error("surface has {vertices} vertices but {normals} normals")]
    NormalCount {
        vertices: MeshVertexCount,
        normals: usize,
    },
    #[error("surface triangle {triangle}, corner {corner}, references vertex {index} outside {vertices} vertices")]
    VertexIndex {
        triangle: usize,
        corner: usize,
        index: u32,
        vertices: MeshVertexCount,
    },
}

impl TryFrom<&Surface> for SurfaceRenderLayout {
    type Error = SurfaceRenderError;

    fn try_from(surface: &Surface) -> SurfaceRenderResult<Self> {
        let vertices = MeshVertexCount::try_from(surface.positions.len())
            .map_err(SurfaceRenderError::VertexCount)?;
        let triangles = MeshTriangleCount::try_from(surface.triangles.len())
            .map_err(SurfaceRenderError::TriangleCount)?;
        let corners = triangles
            .checked_corners()
            .ok_or(SurfaceRenderError::CornerCount { triangles })?;
        if surface.normals.len() != usize::from(vertices) {
            return Err(SurfaceRenderError::NormalCount {
                vertices,
                normals: surface.normals.len(),
            });
        }
        for (triangle, indices) in surface.triangles.iter().enumerate() {
            for (corner, &index) in indices.iter().enumerate() {
                if index >= vertices.native_elements() {
                    return Err(SurfaceRenderError::VertexIndex {
                        triangle,
                        corner,
                        index,
                        vertices,
                    });
                }
            }
        }
        Ok(Self {
            vertices,
            triangles,
            corners,
        })
    }
}

impl SurfaceRenderLayout {
    pub fn vertices(self) -> MeshVertexCount {
        self.vertices
    }

    pub fn triangles(self) -> MeshTriangleCount {
        self.triangles
    }

    pub fn corners(self) -> MeshCornerCount {
        self.corners
    }
}
