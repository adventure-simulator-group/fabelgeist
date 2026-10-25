//! Target selection keeps candidate buffers paired with candidate populations.

use super::{CandidateCount, QueryLayoutError};
use fabelgeist_gpu::prelude::*;
use fabelgeist_mesh::{MeshTriangleCount, MeshVertexCount};

/// Point records searched by the nearest-point query.
#[derive(Clone, Copy, Debug)]
pub struct PointTargets<'a> {
    /// Packed native f32 triples; indices must address this allocation.
    pub positions: &'a Buffer,
    pub selection: PointSelection<'a>,
}

#[derive(Clone, Copy, Debug)]
pub enum PointSelection<'a> {
    All(MeshVertexCount),
    Candidates { indices: &'a Buffer, count: CandidateCount },
}

/// Triangle records searched by surface and ray queries.
#[derive(Clone, Copy, Debug)]
pub struct TriangleTargets<'a> {
    pub positions: &'a Buffer,
    /// Native u32 triples indexing positions; membership is caller-owned.
    pub triangles: &'a Buffer,
    pub selection: TriangleSelection<'a>,
}

#[derive(Clone, Copy, Debug)]
pub enum TriangleSelection<'a> {
    All(MeshTriangleCount),
    Candidates { indices: &'a Buffer, count: CandidateCount },
}

impl PointSelection<'_> {
    pub(super) fn bind(self, parameters: &mut PassParameters, unused: &Buffer) -> Result<()> {
        match self {
            Self::All(vertices) => {
                u32::try_from(u64::from(vertices.native_elements()) * 3)
                    .map_err(|source| QueryLayoutError::Points { vertices, source })?;
                bind_all(parameters, vertices.into(), unused);
            }
            Self::Candidates { indices, count } => bind_candidates(parameters, count, indices),
        }
        Ok(())
    }
}

impl TriangleSelection<'_> {
    pub(super) fn bind(self, parameters: &mut PassParameters, unused: &Buffer) -> Result<()> {
        match self {
            Self::All(triangles) => {
                u32::try_from(u64::from(triangles.native_elements()) * 3)
                    .map_err(|source| QueryLayoutError::Triangles { triangles, source })?;
                bind_all(parameters, triangles.into(), unused);
            }
            Self::Candidates { indices, count } => bind_candidates(parameters, count, indices),
        }
        Ok(())
    }
}

// These functions encode the actual reflected shader fields. The application
// selection has already established the population's role before this port.
fn bind_all(parameters: &mut PassParameters, count: PassParameter, unused: &Buffer) {
    parameters.insert("target_count", count);
    parameters.insert("use_candidates", 0u32);
    parameters.insert("candidates", unused.clone());
}

fn bind_candidates(parameters: &mut PassParameters, count: CandidateCount, indices: &Buffer) {
    parameters.insert("target_count", count);
    parameters.insert("use_candidates", 1u32);
    parameters.insert("candidates", indices.clone());
}
