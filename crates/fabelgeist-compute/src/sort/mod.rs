//! Least-significant-digit radix sort of `u32` keys, carrying a `u32` payload.
//!
//! Two things upstream need it. A linear BVH sorts primitives by Morton code;
//! a spatial hash sorts particles by cell index. Both want the sort recorded
//! into the frame's own [`KernelBatch`], never read back to the host, so this
//! is written as passes over GPU buffers with no synchronisation points.
//!
//! Eight bits per pass, so a full 32-bit key takes four passes and a 30-bit
//! Morton code takes four as well (`bits` rounds up to a whole digit). Each
//! pass is three dispatches:
//!
//! 1. **histogram** -- one workgroup per tile counts its 256 digits into
//!    shared memory and writes the counts to `histogram[digit][tile]`.
//! 2. **scan** -- a single workgroup turns those counts into the global offset
//!    each `(digit, tile)` pair writes at. One workgroup can do it because the
//!    array is only `256 x tiles`, and laying it out digit-major means thread
//!    `d` owns row `d` outright.
//! 3. **scatter** -- each element's destination is its tile's base for that
//!    digit, plus its rank among the same-digit elements earlier in the tile.
//!
//! The rank in step 3 is computed by counting matching digits over the tile in
//! shared memory: `TILE` reads per element, all 256 threads in parallel. A
//! subgroup ballot would make it a couple of instructions instead, but ballots
//! are an optional wgpu feature and this is nowhere near the bottleneck for
//! the tens of thousands of elements a cloth solve sorts.

use crate::prelude::*;
use std::sync::Arc;
mod error;
mod quantities;
pub use error::{
    SortBufferRole, SortBuildError, SortDispatchStage, SortError, SortKernelRole, SortScratchError,
};
pub use quantities::{
    ScratchGrowth, SortDigit, SortDigits, SortItemCount, SortKeyWidth, SortPassCount,
};
use quantities::{SortCopyBack, SortWork};

/// Elements per tile, and threads per workgroup. One element per thread.
const TILE: u32 = 256;
/// Digits per pass. Tied to `TILE` only because the scan pass gives each of
/// its 256 threads one digit row.
const RADIX: u32 = 256;
/// Bits consumed per pass.
const BITS_PER_PASS: u32 = 8;

mod source;
use source::{histogram_code, scan_code, scatter_code};
mod passes;
use passes::SortPassBuffers;

/// Scratch buffers a sort needs: one ping-pong pair and the histogram.
///
/// Allocated for a capacity and reused, so that a per-frame sort does not
/// allocate. Grow it with [`SortScratch::ensure`].
#[derive(Clone, Debug)]
pub struct SortScratch {
    pub keys: Buffer,
    pub values: Buffer,
    pub histogram: Buffer,
    capacity: SortItemCount,
}

impl SortScratch {
    pub fn new(
        context: &WgpuContext,
        capacity: SortItemCount,
    ) -> std::result::Result<Self, SortScratchError> {
        let capacity = capacity.with_sentinel();
        let tiles = capacity.tiles();
        let storage = BufferDefinition::storage();
        Ok(Self {
            keys: Buffer::new(
                context,
                capacity.word_bytes(),
                storage.clone().with_label(("sort keys").into()),
            )
            .map_err(|source: BufferCreationError| -> SortScratchError {
                SortScratchError {
                    role: SortBufferRole::Keys,
                    capacity,
                    bytes: capacity.word_bytes(),
                    source,
                }
            })?,
            values: Buffer::new(
                context,
                capacity.word_bytes(),
                storage.clone().with_label(("sort values").into()),
            )
            .map_err(|source: BufferCreationError| -> SortScratchError {
                SortScratchError {
                    role: SortBufferRole::Values,
                    capacity,
                    bytes: capacity.word_bytes(),
                    source,
                }
            })?,
            histogram: Buffer::new(
                context,
                tiles.histogram_bytes(),
                storage.with_label(("sort histogram").into()),
            )
            .map_err(|source: BufferCreationError| -> SortScratchError {
                SortScratchError {
                    role: SortBufferRole::Histogram,
                    capacity,
                    bytes: tiles.histogram_bytes(),
                    source,
                }
            })?,
            capacity,
        })
    }

