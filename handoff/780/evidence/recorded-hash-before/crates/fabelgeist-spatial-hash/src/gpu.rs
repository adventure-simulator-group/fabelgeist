//! The GPU grid: hash, radix sort and bucket ranges as compute passes.
//!
//! # Buffer layout
//!
//! * `points: array<vec4<f32>>` -- the caller's; `.xyz` is hashed, `.w` is
//!   free for a radius or anything else.
//! * `hash_indices: array<u32>` -- entry indices, grouped by bucket.
//! * `hash_ranges: array<u32>` -- two words per bucket, `[begin, end)` into
//!   `hash_indices`; `begin` is [`EMPTY`](crate::EMPTY) for an empty bucket.
//!
//! A kernel that queries the grid binds `hash_indices` and `hash_ranges` under
//! those names ([`GpuHashGrid::bind`] does it), pastes [`HASH_WGSL`] and
//! [`RANGE_WGSL`], and
//! walks the cells around its point:
//!
//! ```wgsl
//! let base = spatial_hash_cell(p, inverse_cell);
//! for (var dz = -1; dz <= 1; dz++) { for (var dy = -1; dy <= 1; dy++) { for (var dx = -1; dx <= 1; dx++) {
//!     let cell = base + vec3<i32>(dx, dy, dz);
//!     let range = spatial_hash_range(cell, table_size);
//!     for (var k = range.x; k < range.y; k++) {
//!         let j = hash_indices[k];
//!         if (any(spatial_hash_cell(points[j].xyz, inverse_cell) != cell)) { continue; }
//!         // ... j is a neighbour candidate
//!     }
//! }}}
//! ```

use anyhow::{Result, anyhow};
use fabelgeist_compute::{
    FOLDED_INDEX_WGSL, Kernel, KernelBatch, KernelCache, RadixSort, SortScratch,
};
use fabelgeist_gpu::data::gpu::buffer::{Buffer, BufferCapacityChange, BufferDefinition};
use fabelgeist_gpu::data::gpu::parameters::PassParameters;
use fabelgeist_gpu::globals::WgpuContext;
use std::sync::Arc;

use crate::{CellScale, HashBucketCount, HashLayout, HashPointCount};

/// The cell and bucket functions alone, with no bindings.
pub const HASH_WGSL: &str = r#"
fn spatial_hash_cell(p: vec3<f32>, inverse_cell: f32) -> vec3<i32> {
    return vec3<i32>(floor(p * inverse_cell));
}

fn spatial_hash_bucket(cell: vec3<i32>, table_size: u32) -> u32 {
    let h = (u32(cell.x) * 73856093u) ^ (u32(cell.y) * 19349663u) ^ (u32(cell.z) * 83492791u);
    return h & (table_size - 1u);
}
"#;

/// The range lookup, for kernels to paste in after [`HASH_WGSL`]. Expects
/// `hash_ranges: array<u32>` bound in the including kernel.
pub const RANGE_WGSL: &str = r#"
const SPATIAL_HASH_EMPTY: u32 = 0xFFFFFFFFu;

// [begin, end) into `hash_indices`; empty when begin == end.
fn spatial_hash_range(cell: vec3<i32>, table_size: u32) -> vec2<u32> {
    let bucket = spatial_hash_bucket(cell, table_size);
    let begin = hash_ranges[bucket * 2u];
    if (begin == SPATIAL_HASH_EMPTY) {
        return vec2<u32>(0u, 0u);
    }
    return vec2<u32>(begin, hash_ranges[bucket * 2u + 1u]);
}
"#;

const PARAMS: &str = r#"
struct HashParams {
    count: u32,
    table_size: u32,
    inverse_cell: f32,
    pad: u32,
};
"#;

/// The compiled passes, shareable between grids.
#[derive(Clone, Debug)]
pub struct HashKernels {
    hash: Arc<Kernel>,
    clear: Arc<Kernel>,
    ranges: Arc<Kernel>,
    sort: RadixSort,
}

/// A spatial hash over the checked layout capacity, built on the GPU.
pub struct GpuHashGrid {
    kernels: HashKernels,
    layout: HashLayout,
    recorded: HashBuild,
    keys: Buffer,
    indices: Buffer,
    ranges: Buffer,
    scratch: SortScratch,
}

/// Metadata of the last successfully recorded build, in caller coordinates.
#[derive(Clone, Copy, Debug)]
struct HashBuild {
    count: HashPointCount,
    scale: CellScale,
}

impl HashKernels {
    pub fn new(context: &WgpuContext, cache: &KernelCache) -> Result<Self> {
        Ok(Self {
            hash: cache.get(context, &hash_source())?,
            clear: cache.get(context, &clear_source())?,
            ranges: cache.get(context, &ranges_source())?,
            sort: RadixSort::with_cache(context, cache)?,
        })
    }
}

