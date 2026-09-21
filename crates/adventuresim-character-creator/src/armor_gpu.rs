//! The one armor compute device this process generates on.

use std::sync::{Condvar, Mutex, OnceLock};

use adventuresim_armor_model::ArmorGpu;

/// The shared armor device, opened on first use.
///
/// Opening a device and compiling the armor kernels takes a noticeable
/// fraction of a second, and every fitted piece in every thread wants the
/// same kernels, so the process keeps one.
pub fn armor_gpu() -> anyhow::Result<&'static ArmorGpu> {
    static GPU: OnceLock<Result<ArmorGpu, String>> = OnceLock::new();
    GPU.get_or_init(|| ArmorGpu::open().map_err(|error| error.to_string()))
        .as_ref()
        .map_err(|error| anyhow::anyhow!("opening the armor GPU: {error}"))
}

/// Pieces fitted on the armor device at once.
///
/// Every piece's work lands on the one device queue, and every readback waits
/// for that queue. Past a handful of pieces in flight, more threads only add
/// queued work and buffers each readback must wait behind: fitting the whole
/// catalog with morphs took four times as long on 32 threads as on four.
const PIECES_IN_FLIGHT: usize = 4;

/// Permission to fit one piece on the armor device; released on drop.
pub struct FittingSlot(());

static IN_FLIGHT: (Mutex<usize>, Condvar) = (Mutex::new(0), Condvar::new());

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

impl Drop for FittingSlot {
    fn drop(&mut self) {
        let (count, freed) = &IN_FLIGHT;
        *count
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) -= 1;
        freed.notify_one();
    }
}
