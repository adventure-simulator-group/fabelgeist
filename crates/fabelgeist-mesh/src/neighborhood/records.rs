//! Vertex-neighbor admission and the existing fixed-stride device records.

use crate::topology::{DrawEdge, MeshEdges, MeshLineUpload};
use crate::{DrawVertexCount, DrawVertexIndex, DrawVertexMembership};
use fabelgeist_gpu::prelude::{Buffer, BufferUpload, ReadbackError, WgpuContext};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NeighborCapacity(usize);
impl From<usize> for NeighborCapacity {
    fn from(neighbors: usize) -> Self {
        Self(neighbors)
    }
}

pub(super) struct MeshAdjacency(Vec<HashSet<DrawVertexIndex>>);
impl MeshAdjacency {
    pub(super) fn from_edges(vertices: DrawVertexCount, edges: &MeshEdges) -> Self {
        let mut rows = vec![HashSet::new(); usize::from(vertices)];
        for edge in edges.as_slice() {
            let [first, second] = edge.vertices();
            if first.membership(vertices) == DrawVertexMembership::Present
                && second.membership(vertices) == DrawVertexMembership::Present
                && first != second
            {
                rows[usize::from(first)].insert(second);
                rows[usize::from(second)].insert(first);
            }
        }
        Self(rows)
    }
}

pub(crate) struct NeighborRecords(Vec<u32>);
impl NeighborRecords {
    pub(super) fn from_adjacency(rows: &MeshAdjacency, capacity: NeighborCapacity) -> Self {
        let stride = 1 + capacity.0;
        let mut words = vec![0; rows.0.len() * stride];
        for (vertex, neighbors) in rows.0.iter().enumerate() {
            let count = neighbors.len().min(capacity.0);
            let base = vertex * stride;
            words[base] = count as u32;
            for (ordinal, &neighbor) in neighbors.iter().take(count).enumerate() {
                words[base + 1 + ordinal] = u32::from(neighbor);
            }
        }
        Self(words)
    }
    pub(crate) async fn read(
        context: &WgpuContext,
        buffer: &Buffer,
    ) -> Result<Self, ReadbackError> {
        Ok(Self(buffer.read(context).await?))
    }
    pub(crate) fn neighbor_lines(
        &self,
        capacity: NeighborCapacity,
        target: DrawVertexIndex,
    ) -> MeshLineUpload {
        let stride = 1 + capacity.0;
        let base = usize::from(target) * stride;
        let mut edges = Vec::new();
        if base + stride <= self.0.len() {
            let count = (self.0[base] as usize).min(capacity.0);
            for ordinal in 0..count {
                let neighbor = DrawVertexIndex::from(self.0[base + 1 + ordinal]);
                edges.push(DrawEdge::new(target, neighbor));
            }
        }
        MeshLineUpload::from_edges(&edges)
    }
    pub(super) fn payload(&self) -> BufferUpload<'_> {
        BufferUpload::from_elements(&self.0)
    }
}

#[cfg(test)]
mod tests;
