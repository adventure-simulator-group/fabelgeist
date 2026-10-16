//! The GPU hair is grown and prepared on.
//!
//! The hair crate grooms on the GPU ([`GpuGroomer`]): a hundred thousand
//! strands are a hundred thousand independent sweeps, which compute kernels
//! run about ten times faster than every core of the CPU. A studio grooms on
//! the device it draws with, so that what is grown can be drawn from where
//! it lies; an export or a test without a window opens a device of its own.

use std::sync::{Mutex, MutexGuard, PoisonError};

use anyhow::Result;
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_hair::gpu::{GpuGroomer, headless_context};

/// The hair kernels on one device. Grooms take turns: one at a time
/// records into the groomer's scratch buffers.
pub struct HairGpu {
    groomer: Mutex<GpuGroomer>,
}

impl HairGpu {
    /// The hair kernels compiled for `context`'s device.
    pub fn new(context: WgpuContext) -> Result<Self> {
        Ok(Self {
            groomer: Mutex::new(GpuGroomer::new(context)?),
        })
    }

    /// The hair kernels on a device of their own, for work without a
    /// window.
    pub fn headless() -> Result<Self> {
        Self::new(headless_context()?)
    }

    /// The groomer, once no other groom is using it. A groom that panicked
    /// leaves nothing a later one relies on: every groom sizes and clears
    /// what it uses.
    pub(super) fn groomer(&self) -> MutexGuard<'_, GpuGroomer> {
        self.groomer.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
