//! The index built on the GPU, from a buffer of shapes already there.
//!
//! A scene that changes every frame -- plants growing, things moving --
//! cannot wait on the host to bin its shapes, and its shapes may never have
//! been on the host at all. This builds the same index as
//! [`crate::SurfaceIndex::build`] in five passes recorded into the caller's
//! encoder, needing nothing but a `wgpu` device:
//!
//! 1. **count**: each shape adds one to every bucket its cells hash to;
//! 2. **scan**: an exclusive prefix sum of the counts gives each bucket's
//!    start;
//! 3. **place**: each shape writes its index into its buckets' slots;
//! 4. **tidy**: each bucket sorts its few entries and blanks the duplicates
//!    ([`crate::index::EMPTY`], which queries skip) -- so the index is the
//!    same whatever order the atomics ran in, and the host's.
//!
//! Keys are buckets, so a counting sort does the work of a general sort in
//! two passes over the shapes. Entries are capped at a capacity the caller
//! chooses; [`crate::SurfaceIndex::entry_count`] gives the exact figure for
//! shapes known on the host, and an upper bound serves for ones that are
//! not. Past it, entries are dropped rather than written out of bounds.

use crate::index::{SurfaceParams, table_size_for};

/// Elements per scan block, and threads per workgroup.
const BLOCK: u32 = 256;
/// Threads per workgroup of the per-shape passes.
const SHAPES_PER_GROUP: u32 = 64;
/// The most dispatch groups along one dimension.
const MAX_GROUPS: u32 = 65_535;

/// The shape layout and each kind's cell span, shared by the passes.
const COMMON: &str = r#"
struct SurfaceShape {
    p0: vec4<f32>,
    p1: vec4<f32>,
    p2: vec4<f32>,
    object: u32,
    payload: u32,
    reach: f32,
    mask: u32,
};

struct BuildParams {
    count: u32,
    table_size: u32,
    inverse_cell: f32,
    capacity: u32,
};

struct Span {
    lo: vec3<i32>,
    hi: vec3<i32>,
};

fn bucket_of(cell: vec3<i32>, table_size: u32) -> u32 {
    let h = (u32(cell.x) * 73856093u) ^ (u32(cell.y) * 19349663u) ^ (u32(cell.z) * 83492791u);
    return h & (table_size - 1u);
}

// Everything within `reach` of the shape's surface, as a box: as
// `Shape::bounds` on the host.
fn bounds_of(s: SurfaceShape) -> array<vec3<f32>, 2> {
    let reach = s.reach;
    switch (s.mask & 0xFFu) {
        case 1u: {
            let q = s.p2;
            let x = q.x; let y = q.y; let z = q.z; let w = q.w;
            let r0 = vec3<f32>(1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - z * w), 2.0 * (x * z + y * w));
            let r1 = vec3<f32>(2.0 * (x * y + z * w), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - x * w));
            let r2 = vec3<f32>(2.0 * (x * z - y * w), 2.0 * (y * z + x * w), 1.0 - 2.0 * (x * x + y * y));
            let half = s.p1.xyz;
            let extent = vec3<f32>(dot(abs(r0), half), dot(abs(r1), half), dot(abs(r2), half)) + s.p0.w + reach;
            return array<vec3<f32>, 2>(s.p0.xyz - extent, s.p0.xyz + extent);
        }
        case 2u: {
            let t = s.p0.w + reach;
            return array<vec3<f32>, 2>(
                min(min(s.p0.xyz, s.p1.xyz), s.p2.xyz) - t,
                max(max(s.p0.xyz, s.p1.xyz), s.p2.xyz) + t,
            );
        }
        case 3u: {
            let h = s.p1;
            let low = min(min(h.x, h.y), min(h.z, h.w));
            let high = max(max(h.x, h.y), max(h.z, h.w));
            let w = max(s.p0.z - s.p0.x, 1e-9);
            let d = max(s.p0.w - s.p0.y, 1e-9);
            let gx = max(abs(h.y - h.x), abs(h.w - h.z)) / w;
            let gz = max(abs(h.z - h.x), abs(h.w - h.y)) / d;
            let vertical = reach * sqrt(1.0 + gx * gx + gz * gz);
            return array<vec3<f32>, 2>(
                vec3<f32>(s.p0.x, low - s.p2.x - vertical, s.p0.y),
                vec3<f32>(s.p0.z, high + vertical, s.p0.w),
            );
        }
        default: {
            let a = s.p0;
            let b = s.p1;
            return array<vec3<f32>, 2>(
                min(a.xyz - a.w, b.xyz - b.w) - reach,
                max(a.xyz + a.w, b.xyz + b.w) + reach,
            );
        }
    }
}

