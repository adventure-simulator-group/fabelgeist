//! The armor metal device: every armor metal bake runs on the armor compute
//! device.

use std::sync::{Arc, OnceLock};

use fabelgeist_armor::gpu::metal::MetalGpu;
use fabelgeist_armor::{GenerateError, material::MetalError};

#[derive(Debug, Clone, thiserror::Error)]
pub enum MetalGpuOpenError {
    #[error("opening the armor GPU: {0}")]
    Armor(#[source] Arc<GenerateError>),
    #[error("compiling the armor metal kernels: {0}")]
    Metal(#[source] Arc<MetalError>),
}

/// The shared metal kernels, compiled on first use into the armor device's
/// kernel cache.
pub fn metal_gpu() -> Result<&'static MetalGpu, MetalGpuOpenError> {
    static GPU: OnceLock<Result<MetalGpu, MetalGpuOpenError>> = OnceLock::new();
    GPU.get_or_init(|| -> Result<MetalGpu, MetalGpuOpenError> {
        let armor = crate::armor_gpu().map_err(MetalGpuOpenError::Armor)?;
        MetalGpu::new(armor.context(), armor.cache())
            .map_err(Arc::new)
            .map_err(MetalGpuOpenError::Metal)
    })
    .as_ref()
    .map_err(MetalGpuOpenError::clone)
}
