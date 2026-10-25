//! Exclusive prefix sum of `u32`s, recorded into a [`KernelBatch`].
//!
//! [`Scan`] folds with any WGSL operator, but each call submits on its own
//! encoder, so it cannot sit between two dispatches of a frame's batch. Tree
//! builds need exactly that: count how many nodes each element emits, sum the
//! counts into offsets, and write the nodes -- three passes that have to run
//! back to back without a trip to the host. This is that middle pass.
//!
//! The classic three-step scan, applied recursively: each workgroup scans its
//! own block of 256 in shared memory and writes the block's total aside; the
//! totals are scanned the same way; then every block adds its total's offset
//! back. Additional levels are recorded within the admitted element count;
//! device limits still constrain storage and dispatch.

use crate::prelude::*;
use fabelgeist_gpu::prelude::BufferByteLength;
use std::sync::Arc;
use wgsl::{add_offsets_code, scan_blocks_code};

mod wgsl;

/// Elements per block, and threads per workgroup in the native scan.
const BLOCK: u32 = 256;

type Result<T> = std::result::Result<T, ScanError>;

/// Number of `u32` elements in an exclusive scan, not a byte extent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScanElementCount(u32);

/// A scan block emits one total and occupies one workgroup.
#[derive(Clone, Copy, Debug)]
struct ScanBlockCount(u32);


/// The block totals of every level of the recursion, allocated for a
/// capacity and reused.
#[derive(Clone, Debug)]
pub struct PrefixSumScratch {
    /// One buffer per level: level `k` holds the block totals of level `k - 1`.
    levels: Vec<Buffer>,
    /// Bound as `sums` where a pass writes none, because a binding cannot be
    /// left empty.
    unused: Buffer,
    capacity: ScanElementCount,
}

/// The two compiled passes.
#[derive(Clone, Debug)]
pub struct PrefixSum {
    scan_blocks: Arc<Kernel>,
    add_offsets: Arc<Kernel>,
}

/// The scan's application-owned failures, retaining lower-level GPU causes.
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("scan requires {requested} elements but scratch holds {capacity}")]
    ScratchTooSmall {
        requested: ScanElementCount,
        capacity: ScanElementCount,
    },
    #[error("scan requires {required:?} but the input buffer holds {available:?}")]
    BufferTooSmall {
        required: BufferByteLength,
        available: BufferByteLength,
    },
    #[error("scan scratch is missing a recursion level")]
    MissingScratchLevel,
    #[error("scan GPU operation failed")]
    Gpu(#[from] anyhow::Error),
}

impl ScanElementCount {
    /// Admit an element count at a native collection or GPU-word boundary.
    pub const fn from_elements(elements: u32) -> Self {
        Self(elements)
    }

    /// The GPU uniform's fixed-width representation.
    pub const fn elements(self) -> u32 {
        self.0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    fn spans_multiple_blocks(self) -> bool {
        self.0 > BLOCK
    }

    fn blocks(self) -> ScanBlockCount {
        ScanBlockCount(self.0.div_ceil(BLOCK))
    }

    pub fn byte_length(self) -> BufferByteLength {
        BufferByteLength::from(u64::from(self.0) * std::mem::size_of::<u32>() as u64)
    }
}

impl From<ScanElementCount> for PassParameter {
    fn from(count: ScanElementCount) -> Self {
        Self::Unsigned(count.0)
    }
}

impl ScanBlockCount {
    fn is_single(self) -> bool {
        self.0 == 1
    }

    /// One recorded total becomes one element of the next recursive scan.
    fn totals(self) -> ScanElementCount {
        ScanElementCount(self.0)
    }

    fn workgroups(self) -> WorkgroupGrid {
        WorkgroupGrid::folded_x(self.0)
    }
}

impl std::fmt::Display for ScanElementCount {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl PrefixSumScratch {
    pub fn new(context: &WgpuContext, capacity: ScanElementCount) -> Result<Self> {
        let capacity = capacity.max(ScanElementCount::from_elements(1));
        let storage = BufferDefinition::storage();
        let mut levels = Vec::new();
        let mut count = capacity;
        while count.spans_multiple_blocks() {
            count = count.blocks().totals();
            levels.push(Buffer::new(
                context,
                count.byte_length(),
                storage.clone().with_label("prefix sum block totals".into()),
            )?);
        }
        Ok(Self {
            levels,
            unused: Buffer::new(
                context,
                BufferByteLength::from(std::mem::size_of::<u32>()),
                storage.with_label("prefix sum unused".into()),
            )?,
            capacity,
        })
    }

    pub fn capacity(&self) -> ScanElementCount {
        self.capacity
    }

    /// Reallocate if `capacity` no longer fits. Returns whether it grew.
    pub fn ensure(&mut self, context: &WgpuContext, capacity: ScanElementCount) -> Result<bool> {
        if capacity <= self.capacity {
            return Ok(false);
        }
        *self = Self::new(context, capacity)?;
        Ok(true)
    }
}

impl PrefixSum {
    pub fn new(context: &WgpuContext) -> Result<Self> {
        Self::with_cache(context, &KernelCache::new())
    }

    pub fn with_cache(context: &WgpuContext, cache: &KernelCache) -> Result<Self> {
        Ok(Self {
            scan_blocks: cache.get(context, &scan_blocks_code())?,
            add_offsets: cache.get(context, &add_offsets_code())?,
        })
    }

