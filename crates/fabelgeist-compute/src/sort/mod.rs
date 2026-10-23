//! Least-significant-digit radix sort of `u32` keys, carrying a `u32` payload.
//!
//! A linear BVH sorts primitives by Morton code; cloth self-collision groups
//! particles by hash bucket. Vertex normals group corners by vertex, and
//! underlayer fitting groups weld hashes and triangle-grid entries. Each
//! consumer records into its own [`KernelBatch`] without a host readback.
//!
//! Eight bits per pass, so a full 32-bit key takes four passes and a 30-bit
//! Morton code takes four as well (`bits` rounds up to a whole digit). Each
//! pass has three stages:
//!
//! 1. **histogram** -- one workgroup per tile counts its 256 digits into
//!    shared memory and writes the counts to `histogram[digit][tile]`.
//! 2. **scan** -- an exclusive [`PrefixSum`] over the whole histogram turns
//!    those counts into the global offset each `(digit, tile)` pair writes
//!    at. Digit-major is what makes one flat scan enough: everything before
//!    `(d, t)` in the array is every smaller digit from every tile, then
//!    digit `d` from the earlier tiles -- exactly what lands before it in the
//!    output. (A single workgroup used to do this, each thread walking one
//!    digit's row; past a few hundred thousand keys that serial walk was most
//!    of the sort.)
//! 3. **scatter** -- each element's destination is its tile's base for that
//!    digit, plus its rank among the same-digit elements earlier in the tile.
//!
//! The rank in step 3 is computed by counting matching digits over the tile in
//! shared memory: `TILE` reads per element, all 256 threads in parallel. A
//! subgroup ballot would make it a couple of instructions instead, but ballots
//! are an optional wgpu feature and this is nowhere near the bottleneck for
//! the tens of thousands of elements a cloth solve sorts.

use crate::prelude::*;
pub use quantities::{
    ScratchGrowth, SortDigit, SortDigits, SortItemCount, SortKeyWidth, SortLayoutError, SortPassCount,
};
use quantities::{SortCopyBack, SortWork};
use std::sync::Arc;

mod quantities;

/// Elements per tile, and threads per workgroup. One element per thread.
const TILE: u32 = 256;
/// Digit values per eight-bit pass; each histogram tile writes one row of
/// counts across these bins.
const RADIX: u32 = 256;
/// Bits consumed per pass.
const BITS_PER_PASS: u32 = 8;

fn histogram_code() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> keys: array<u32>;
@group(0) @binding(1) var<storage, read_write> histogram: array<u32>;

struct Params {{
    count: u32,
    shift: u32,
    tiles: u32,
    pad: u32,
}};
@group(0) @binding(2) var<uniform> params: Params;

var<workgroup> local_histogram: array<atomic<u32>, {RADIX}u>;

@compute @workgroup_size({TILE}u)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
) {{
    atomicStore(&local_histogram[local_id.x], 0u);
    workgroupBarrier();

    let index = global_id.x;
    if (index < params.count) {{
        let digit = (keys[index] >> params.shift) & {mask}u;
        atomicAdd(&local_histogram[digit], 1u);
    }}
    workgroupBarrier();

    // Digit-major ordering lets one flat exclusive scan include every
    // smaller digit, then this digit's earlier tiles.
    histogram[local_id.x * params.tiles + group_id.x] =
        atomicLoad(&local_histogram[local_id.x]);
}}
"#,
        mask = RADIX - 1
    )
}

fn scatter_code() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> keys_in: array<u32>;
@group(0) @binding(1) var<storage, read> values_in: array<u32>;
@group(0) @binding(2) var<storage, read_write> keys_out: array<u32>;
@group(0) @binding(3) var<storage, read_write> values_out: array<u32>;
@group(0) @binding(4) var<storage, read> histogram: array<u32>;

struct Params {{
    count: u32,
    shift: u32,
    tiles: u32,
    pad: u32,
}};
@group(0) @binding(5) var<uniform> params: Params;

var<workgroup> tile_digits: array<u32, {TILE}u>;

@compute @workgroup_size({TILE}u)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
) {{
    let index = global_id.x;
    let in_range = index < params.count;

    // Out-of-range lanes park on a digit no live lane can hold. They sit past
    // the end of the tile, so a live lane would never count them anyway; this
    // just keeps the shared array from carrying a digit that means something.
    var digit = {RADIX}u;
    var key = 0u;
    if (in_range) {{
        key = keys_in[index];
        digit = (key >> params.shift) & {mask}u;
    }}
    tile_digits[local_id.x] = digit;
    workgroupBarrier();

    if (in_range) {{
        // Rank among the same-digit elements earlier in this tile. Counting
        // rather than scanning keeps the sort stable, which is what makes the
        // whole least-significant-digit scheme work.
        var rank = 0u;
        for (var i = 0u; i < local_id.x; i = i + 1u) {{
            if (tile_digits[i] == digit) {{
                rank = rank + 1u;
            }}
        }}
        let destination = histogram[digit * params.tiles + group_id.x] + rank;
        keys_out[destination] = key;
        values_out[destination] = values_in[index];
    }}
}}
"#,
        mask = RADIX - 1
    )
}

