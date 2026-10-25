//! The metal every rigid armor piece is shaded with, baked on a compute
//! device.
//!
//! [`MetalGpu::textures`] bakes a [`Metal`]'s maps; the host only decodes the
//! engraving image. Per-texel arithmetic uses the operation order and fences
//! in [`fabelgeist_compute::host_float`], subject to its documented numerical
//! and device limits.
//!
//! The kernels compile into a caller's [`KernelCache`] on a caller's
//! [`WgpuContext`], so an application that already generates armor on a
//! device shares it; [`MetalGpu::open`] opens one of its own.

use crate::material::{Metal, MetalTextureSize, MetalTextures};
use anyhow::Result;
use fabelgeist_compute::{Kernel, KernelBatch, KernelCache};
use fabelgeist_gpu::prelude::{
    Buffer, BufferDefinition, BufferElementCount, BufferElementStride, BufferLabel, BufferUpload,
    BufferUse, InvocationCount, PassParameters, WgpuContext,
};
use std::sync::Arc;

mod bake;
mod finish;
mod ornament;
mod textures;
mod wgsl;

/// The metal kernels, compiled for one device.
pub struct MetalGpu {
    context: WgpuContext,
    scratches: Arc<Kernel>,
    engraving: Arc<Kernel>,
    cut_slopes: Arc<Kernel>,
    finish: Arc<Kernel>,
    ornament: Arc<Kernel>,
    bake: Arc<Kernel>,
}

/// Context for a failed GPU-backed metal operation, retaining its cause.
#[derive(Debug, thiserror::Error)]
#[error("armor metal device: {source}")]
pub struct MetalDeviceError {
    #[source]
    source: anyhow::Error,
}

impl std::fmt::Debug for MetalGpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetalGpu").finish_non_exhaustive()
    }
}

impl MetalGpu {
    /// Compile the kernels on `context`, sharing `cache`.
    ///
    /// Obtain all six kernel handles here. Bakes reuse those handles rather
    /// than compiling their kernels on demand.
    pub fn new(context: &WgpuContext, cache: &KernelCache) -> Result<Self> {
        let kernel = |source: String| cache.get(context, &source).map_err(device_error);
        Ok(Self {
            scratches: kernel(textures::scratches_source())?,
            engraving: kernel(textures::engraving_source())?,
            cut_slopes: kernel(textures::cut_slopes_source())?,
            finish: kernel(finish::finish_source())?,
            ornament: kernel(ornament::ornament_source())?,
            bake: kernel(textures::bake_source())?,
            context: context.clone(),
        })
    }

    /// Open the default adapter's compute-only device without blocking
    /// browser adapter and device callbacks.
    pub async fn open() -> Result<Self> {
        let context = WgpuContext::new_compute().await.map_err(device_error)?;
        Self::new(&context, &KernelCache::new())
    }

    /// Bake the metal's maps onto a `size` square tile, reading the engraving
    /// image if there is one. `size` has already admitted the supported range.
    /// Await readback so browser mapping callbacks can make progress.
    pub async fn textures(&self, metal: &Metal, size: MetalTextureSize) -> Result<MetalTextures> {
        bake::textures(self, metal, size).await
    }

    fn batch(&self, label: &str) -> KernelBatch<'_> {
        KernelBatch::labelled(&self.context, label)
    }

    fn upload(&self, data: BufferUpload<'_>) -> Result<Buffer> {
        Buffer::from_upload(
            &self.context,
            data.with_empty_word(),
            BufferDefinition::storage().with_usage(BufferUse::CopySource),
        )
        .map_err(device_error)
    }

    fn scratch(&self, words: BufferElementCount, label: BufferLabel) -> Result<Buffer> {
        // Every native storage word is one u32/f32 lane; retain a word for
        // an empty allocation, independently of the logical population.
        let stride = BufferElementStride::try_from(std::mem::size_of::<u32>() as u32)
            .map_err(device_error)?;
        let words = if words.is_empty() {
            BufferElementCount::from(1)
        } else {
            words
        };
        Buffer::new(
            &self.context,
            words.byte_length(stride),
            BufferDefinition::storage()
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
                .with_label(label),
        )
        .map_err(device_error)
    }

    /// Read a logical prefix after its producing submission succeeded.
    /// Mapping ownership and native/browser progress follow `Buffer::read`.
    async fn read<T: bytemuck::AnyBitPattern>(&self, buffer: &Buffer) -> Result<Vec<T>> {
        buffer.read(&self.context).await.map_err(device_error)
    }

    /// Record `kernel` over `items` invocations.
    fn dispatch(
        &self,
        batch: &mut KernelBatch,
        kernel: &Kernel,
        parameters: &PassParameters,
        items: InvocationCount,
    ) -> Result<()> {
        batch
            .dispatch_items(kernel, parameters, items)
            .map(|_| ())
            .map_err(device_error)
    }
}

/// Preserve native context without classifying every failure as device loss.
fn device_error(error: impl Into<anyhow::Error>) -> anyhow::Error {
    MetalDeviceError {
        source: error.into(),
    }
    .into()
}

#[cfg(test)]
mod tests;
