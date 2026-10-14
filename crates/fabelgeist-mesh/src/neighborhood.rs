use crate::PrimitiveTopology;
use anyhow::Result;
use fabelgeist_gpu::data::gpu::buffer::{Buffer, BufferDefinition};
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_gpu::prelude::BufferUpload;
use std::collections::HashSet;

pub struct MeshNeighborhood {
    pub buffer: Buffer,
    pub max_neighbors: u32,
}

impl MeshNeighborhood {
    pub async fn build(
        context: &WgpuContext,
        vertex_count: u32,
        topology: PrimitiveTopology,
        index_buffer: Option<&Buffer>,
        max_neighbors: usize,
    ) -> Result<Buffer> {
        let indices = match index_buffer {
            Some(buf) => buf.read::<u32>(context).await?,
            None => (0..vertex_count).collect::<Vec<u32>>(),
        };

        let mut adjacency: Vec<HashSet<u32>> = vec![HashSet::new(); vertex_count as usize];

        let add_edge = |adj: &mut Vec<HashSet<u32>>, u: u32, v: u32| {
            if u < vertex_count && v < vertex_count && u != v {
                adj[u as usize].insert(v);
                adj[v as usize].insert(u);
            }
        };

        match topology {
            PrimitiveTopology::PointList => {}
            PrimitiveTopology::LineList => {
                for chunk in indices.as_chunks::<2>().0 {
                    add_edge(&mut adjacency, chunk[0], chunk[1]);
                }
            }
            PrimitiveTopology::LineStrip => {
                for i in 0..indices.len().saturating_sub(1) {
                    add_edge(&mut adjacency, indices[i], indices[i + 1]);
                }
            }
            PrimitiveTopology::TriangleList => {
                for chunk in indices.as_chunks::<3>().0 {
                    add_edge(&mut adjacency, chunk[0], chunk[1]);
                    add_edge(&mut adjacency, chunk[1], chunk[2]);
                    add_edge(&mut adjacency, chunk[2], chunk[0]);
                }
            }
            PrimitiveTopology::TriangleStrip => {
                for i in 0..indices.len().saturating_sub(2) {
                    add_edge(&mut adjacency, indices[i], indices[i + 1]);
                    add_edge(&mut adjacency, indices[i + 1], indices[i + 2]);
                    add_edge(&mut adjacency, indices[i], indices[i + 2]);
                }
            }
        }

        let mut buffer_data = vec![0u32; vertex_count as usize * (1 + max_neighbors)];
        for (i, neighbors_set) in adjacency.iter().enumerate() {
            let count = neighbors_set.len().min(max_neighbors);

            let base_offset = i * (1 + max_neighbors);
            buffer_data[base_offset] = count as u32;

            for (j, &neighbor) in neighbors_set.iter().take(count).enumerate() {
                buffer_data[base_offset + 1 + j] = neighbor;
            }
        }

        Buffer::from_upload(
            context,
            BufferUpload::from_elements(&buffer_data),
            BufferDefinition::storage().with_label(("Vertex Neighbors").into()),
        )
        .map_err(anyhow::Error::from)
    }
}
