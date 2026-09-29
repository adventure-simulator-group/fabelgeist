use crate::PrimitiveTopology;
use anyhow::Result;
use fabelgeist_gpu::data::gpu::buffer::{Buffer, BufferDefinition};
use fabelgeist_gpu::globals::WgpuContext;
use std::collections::HashSet;

pub struct MeshWireframe;

impl MeshWireframe {
    pub async fn build_wireframe_indices(
        context: &WgpuContext,
        vertex_count: u32,
        topology: PrimitiveTopology,
        index_buffer: Option<&Buffer>,
    ) -> Result<Buffer> {
        let indices = match index_buffer {
            Some(buf) => buf.read::<u32>(context).await?,
            None => (0..vertex_count).collect::<Vec<u32>>(),
        };

        let mut unique_edges = HashSet::new();

        let add_edge = |edges: &mut HashSet<(u32, u32)>, u: u32, v: u32| {
            if u < vertex_count && v < vertex_count && u != v {
                let min = u.min(v);
                let max = u.max(v);
                edges.insert((min, max));
            }
        };

        match topology {
            PrimitiveTopology::PointList => {}
            PrimitiveTopology::LineList => {
                for chunk in indices.chunks_exact(2) {
                    add_edge(&mut unique_edges, chunk[0], chunk[1]);
                }
            }
            PrimitiveTopology::LineStrip => {
                for i in 0..indices.len().saturating_sub(1) {
                    add_edge(&mut unique_edges, indices[i], indices[i + 1]);
                }
            }
            PrimitiveTopology::TriangleList => {
                for chunk in indices.chunks_exact(3) {
                    add_edge(&mut unique_edges, chunk[0], chunk[1]);
                    add_edge(&mut unique_edges, chunk[1], chunk[2]);
                    add_edge(&mut unique_edges, chunk[2], chunk[0]);
                }
            }
            PrimitiveTopology::TriangleStrip => {
                for i in 0..indices.len().saturating_sub(2) {
                    add_edge(&mut unique_edges, indices[i], indices[i + 1]);
                    add_edge(&mut unique_edges, indices[i + 1], indices[i + 2]);
                    add_edge(&mut unique_edges, indices[i], indices[i + 2]);
                }
            }
        }

        let mut line_indices = Vec::with_capacity(unique_edges.len() * 2);
        for &(u, v) in &unique_edges {
            line_indices.push(u);
            line_indices.push(v);
        }

        if line_indices.is_empty() {
            line_indices.push(0);
            line_indices.push(0);
        }

        Buffer::from_slice(
            context,
            &line_indices,
            BufferDefinition::index().with_label("Wireframe Line Indices"),
        )
    }

    pub async fn build_neighbor_lines_indices(
        context: &WgpuContext,
        vertex_count: u32,
        topology: PrimitiveTopology,
        index_buffer: Option<&Buffer>,
        target_vertex: u32,
    ) -> Result<Buffer> {
        let indices = match index_buffer {
            Some(buf) => buf.read::<u32>(context).await?,
            None => (0..vertex_count).collect::<Vec<u32>>(),
        };

        let mut neighbors = HashSet::new();

        let add_edge = |n: &mut HashSet<u32>, u: u32, v: u32| {
            if u == target_vertex && v < vertex_count && u != v {
                n.insert(v);
            } else if v == target_vertex && u < vertex_count && u != v {
                n.insert(u);
            }
        };

        match topology {
            PrimitiveTopology::PointList => {}
            PrimitiveTopology::LineList => {
                for chunk in indices.chunks_exact(2) {
                    add_edge(&mut neighbors, chunk[0], chunk[1]);
                }
            }
            PrimitiveTopology::LineStrip => {
                for i in 0..indices.len().saturating_sub(1) {
                    add_edge(&mut neighbors, indices[i], indices[i + 1]);
                }
            }
            PrimitiveTopology::TriangleList => {
                for chunk in indices.chunks_exact(3) {
                    add_edge(&mut neighbors, chunk[0], chunk[1]);
                    add_edge(&mut neighbors, chunk[1], chunk[2]);
                    add_edge(&mut neighbors, chunk[2], chunk[0]);
                }
            }
            PrimitiveTopology::TriangleStrip => {
                for i in 0..indices.len().saturating_sub(2) {
                    add_edge(&mut neighbors, indices[i], indices[i + 1]);
                    add_edge(&mut neighbors, indices[i + 1], indices[i + 2]);
                    add_edge(&mut neighbors, indices[i], indices[i + 2]);
                }
            }
        }

        let mut line_indices = Vec::with_capacity(neighbors.len() * 2);
        for &v in &neighbors {
            line_indices.push(target_vertex);
            line_indices.push(v);
        }

        if line_indices.is_empty() {
            line_indices.push(0);
            line_indices.push(0);
        }

        Buffer::from_slice(
            context,
            &line_indices,
            BufferDefinition::index().with_label("Neighbor Lines Indices"),
        )
    }

    pub async fn build_neighbor_lines_from_neighborhood(
        context: &WgpuContext,
        neighbors_buf: &Buffer,
        max_neighbors: usize,
        target_vertex: u32,
    ) -> Result<Buffer> {
        let neighbors_data = neighbors_buf.read::<u32>(context).await?;

        let stride = 1 + max_neighbors;
        let base_offset = target_vertex as usize * stride;

        let mut line_indices = Vec::new();
        if base_offset + stride <= neighbors_data.len() {
            let count = neighbors_data[base_offset] as usize;
            let actual_count = count.min(max_neighbors);

            for j in 0..actual_count {
                let neighbor = neighbors_data[base_offset + 1 + j];
                line_indices.push(target_vertex);
                line_indices.push(neighbor);
            }
        }

        if line_indices.is_empty() {
            line_indices.push(0);
            line_indices.push(0);
        }

        Buffer::from_slice(
            context,
            &line_indices,
            BufferDefinition::index().with_label("Neighbor Lines Indices"),
        )
    }
}
