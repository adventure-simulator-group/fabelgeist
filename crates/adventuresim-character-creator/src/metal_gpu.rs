//! The armor metal device: every armor metal bake runs on the armor compute
//! device.

use crate::GpuInitializationError;
use anyhow::Context;
use fabelgeist_armor::gpu::metal::MetalGpu;

/// The shared metal kernels, initialized once in the armor device's kernel cache.
/// The process retains that first outcome, including failure. Awaiting device
/// acquisition leaves browser callbacks free to run.
pub async fn metal_gpu() -> anyhow::Result<&'static MetalGpu> {
    static GPU: async_lock::OnceCell<Result<MetalGpu, GpuInitializationError>> =
        async_lock::OnceCell::new();
    GPU.get_or_init(|| async {
        let armor = crate::armor_gpu_async().await.map_err(GpuInitializationError::new)?;
        MetalGpu::new(armor.context(), armor.cache()).map_err(GpuInitializationError::new)
    })
    .await
    .as_ref()
    .map_err(Clone::clone)
    .context("opening the armor metal GPU")
}
