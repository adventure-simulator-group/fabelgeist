//! Owned normal buffers and the mesh populations that size them.

use super::{DEGENERATE_TRIANGLE, VANISHED_NORMAL};
use crate::prelude::*;
use fabelgeist_gpu::prelude::{BufferByteLength, BufferUse};
use fabelgeist_mesh::{MeshCornerCount, MeshTriangleCount, MeshVertexCount};
use std::num::TryFromIntError;

/// A mesh population whose native normal layout cannot be addressed.
#[derive(Debug, thiserror::Error)]
pub enum NormalLayoutError {
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
            normals: buffer((u64::from(vertex_capacity.native_elements()) * 12).into(), "vertex normals")?,
            status: buffer(4u64.into(), "vertex normal status")?,
            contributions: buffer((u64::from(corner_capacity.native_elements()) * 12).into(), "corner normals")?,
            keys: buffer((u64::from(corner_capacity.native_elements()) * 4).into(), "corner vertices")?,
            corners: buffer((u64::from(corner_capacity.native_elements()) * 4).into(), "corners by vertex")?,
            starts: buffer((u64::from(vertex_capacity.native_elements()) * 4).into(), "vertex corner starts")?,
            ends: buffer((u64::from(vertex_capacity.native_elements()) * 4).into(), "vertex corner ends")?,
            sort: SortScratch::new(context, corner_capacity.into())?,
            vertex_capacity,
            triangle_capacity,
        })
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

