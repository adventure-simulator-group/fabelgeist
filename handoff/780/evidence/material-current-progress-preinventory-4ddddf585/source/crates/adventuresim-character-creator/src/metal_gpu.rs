//! The armor metal device: every armor metal bake runs on the armor compute
//! device.

use crate::GpuInitializationError;
use anyhow::Context;
use fabelgeist_armor::gpu::metal::MetalGpu;
use std::sync::OnceLock;

/// The shared metal kernels, initialized once in the armor device's kernel cache.
/// The process retains that first outcome, including failure.
pub fn metal_gpu() -> anyhow::Result<&'static MetalGpu> {
    static GPU: OnceLock<Result<MetalGpu, GpuInitializationError>> = OnceLock::new();
    GPU.get_or_init(|| {
        let armor = crate::armor_gpu().map_err(GpuInitializationError::new)?;
        MetalGpu::new(armor.context(), armor.cache()).map_err(GpuInitializationError::new)
    })
    .as_ref()
    .map_err(Clone::clone)
    .context("opening the armor metal GPU")
}