    /// Record an exclusive prefix sum of the first `count` words of `data`,
    /// in place: afterwards `data[i]` is the sum of the original
    /// `data[0..i]`.
    ///
    /// Sums wrap on overflow, as `u32` addition in WGSL does. To get the
    /// total as well, scan one element more than there are values and leave
    /// the extra one zero; it ends up holding the sum of everything before it.
    pub fn record(
        &self,
        batch: &mut KernelBatch,
        data: &Buffer,
        scratch: &PrefixSumScratch,
        count: ScanElementCount,
    ) -> Result<()> {
        if count.is_empty() {
            return Ok(());
        }
        if count > scratch.capacity {
            return Err(ScanError::ScratchTooSmall {
                requested: count,
                capacity: scratch.capacity,
            });
        }
        let required = count.byte_length();
        if data.size < required {
            return Err(ScanError::BufferTooSmall {
                required,
                available: data.size,
            });
        }
        self.record_level(batch, data, &scratch.levels, &scratch.unused, count)
    }

    fn record_level(
        &self,
        batch: &mut KernelBatch,
        data: &Buffer,
        levels: &[Buffer],
        unused: &Buffer,
        count: ScanElementCount,
    ) -> Result<()> {
        let blocks = count.blocks();
        let mut parameters = PassParameters::new();
        parameters.insert("count", PassParameter::from(count));
        parameters.insert("pad0", 0u32);
        parameters.insert("pad1", 0u32);
        parameters.insert("data", data.clone());

        if blocks.is_single() {
            parameters.insert("write_sums", 0u32);
            parameters.insert("sums", unused.clone());
            batch.dispatch(&self.scan_blocks, &parameters, blocks.workgroups())?;
            return Ok(());
        }

        let (sums, deeper) = levels.split_first().ok_or(ScanError::MissingScratchLevel)?;
        parameters.insert("write_sums", 1u32);
        parameters.insert("sums", sums.clone());
        batch.dispatch(&self.scan_blocks, &parameters, blocks.workgroups())?;

        self.record_level(batch, sums, deeper, unused, blocks.totals())?;

        batch.dispatch(&self.add_offsets, &parameters, blocks.workgroups())?;
        Ok(())
    }

    /// Scan on a batch of its own and submit. For a one-off; inside a frame,
    /// record into the frame's batch.
    pub fn run(
        &self,
        context: &WgpuContext,
        data: &Buffer,
        scratch: &PrefixSumScratch,
        count: ScanElementCount,
    ) -> Result<()> {
        let mut batch = KernelBatch::labelled(context, "PrefixSum");
        self.record(&mut batch, data, scratch, count)?;
        batch.submit();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fabelgeist_gpu::prelude::BufferUpload;

    fn exclusive(values: &[u32]) -> Vec<u32> {
        let mut running = 0u32;
        values
            .iter()
            .map(|value| {
                let before = running;
                running = running.wrapping_add(*value);
                before
            })
            .collect()
    }

    #[tokio::test]
    async fn matches_the_host_at_every_depth() -> anyhow::Result<()> {
        let context = WgpuContext::new().await?;
        let prefix_sum = PrefixSum::new(&context)?;

        // One block, a ragged last block, exactly two levels, and three.
        for count in [1u32, 7, 256, 257, 1000, 65_536, 65_537, 300_001] {
            let values: Vec<u32> = (0..count)
                .map(|i| i.wrapping_mul(2_654_435_761) >> 28)
                .collect();
            let buffer = Buffer::from_upload(
                &context,
                BufferUpload::from_elements(&values),
                BufferDefinition::storage(),
            )?;
            let scratch = PrefixSumScratch::new(&context, ScanElementCount::from_elements(count))?;
            prefix_sum.run(
                &context,
                &buffer,
                &scratch,
                ScanElementCount::from_elements(count),
            )?;

            let scanned: Vec<u32> = buffer.read(&context).await?;
            assert_eq!(scanned, exclusive(&values), "count {count}");
        }
        Ok(())
    }

    #[tokio::test]
    async fn leaves_the_words_past_count_alone() -> anyhow::Result<()> {
        let context = WgpuContext::new().await?;
        let prefix_sum = PrefixSum::new(&context)?;
        let values = vec![1u32; 600];
        let buffer = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&values),
            BufferDefinition::storage(),
        )?;
        let scratch = PrefixSumScratch::new(&context, ScanElementCount::from_elements(600))?;
        prefix_sum.run(
            &context,
            &buffer,
            &scratch,
            ScanElementCount::from_elements(513),
        )?;

        let scanned: Vec<u32> = buffer.read(&context).await?;
        assert_eq!(&scanned[..513], &(0..513).collect::<Vec<_>>()[..]);
        assert!(scanned[513..].iter().all(|&value| value == 1));
        Ok(())
    }

    #[tokio::test]
    async fn refuses_more_than_its_scratch_holds() -> anyhow::Result<()> {
        let context = WgpuContext::new().await?;
        let prefix_sum = PrefixSum::new(&context)?;
        let buffer = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[0u32; 1024]),
            BufferDefinition::storage(),
        )?;
        let scratch = PrefixSumScratch::new(&context, ScanElementCount::from_elements(100))?;
        assert!(
            matches!(prefix_sum.run(&context, &buffer, &scratch, ScanElementCount::from_elements(1024)),
            Err(ScanError::ScratchTooSmall { requested, capacity })
                if requested == ScanElementCount::from_elements(1024) && capacity == ScanElementCount::from_elements(100))
        );
        Ok(())
    }
}
