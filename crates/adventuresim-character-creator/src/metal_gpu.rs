//! The armor metal device: every armor metal bake runs on the armor compute
//! device.

use std::sync::OnceLock;

use fabelgeist_armor::gpu::metal::MetalGpu;

/// The shared metal kernels, compiled on first use into the armor device's
/// kernel cache.
pub fn metal_gpu() -> anyhow::Result<&'static MetalGpu> {
    static GPU: OnceLock<Result<MetalGpu, String>> = OnceLock::new();
    GPU.get_or_init(|| {
        let armor = crate::armor_gpu().map_err(|error| error.to_string())?;
        MetalGpu::new(armor.context(), armor.cache())
    })
    .as_ref()
    .map_err(|error| anyhow::anyhow!("opening the armor metal GPU: {error}"))
}
