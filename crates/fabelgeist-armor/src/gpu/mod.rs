//! The device armor is generated on.
//!
//! [`ArmorGpu`] owns a compute context and the compiled kernels every armor
//! family shares. One is enough for a whole process: its kernels are compiled
//! once, and it may be used from any number of threads at once.

use std::sync::Arc;

use fabelgeist_compute::{
    KernelBatch, KernelCache, MeshQuery, NormalWeighting, PointTargets, QueryHits,
    VertexNormalKernels,
};
use fabelgeist_gpu::prelude::{Buffer, BufferDefinition, WgpuContext};

use crate::{GenerateError, PartFrame};

pub mod anatomy;
pub mod body;
pub mod bracer;
mod bracer_contour_wgsl;
mod bracer_wgsl;
pub mod breastplate;
pub(crate) mod chart;
pub(crate) mod close_helmet;
pub(crate) mod close_helmet_dome;
pub(crate) mod close_helmet_profile;
pub(crate) mod close_helmet_shape;
pub(crate) mod coif;
pub(crate) mod coif_drape;
pub(crate) mod coif_shape;
pub(crate) mod coord;
pub(crate) mod coord_topology;
pub(crate) mod extremities;
pub(crate) mod footwear;
pub(crate) mod garment;
pub(crate) mod garment_torso;
pub(crate) mod gorget;
pub(crate) mod helmet;
pub(crate) mod limb;
pub mod metal;
pub(crate) mod mitten;
pub(crate) mod part;
pub(crate) mod placement;
mod puff_and_slash;
pub(crate) mod recipe;
pub(crate) mod shell_plan;
pub mod staging;
pub mod wgsl;
pub use close_helmet::{CLOSE_HELMET_PROFILE_WORDS, FIT_PROFILE_WORD, record_close_helmet};
pub use close_helmet_profile::CloseHelmetProfile;
pub use coif::record_coif;
pub use coif_drape::{
    COIF_DRAPE_SECTIONS, COIF_DRAPE_WORDS, CoifDrapeProfile, CoifDrapeSection, CoifFlapDrape,
    CoifNeckDrape,
};
pub use extremities::record_extremity_armor;
pub use garment::{record_fauld, record_garment_tube, record_tassets};
pub use garment_torso::record_garment_torso;
pub use gorget::record_gorget_plates;
pub use helmet::{generate_helmet_on, record_helmet};
pub use limb::{generate_limb_armor_on, record_limb_armor};
pub use part::{BuiltPart, PartSlots};
pub use puff_and_slash::record_puff_and_slash;
pub use recipe::frame_words;
pub use staging::{Staged, StagedResults, Staging};

/// A part under construction on the device: its carriers, until
/// [`DevicePart::record_shells`] thickens them, then its final mesh.
pub struct DevicePart(part::PartBuild);

impl DevicePart {
    /// Carrier points of every shell, packed `f32` triples in shell order.
    pub fn carriers(&self) -> &Buffer {
        &self.0.carriers
    }

    /// Nonzero once any stage has found the part invalid.
    pub fn status(&self) -> &Buffer {
        &self.0.status
    }

    pub fn shell_count(&self) -> usize {
        self.0.layout().shell_count()
    }

    /// The carriers of one shell, as indices into [`DevicePart::carriers`].
    pub fn shell_carriers(&self, shell: usize) -> std::ops::Range<u32> {
        let layout = self.0.layout();
        let first = layout.first_carrier(shell);
        first..first + layout.carrier_count(shell)
    }

    /// The carrier triangles of one shell as authored, before any
    /// reflection rewinds them, indexed within the shell.
    pub fn shell_carrier_triangles(&self, shell: usize) -> &[u32] {
        self.0.layout().carrier_triangles(shell)
    }

    pub fn carrier_count(&self) -> u32 {
        self.0.layout().total_carriers()
    }

    /// Place the part by the frame at the start of `frame` once its shells
    /// are thickened: its shapes then evaluate it in its own frame.
    pub fn place_by(&mut self, frame: &Buffer) {
        self.0.placement = Some(frame.clone());
    }

    /// Thicken every shell and compute the final normals.
    pub fn record_shells(
        &mut self,
        gpu: &ArmorGpu,
        batch: &mut KernelBatch,
    ) -> Result<(), GenerateError> {
        self.0.record_shells(gpu, batch)
    }

    /// Final positions, once the shells are recorded.
    pub fn positions(&self) -> &Buffer {
        self.0.positions()
    }

    pub fn vertex_count(&self) -> u32 {
        self.0.final_count()
    }

    /// Final triangles, once the shells are recorded.
    pub fn indices(&self) -> &Buffer {
        self.0.final_indices()
    }

    pub fn triangle_count(&self) -> u32 {
        self.0.final_triangle_count()
    }

    /// Read the finished part back, after its batch has been submitted.
    pub fn read(&self, gpu: &ArmorGpu) -> Result<BuiltPart, GenerateError> {
        let mut staging = Staging::new();
        let slots = self.stage(&mut staging);
        self.finish(&gpu.read_staged(staging)?, slots)
    }

    /// Stage the finished part for a shared readback.
    pub fn stage<'a>(&'a self, staging: &mut Staging<'a>) -> part::PartSlots {
        self.0.stage(staging)
    }

    /// The finished part, from a shared readback.
    pub fn finish(
        &self,
        results: &StagedResults,
        slots: part::PartSlots,
    ) -> Result<BuiltPart, GenerateError> {
        self.0.finish(results, slots)
    }
}