// The cells a shape is binned into; none for a negative reach.
fn span_of(s: SurfaceShape, inverse_cell: f32) -> Span {
    if (s.reach < 0.0) {
        return Span(vec3<i32>(0), vec3<i32>(-1));
    }
    let b = bounds_of(s);
    return Span(vec3<i32>(floor(b[0] * inverse_cell)), vec3<i32>(floor(b[1] * inverse_cell)));
}

fn shape_index(id: vec3<u32>, groups: vec3<u32>) -> u32 {
    return id.x + id.y * groups.x * 64u;
}
"#;

fn count_source() -> String {
    format!(
        r#"{COMMON}
@group(0) @binding(0) var<storage, read> shapes: array<SurfaceShape>;
@group(0) @binding(1) var<uniform> build: BuildParams;
@group(0) @binding(2) var<storage, read_write> counts: array<atomic<u32>>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let i = shape_index(id, groups);
    if (i >= build.count) {{
        return;
    }}
    let span = span_of(shapes[i], build.inverse_cell);
    for (var z = span.lo.z; z <= span.hi.z; z++) {{
        for (var y = span.lo.y; y <= span.hi.y; y++) {{
            for (var x = span.lo.x; x <= span.hi.x; x++) {{
                atomicAdd(&counts[bucket_of(vec3<i32>(x, y, z), build.table_size)], 1u);
            }}
        }}
    }}
}}
"#
    )
}

fn place_source() -> String {
    format!(
        r#"{COMMON}
@group(0) @binding(0) var<storage, read> shapes: array<SurfaceShape>;
@group(0) @binding(1) var<uniform> build: BuildParams;
@group(0) @binding(2) var<storage, read> starts: array<u32>;
@group(0) @binding(3) var<storage, read_write> cursors: array<atomic<u32>>;
@group(0) @binding(4) var<storage, read_write> entries: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let i = shape_index(id, groups);
    if (i >= build.count) {{
        return;
    }}
    let span = span_of(shapes[i], build.inverse_cell);
    for (var z = span.lo.z; z <= span.hi.z; z++) {{
        for (var y = span.lo.y; y <= span.hi.y; y++) {{
            for (var x = span.lo.x; x <= span.hi.x; x++) {{
                let bucket = bucket_of(vec3<i32>(x, y, z), build.table_size);
                let slot = starts[bucket] + atomicAdd(&cursors[bucket], 1u);
                if (slot < build.capacity) {{
                    entries[slot] = i;
                }}
            }}
        }}
    }}
}}
"#
    )
}

/// Buckets past this many entries are left in the order they were placed:
/// a long insertion sort would stall its workgroup, and the index is still
/// right, only not in index order.
const TIDY_LIMIT: u32 = 128;

fn tidy_source() -> String {
    format!(
        r#"{COMMON}
@group(0) @binding(0) var<uniform> build: BuildParams;
@group(0) @binding(1) var<storage, read> starts: array<u32>;
@group(0) @binding(2) var<storage, read_write> entries: array<u32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let bucket = id.x + id.y * groups.x * 256u;
    if (bucket >= build.table_size) {{
        return;
    }}
    let begin = min(starts[bucket], build.capacity);
    let end = min(starts[bucket + 1u], build.capacity);
    if (end - begin > {TIDY_LIMIT}u) {{
        return;
    }}
    // Insertion sort, then blank each repeat of the one before.
    for (var i = begin + 1u; i < end; i++) {{
        let value = entries[i];
        var j = i;
        while (j > begin && entries[j - 1u] > value) {{
            entries[j] = entries[j - 1u];
            j--;
        }}
        entries[j] = value;
    }}
    var last = 0xFFFFFFFFu;
    for (var i = begin; i < end; i++) {{
        let value = entries[i];
        if (value == last) {{
            entries[i] = 0xFFFFFFFFu;
        }}
        last = value;
    }}
}}
"#
    )
}

