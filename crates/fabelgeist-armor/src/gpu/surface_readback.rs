//! Named tickets for an armor surface's checked native vertex prefixes.

use super::{GenerateError, Staged, StagedResults, Staging};
use fabelgeist_compute::VertexNormals;
use fabelgeist_gpu::prelude::Buffer;

pub(super) struct SurfaceSlots {
    status: Staged,
    positions: Staged,
    normals: Staged,
}

/// Host attributes decoded from the same logical vertex population.
pub(super) struct SurfaceVertices {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
}

impl SurfaceSlots {
    pub(super) fn stage<'a>(
        staging: &mut Staging<'a>,
        positions: &'a Buffer,
        normals: &'a VertexNormals,
    ) -> Self {
        Self {
            status: staging.stage(&normals.status),
            positions: staging.stage(positions),
            normals: staging.stage(&normals.normals),
        }
    }

    /// `count` is the host slice length at this native decoding boundary.
    pub(super) fn read(
        &self,
        results: &StagedResults,
        count: usize,
    ) -> Result<SurfaceVertices, GenerateError> {
        if results.status(&self.status)? != 0 {
            return Err(GenerateError::Degenerate);
        }
        Ok(SurfaceVertices {
            positions: results.prefix(&self.positions, count)?,
            normals: results.prefix(&self.normals, count)?,
        })
    }
}
