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
//! back. Three levels cover 16.7 million elements, and nothing limits the
//! depth beyond that.

use crate::prelude::*;
use fabelgeist_gpu::prelude::BufferByteLength;
use std::sync::Arc;

/// Elements per block, and threads per workgroup.
const BLOCK: u32 = 256;

fn scan_blocks_code() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read_write> data: array<u32>;
@group(0) @binding(1) var<storage, read_write> sums: array<u32>;

struct Params {{
    count: u32,
    write_sums: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(2) var<uniform> params: Params;

var<workgroup> shared_values: array<u32, {BLOCK}u>;

@compute @workgroup_size({BLOCK}u)
fn main(
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
    @builtin(num_workgroups) groups: vec3<u32>,
) {{
    // Blocks fold into y past the 65535 groups one dimension holds.
    let block = group_id.y * groups.x + group_id.x;
    let index = block * {BLOCK}u + local_id.x;

    var value = 0u;
    if (index < params.count) {{
        value = data[index];
    }}
    shared_values[local_id.x] = value;
    workgroupBarrier();

    // Hillis-Steele, inclusive. Every lane reads before any lane writes, so
    // the barrier between them is all the double-buffering it needs.
    for (var offset = 1u; offset < {BLOCK}u; offset = offset * 2u) {{
        var carried = 0u;
        if (local_id.x >= offset) {{
            carried = shared_values[local_id.x - offset];
        }}
        workgroupBarrier();
        shared_values[local_id.x] = shared_values[local_id.x] + carried;
        workgroupBarrier();
    }}

    let inclusive = shared_values[local_id.x];
    if (index < params.count) {{
        data[index] = inclusive - value;
    }}
    if (local_id.x == {BLOCK}u - 1u && params.write_sums != 0u) {{
        sums[block] = inclusive;
    }}
}}
"#
    )
}

fn add_offsets_code() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read_write> data: array<u32>;
@group(0) @binding(1) var<storage, read> sums: array<u32>;

struct Params {{
    count: u32,
    write_sums: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size({BLOCK}u)
fn main(
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
    @builtin(num_workgroups) groups: vec3<u32>,
) {{
    let block = group_id.y * groups.x + group_id.x;
    let index = block * {BLOCK}u + local_id.x;
    if (index < params.count) {{
        data[index] = data[index] + sums[block];
    }}
}}
"#
    )
}

/// Workgroups for `blocks` blocks, folded into y past one dimension's limit.
fn grid(blocks: u32) -> [u32; 3] {
    let x = blocks.clamp(1, MAX_GROUPS);
    [x, blocks.div_ceil(x), 1]
}

/// The block totals of every level of the recursion, allocated for a
/// capacity and reused.
#[derive(Clone, Debug)]
pub struct PrefixSumScratch {
    /// One buffer per level: level `k` holds the block totals of level `k - 1`.
    levels: Vec<Buffer>,
    /// Bound as `sums` where a pass writes none, because a binding cannot be
    /// left empty.
    unused: Buffer,
    capacity: u32,
}

impl PrefixSumScratch {
    pub fn new(context: &WgpuContext, capacity: u32) -> Result<Self> {
        let capacity = capacity.max(1);
        let storage = BufferDefinition::storage();
        let mut levels = Vec::new();
        let mut count = capacity;
        while count > BLOCK {
            count = count.div_ceil(BLOCK);
            levels.push(Buffer::new(
                context,
                (count as u64 * 4).into(),
                storage
                    .clone()
                    .with_label(("prefix sum block totals").into()),
            )?);
        }
        Ok(Self {
            levels,
            unused: Buffer::new(
                context,
                BufferByteLength::from(4_u64),
                storage.with_label(("prefix sum unused").into()),
            )?,
            capacity,
        })
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Reallocate if `capacity` no longer fits. Returns whether it grew.
    pub fn ensure(&mut self, context: &WgpuContext, capacity: u32) -> Result<bool> {
        if capacity <= self.capacity {
            return Ok(false);
        }
        *self = Self::new(context, capacity)?;
        Ok(true)
    }
}

/// The two compiled passes.
#[derive(Clone, Debug)]
pub struct PrefixSum {
    scan_blocks: Arc<Kernel>,
    add_offsets: Arc<Kernel>,
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
        count: u32,
    ) -> Result<()> {
        if count == 0 {
            return Ok(());
        }
        if count > scratch.capacity {
            return Err(anyhow!(
                "PrefixSum: scratch holds {} elements, asked to scan {count}",
                scratch.capacity
            ));
        }
        if data.size < (count as u64 * 4).into() {
            return Err(anyhow!(
                "PrefixSum: {count} elements need {} bytes; the buffer holds {}",
                count as u64 * 4,
                data.size
            ));
        }
        self.record_level(batch, data, &scratch.levels, &scratch.unused, count)
    }