fn scan_blocks_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read_write> data: array<u32>;
@group(0) @binding(1) var<storage, read_write> sums: array<u32>;
struct ScanParams {{
    count: u32,
    write_sums: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(2) var<uniform> params: ScanParams;

var<workgroup> shared_values: array<u32, {BLOCK}u>;

@compute @workgroup_size({BLOCK}u)
fn main(
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
    @builtin(num_workgroups) groups: vec3<u32>,
) {{
    let block = group_id.y * groups.x + group_id.x;
    let index = block * {BLOCK}u + local_id.x;
    var value = 0u;
    if (index < params.count) {{
        value = data[index];
    }}
    shared_values[local_id.x] = value;
    workgroupBarrier();
    // Hillis-Steele, inclusive.
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

fn add_offsets_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read_write> data: array<u32>;
@group(0) @binding(1) var<storage, read> sums: array<u32>;
struct ScanParams {{
    count: u32,
    write_sums: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(2) var<uniform> params: ScanParams;

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

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BuildParams {
    count: u32,
    table_size: u32,
    inverse_cell: f32,
    capacity: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScanParams {
    count: u32,
    write_sums: u32,
    pad: [u32; 2],
}

/// Workgroups for `items` items of `width` per group, folded into y past one
/// dimension's limit.
fn grid(items: u32, width: u32) -> (u32, u32) {
    let groups = items.div_ceil(width).max(1);
    let x = groups.min(MAX_GROUPS);
    (x, groups.div_ceil(x))
}

/// One level of the scan's recursion: the block totals it writes, and its
/// parameters.
struct Level {
    sums: wgpu::Buffer,
    params: wgpu::Buffer,
    /// The same parameters for the level's add pass, which writes none.
    count: u32,
}

/// A surface index built on the GPU: its buffers, and the passes that fill
/// them. Bind [`GpuSurfaceIndex::starts`] and [`GpuSurfaceIndex::entries`]
/// as `surface_starts` and `surface_entries`, the shapes buffer as
/// `surface_shapes`, and [`GpuSurfaceIndex::params`] in a uniform, exactly
/// as for a host-built index.
pub struct GpuSurfaceIndex {
    count: wgpu::ComputePipeline,
    place: wgpu::ComputePipeline,
    tidy: wgpu::ComputePipeline,
    scan_blocks: wgpu::ComputePipeline,
    add_offsets: wgpu::ComputePipeline,
    build: wgpu::Buffer,
    starts: wgpu::Buffer,
    cursors: wgpu::Buffer,
    entries: wgpu::Buffer,
    /// The scan's levels, for `table_size + 1` elements.
    levels: Vec<Level>,
    /// The scan of the level-0 data (the starts), when one block holds it.
    top: wgpu::Buffer,
    unused: wgpu::Buffer,
    table_size: u32,
    capacity: u32,
    params: SurfaceParams,
}

fn storage(
    device: &wgpu::Device,
    label: &str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(16).next_multiple_of(16),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC
            | usage,
        mapped_at_creation: false,
    })
}

fn uniform(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn pipeline(device: &wgpu::Device, label: &str, source: &str) -> wgpu::ComputePipeline {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    })
}

fn bind(
    device: &wgpu::Device,
    pipeline: &wgpu::ComputePipeline,
    buffers: &[&wgpu::Buffer],
) -> wgpu::BindGroup {
    let entries: Vec<wgpu::BindGroupEntry> = buffers
        .iter()
        .enumerate()
        .map(|(i, buffer)| wgpu::BindGroupEntry {
            binding: i as u32,
            resource: buffer.as_entire_binding(),
        })
        .collect();
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &entries,
    })
}

impl GpuSurfaceIndex {
    pub fn new(device: &wgpu::Device) -> Self {
        let none = wgpu::BufferUsages::empty();
        let mut index = Self {
            count: pipeline(device, "surface index count", &count_source()),
            place: pipeline(device, "surface index place", &place_source()),
            tidy: pipeline(device, "surface index tidy", &tidy_source()),
            scan_blocks: pipeline(device, "surface index scan", &scan_blocks_source()),
            add_offsets: pipeline(device, "surface index scan offsets", &add_offsets_source()),
            build: uniform(
                device,
                "surface index build",
                std::mem::size_of::<BuildParams>() as u64,
            ),
            starts: storage(device, "surface starts", 16, none),
            cursors: storage(device, "surface cursors", 16, none),
            entries: storage(device, "surface entries", 16, none),
            levels: Vec::new(),
            top: uniform(
                device,
                "surface index scan top",
                std::mem::size_of::<ScanParams>() as u64,
            ),
            unused: storage(device, "surface index unused", 16, none),
            table_size: 0,
            capacity: 0,
            params: SurfaceParams::default(),
        };
        index.allocate(device, 64, 1);
        index
    }

