//! The GPU hair is grown and prepared on.
//!
//! The hair crate grooms on the GPU ([`GpuGroomer`]): a hundred thousand
//! strands are a hundred thousand independent sweeps, which compute kernels
//! run about ten times faster than every core of the CPU. Placing hair on a
//! character runs there too: the character's body sampled as a distance
//! field ([`field`]), the strands carried onto its head and draped on it
//! ([`place`]), and the levels of detail kept clear of it, all colliding
//! with the field as the hair crate's kernels collide with its mannequin. A
//! studio grooms on the device it draws with, so that what is grown can be
//! drawn from where it lies; an export or a test without a window opens a
//! device of its own.

mod bake;
mod field;
mod place;
mod strips;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use anyhow::Result;
use fabelgeist_compute::{Kernel, KernelCache};
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_gpu::prelude::{Buffer, BufferDefinition, BufferUpload};
use fabelgeist_hair::gpu::{GpuGroomer, headless_context};

pub(crate) use bake::longest;
pub use bake::{GpuScalpTexture, Level as ScalpLevel, ScalpTexture, levels as scalp_levels};
pub(crate) use field::BodyField;
pub(crate) use place::Draping;
pub use strips::GpuGeometry;
pub(crate) use strips::StripSource;

/// A kernel's parameters: buffers and values bound by name.
pub(super) type Params = fabelgeist_gpu::data::gpu::parameters::PassParameters;

/// Parameters by name, as the hair crate's kernels bind theirs.
macro_rules! params {
    ($($key:literal => $value:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut p = $crate::hair::gpu::Params::new();
        $(p.insert($key, $value);)*
        p
    }};
}
pub(super) use params;

/// The hair kernels on one device: the hair crate's groomer, and the
/// kernels placing its hair on a character. Grooms take turns: one at a
/// time records into the groomer's scratch buffers.
pub struct HairGpu {
    context: WgpuContext,
    groomer: Mutex<GpuGroomer>,
    field: field::FieldKernels,
    place: place::PlaceKernels,
    strips: strips::StripKernels,
    bake: bake::BakeKernels,
}

/// Compiles kernels for one device, each source once.
pub(super) struct KernelSource {
    context: WgpuContext,
    cache: KernelCache,
}

impl KernelSource {
    pub(super) fn compile(&self, code: &str) -> Result<Arc<Kernel>> {
        self.cache.get(&self.context, code)
    }
}

impl HairGpu {
    /// The hair kernels compiled for `context`'s device.
    pub fn new(context: WgpuContext) -> Result<Self> {
        let source = KernelSource {
            context: context.clone(),
            cache: KernelCache::new(),
        };
        Ok(Self {
            groomer: Mutex::new(GpuGroomer::new(context.clone())?),
            field: field::FieldKernels::new(&source)?,
            place: place::PlaceKernels::new(&source)?,
            strips: strips::StripKernels::new(&source)?,
            bake: bake::BakeKernels::new(&source)?,
            context,
        })
    }

    /// The hair kernels on a device of their own, for work without a
    /// window.
    pub fn headless() -> Result<Self> {
        Self::new(headless_context()?)
    }

    /// The device the kernels run on.
    pub(super) fn context(&self) -> &WgpuContext {
        &self.context
    }

    /// Wait, however long it takes, for everything submitted to the
    /// kernels' device to finish: before letting go of a device of their
    /// own, which waits for its work only so long, and on a busy GPU might
    /// not long enough.
    pub fn finish(&self) {
        self.groomer().finish();
    }

    /// The groomer, once no other groom is using it. A groom that panicked
    /// leaves nothing a later one relies on: every groom sizes and clears
    /// what it uses.
    pub(super) fn groomer(&self) -> MutexGuard<'_, GpuGroomer> {
        self.groomer.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A fresh storage buffer of `bytes`, never empty.
pub(super) fn storage(context: &WgpuContext, bytes: u64, label: &str) -> Result<Buffer> {
    Buffer::new(
        context,
        bytes.max(16).into(),
        BufferDefinition::storage().with_label(label.into()),
    )
}

/// A storage buffer holding `data`, never empty.
pub(crate) fn upload<T: bytemuck::NoUninit + Default>(
    context: &WgpuContext,
    data: &[T],
    label: &str,
) -> Result<Buffer> {
    let padding = [T::default(); 4];
    let data = if data.is_empty() { &padding[..] } else { data };
    Buffer::from_upload(
        context,
        BufferUpload::from_elements(data),
        BufferDefinition::storage().with_label(label.into()),
    )
}
