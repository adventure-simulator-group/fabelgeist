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

/// Elements per tile, and threads per workgroup. One element per thread.
const TILE: u32 = 256;
/// Digits per pass. Tied to `TILE` only because the scan pass gives each of
/// its 256 threads one digit row.
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

    // Digit-major: every tile's count for one digit is contiguous, which is
    // what lets the scan pass hand one whole row to one thread.
    histogram[local_id.x * params.tiles + group_id.x] =
        atomicLoad(&local_histogram[local_id.x]);
}}
"#,
        mask = RADIX - 1
    )
}

fn scan_code() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read_write> histogram: array<u32>;

struct Params {{
    count: u32,
    shift: u32,
    tiles: u32,
    pad: u32,
}};
@group(0) @binding(1) var<uniform> params: Params;

var<workgroup> digit_totals: array<u32, {RADIX}u>;
var<workgroup> scratch: array<u32, {RADIX}u>;

@compute @workgroup_size({RADIX}u)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {{
    let digit = local_id.x;
    let row = digit * params.tiles;

    // Exclusive scan along this digit's row, leaving each tile's offset
    // *within* the digit in place, and the digit's total in hand.
    var running = 0u;
    for (var tile = 0u; tile < params.tiles; tile = tile + 1u) {{
        let value = histogram[row + tile];
        histogram[row + tile] = running;
        running = running + value;
    }}
    digit_totals[digit] = running;
    workgroupBarrier();

    // Exclusive scan across the 256 digit totals: where each digit's block
    // starts in the output. Hillis-Steele, double-buffered through `scratch`
    // so no thread reads a slot another has already overwritten.
    var value = digit_totals[digit];
    for (var offset = 1u; offset < {RADIX}u; offset = offset * 2u) {{
        scratch[digit] = value;
        workgroupBarrier();
        if (digit >= offset) {{
            value = value + scratch[digit - offset];
        }}
        workgroupBarrier();
    }}
    // `value` is now the inclusive sum; the exclusive one drops this digit.
    let digit_base = value - digit_totals[digit];

    for (var tile = 0u; tile < params.tiles; tile = tile + 1u) {{
        histogram[row + tile] = histogram[row + tile] + digit_base;
    }}
}}
"#
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

/// Scratch buffers a sort needs: one ping-pong pair and the histogram.
///
/// Allocated for a capacity and reused, so that a per-frame sort does not
/// allocate. Grow it with [`SortScratch::ensure`].
#[derive(Clone, Debug)]
pub struct SortScratch {
    pub keys: Buffer,
    pub values: Buffer,
    pub histogram: Buffer,
    capacity: u32,
}

impl SortScratch {
    pub fn new(context: &WgpuContext, capacity: u32) -> Result<Self> {
        let capacity = capacity.max(1);
        let tiles = capacity.div_ceil(TILE);
        let storage = BufferDefinition::storage();
        Ok(Self {
            keys: Buffer::new(
                context,
                (capacity as u64) * 4,
                storage.clone().with_label("sort keys"),
            )?,
            values: Buffer::new(
                context,
                (capacity as u64) * 4,
                storage.clone().with_label("sort values"),
            )?,
            histogram: Buffer::new(
                context,
                (RADIX as u64) * (tiles as u64) * 4,
                storage.with_label("sort histogram"),
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

/// The three compiled passes.
#[derive(Clone, Debug)]
pub struct RadixSort {
    histogram: Arc<Kernel>,
    scan: Arc<Kernel>,
    scatter: Arc<Kernel>,
}

impl RadixSort {
    pub fn new(context: &WgpuContext) -> Result<Self> {
        Self::with_cache(context, &KernelCache::new())
    }

    pub fn with_cache(context: &WgpuContext, cache: &KernelCache) -> Result<Self> {
        Ok(Self {
            histogram: cache.get(context, &histogram_code())?,
            scan: cache.get(context, &scan_code())?,
            scatter: cache.get(context, &scatter_code())?,
        })
    }

    /// Number of passes a key of `bits` significant bits needs.
    pub fn passes_for(bits: u32) -> u32 {
        bits.clamp(1, 32).div_ceil(BITS_PER_PASS)
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
        count: u32,
        bits: u32,
    ) -> Result<()> {
        if count <= 1 {
            return Ok(());
        }
        if scratch.capacity < count {
            return Err(anyhow!(
                "RadixSort: scratch holds {} elements, asked to sort {count}",
                scratch.capacity
            ));
        }
        let needed = (count as u64) * 4;
        if keys.size < needed || values.size < needed {
            return Err(anyhow!(
                "RadixSort: {count} elements need {needed} bytes; keys hold {}, values hold {}",
                keys.size,
                values.size
            ));
        }

        let tiles = count.div_ceil(TILE);
        let passes = Self::passes_for(bits);

        let mut source_keys = keys.clone();
        let mut source_values = values.clone();
        let mut target_keys = scratch.keys.clone();
        let mut target_values = scratch.values.clone();

        for pass in 0..passes {
            let shift = pass * BITS_PER_PASS;

            let mut parameters = PassParameters::new();
            parameters.insert("count", count);
            parameters.insert("shift", shift);
            parameters.insert("tiles", tiles);
            parameters.insert("pad", 0u32);
            parameters.insert("histogram", scratch.histogram.clone());

            let mut histogram_parameters = parameters.clone();
            histogram_parameters.insert("keys", source_keys.clone());
            batch.dispatch(&self.histogram, &histogram_parameters, [tiles, 1, 1])?;

            batch.dispatch(&self.scan, &parameters, [1, 1, 1])?;

            let mut scatter_parameters = parameters;
            scatter_parameters.insert("keys_in", source_keys.clone());
            scatter_parameters.insert("values_in", source_values.clone());
            scatter_parameters.insert("keys_out", target_keys.clone());
            scatter_parameters.insert("values_out", target_values.clone());
            batch.dispatch(&self.scatter, &scatter_parameters, [tiles, 1, 1])?;

            std::mem::swap(&mut source_keys, &mut target_keys);
            std::mem::swap(&mut source_values, &mut target_values);
        }

        if passes % 2 == 1 {
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
        count: u32,
        bits: u32,
    ) -> Result<()> {
        let mut batch = KernelBatch::labelled(context, "RadixSort");
        self.record(&mut batch, keys, values, scratch, count, bits)?;
        batch.submit();
        Ok(())
    }
}

#[cfg(test)]
mod tests;