    /// Size the buffers for a table of `table_size` buckets and `capacity`
    /// entries.
    fn allocate(&mut self, device: &wgpu::Device, table_size: u32, capacity: u32) {
        let none = wgpu::BufferUsages::empty();
        if table_size != self.table_size {
            self.starts = storage(device, "surface starts", (table_size as u64 + 1) * 4, none);
            self.cursors = storage(device, "surface cursors", table_size as u64 * 4, none);
            self.levels.clear();
            let mut count = table_size + 1;
            while count > BLOCK {
                let blocks = count.div_ceil(BLOCK);
                self.levels.push(Level {
                    sums: storage(device, "surface index scan totals", blocks as u64 * 4, none),
                    params: uniform(
                        device,
                        "surface index scan level",
                        std::mem::size_of::<ScanParams>() as u64,
                    ),
                    count,
                });
                count = blocks;
            }
            self.table_size = table_size;
        }
        if capacity > self.capacity {
            let grown = capacity.max(self.capacity + self.capacity / 2);
            self.entries = storage(device, "surface entries", grown as u64 * 4, none);
            self.capacity = grown;
        }
    }

    /// Record a build of the index of the first `count` shapes in `shapes`
    /// (a storage buffer of [`crate::Shape`]s), in cells of edge `cell`,
    /// with room for `capacity` entries. Returns whether its buffers were
    /// made anew, so that bind groups holding them need making again.
    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        shapes: &wgpu::Buffer,
        count: u32,
        cell: f32,
        capacity: u32,
    ) -> bool {
        let before = (self.starts.clone(), self.entries.clone());
        let table_size = table_size_for(capacity.max(1));
        self.allocate(device, table_size, capacity.max(1));
        let reallocated = before.0 != self.starts || before.1 != self.entries;
        let inverse_cell = 1.0 / cell.max(1e-4);
        self.params = SurfaceParams {
            inverse_cell,
            table_size: if count == 0 { 0 } else { table_size },
            pad: [0; 2],
        };
        let build = BuildParams {
            count,
            table_size,
            inverse_cell,
            capacity: self.capacity,
        };
        queue.write_buffer(&self.build, 0, bytemuck::bytes_of(&build));
        encoder.clear_buffer(&self.starts, 0, None);
        encoder.clear_buffer(&self.cursors, 0, None);
        if count == 0 {
            return reallocated;
        }

        let dispatch = |encoder: &mut wgpu::CommandEncoder,
                        pipeline: &wgpu::ComputePipeline,
                        group: &wgpu::BindGroup,
                        (x, y): (u32, u32)| {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("surface index"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.dispatch_workgroups(x, y, 1);
        };

        // 1. Count.
        let group = bind(device, &self.count, &[shapes, &self.build, &self.starts]);
        dispatch(encoder, &self.count, &group, grid(count, SHAPES_PER_GROUP));

        // 2. Scan the counts, and the extra word past them, into starts.
        self.record_scan(
            device,
            queue,
            encoder,
            0,
            &self.starts.clone(),
            table_size + 1,
        );

        // 3. Place.
        let group = bind(
            device,
            &self.place,
            &[
                shapes,
                &self.build,
                &self.starts,
                &self.cursors,
                &self.entries,
            ],
        );
        dispatch(encoder, &self.place, &group, grid(count, SHAPES_PER_GROUP));

        // 4. Tidy.
        let group = bind(
            device,
            &self.tidy,
            &[&self.build, &self.starts, &self.entries],
        );
        dispatch(encoder, &self.tidy, &group, grid(table_size, BLOCK));
        reallocated
    }

    /// An exclusive scan of `count` words of `data` in place, using the
    /// scan levels from `level` down.
    fn record_scan(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        level: usize,
        data: &wgpu::Buffer,
        count: u32,
    ) {
        let blocks = count.div_ceil(BLOCK);
        let run = |encoder: &mut wgpu::CommandEncoder,
                   pipeline: &wgpu::ComputePipeline,
                   group: &wgpu::BindGroup| {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("surface index scan"),
                timestamp_writes: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[]);
            let (x, y) = grid(count, BLOCK);
            pass.dispatch_workgroups(x, y, 1);
        };
        if blocks == 1 {
            queue.write_buffer(
                &self.top,
                0,
                bytemuck::bytes_of(&ScanParams {
                    count,
                    write_sums: 0,
                    pad: [0; 2],
                }),
            );
            let group = bind(device, &self.scan_blocks, &[data, &self.unused, &self.top]);
            run(encoder, &self.scan_blocks, &group);
            return;
        }
        let this = &self.levels[level];
        debug_assert_eq!(this.count, count);
        queue.write_buffer(
            &this.params,
            0,
            bytemuck::bytes_of(&ScanParams {
                count,
                write_sums: 1,
                pad: [0; 2],
            }),
        );
        let group = bind(device, &self.scan_blocks, &[data, &this.sums, &this.params]);
        run(encoder, &self.scan_blocks, &group);
        self.record_scan(device, queue, encoder, level + 1, &this.sums, blocks);
        let group = bind(device, &self.add_offsets, &[data, &this.sums, &this.params]);
        run(encoder, &self.add_offsets, &group);
    }

