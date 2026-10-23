//! GPU advancing-front extraction for implicit surfaces.
//!
//! On a regular distance-field lattice, every sign-changing lattice edge is an
//! active front.  The four incident cells supply projected vertex candidates;
//! joining those candidates closes the front with two triangles.  Ownership by
//! lattice edge makes the candidates non-overlapping, which is the structured
//! grid equivalent of SAFT's collision/selection phase.
//!
//! The underlying surface-front pipeline is shared with dual contouring: gather
//! finds active cells/fronts, scan allocates compact output ranges, and stream
//! projects candidates and emits triangles.  Keeping this specialization as a
//! thin facade avoids maintaining a second copy of the sizeable WGSL kernels.

use crate::dual_contouring::{DualContouring, DualContouringDefinition};
use crate::{ExtractionGrid, ExtractionIndexCapacity, IndexedExtraction};
use anyhow::Result;
use fabelgeist_mesh::MeshVertexCount;
use fabelgeist_gpu::data::gpu::resource::GpuResource;
use fabelgeist_gpu::globals::WgpuContext;

pub struct AdvancingFront;

impl AdvancingFront {
    /// Extract an indexed triangle mesh from a 3D distance field.
    ///
    /// The same `max_vertices` budget bounds projected front candidates and
    /// index slots. Only complete two-triangle quads are included in the draw
    /// count; quads touching a suppressed candidate are omitted.
    #[expect(
        clippy::too_many_arguments,
        reason = "surface extraction passes grid, threshold, output bounds and world transform to the GPU"
    )]
    pub fn execute(
        context: &WgpuContext,
        definition: &DualContouringDefinition,
        sdf: &GpuResource,
        grid: ExtractionGrid,
        threshold: f32,
        max_vertices: MeshVertexCount,
        scale: (f32, f32, f32),
        offset: (f32, f32, f32),
    ) -> Result<IndexedExtraction> {
        let max_indices = ExtractionIndexCapacity::from_vertex_budget(max_vertices)?;
        DualContouring::execute_advancing_front(
            context,
            definition,
            sdf,
            grid,
            threshold,
            max_vertices,
            max_indices,
            scale,
            offset,
        )
    }
}