impl GpuHashGrid {
    pub fn new(context: &WgpuContext, kernels: HashKernels, layout: HashLayout) -> Result<Self> {
        let layout = layout.with_sentinel();
        let capacity = layout.point_count();
        let storage = BufferDefinition::storage();
        Ok(Self {
            kernels,
            layout,
            recorded: HashBuild {
                count: HashPointCount::default(),
                scale: CellScale::from_cell_size(1.0),
            },
            keys: Buffer::new(
                context,
                capacity.word_bytes(),
                storage.clone().with_label(("hash keys").into()),
            )?,
            indices: Buffer::new(
                context,
                capacity.word_bytes(),
                storage.clone().with_label(("hash indices").into()),
            )?,
            ranges: Buffer::new(
                context,
                layout.bucket_count().range_bytes(),
                storage.with_label(("hash ranges").into()),
            )?,
            scratch: SortScratch::new(context, capacity.into())?,
        })
    }

    /// Reallocate if `requested` no longer fits, reporting the allocation outcome.
    /// Grows by half again, so a slowly growing population does not
    /// reallocate every step.
    pub fn ensure_capacity(
        &mut self,
        context: &WgpuContext,
        requested: HashLayout,
    ) -> Result<BufferCapacityChange> {
        if requested.point_count() <= self.capacity() {
            return Ok(BufferCapacityChange::Unchanged);
        }
        let grown = self.layout.grown_for(requested);
        *self = Self::new(context, self.kernels.clone(), grown)?;
        Ok(BufferCapacityChange::Grown)
    }

    pub fn capacity(&self) -> HashPointCount {
        self.layout.point_count()
    }

    pub fn table_size(&self) -> HashBucketCount {
        self.layout.bucket_count()
    }

    pub fn cell_scale(&self) -> CellScale {
        self.recorded.scale
    }

    pub fn count(&self) -> HashPointCount {
        self.recorded.count
    }

    /// Record a build over the first `count` entries of `points`
    /// (`array<vec4<f32>>`), using `scale` in their coordinate units.
    /// Projected cells and query offsets must fit native signed cell coordinates.
    /// On error, discard the partially recorded batch. Successful encoding does
    /// not establish execution or certify the external point buffer's identity.
    pub fn record_build(
        &mut self,
        batch: &mut KernelBatch,
        points: &Buffer,
        count: HashPointCount,
        scale: CellScale,
    ) -> Result<()> {
        if count > self.capacity() {
            return Err(anyhow!(
                "GpuHashGrid: built for {} points, asked for {count}",
                self.capacity()
            ));
        }
        let available = points.size.min(points.buffer.size().into());
        if count.point_bytes() > available {
            return Err(anyhow!(
                "GpuHashGrid: {count} points require {} bytes, buffer holds {available}",
                count.point_bytes()
            ));
        }
        let build = HashBuild { count, scale };

        let mut clear = self.params(build);
        clear.insert("hash_ranges", self.ranges.clone());
        batch.dispatch_folded(&self.kernels.clear, &clear, self.table_size().into())?;
        if count.is_empty() {
            self.recorded = build;
            return Ok(());
        }

        let mut hash = self.params(build);
        hash.insert("points", points.clone());
        hash.insert("hash_keys", self.keys.clone());
        hash.insert("hash_indices", self.indices.clone());
        batch.dispatch_folded(&self.kernels.hash, &hash, count.into())?;

        let bits = self.table_size().sort_key_width();
        self.kernels.sort.record(
            batch,
            &self.keys,
            &self.indices,
            &mut self.scratch,
            count.into(),
            bits,
        )?;

        let mut ranges = self.params(build);
        ranges.insert("hash_keys", self.keys.clone());
        ranges.insert("hash_ranges", self.ranges.clone());
        batch.dispatch_folded(&self.kernels.ranges, &ranges, count.into())?;
        self.recorded = build;
        Ok(())
    }

    /// Bind `hash_indices` and `hash_ranges` for a querying kernel.
    pub fn bind(&self, parameters: &mut PassParameters) {
        parameters.insert("hash_indices", self.indices.clone());
        parameters.insert("hash_ranges", self.ranges.clone());
    }

    fn params(&self, build: HashBuild) -> PassParameters {
        let mut parameters = PassParameters::new();
        parameters.insert("count", build.count);
        parameters.insert("table_size", self.table_size());
        parameters.insert("inverse_cell", build.scale);
        parameters.insert("pad", 0u32);
        parameters
    }
}

fn hash_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> points: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> hash_keys: array<u32>;
@group(0) @binding(2) var<storage, read_write> hash_indices: array<u32>;
{PARAMS}
@group(0) @binding(3) var<uniform> params: HashParams;
{HASH_WGSL}
{FOLDED_INDEX_WGSL}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let i = folded_index(id, groups, 256u);
    if (i >= params.count) {{
        return;
    }}
    hash_keys[i] = spatial_hash_bucket(spatial_hash_cell(points[i].xyz, params.inverse_cell), params.table_size);
    hash_indices[i] = i;
}}
"#
    )
}