    pub fn capacity(&self) -> SortItemCount {
        self.capacity
    }

    /// Reallocate if `capacity` no longer fits; report the growth outcome.
    pub fn ensure(
        &mut self,
        context: &WgpuContext,
        capacity: SortItemCount,
    ) -> std::result::Result<ScratchGrowth, SortScratchError> {
        if capacity <= self.capacity {
            return Ok(ScratchGrowth::Unchanged);
        }
        *self = Self::new(context, capacity)?;
        Ok(ScratchGrowth::Grown)
    }
}

/// The three compiled passes.
#[derive(Clone, Debug)]
pub struct RadixSort {
    histogram: Arc<Kernel>,
    scan: Arc<Kernel>,
    scatter: Arc<Kernel>,
}

impl RadixSort {
    pub fn new(context: &WgpuContext) -> std::result::Result<Self, SortBuildError> {
        Self::with_cache(context, &KernelCache::new())
    }

    pub fn with_cache(
        context: &WgpuContext,
        cache: &KernelCache,
    ) -> std::result::Result<Self, SortBuildError> {
        Ok(Self {
            histogram: cache.get(context, &histogram_code()).map_err(
                |source: KernelCacheError| -> SortBuildError {
                    SortBuildError {
                        role: SortKernelRole::Histogram,
                        source: Box::new(source),
                    }
                },
            )?,
            scan: cache.get(context, &scan_code()).map_err(
                |source: KernelCacheError| -> SortBuildError {
                    SortBuildError {
                        role: SortKernelRole::Scan,
                        source: Box::new(source),
                    }
                },
            )?,
            scatter: cache.get(context, &scatter_code()).map_err(
                |source: KernelCacheError| -> SortBuildError {
                    SortBuildError {
                        role: SortKernelRole::Scatter,
                        source: Box::new(source),
                    }
                },
            )?,
        })
    }

    /// Record a sort of `keys`/`values` in place, ascending by key.
    ///
    /// `bits` is how many low bits of the key actually vary -- 30 for a Morton
    /// code, 32 for an arbitrary hash. Passing fewer is purely a saving; it
    /// never changes the result as long as the higher bits really are zero.
    ///
    /// Every pass ping-pongs into `scratch`, so an odd pass count would leave
    /// the result there. Rather than make the caller track that, an odd count
    /// gets one extra copy back at the end.
    pub fn record(
        &self,
        batch: &mut KernelBatch,
        keys: &Buffer,
        values: &Buffer,
        scratch: &mut SortScratch,
        count: SortItemCount,
        bits: SortKeyWidth,
    ) -> std::result::Result<(), SortError> {
        if count.work() == SortWork::NoOp {
            return Ok(());
        }
        if scratch.capacity < count {
            return Err(SortError::ScratchCapacity {
                capacity: scratch.capacity,
                count,
            });
        }
        let needed = count.word_bytes();
        if keys.length() < needed || values.length() < needed {
            return Err(SortError::BufferLength {
                count,
                needed,
                keys: keys.length(),
                values: values.length(),
            });
        }
        let mut buffers = SortPassBuffers::new(keys, values, scratch, count);
        for digit in bits.digits() {
            buffers.record_digit(batch, self, digit)?;
        }
        if bits.pass_count().copy_back() == SortCopyBack::Required {
            buffers.copy_back(batch, keys, values)?;
        }

        Ok(())
    }

    /// Sort on its own encoder and submit. For a one-off; inside a frame,
    /// record into the frame's batch instead.
    pub fn run(
        &self,
        context: &WgpuContext,
        keys: &Buffer,
        values: &Buffer,
        scratch: &mut SortScratch,
        count: SortItemCount,
        bits: SortKeyWidth,
    ) -> std::result::Result<(), SortError> {
        let mut batch = KernelBatch::labelled(context, ("RadixSort").into());
        self.record(&mut batch, keys, values, scratch, count, bits)?;
        batch.submit();
        Ok(())
    }
}

#[cfg(test)]
mod tests;