/// A compute device and the armor kernels compiled for it.
pub struct ArmorGpu {
    context: WgpuContext,
    cache: KernelCache,
    query: MeshQuery,
    area_normals: VertexNormalKernels,
    angle_normals: VertexNormalKernels,
}

impl std::fmt::Debug for ArmorGpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArmorGpu")
            .field("kernels", &self.cache.len())
            .finish()
    }
}

impl ArmorGpu {
    /// Compile the shared kernels on `context`.
    pub fn new(context: WgpuContext) -> Result<Self, GenerateError> {
        let cache = KernelCache::new();
        Ok(Self {
            query: MeshQuery::with_cache(&context, &cache).map_err(device_error)?,
            area_normals: VertexNormalKernels::with_cache(&context, &cache, NormalWeighting::Area)
                .map_err(device_error)?,
            angle_normals: VertexNormalKernels::with_cache(
                &context,
                &cache,
                NormalWeighting::Angle,
            )
            .map_err(device_error)?,
            context,
            cache,
        })
    }

    /// Open the default adapter's device.
    pub fn open() -> Result<Self, GenerateError> {
        pollster::block_on(Self::open_async())
    }

    /// Open a device without blocking the browser event loop.
    pub async fn open_async() -> Result<Self, GenerateError> {
        Self::new(WgpuContext::new_compute().await.map_err(device_error)?)
    }

    pub fn context(&self) -> &WgpuContext {
        &self.context
    }

    pub fn cache(&self) -> &KernelCache {
        &self.cache
    }

    pub fn query(&self) -> &MeshQuery {
        &self.query
    }

    pub fn normals(&self, weighting: NormalWeighting) -> &VertexNormalKernels {
        match weighting {
            NormalWeighting::Area => &self.area_normals,
            NormalWeighting::Angle => &self.angle_normals,
        }
    }

    pub fn batch(&self, label: &str) -> KernelBatch<'_> {
        KernelBatch::labelled(&self.context, label)
    }

    /// Build a part placed by host frames, as previews and tests place one:
    /// `record` records it against each of `frames` on the device, then its
    /// shells are thickened and it is read back.
    pub fn build_in(
        &self,
        frames: &[PartFrame],
        record: impl FnOnce(&mut KernelBatch, &[&Buffer]) -> Result<DevicePart, GenerateError>,
    ) -> Result<BuiltPart, GenerateError> {
        let buffers = frames
            .iter()
            .map(|frame| {
                frame.validate()?;
                self.upload(&frame_words(frame))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let buffers = buffers.iter().collect::<Vec<_>>();
        let mut batch = self.batch("armor in host frames");
        let mut part = record(&mut batch, &buffers)?;
        part.record_shells(self, &mut batch)?;
        batch.submit();
        part.read(self)
    }

    /// Upload plain data into a new storage buffer.
    pub fn upload<T: bytemuck::NoUninit>(&self, data: &[T]) -> Result<Buffer, GenerateError> {
        let bytes = bytemuck::cast_slice::<T, u8>(data);
        // A buffer cannot be empty; an empty array still needs something bound.
        let padded;
        let bytes = if bytes.is_empty() {
            padded = [0u8; 4];
            &padded[..]
        } else {
            bytes
        };
        Buffer::from_bytes(
            &self.context,
            bytes,
            BufferDefinition::storage().with_copy_src(),
        )
        .map_err(device_error)
    }

    /// A zeroed storage buffer of `bytes` bytes.
    pub fn scratch(&self, bytes: u64, label: &str) -> Result<Buffer, GenerateError> {
        Buffer::new(
            &self.context,
            bytes.max(4),
            BufferDefinition::storage()
                .with_copy_src()
                .with_label(label),
        )
        .map_err(device_error)
    }

    /// Read a buffer back. Stalls until the device has finished writing it.
    pub fn read<T: bytemuck::AnyBitPattern>(
        &self,
        buffer: &Buffer,
    ) -> Result<Vec<T>, GenerateError> {
        pollster::block_on(self.read_async(buffer))
    }

    pub async fn read_async<T: bytemuck::AnyBitPattern>(
        &self,
        buffer: &Buffer,
    ) -> Result<Vec<T>, GenerateError> {
        buffer.read(&self.context).await.map_err(device_error)
    }

    /// The nearest of `targets` to each query, ties going to the lowest index.
    pub fn nearest_points(
        &self,
        queries: &[[f32; 3]],
        targets: &[[f32; 3]],
    ) -> Result<Vec<u32>, GenerateError> {
        if queries.is_empty() {
            return Ok(Vec::new());
        }
        if targets.is_empty() {
            return Err(GenerateError::InvalidSurface);
        }
        let count = queries.len() as u32;
        let hits = QueryHits::new(&self.context, count).map_err(device_error)?;
        let query_buffer = self.upload(queries)?;
        let target_buffer = self.upload(targets)?;
        let mut batch = self.batch("armor nearest points");
        self.query
            .record_nearest_points(
                &mut batch,
                &query_buffer,
                count,
                PointTargets {
                    positions: &target_buffer,
                    candidates: None,
                    count: targets.len() as u32,
                },
                &hits,
            )
            .map_err(device_error)?;
        batch.submit();
        let mut nearest: Vec<u32> = self.read(&hits.nearest)?;
        nearest.truncate(queries.len());
        Ok(nearest)
    }
}

/// A device failure, which armor generation cannot recover from.
pub fn device_error(error: impl std::fmt::Display) -> GenerateError {
    GenerateError::Gpu(Arc::from(error.to_string()))
}
