//! Branch meshes swept on the GPU.
//!
//! The paper builds branch cylinders in a geometry shader from the skeleton's
//! node attributes. The equivalent here is a compute pass (`fabelgeist-tube`) over
//! the segment list, writing the vertex and index buffers a
//! [`GpuMesh`] draws from -- one dispatch per mesh, all in one submit.

use anyhow::Result;
use fabelgeist_compute::{KernelBatch, KernelCache};
use fabelgeist_gpu::data::gpu::buffer::{Buffer, BufferDefinition};
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_mesh::GpuMesh;
use fabelgeist_tube::gpu::TubeMesher;

use crate::ecosystem::Ecosystem;
use fabelgeist_gpu::prelude::BufferUpload;

/// Segments per mesh. A storage binding may not exceed 2 GiB, and a mesh's
/// positions take `14 * 12` bytes a segment at six sides; this keeps well
/// under the limit whatever the side count.
pub const SEGMENTS_PER_MESH: usize = 4 << 20;

/// One sweep's buffers, kept and reused step to step.
struct Slot {
    mesher: TubeMesher,
    segments: Option<Buffer>,
}

/// Sweeps an ecosystem's branches: a mesh per species, split into chunks of
/// at most [`SEGMENTS_PER_MESH`] segments.
pub struct BranchMesher {
    cache: KernelCache,
    sides: u32,
    slots: Vec<Slot>,
}

/// The meshes of one species.
pub struct SpeciesBranches {
    pub species: usize,
    pub meshes: Vec<GpuMesh>,
}

impl BranchMesher {
    pub fn new(sides: u32) -> Self {
        Self {
            cache: KernelCache::new(),
            sides,
            slots: Vec::new(),
        }
    }

    /// Sweep every species' branches. The meshes share storage with the next
    /// call's; draw them before meshing again.
    pub fn mesh(
        &mut self,
        context: &WgpuContext,
        ecosystem: &Ecosystem,
    ) -> Result<Vec<SpeciesBranches>> {
        let per_species = ecosystem.branch_segments();
        let mut batch = KernelBatch::labelled(context, "botany branches");
        let mut out = Vec::with_capacity(per_species.len());
        let mut used = 0;
        for (species, segments) in per_species.iter().enumerate() {
            let mut meshes = Vec::new();
            for chunk in segments.chunks(SEGMENTS_PER_MESH) {
                if used == self.slots.len() {
                    self.slots.push(Slot {
                        mesher: TubeMesher::new(context, &self.cache, self.sides)?,
                        segments: None,
                    });
                }
                let slot = &mut self.slots[used];
                used += 1;
                let bytes = std::mem::size_of_val(chunk) as u64;
                if slot
                    .segments
                    .as_ref()
                    .is_none_or(|b| b.buffer.size() < bytes)
                {
                    let grown = bytes.max(
                        slot.segments
                            .as_ref()
                            .map_or(0, |b| b.buffer.size() * 3 / 2),
                    );
                    slot.segments = Some(Buffer::new(
                        context,
                        (grown).into(),
                        BufferDefinition::storage(),
                    )?);
                }
                let mut buffer = slot.segments.clone().expect("allocated above");
                buffer.size = (bytes).into();
                buffer.write(context, BufferUpload::from_elements(chunk))?;
                meshes.push(slot.mesher.record(
                    context,
                    &mut batch,
                    &buffer,
                    chunk.len() as u32,
                )?);
            }
            out.push(SpeciesBranches { species, meshes });
        }
        batch.submit();
        Ok(out)
    }
}

/// Segments in a mesh swept with `sides` sides.
pub fn segment_count(mesh: &GpuMesh, sides: u32) -> u32 {
    mesh.vertex_count / fabelgeist_tube::vertices_per_segment(sides)
}