/// Scratch buffers a sort needs: one ping-pong pair, the histogram, and the
/// histogram scan's own scratch.
///
/// Allocated for a capacity and reused, so that a per-frame sort does not
/// allocate. Grow it with [`SortScratch::ensure`].
#[derive(Clone, Debug)]
pub struct SortScratch {
    keys: Buffer,
    values: Buffer,
    histogram: Buffer,
    scan: PrefixSumScratch,
    capacity: SortItemCount,
}

impl SortScratch {
    pub fn new(context: &WgpuContext, capacity: SortItemCount) -> Result<Self> {
        let capacity = capacity.with_sentinel();
        let tiles = capacity.tiles();
        let histogram_count = tiles.scan_elements()?;
        let storage = BufferDefinition::storage();
        Ok(Self {
            keys: Buffer::new(
                context,
                capacity.word_bytes(),
                storage.clone().with_label(("sort keys").into()),
            )?,
            values: Buffer::new(
                context,
                capacity.word_bytes(),
                storage.clone().with_label(("sort values").into()),
            )?,
            histogram: Buffer::new(
                context,
                tiles.histogram_bytes(),
                storage.with_label(("sort histogram").into()),
            )?,
            scan: PrefixSumScratch::new(context, histogram_count)?,
            capacity,
        })
    }

    pub fn capacity(&self) -> SortItemCount {
        self.capacity
    }

    /// Reallocate if `capacity` no longer fits. Returns the resulting growth state.
    pub fn ensure(
        &mut self,
        context: &WgpuContext,
        capacity: SortItemCount,
    ) -> Result<ScratchGrowth> {
        if capacity <= self.capacity {
            return Ok(ScratchGrowth::Unchanged);
        }
        *self = Self::new(context, capacity)?;
        Ok(ScratchGrowth::Grown)
    }
}

/// The compiled passes.
#[derive(Clone, Debug)]
pub struct RadixSort {
    histogram: Arc<Kernel>,
    scan: PrefixSum,
    scatter: Arc<Kernel>,
}

impl RadixSort {
    pub fn new(context: &WgpuContext) -> Result<Self> {
        Self::with_cache(context, &KernelCache::new())
    }

    pub fn with_cache(context: &WgpuContext, cache: &KernelCache) -> Result<Self> {
        Ok(Self {
            histogram: cache.get(context, &histogram_code())?,
            scan: PrefixSum::with_cache(context, cache)?,
            scatter: cache.get(context, &scatter_code())?,
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
    ) -> Result<()> {
        if count.work() == SortWork::NoOp {
            return Ok(());
        }
        if scratch.capacity < count {
            return Err(anyhow!(
                "RadixSort: scratch holds {} elements, asked to sort {count}",
                scratch.capacity
            ));
        }
        let needed = count.word_bytes();
        if keys.size < needed || values.size < needed {
            return Err(anyhow!(
                "RadixSort: {count} elements need {needed} bytes; keys hold {}, values hold {}",
                u64::from(keys.size),
                u64::from(values.size)
            ));
        }

        let tiles = count.tiles();
        let histogram_count = tiles.scan_elements()?;
        let passes = bits.pass_count();

        let mut source_keys = keys.clone();
        let mut source_values = values.clone();
        let mut target_keys = scratch.keys.clone();
        let mut target_values = scratch.values.clone();

        for digit in bits.digits() {
            let mut parameters = PassParameters::new();
            count.bind(&mut parameters);
            digit.bind(&mut parameters);
            tiles.bind(&mut parameters);
            parameters.insert("pad", 0u32);
            parameters.insert("histogram", scratch.histogram.clone());

            let mut histogram_parameters = parameters.clone();
            histogram_parameters.insert("keys", source_keys.clone());
            batch.dispatch(
                &self.histogram,
                &histogram_parameters,
                tiles.workgroups(),
            )?;

            self.scan.record(
                batch,
                &scratch.histogram,
                &scratch.scan,
                histogram_count,
            )?;

            let mut scatter_parameters = parameters;
            scatter_parameters.insert("keys_in", source_keys.clone());
            scatter_parameters.insert("values_in", source_values.clone());
            scatter_parameters.insert("keys_out", target_keys.clone());
            scatter_parameters.insert("values_out", target_values.clone());
            batch.dispatch(&self.scatter, &scatter_parameters, tiles.workgroups())?;

            std::mem::swap(&mut source_keys, &mut target_keys);
            std::mem::swap(&mut source_values, &mut target_values);
        }

        if passes.copy_back() == SortCopyBack::Required {
            batch.copy_buffer(&source_keys, keys, needed)?;
            batch.copy_buffer(&source_values, values, needed)?;
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
    ) -> Result<()> {
        let mut batch = KernelBatch::labelled(context, "RadixSort");
        self.record(&mut batch, keys, values, scratch, count, bits)?;
        batch.submit();
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod contract_tests;
