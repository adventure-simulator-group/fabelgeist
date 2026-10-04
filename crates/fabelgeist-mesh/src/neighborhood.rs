//! Derived vertex adjacency, with capacity distinct from draw addresses.
mod records;
use crate::topology::MeshEdges;
use crate::{DerivedMeshBuffer, DrawVertexCount, MeshTopologyTransferError, PrimitiveTopology};
use fabelgeist_gpu::prelude::{Buffer, WgpuContext};
use records::MeshAdjacency;
pub use records::NeighborCapacity;
pub(crate) use records::NeighborRecords;

pub struct MeshNeighborhood {
    pub buffer: Buffer,
    pub max_neighbors: NeighborCapacity,
}
impl MeshNeighborhood {
    pub async fn build(
        context: &WgpuContext,
        vertex_count: DrawVertexCount,
        topology: PrimitiveTopology,
        index_buffer: Option<&Buffer>,
        max_neighbors: NeighborCapacity,
    ) -> Result<Buffer, MeshTopologyTransferError> {
        let edges = MeshEdges::read(context, vertex_count, topology, index_buffer)
            .await
            .map_err(MeshTopologyTransferError::ReadIndices)?;
        let rows = MeshAdjacency::from_edges(vertex_count, &edges);
        let records = NeighborRecords::from_adjacency(&rows, max_neighbors);
        DerivedMeshBuffer::Neighborhood.upload(context, records.payload())
    }
}
