//! Native GPU words and invocation requests for mesh populations.

use super::{MeshCornerCount, MeshTriangleCount, MeshVertexCount};
use fabelgeist_gpu::prelude::{InvocationCount, PassParameter};

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
