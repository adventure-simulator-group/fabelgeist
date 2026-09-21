//! Device buffers one underlayer fit works in.

use adventuresim_armor_model::ArmorGpu;
use adventuresim_armor_model::gpu::device_error;
use anyhow::Result;
use fabelgeist_compute::{RadixSort, SortScratch};
use fabelgeist_gpu::prelude::Buffer;

use super::gap::{BUCKETS, CELLS_PER_FACE};
use super::plan::{CutPlan, incidence};
use super::sweep::WORKLIST_HEADER;

/// Scratch shared by every stage of a fit. Stages run in submission order,
/// so each may reuse what the one before it has finished with.
pub(super) struct Workspace<'a> {
    pub gpu: &'a ArmorGpu,
    pub plan: &'a CutPlan,
    pub vertex_count: u32,
    pub face_count: u32,
    pub faces: Buffer,
    /// [`incidence`] of the body's faces.
    pub incidence: Buffer,
    pub table: Buffer,
    pub sources: Buffer,
    /// Fit failures; see `wgsl::STATUS`.
    pub status: Buffer,
    pub sort: RadixSort,
    pub sort_scratch: SortScratch,
    /// Sort keys and payloads: weld hashes, then grid cells of triangles.
    pub keys: Buffer,
    pub values: Buffer,
    /// Rounded positions of the weld.
    pub quantized: Buffer,
    /// First and one-past-last sorted slot of each grid bucket.
    pub ranges: Buffer,
    /// Count, then triangles too large for the grid.
    pub overflow: Buffer,
    /// A realization's room per vertex, then its room at the start of a sweep.
    pub rooms: Buffer,
    /// The prism sweeps' candidates and verdicts; see [`super::sweep`].
    pub worklist: Buffer,
    /// Face-normal constraint sets: the realization's own, then the samples'.
    pub constraints: Buffer,
    /// The frozen layer-stack compression, per body vertex.
    pub compression: Buffer,
}

impl<'a> Workspace<'a> {
    pub fn new(
        gpu: &'a ArmorGpu,
        plan: &'a CutPlan,
        faces: &[[u32; 3]],
        vertex_count: usize,
        constraint_sets: u32,
    ) -> Result<Self> {
        let face_count = faces.len() as u32;
        let vertices = vertex_count as u64;
        let pairs = face_count * CELLS_PER_FACE;
        let capacity = pairs.max(vertex_count as u32);
        Ok(Self {
            gpu,
            plan,
            vertex_count: vertex_count as u32,
            face_count,
            faces: gpu.upload(faces)?,
            incidence: gpu.upload(&incidence(vertex_count, faces))?,
            table: gpu.upload(&plan.table)?,
            sources: gpu.upload(&plan.sources)?,
            status: gpu.scratch(4, "underlayer status")?,
            sort: RadixSort::with_cache(gpu.context(), gpu.cache()).map_err(device_error)?,
            sort_scratch: SortScratch::new(gpu.context(), capacity).map_err(device_error)?,
            keys: gpu.scratch(capacity as u64 * 4, "underlayer sort keys")?,
            values: gpu.scratch(capacity as u64 * 4, "underlayer sort values")?,
            quantized: gpu.scratch(vertices * 12, "underlayer weld keys")?,
            ranges: gpu.scratch(BUCKETS as u64 * 8, "underlayer grid buckets")?,
            overflow: gpu.scratch((face_count as u64 + 1) * 4, "underlayer large faces")?,
            rooms: gpu.scratch(vertices * 8, "underlayer rooms")?,
            worklist: gpu.scratch(
                (WORKLIST_HEADER as u64 + face_count as u64 * 4) * 4,
                "underlayer prism worklist",
            )?,
            constraints: gpu.scratch(
                constraint_sets.max(1) as u64 * face_count as u64 * 16,
                "underlayer face constraints",
            )?,
            compression: gpu.upload(&vec![f32::MAX; vertex_count])?,
        })
    }
}
