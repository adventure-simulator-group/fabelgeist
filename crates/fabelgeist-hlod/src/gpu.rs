//! The cut taken on the GPU, straight into draw lists.
//!
//! One workgroup per node. Its first thread takes the test of
//! [`crate::Hierarchy::select`] and, for a drawn node, reserves room in each
//! stream's list for the chosen level with one atomic add -- which is also
//! the instance count of that list's indirect draw; then the workgroup
//! writes the node's instance indices there together, so that a leaf with
//! thousands of instances does not hold one thread for thousands of writes.
//!
//! The caller owns the list and draw buffers:
//!
//! * `lists: array<u32>`: stream `s`, level `l` from
//!   `stream_base[s] + l * stream_stride[s]`;
//! * `draws`: indirect draws of four words (vertex count, instance count,
//!   first vertex, first instance), stream by stream, `total_levels` a
//!   stream. The instance counts start at zero -- clear them -- and the
//!   vertex counts are the caller's to write.
//!
//! A cut draws each instance at most once, so a stream's list at one level
//! needs room for no more than all its instances.

use crate::hierarchy::{Hierarchy, Node, View};

const SOURCE: &str = r#"
struct HlodNode {
    sphere: vec4<f32>,
    parent_sphere: vec4<f32>,
    error: f32,
    parent_error: f32,
    finest_level: u32,
    pad: u32,
};

struct HlodView {
    planes: array<vec4<f32>, 6>,
    eye: vec4<f32>,
    tolerance: f32,
    texel: f32,
    levels_px: vec2<f32>,
    count: u32,
    streams: u32,
    levels: u32,
    level_offset: u32,
    total_levels: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
    stream_base: vec4<u32>,
    stream_stride: vec4<u32>,
};

@group(0) @binding(0) var<storage, read> nodes: array<HlodNode>;
@group(0) @binding(1) var<storage, read> ranges: array<vec2<u32>>;
@group(0) @binding(2) var<uniform> view: HlodView;
@group(0) @binding(3) var<storage, read_write> lists: array<u32>;
@group(0) @binding(4) var<storage, read_write> draws: array<atomic<u32>>;

const NOT_DRAWN: u32 = 0xFFFFFFFFu;

// An error at a sphere, in pixels, over the distance to its nearest point.
fn projected(error: f32, sphere: vec4<f32>) -> f32 {
    if (view.texel > 0.0) {
        return error / view.texel;
    }
    let distance = length(sphere.xyz - view.eye.xyz) - sphere.w;
    return error / (max(distance, 1e-4) * view.eye.w);
}

// The level a node is drawn at, or NOT_DRAWN.
fn decide(node: HlodNode) -> u32 {
    for (var i = 0u; i < 6u; i++) {
        let plane = view.planes[i];
        if (dot(plane.xyz, node.sphere.xyz) + plane.w < -node.sphere.w) {
            return NOT_DRAWN;
        }
    }
    if (projected(node.error, node.sphere) > view.tolerance
        || projected(node.parent_error, node.parent_sphere) <= view.tolerance) {
        return NOT_DRAWN;
    }
    let size = projected(node.sphere.w, node.sphere);
    var level = 2u;
    if (view.texel > 0.0 || size >= view.levels_px.x) {
        level = 0u;
    } else if (size >= view.levels_px.y) {
        level = 1u;
    }
    return min(max(level, node.finest_level), max(view.levels, 1u) - 1u);
}

// What a drawn node's workgroup writes: per stream, where to and what.
struct Plan {
    drawn: u32,
    dest: vec4<u32>,
    first: vec4<u32>,
    count: vec4<u32>,
};

var<workgroup> plan: Plan;

@compute @workgroup_size(64)
fn main(
    @builtin(workgroup_id) group: vec3<u32>,
    @builtin(num_workgroups) groups: vec3<u32>,
    @builtin(local_invocation_index) lane: u32,
) {
    let index = group.x + group.y * groups.x;
    if (lane == 0u) {
        var p: Plan;
        p.drawn = 0u;
        if (index < view.count) {
            let level = decide(nodes[index]);
            if (level != NOT_DRAWN) {
                p.drawn = 1u;
                let streams = min(view.streams, 4u);
                for (var s = 0u; s < streams; s++) {
                    let range = ranges[index * view.streams + s];
                    if (range.y == 0u) {
                        continue;
                    }
                    let draw = s * view.total_levels + view.level_offset + level;
                    let slot = atomicAdd(&draws[draw * 4u + 1u], range.y);
                    p.dest[s] = view.stream_base[s] + (view.level_offset + level) * view.stream_stride[s] + slot;
                    p.first[s] = range.x;
                    p.count[s] = min(range.y, view.stream_stride[s] - min(slot, view.stream_stride[s]));
                }
            }
        }
        plan = p;
    }
    let p = workgroupUniformLoad(&plan);
    if (p.drawn == 0u) {
        return;
    }
    for (var s = 0u; s < 4u; s++) {
        for (var i = lane; i < p.count[s]; i += 64u) {
            lists[p.dest[s] + i] = p.first[s] + i;
        }
    }
}
"#;

/// The hierarchy on the GPU, and the pass that takes cuts through it.
pub struct GpuHierarchy {
    pipeline: wgpu::ComputePipeline,
    nodes: wgpu::Buffer,
    ranges: wgpu::Buffer,
    /// A uniform per view recorded in one call.
    views: Vec<wgpu::Buffer>,
    count: u32,
    streams: u32,
}

