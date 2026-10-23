//! Native GPU words and invocation requests for mesh populations.

use super::{MeshCornerCount, MeshTriangleCount, MeshVertexCount};
use fabelgeist_gpu::prelude::{BufferByteLength, InvocationCount, PassParameter};

impl MeshVertexCount {
    /// Tightly packed XYZ `f32` triples, without native `vec3` array padding.
    pub fn packed_vec3_bytes(self) -> BufferByteLength {
        (u64::from(self.0) * size_of::<[f32; 3]>() as u64).into()
    }

    /// One native `u32` word for each vertex, such as a corner-range endpoint.
    pub fn word_bytes(self) -> BufferByteLength {
        (u64::from(self.0) * size_of::<u32>() as u64).into()
    }
}

impl MeshTriangleCount {
    /// Three tightly packed native `u32` vertex indices for each triangle.
    pub fn index_bytes(self) -> BufferByteLength {
        (u64::from(self.0) * size_of::<[u32; 3]>() as u64).into()
    }
}

impl MeshCornerCount {
    /// Tightly packed XYZ `f32` contributions, without `vec3` array padding.
    pub fn packed_vec3_bytes(self) -> BufferByteLength {
        (u64::from(self.0) * size_of::<[f32; 3]>() as u64).into()
    }

    /// One native `u32` word for each corner, such as its vertex index.
    pub fn word_bytes(self) -> BufferByteLength {
        (u64::from(self.0) * size_of::<u32>() as u64).into()
    }
}

impl From<MeshVertexCount> for InvocationCount {
    fn from(vertices: MeshVertexCount) -> Self {
        Self::from(vertices.0)
    }
}

impl From<MeshTriangleCount> for InvocationCount {
    fn from(triangles: MeshTriangleCount) -> Self {
        Self::from(triangles.0)
    }
}

impl From<MeshCornerCount> for InvocationCount {
    fn from(corners: MeshCornerCount) -> Self {
        Self::from(corners.0)
    }
}

impl From<MeshVertexCount> for PassParameter {
    fn from(vertices: MeshVertexCount) -> Self {
        Self::Unsigned(vertices.0)
    }
}

impl From<MeshTriangleCount> for PassParameter {
    fn from(triangles: MeshTriangleCount) -> Self {
        Self::Unsigned(triangles.0)
    }
}

impl From<MeshCornerCount> for PassParameter {
    fn from(corners: MeshCornerCount) -> Self {
        Self::Unsigned(corners.0)
    }
}