    /// Where each bucket's entries begin, and one more word.
    pub fn starts(&self) -> &wgpu::Buffer {
        &self.starts
    }

    /// Shape indices grouped by bucket, [`crate::index::EMPTY`] where a
    /// duplicate was dropped.
    pub fn entries(&self) -> &wgpu::Buffer {
        &self.entries
    }

    /// For the shaders' uniform, once the build is recorded.
    pub fn params(&self) -> SurfaceParams {
        self.params
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Shape, SurfaceIndex};

    fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
        pollster::block_on(async {
            let instance = wgpu::Instance::default();
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                .await
                .ok()?;
            adapter
                .request_device(&wgpu::DeviceDescriptor::default())
                .await
                .ok()
        })
    }

    fn read(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer: &wgpu::Buffer,
        words: usize,
    ) -> Vec<u32> {
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (words * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, (words * 4) as u64);
        queue.submit([encoder.finish()]);
        staging.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let words = bytemuck::cast_slice(&staging.slice(..).get_mapped_range()).to_vec();
        staging.unmap();
        words
    }

    /// The host's scene of every kind, and a large one so the scan recurses.
    fn scene(scale: usize) -> Vec<Shape> {
        let mut shapes = Vec::new();
        let mut seed = 7u64;
        let mut next = || {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (seed >> 40) as f32 / (1u64 << 24) as f32
        };
        for i in 0..scale as u32 {
            let a = [next() * 40.0, next() * 10.0, next() * 40.0];
            let r = 0.02 + 0.3 * next().powi(3);
            let shape = match i % 4 {
                0 => Shape::round_cone(
                    a,
                    r,
                    [a[0] + next() - 0.5, a[1] + next(), a[2] + next() - 0.5],
                    r * 0.5,
                ),
                1 => Shape::oriented_box(
                    a,
                    [r * 2.0, r, r],
                    [0.0, 0.3826834, 0.0, 0.9238795],
                    r * 0.1,
                ),
                2 => Shape::triangle(
                    a,
                    [a[0] + 0.5, a[1], a[2]],
                    [a[0], a[1] + 0.5, a[2] + 0.2],
                    r * 0.1,
                ),
                _ => Shape::height_tile(
                    [a[0], a[2]],
                    [a[0] + 1.0, a[2] + 1.0],
                    [0.0, 0.3, 0.1, 0.5],
                    0.5,
                ),
            };
            let reach = if i % 17 == 0 { -1.0 } else { r * 0.8 + 0.02 };
            shapes.push(shape.with_reach(reach).with_object(i).with_payload(i));
        }
        shapes
    }

    #[test]
    fn the_gpu_builds_the_hosts_index() {
        let Some((device, queue)) = device() else {
            eprintln!("no GPU; skipped");
            return;
        };
        for scale in [300, 40_000] {
            let shapes = scene(scale);
            let cell = SurfaceIndex::cell_for(&shapes);
            let capacity = SurfaceIndex::entry_count(&shapes, cell) as u32;
            let host = SurfaceIndex::build(shapes.clone(), cell);

            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of_val(shapes.as_slice()) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&shapes));
            let mut gpu = GpuSurfaceIndex::new(&device);
            let mut encoder = device.create_command_encoder(&Default::default());
            gpu.record(
                &device,
                &queue,
                &mut encoder,
                &buffer,
                shapes.len() as u32,
                cell,
                capacity,
            );
            queue.submit([encoder.finish()]);

            assert_eq!(gpu.params(), host.params);
            let table = host.params.table_size as usize;
            let starts = read(&device, &queue, gpu.starts(), table + 1);
            let entries = read(&device, &queue, gpu.entries(), capacity as usize);
            assert_eq!(
                starts[table] as usize, capacity as usize,
                "every entry counted"
            );
            for bucket in 0..table {
                let gpu_bucket: Vec<u32> = entries
                    [starts[bucket] as usize..starts[bucket + 1] as usize]
                    .iter()
                    .copied()
                    .filter(|&e| e != crate::index::EMPTY)
                    .collect();
                let host_bucket =
                    &host.entries[host.starts[bucket] as usize..host.starts[bucket + 1] as usize];
                assert_eq!(gpu_bucket, host_bucket, "bucket {bucket} of {scale} shapes");
            }
        }
    }
}
