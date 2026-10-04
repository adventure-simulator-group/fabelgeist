//! Ordered topology edges and their terminal line-index encoding.

use crate::{DrawVertexCount, DrawVertexIndex, DrawVertexMembership, PrimitiveTopology};
use fabelgeist_gpu::prelude::{Buffer, BufferUpload, ReadbackError, WgpuContext};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct DrawEdge(DrawVertexIndex, DrawVertexIndex);
impl DrawEdge {
    pub(crate) fn new(first: DrawVertexIndex, second: DrawVertexIndex) -> Self {
        Self(first, second)
    }
    pub(crate) fn vertices(self) -> [DrawVertexIndex; 2] {
        [self.0, self.1]
    }
    pub(crate) fn canonical(self) -> Self {
        Self(self.0.min(self.1), self.0.max(self.1))
    }
}

pub(crate) struct MeshEdges(Vec<DrawEdge>);
impl MeshEdges {
    pub(crate) async fn read(
        context: &WgpuContext,
        vertices: DrawVertexCount,
        topology: PrimitiveTopology,
        index_buffer: Option<&Buffer>,
    ) -> Result<Self, ReadbackError> {
        let indices = match index_buffer {
            Some(buffer) => buffer
                .read::<u32>(context)
                .await?
                .into_iter()
                .map(DrawVertexIndex::from)
                .collect(),
            None => vertices.sequential_indices(),
        };
        Ok(Self::from_indices(indices, topology))
    }
    pub(crate) fn from_indices(indices: Vec<DrawVertexIndex>, topology: PrimitiveTopology) -> Self {
        let mut edges = Vec::new();
        match topology {
            PrimitiveTopology::PointList => {}
            PrimitiveTopology::LineList => {
                for pair in indices.as_chunks::<2>().0 {
                    edges.push(DrawEdge(pair[0], pair[1]));
                }
            }
            PrimitiveTopology::LineStrip => {
                for pair in indices.windows(2) {
                    edges.push(DrawEdge(pair[0], pair[1]));
                }
            }
            PrimitiveTopology::TriangleList => {
                for triangle in indices.as_chunks::<3>().0 {
                    edges.push(DrawEdge(triangle[0], triangle[1]));
                    edges.push(DrawEdge(triangle[1], triangle[2]));
                    edges.push(DrawEdge(triangle[2], triangle[0]));
                }
            }
            PrimitiveTopology::TriangleStrip => {
                for triangle in indices.windows(3) {
                    edges.push(DrawEdge(triangle[0], triangle[1]));
                    edges.push(DrawEdge(triangle[1], triangle[2]));
                    edges.push(DrawEdge(triangle[0], triangle[2]));
                }
            }
        }
        Self(edges)
    }
    pub(crate) fn as_slice(&self) -> &[DrawEdge] {
        &self.0
    }
}

/// Serialized line-index words; the empty line sentinel belongs to this ABI.
pub(crate) struct MeshLineUpload(Vec<u32>);
impl MeshLineUpload {
    pub(crate) fn wireframe(vertex_count: DrawVertexCount, edges: &MeshEdges) -> Self {
        let mut unique = HashSet::new();
        for &edge in edges.as_slice() {
            let [first, second] = edge.vertices();
            if first.membership(vertex_count) == DrawVertexMembership::Present
                && second.membership(vertex_count) == DrawVertexMembership::Present
                && first != second
            {
                unique.insert(edge.canonical());
            }
        }
        Self::from_edges(&unique.into_iter().collect::<Vec<_>>())
    }
    pub(crate) fn neighbors(
        vertex_count: DrawVertexCount,
        edges: &MeshEdges,
        target_vertex: DrawVertexIndex,
    ) -> Self {
        let mut neighbors = HashSet::new();
        for edge in edges.as_slice() {
            let [first, second] = edge.vertices();
            if first == target_vertex
                && second.membership(vertex_count) == DrawVertexMembership::Present
                && first != second
            {
                neighbors.insert(second);
            } else if second == target_vertex
                && first.membership(vertex_count) == DrawVertexMembership::Present
                && first != second
            {
                neighbors.insert(first);
            }
        }
        let mut lines = Vec::new();
        for neighbor in neighbors {
            lines.push(DrawEdge::new(target_vertex, neighbor));
        }
        Self::from_edges(&lines)
    }

    pub(crate) fn from_edges(edges: &[DrawEdge]) -> Self {
        let mut words = Vec::with_capacity(edges.len() * 2);
        for edge in edges {
            words.extend(edge.vertices().map(u32::from));
        }
        if words.is_empty() {
            words.extend([0, 0]);
        }
        Self(words)
    }
    pub(crate) fn payload(&self) -> BufferUpload<'_> {
        BufferUpload::from_elements(&self.0)
    }
}

#[cfg(test)]
mod tests;
