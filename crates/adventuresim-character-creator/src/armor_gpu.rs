//! The one armor compute device this process generates on.

use crate::GpuInitializationError;
use anyhow::Context;
use std::sync::{Condvar, Mutex};

use fabelgeist_armor::ArmorGpu;

/// Pieces fitted on the armor device at once.
///
/// Every piece's work lands on the one device queue, and every readback waits
/// for that queue. Past a handful of pieces in flight, more threads only add
/// queued work and buffers each readback must wait behind: fitting the whole
/// catalog with morphs took four times as long on 32 threads as on four.
const PIECES_IN_FLIGHT: usize = 4;

static IN_FLIGHT: (Mutex<usize>, Condvar) = (Mutex::new(0), Condvar::new());

/// Permission to fit one piece on the armor device; released on drop.
pub struct FittingSlot(());

impl Drop for FittingSlot {
    fn drop(&mut self) {
        let (count, freed) = &IN_FLIGHT;
        *count
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) -= 1;
        freed.notify_one();
    }
}

/// The shared armor device, opened on first use.
///
/// The process retains the first initialization outcome, including failure.
/// Opening a device and compiling its kernels is shared by every fitting worker.
pub fn armor_gpu() -> anyhow::Result<&'static ArmorGpu> {
    pollster::block_on(armor_gpu_async())
}

/// Initialize once without blocking the browser event loop while opening a device.
pub async fn armor_gpu_async() -> anyhow::Result<&'static ArmorGpu> {
    static GPU: async_lock::OnceCell<Result<ArmorGpu, GpuInitializationError>> =
        async_lock::OnceCell::new();
    GPU.get_or_init(|| async {
        ArmorGpu::open_async()
            .await
            .map_err(GpuInitializationError::new)
    })
    .await
    .as_ref()
    .map_err(Clone::clone)
    .context("opening the armor GPU")
}

/// Wait until fewer than [`PIECES_IN_FLIGHT`] pieces are being fitted.
pub fn fitting_slot() -> FittingSlot {
    let (count, freed) = &IN_FLIGHT;
    let mut count = count
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    while *count >= PIECES_IN_FLIGHT {
        count = freed
            .wait(count)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }
    *count += 1;
    FittingSlot(())
}
