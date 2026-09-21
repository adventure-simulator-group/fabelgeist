//! The plate armor device: the parametric plate armor and every armor metal
//! bake run on the armor compute device.

use std::sync::OnceLock;

use fabelgeist_armor::gpu::PlateGpu;

/// The shared plate armor kernels, compiled on first use into the armor
/// device's kernel cache.
pub fn plate_gpu() -> anyhow::Result<&'static PlateGpu> {
    static GPU: OnceLock<Result<PlateGpu, String>> = OnceLock::new();
    GPU.get_or_init(|| {
        let armor = crate::armor_gpu().map_err(|error| error.to_string())?;
        PlateGpu::new(armor.context(), armor.cache())
    })
    .as_ref()
    .map_err(|error| anyhow::anyhow!("opening the plate armor GPU: {error}"))
}
