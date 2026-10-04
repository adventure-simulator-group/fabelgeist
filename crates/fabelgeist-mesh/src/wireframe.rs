//! Canonical wire edges and target-neighbor lines in draw address space.
use crate::neighborhood::{NeighborCapacity, NeighborRecords};
use crate::topology::{MeshEdges, MeshLineUpload};
use crate::{
    DerivedMeshBuffer, DrawVertexCount, DrawVertexIndex, MeshTopologyTransferError,
    PrimitiveTopology,
};
use fabelgeist_gpu::prelude::{Buffer, WgpuContext};

pub struct MeshWireframe;
impl MeshWireframe {
    pub async fn build_wireframe_indices(
        context: &WgpuContext,
        vertex_count: DrawVertexCount,
        topology: PrimitiveTopology,
        index_buffer: Option<&Buffer>,
    ) -> Result<Buffer, MeshTopologyTransferError> {
        let edges = MeshEdges::read(context, vertex_count, topology, index_buffer)
            .await
            .map_err(MeshTopologyTransferError::ReadIndices)?;
        let lines = MeshLineUpload::wireframe(vertex_count, &edges);
        DerivedMeshBuffer::Wireframe.upload(context, lines.payload())
    }
    pub async fn build_neighbor_lines_indices(
        context: &WgpuContext,
        vertex_count: DrawVertexCount,
        topology: PrimitiveTopology,
        index_buffer: Option<&Buffer>,
        target_vertex: DrawVertexIndex,
    ) -> Result<Buffer, MeshTopologyTransferError> {
        let edges = MeshEdges::read(context, vertex_count, topology, index_buffer)
            .await
            .map_err(MeshTopologyTransferError::ReadIndices)?;
        let upload = MeshLineUpload::neighbors(vertex_count, &edges, target_vertex);
        DerivedMeshBuffer::NeighborLines.upload(context, upload.payload())
    }
    pub async fn build_neighbor_lines_from_neighborhood(
        context: &WgpuContext,
        neighbors_buf: &Buffer,
        max_neighbors: NeighborCapacity,
        target_vertex: DrawVertexIndex,
    ) -> Result<Buffer, MeshTopologyTransferError> {
        let records = NeighborRecords::read(context, neighbors_buf)
            .await
            .map_err(MeshTopologyTransferError::ReadNeighbors)?;
        let lines = records.neighbor_lines(max_neighbors, target_vertex);
        DerivedMeshBuffer::NeighborLines.upload(context, lines.payload())
    }
}
