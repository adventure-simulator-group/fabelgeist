//! Owned normal buffers and the mesh populations that size them.

use super::{DEGENERATE_TRIANGLE, VANISHED_NORMAL};
use crate::prelude::*;
use fabelgeist_gpu::prelude::{BufferByteLength, BufferUse};
use fabelgeist_mesh::{MeshCornerCount, MeshTriangleCount, MeshVertexCount};
use std::num::TryFromIntError;

/// A normal layout that exceeds native addresses or its available buffers.
#[derive(Debug, thiserror::Error)]
pub enum NormalLayoutError {
    #[error("VertexNormals: {vertices} vertices and {triangles} triangles exceed {vertex_capacity} and {triangle_capacity}")]
    Capacity {
        vertices: MeshVertexCount,
        triangles: MeshTriangleCount,
        vertex_capacity: MeshVertexCount,
        triangle_capacity: MeshTriangleCount,
    },
    #[error("normal positions require {required} bytes, but only {available} are available")]
    Positions {
        required: BufferByteLength,
        available: BufferByteLength,
    },
    #[error("normal triangle indices require {required} bytes, but only {available} are available")]
    TriangleIndices {
        required: BufferByteLength,
        available: BufferByteLength,
    },
    #[error("normal corners for {triangles} triangles exceed the native address space")]
    Corners { triangles: MeshTriangleCount },
    #[error("normal words for {vertices} vertices exceed native addresses")]
    VertexWords {
        vertices: MeshVertexCount,
        #[source]
        source: TryFromIntError,
    },
    #[error("normal contribution words for {corners} corners exceed native addresses")]
    CornerWords {
        corners: MeshCornerCount,
        #[source]
        source: TryFromIntError,
    },
}

/// Buffers for the normals of meshes up to a vertex and triangle capacity.
#[derive(Clone, Debug)]
pub struct VertexNormals {
    /// Unit normals as `f32` triples. A vanished normal is written as zero.
    pub(super) normals: Buffer,
    /// [`DEGENERATE_TRIANGLE`] and [`VANISHED_NORMAL`] bits, accumulated
    /// until cleared.
    pub(super) status: Buffer,
    pub(super) contributions: Buffer,
    pub(super) keys: Buffer,
    pub(super) corners: Buffer,
    pub(super) starts: Buffer,
    pub(super) ends: Buffer,
    pub(super) sort: SortScratch,
    pub(super) vertex_capacity: MeshVertexCount,
    pub(super) triangle_capacity: MeshTriangleCount,
}

impl VertexNormals {
    /// Unit normals as native float triples, without replacing their allocation.
    pub fn normals(&self) -> &Buffer {
        &self.normals
    }

    /// Accumulated native status bits, without replacing their allocation.
    pub fn status(&self) -> &Buffer {
        &self.status
    }

    pub fn new(
        context: &WgpuContext,
        vertex_capacity: MeshVertexCount,
        triangle_capacity: MeshTriangleCount,
    ) -> Result<Self> {
        let vertex_capacity = vertex_capacity.max(MeshVertexCount::from(1));
        let triangle_capacity = triangle_capacity.max(MeshTriangleCount::from(1));
        let corner_capacity = triangle_capacity.checked_corners()
            .ok_or(NormalLayoutError::Corners { triangles: triangle_capacity })?;
        u32::try_from(u64::from(vertex_capacity.native_elements()) * 3)
            .map_err(|source| NormalLayoutError::VertexWords {
                vertices: vertex_capacity, source,
            })?;
        u32::try_from(u64::from(corner_capacity.native_elements()) * 3)
            .map_err(|source| NormalLayoutError::CornerWords {
                corners: corner_capacity, source,
            })?;
        let storage = BufferDefinition::storage().with_usage(BufferUse::CopySource);
        let buffer = |bytes: BufferByteLength, label: &str| {
            Buffer::new(
                context,
                bytes,
                storage.clone().with_label((label).into()),
            )
        };
        Ok(Self {
            normals: buffer(vertex_capacity.packed_vec3_bytes(), "vertex normals")?,
            status: buffer((size_of::<u32>() as u64).into(), "vertex normal status")?,
            contributions: buffer(corner_capacity.packed_vec3_bytes(), "corner normals")?,
            keys: buffer(corner_capacity.word_bytes(), "corner vertices")?,
            corners: buffer(corner_capacity.word_bytes(), "corners by vertex")?,
            starts: buffer(vertex_capacity.word_bytes(), "vertex corner starts")?,
            ends: buffer(vertex_capacity.word_bytes(), "vertex corner ends")?,
            sort: SortScratch::new(context, corner_capacity.into())?,
            vertex_capacity,
            triangle_capacity,
        })
    }

    pub(super) fn check_inputs(
        &self,
        positions: &Buffer,
        triangles: &Buffer,
        vertex_count: MeshVertexCount,
        triangle_count: MeshTriangleCount,
    ) -> Result<MeshCornerCount> {
        if vertex_count > self.vertex_capacity || triangle_count > self.triangle_capacity {
            return Err(NormalLayoutError::Capacity {
                vertices: vertex_count,
                triangles: triangle_count,
                vertex_capacity: self.vertex_capacity,
                triangle_capacity: self.triangle_capacity,
            }
            .into());
        }
        let corners = triangle_count
            .checked_corners()
            .ok_or(NormalLayoutError::Corners {
                triangles: triangle_count,
            })?;
        let required = vertex_count.packed_vec3_bytes();
        let available = positions.size.min(positions.buffer.size().into());
        if available < required {
            return Err(NormalLayoutError::Positions { required, available }.into());
        }
        let required = triangle_count.index_bytes();
        let available = triangles.size.min(triangles.buffer.size().into());
        if available < required {
            return Err(NormalLayoutError::TriangleIndices { required, available }.into());
        }
        Ok(corners)
    }

    /// Read the normals of the first `count` vertices, and the status bits.
    /// Stalls on the device.
    pub async fn read(&self, context: &WgpuContext, count: MeshVertexCount) -> Result<(Vec<[f32; 3]>, u32)> {
        let normals: Vec<[f32; 3]> = self.normals.read(context).await?;
        let status: Vec<u32> = self.status.read(context).await?;
        Ok((
            normals[..usize::from(count.min(self.vertex_capacity))].to_vec(),
            status[0],
        ))
    }
}