fn clear_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read_write> hash_ranges: array<u32>;
{PARAMS}
@group(0) @binding(1) var<uniform> params: HashParams;
{FOLDED_INDEX_WGSL}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let i = folded_index(id, groups, 256u);
    if (i >= params.table_size) {{
        return;
    }}
    hash_ranges[i * 2u] = 0xFFFFFFFFu;
    hash_ranges[i * 2u + 1u] = 0xFFFFFFFFu;
}}
"#
    )
}

fn ranges_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> hash_keys: array<u32>;
@group(0) @binding(1) var<storage, read_write> hash_ranges: array<u32>;
{PARAMS}
@group(0) @binding(2) var<uniform> params: HashParams;
{FOLDED_INDEX_WGSL}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let i = folded_index(id, groups, 256u);
    if (i >= params.count) {{
        return;
    }}
    let key = hash_keys[i];
    if (i == 0u || hash_keys[i - 1u] != key) {{
        hash_ranges[key * 2u] = i;
    }}
    if (i + 1u == params.count || hash_keys[i + 1u] != key) {{
        hash_ranges[key * 2u + 1u] = i + 1u;
    }}
}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HashGrid;
    use fabelgeist_gpu::prelude::BufferUpload;

    const COUNT_KERNEL: &str = r#"
@group(0) @binding(0) var<storage, read> points: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> hash_indices: array<u32>;
@group(0) @binding(2) var<storage, read> hash_ranges: array<u32>;
@group(0) @binding(3) var<storage, read_write> counts: array<u32>;
struct QueryParams {
    count: u32,
    table_size: u32,
    inverse_cell: f32,
    radius: f32,
};
@group(0) @binding(4) var<uniform> params: QueryParams;
"#;

    const COUNT_MAIN: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) { return; }
    let p = points[i].xyz;
    let base = spatial_hash_cell(p, params.inverse_cell);
    var n = 0u;
    for (var dz = -1; dz <= 1; dz++) { for (var dy = -1; dy <= 1; dy++) { for (var dx = -1; dx <= 1; dx++) {
        let cell = base + vec3<i32>(dx, dy, dz);
        let range = spatial_hash_range(cell, params.table_size);
        for (var k = range.x; k < range.y; k++) {
            let j = hash_indices[k];
            if (any(spatial_hash_cell(points[j].xyz, params.inverse_cell) != cell)) { continue; }
            if (distance(points[j].xyz, p) <= params.radius) { n += 1u; }
        }
    }}}
    counts[i] = n;
}
"#;

    #[tokio::test]
    async fn gpu_neighbour_counts_match_the_host_grid() -> Result<()> {
        let context = WgpuContext::new().await?;
        let cache = KernelCache::new();
        let kernels = HashKernels::new(&context, &cache)?;

        let mut state = 12345u32;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state as f32 / u32::MAX as f32) * 16.0 - 8.0
        };
        let points: Vec<[f32; 4]> = (0..5000).map(|_| [next(), next(), next(), 0.0]).collect();
        let radius = 0.9;

        let buffer = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&points),
            BufferDefinition::storage(),
        )?;
        let counts = Buffer::new(
            &context,
            (points.len() as u64 * 4).into(),
            BufferDefinition::storage(),
        )?;
        let count = HashPointCount::try_from(points.len())?;
        let scale = CellScale::from_cell_size(radius);
        let mut grid = GpuHashGrid::new(&context, kernels, HashLayout::try_from(count)?)?;

        let kernel = Kernel::new(
            &context,
            format!("{COUNT_KERNEL}{HASH_WGSL}{RANGE_WGSL}{COUNT_MAIN}"),
        )?;
        let mut batch = KernelBatch::new(&context);
        grid.record_build(&mut batch, &buffer, count, scale)?;
        let mut parameters = PassParameters::new();
        parameters.insert("points", buffer.clone());
        parameters.insert("counts", counts.clone());
        parameters.insert("count", count);
        parameters.insert("table_size", grid.table_size());
        parameters.insert("inverse_cell", grid.cell_scale());
        parameters.insert("radius", radius);
        grid.bind(&mut parameters);
        batch.dispatch_items(&kernel, &parameters, count.into())?;
        batch.submit();

        let gpu: Vec<u32> = counts.read(&context).await?;
        let host = HashGrid::new(points.iter().map(|p| [p[0], p[1], p[2]]), scale)?;
        for (i, p) in points.iter().enumerate() {
            let mut n = 0;
            host.for_each_near([p[0], p[1], p[2]], 1, |j| {
                let q = points[j as usize];
                let d =
                    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
                if d <= radius {
                    n += 1;
                }
            });
            assert_eq!(gpu[i], n, "point {i}");
        }
        Ok(())
    }
}