fn storage(device: &wgpu::Device, label: &str, bytes: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.max(16).next_multiple_of(16),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

impl GpuHierarchy {
    pub fn new(device: &wgpu::Device) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hlod select"),
            source: wgpu::ShaderSource::Wgsl(SOURCE.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("hlod select"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self {
            pipeline,
            nodes: storage(device, "hlod nodes", 16),
            ranges: storage(device, "hlod ranges", 16),
            views: Vec::new(),
            count: 0,
            streams: 0,
        }
    }

    /// Take a new hierarchy.
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, hierarchy: &Hierarchy) {
        let nodes: &[u8] = bytemuck::cast_slice(&hierarchy.nodes);
        let ranges: &[u8] = bytemuck::cast_slice(&hierarchy.ranges);
        if self.nodes.size() < nodes.len() as u64 {
            self.nodes = storage(device, "hlod nodes", nodes.len() as u64 * 3 / 2);
        }
        if self.ranges.size() < ranges.len() as u64 {
            self.ranges = storage(device, "hlod ranges", ranges.len() as u64 * 3 / 2);
        }
        if !nodes.is_empty() {
            queue.write_buffer(&self.nodes, 0, nodes);
            queue.write_buffer(&self.ranges, 0, ranges);
        }
        self.count = hierarchy.nodes.len() as u32;
        self.streams = hierarchy.streams;
    }

    /// Record the cut each of `views` takes, appending to `lists` and
    /// counting into `draws` (see the module's documentation). The views'
    /// node and stream counts are filled in.
    pub fn record(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        views: &[View],
        lists: &wgpu::Buffer,
        draws: &wgpu::Buffer,
    ) {
        if self.count == 0 {
            return;
        }
        while self.views.len() < views.len() {
            self.views
                .push(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("hlod view"),
                    size: std::mem::size_of::<View>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
        }
        let groups_x = self.count.min(65_535);
        let groups_y = self.count.div_ceil(groups_x);
        for (view, uniform) in views.iter().zip(&self.views) {
            let view = View {
                count: self.count,
                streams: self.streams,
                ..*view
            };
            queue.write_buffer(uniform, 0, bytemuck::bytes_of(&view));
            let resources = [&self.nodes, &self.ranges, uniform, lists, draws];
            let entries: Vec<wgpu::BindGroupEntry> = resources
                .iter()
                .enumerate()
                .map(|(i, buffer)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: buffer.as_entire_binding(),
                })
                .collect();
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("hlod select"),
                layout: &self.pipeline.get_bind_group_layout(0),
                entries: &entries,
            });
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("hlod select"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.dispatch_workgroups(groups_x, groups_y, 1);
        }
    }

    pub fn node_count(&self) -> u32 {
        self.count
    }
}

/// The selection's WGSL, for tools that want to validate or embed it.
pub fn source() -> &'static str {
    SOURCE
}

const _: () = assert!(std::mem::size_of::<Node>() == 48);
const _: () = assert!(std::mem::size_of::<View>() == 192);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchy::tests::{grove, view_from};

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

    fn read(device: &wgpu::Device, queue: &wgpu::Queue, buffer: &wgpu::Buffer) -> Vec<u32> {
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, buffer.size());
        queue.submit([encoder.finish()]);
        staging.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let words = bytemuck::cast_slice(&staging.slice(..).get_mapped_range()).to_vec();
        staging.unmap();
        words
    }

    #[test]
    fn the_wgsl_validates() {
        let module = naga::front::wgsl::parse_str(SOURCE).expect("parses");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("validates");
    }

    #[test]
    fn the_gpu_takes_the_hosts_cut() {
        let Some((device, queue)) = device() else {
            eprintln!("no GPU; skipped");
            return;
        };
        let (_, hierarchy) = grove(3000);
        let streams = 2usize;
        // Room for every instance of each stream, at each of four levels.
        let totals: Vec<u32> = (0..streams)
            .map(|s| {
                hierarchy
                    .ranges
                    .iter()
                    .skip(s)
                    .step_by(streams)
                    .map(|r| r[0] + r[1])
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        let base = [0, 4 * totals[0], 0, 0];
        let stride = [totals[0], totals[1], 0, 0];
        let list_words = (4 * (totals[0] + totals[1])) as u64;
        let usage = wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST;
        let lists = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: list_words * 4,
            usage,
            mapped_at_creation: false,
        });
        let draws = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 8 * 16,
            usage,
            mapped_at_creation: false,
        });
        let mut gpu = GpuHierarchy::new(&device);
        gpu.upload(&device, &queue, &hierarchy);

        for eye in [[0.0, 2.0, 0.0], [150.0, 30.0, 150.0], [-400.0, 50.0, 900.0]] {
            let view = View {
                count: hierarchy.nodes.len() as u32,
                streams: streams as u32,
                levels: 3,
                stream_base: base,
                stream_stride: stride,
                ..view_from(eye, 0.001)
            };
            let mut encoder = device.create_command_encoder(&Default::default());
            encoder.clear_buffer(&draws, 0, None);
            gpu.record(&device, &queue, &mut encoder, &[view], &lists, &draws);
            queue.submit([encoder.finish()]);
            let counts = read(&device, &queue, &draws);
            let lists_read = read(&device, &queue, &lists);

            let cut = hierarchy.select(&view);
            for s in 0..streams {
                for level in 0..3u32 {
                    let mut expected: Vec<u32> = cut
                        .iter()
                        .filter(|&&(_, l)| l == level)
                        .flat_map(|&(n, _)| {
                            let r = hierarchy.ranges[n as usize * streams + s];
                            r[0]..r[0] + r[1]
                        })
                        .collect();
                    expected.sort();
                    let draw = s * 4 + level as usize;
                    let count = counts[draw * 4 + 1] as usize;
                    let start = (base[s] + level * stride[s]) as usize;
                    let mut got = lists_read[start..start + count].to_vec();
                    got.sort();
                    assert_eq!(got, expected, "stream {s} level {level} from {eye:?}");
                }
            }
        }
    }
}
