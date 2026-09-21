//! Plate armor and its metal, generated on a compute device.
//!
//! [`PlateGpu::build`] generates an [`Armor`]'s welded parts and
//! [`PlateGpu::textures`] bakes a [`Metal`]'s maps. The host plans only what
//! depends on the design alone -- the part list, the tile layout and the
//! template tile's CSG-cut rivet holes -- and decodes the engraving image.
//! Everything per vertex, per triangle and per texel runs on the device, in
//! correctly rounded arithmetic (see [`fabelgeist_compute::host_float`]), so
//! that the finite-difference normals and the micrometre weld do not depend
//! on how a device compiler fuses or reassociates.
//!
//! The kernels compile into a caller's [`KernelCache`] on a caller's
//! [`WgpuContext`], so an application that already generates other armor on
//! a device shares it; [`PlateGpu::open`] opens one of its own.

mod armor;
mod geometry;
mod metal;
mod plan;
mod textures;
mod weld;
mod wgsl;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use fabelgeist_compute::{Kernel, KernelBatch, KernelCache, RadixSort, ScanDefinition};
use fabelgeist_gpu::prelude::{Buffer, BufferDefinition, PassParameters, WgpuContext};

use crate::{
    Armor, ArmorPart,
    material::{Metal, MetalTextures},
};

/// The plate armor kernels, compiled for one device.
pub struct PlateGpu {
    context: WgpuContext,
    emit: Arc<Kernel>,
    keys: Arc<Kernel>,
    representatives: Arc<Kernel>,
    weld_vertices: Arc<Kernel>,
    weld_faces: Arc<Kernel>,
    compact: Arc<Kernel>,
    ranges: Arc<Kernel>,
    scratches: Arc<Kernel>,
    engraving: Arc<Kernel>,
    cut_slopes: Arc<Kernel>,
    bake: Arc<Kernel>,
    sort: RadixSort,
    scan: ScanDefinition,
    tiles: plan::TileCache,
}

impl std::fmt::Debug for PlateGpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlateGpu").finish_non_exhaustive()
    }
}

impl PlateGpu {
    /// Compile the kernels on `context`, sharing `cache`.
    ///
    /// Everything compiles here, the scan's pipelines included: compiling
    /// pushes and pops the device's error scopes, which must not interleave
    /// with another thread's, so no build or bake compiles anything later.
    pub fn new(context: &WgpuContext, cache: &KernelCache) -> Result<Self, String> {
        let kernel = |source: String| cache.get(context, &source).map_err(device_error);
        let scan = ScanDefinition::new(
            context,
            "fn scan(a: u32, b: u32) -> u32 { return a + b; }".to_string(),
        )
        .map_err(device_error)?;
        scan.get_or_create_pipelines(context)
            .map_err(device_error)?;
        Ok(Self {
            emit: kernel(geometry::emit_source())?,
            keys: kernel(weld::keys_source())?,
            representatives: kernel(weld::representatives_source())?,
            weld_vertices: kernel(weld::vertices_source())?,
            weld_faces: kernel(weld::faces_source())?,
            compact: kernel(weld::compact_source())?,
            ranges: kernel(weld::ranges_source())?,
            scratches: kernel(textures::scratches_source())?,
            engraving: kernel(textures::engraving_source())?,
            cut_slopes: kernel(textures::cut_slopes_source())?,
            bake: kernel(textures::bake_source())?,
            sort: RadixSort::with_cache(context, cache).map_err(device_error)?,
            scan,
            tiles: plan::TileCache::default(),
            context: context.clone(),
        })
    }

    /// Open the default adapter's device.
    pub fn open() -> Result<Self, String> {
        let context = pollster::block_on(WgpuContext::new()).map_err(device_error)?;
        Self::new(&context, &KernelCache::new())
    }

    /// Generate the armor's parts: the breastplate (one part, or a part per
    /// tile), then each fauld layer, every one a closed, welded mesh.
    pub fn build(&self, armor: &Armor) -> Result<Vec<ArmorPart>, String> {
        armor::build(self, armor)
    }

    /// Bake the metal's maps onto a `size` square tile, reading the engraving
    /// image if there is one. `size` must be 32–1024.
    pub fn textures(&self, metal: &Metal, size: u32) -> Result<MetalTextures, String> {
        metal::textures(self, metal, size)
    }

    fn batch(&self, label: &str) -> KernelBatch<'_> {
        KernelBatch::labelled(&self.context, label)
    }

    fn upload<T: bytemuck::NoUninit>(&self, data: &[T]) -> Result<Buffer, String> {
        let bytes = bytemuck::cast_slice::<T, u8>(data);
        // A buffer cannot be empty; an empty array still needs something bound.
        let bytes = if bytes.is_empty() {
            &[0u8; 4][..]
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

    /// A zeroed storage buffer of `words` 32-bit words.
    fn scratch(&self, words: u64, label: &str) -> Result<Buffer, String> {
        Buffer::new(
            &self.context,
            words.max(1) * 4,
            BufferDefinition::storage()
                .with_copy_src()
                .with_copy_dst()
                .with_label(label),
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

    /// Inclusive prefix sums of a buffer of exactly as many words as it
    /// sums. Runs after everything submitted before it.
    fn inclusive_scan(&self, input: &Buffer) -> Result<Buffer, String> {
        fabelgeist_compute::Scan::execute(&self.context, &self.scan, input).map_err(device_error)
    }
}

/// The parameters of a pass over `count` items.
fn counted(count: u32) -> PassParameters {
    let mut parameters = PassParameters::new();
    parameters.insert("count", count);
    for pad in ["pad0", "pad1", "pad2"] {
        parameters.insert(pad, 0u32);
    }
    parameters
}

/// A device failure, which plate generation cannot recover from.
fn device_error(error: impl std::fmt::Display) -> String {
    format!("plate armor device: {error}")
}
