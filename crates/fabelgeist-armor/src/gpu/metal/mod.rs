//! The metal every rigid armor piece is shaded with, baked on a compute
//! device.
//!
//! [`MetalGpu::textures`] bakes a [`Metal`]'s maps; the host only decodes the
//! engraving image. Everything per texel runs on the device, in correctly
//! rounded arithmetic (see [`fabelgeist_compute::host_float`]), so that a bake
//! does not depend on how a device compiler fuses or reassociates.
//!
//! The kernels compile into a caller's [`KernelCache`] on a caller's
//! [`WgpuContext`], so an application that already generates armor on a
//! device shares it; [`MetalGpu::open`] opens one of its own.

use fabelgeist_compute::KernelBatchLabel;
use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};
mod bake;
mod finish;
mod ornament;
mod textures;
mod wgsl;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use fabelgeist_compute::{Kernel, KernelBatch, KernelCache};
use fabelgeist_gpu::prelude::{Buffer, BufferDefinition, PassParameters, WgpuContext};

use crate::material::{Metal, MetalTextures};

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

impl std::fmt::Debug for MetalGpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetalGpu").finish_non_exhaustive()
    }
}

impl MetalGpu {
    /// Compile the kernels on `context`, sharing `cache`.
    ///
    /// Everything compiles here: compiling pushes and pops the device's error
    /// scopes, which must not interleave with another thread's, so no bake
    /// compiles anything later.
    pub fn new(context: &WgpuContext, cache: &KernelCache) -> Result<Self, String> {
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

    /// Open the default adapter's device.
    pub fn open() -> Result<Self, String> {
        let context = pollster::block_on(WgpuContext::new()).map_err(device_error)?;
        Self::new(&context, &KernelCache::new())
    }

    /// Bake the metal's maps onto a `size` square tile, reading the engraving
    /// image if there is one. `size` must be 32–1024.
    pub fn textures(&self, metal: &Metal, size: u32) -> Result<MetalTextures, String> {
        bake::textures(self, metal, size)
    }

    fn batch(&self, label: KernelBatchLabel<'_>) -> KernelBatch<'_> {
        KernelBatch::labelled(&self.context, label)
    }

    fn upload(&self, data: BufferUpload<'_>) -> Result<Buffer, String> {
        Buffer::from_upload(
            &self.context,
            data.with_empty_word(),
            BufferDefinition::storage().with_usage(BufferUse::CopySource),
        )
        .map_err(device_error)
    }

    fn scratch(&self, words: u64, label: &str) -> Result<Buffer, String> {
        Buffer::new(
            &self.context,
            (words.max(1) * 4).into(),
            BufferDefinition::storage()
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination)
                .with_label((label).into()),
        )
        .map_err(device_error)
    }

    /// Read a buffer back. Stalls until the device has finished writing it.
    fn read<T: bytemuck::AnyBitPattern>(&self, buffer: &Buffer) -> Result<Vec<T>, String> {
        pollster::block_on(buffer.read(&self.context)).map_err(device_error)
    }

    /// Record `kernel` over `items` invocations.
    fn dispatch(
        &self,
        batch: &mut KernelBatch,
        kernel: &Kernel,
        parameters: &PassParameters,
        items: u32,
    ) -> Result<(), String> {
        batch
            .dispatch_items(kernel, parameters, items)
            .map(|_| ())
            .map_err(device_error)
    }
}

/// A device failure, which a bake cannot recover from.
fn device_error(error: impl std::fmt::Display) -> String {
    format!("armor metal device: {error}")
}
