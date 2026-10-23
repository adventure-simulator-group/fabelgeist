//! Shared armor kernels and native operations on their producing context.

use super::{BuiltPart, DevicePart, frame_words};
use crate::{GenerateError, PartFrame};
use fabelgeist_compute::{
    KernelBatch, KernelCache, MeshQuery, NormalWeighting, PointSelection, PointTargets, QueryCount,
    QueryHits, VertexNormalKernels,
};
use fabelgeist_gpu::prelude::{
    Buffer, BufferByteLength, BufferDefinition, BufferLabel, BufferUpload, BufferUse, WgpuContext,
};
use fabelgeist_mesh::MeshVertexCount;

type Result<T> = std::result::Result<T, GenerateError>;

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
    pub fn new(context: WgpuContext) -> Result<Self> {
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
    pub fn open() -> Result<Self> {
        pollster::block_on(Self::open_async())
    }

    /// Open a device without blocking the browser event loop.
    pub async fn open_async() -> Result<Self> {
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
        record: impl FnOnce(&mut KernelBatch, &[&Buffer]) -> Result<DevicePart>,
    ) -> Result<BuiltPart> {
        let buffers = frames
            .iter()
            .map(|frame| {
                frame.validate()?;
                self.upload(BufferUpload::from_elements(&frame_words(frame)))
            })
            .collect::<Result<Vec<_>>>()?;
        let buffers = buffers.iter().collect::<Vec<_>>();
        let mut batch = self.batch("armor in host frames");
        let mut part = record(&mut batch, &buffers)?;
        part.record_shells(self, &mut batch)?;
        batch.submit().map_err(device_error)?;
        part.read(self)
    }

    /// Upload plain data into a new storage buffer.
    pub fn upload(&self, data: BufferUpload<'_>) -> Result<Buffer> {
        Buffer::from_upload(
            &self.context,
            data.with_empty_word(),
            BufferDefinition::storage().with_usage(BufferUse::CopySource),
        )
        .map_err(device_error)
    }

    /// Zeroed storage with at least one native u32 word, even for no logical bytes.
    pub fn scratch(&self, bytes: BufferByteLength, label: BufferLabel) -> Result<Buffer> {
        let minimum = BufferByteLength::from(std::mem::size_of::<u32>());
        Buffer::new(
            &self.context,
            bytes.max(minimum),
            BufferDefinition::storage()
                .with_usage(BufferUse::CopySource)
                .with_label(label),
        )
        .map_err(device_error)
    }

    /// Read a logical buffer prefix after its producing submission succeeded.
    ///
    /// Native progress blocks; browser callers use `read_async` to yield to
    /// mapping callbacks. External mapping ownership follows `Buffer::read`.
    pub fn read<T: bytemuck::AnyBitPattern>(&self, buffer: &Buffer) -> Result<Vec<T>> {
        pollster::block_on(self.read_async(buffer))
    }

    pub async fn read_async<T: bytemuck::AnyBitPattern>(&self, buffer: &Buffer) -> Result<Vec<T>> {
        buffer.read(&self.context).await.map_err(device_error)
    }

    /// The nearest of `targets` to each query, ties going to the lowest index.
    pub fn nearest_points(&self, queries: &[[f32; 3]], targets: &[[f32; 3]]) -> Result<Vec<u32>> {
        if queries.is_empty() {
            return Ok(Vec::new());
        }
        if targets.is_empty() {
            return Err(GenerateError::InvalidSurface);
        }
        let count = QueryCount::try_from(queries.len()).map_err(device_error)?;
        let targets_count = MeshVertexCount::try_from(targets.len()).map_err(device_error)?;
        let hits = QueryHits::new(&self.context, count).map_err(device_error)?;
        let query_buffer = self.upload(BufferUpload::from_elements(queries))?;
        let target_buffer = self.upload(BufferUpload::from_elements(targets))?;
        let mut batch = self.batch("armor nearest points");
        self.query
            .record_nearest_points(
                &mut batch,
                &query_buffer,
                count,
                PointTargets {
                    positions: &target_buffer,
                    selection: PointSelection::All(targets_count),
                },
                &hits,
            )
            .map_err(device_error)?;
        batch.submit().map_err(device_error)?;
        let mut nearest: Vec<u32> = self.read(hits.nearest())?;
        nearest.truncate(usize::from(count));
        Ok(nearest)
    }
}

/// Preserve the cause of a failed GPU-backed armor operation.
pub fn device_error(error: impl Into<anyhow::Error>) -> GenerateError {
    GenerateError::Gpu(error.into())
}