    fn record_level(
        &self,
        batch: &mut KernelBatch,
        data: &Buffer,
        levels: &[Buffer],
        unused: &Buffer,
        count: u32,
    ) -> Result<()> {
        let blocks = count.div_ceil(BLOCK);
        let mut parameters = PassParameters::new();
        parameters.insert("count", count);
        parameters.insert("pad0", 0u32);
        parameters.insert("pad1", 0u32);
        parameters.insert("data", data.clone());

        if blocks == 1 {
            parameters.insert("write_sums", 0u32);
            parameters.insert("sums", unused.clone());
            batch.dispatch(&self.scan_blocks, &parameters, grid(1))?;
            return Ok(());
        }

        let (sums, deeper) = levels
            .split_first()
            .ok_or_else(|| anyhow!("PrefixSum: scratch is missing a level"))?;
        parameters.insert("write_sums", 1u32);
        parameters.insert("sums", sums.clone());
        batch.dispatch(&self.scan_blocks, &parameters, grid(blocks))?;

        self.record_level(batch, sums, deeper, unused, blocks)?;

        batch.dispatch(&self.add_offsets, &parameters, grid(blocks))?;
        Ok(())
    }

    /// Scan on a batch of its own and submit. For a one-off; inside a frame,
    /// record into the frame's batch.
    pub fn run(
        &self,
        context: &WgpuContext,
        data: &Buffer,
        scratch: &PrefixSumScratch,
        count: u32,
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
    async fn matches_the_host_at_every_depth() -> Result<()> {
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
            let scratch = PrefixSumScratch::new(&context, count)?;
            prefix_sum.run(&context, &buffer, &scratch, count)?;

            let scanned: Vec<u32> = buffer.read(&context).await?;
            assert_eq!(scanned, exclusive(&values), "count {count}");
        }
        Ok(())
    }

    #[tokio::test]
    async fn leaves_the_words_past_count_alone() -> Result<()> {
        let context = WgpuContext::new().await?;
        let prefix_sum = PrefixSum::new(&context)?;
        let values = vec![1u32; 600];
        let buffer = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&values),
            BufferDefinition::storage(),
        )?;
        let scratch = PrefixSumScratch::new(&context, 600)?;
        prefix_sum.run(&context, &buffer, &scratch, 513)?;

        let scanned: Vec<u32> = buffer.read(&context).await?;
        assert_eq!(&scanned[..513], &(0..513).collect::<Vec<_>>()[..]);
        assert!(scanned[513..].iter().all(|&value| value == 1));
        Ok(())
    }

    #[tokio::test]
    async fn refuses_more_than_its_scratch_holds() -> Result<()> {
        let context = WgpuContext::new().await?;
        let prefix_sum = PrefixSum::new(&context)?;
        let buffer = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[0u32; 1024]),
            BufferDefinition::storage(),
        )?;
        let scratch = PrefixSumScratch::new(&context, 100)?;
        assert!(prefix_sum.run(&context, &buffer, &scratch, 1024).is_err());
        Ok(())
    }
}
