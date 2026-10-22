// How much each view's coverage is raised at each mip level so that as much
// of the view passes the mask's threshold as at full size (Castaño,
// *Computing Alpha Mipmaps*, 2010): sparse leaves that averaging would sink
// under the threshold are grown instead, and a distant crown keeps its
// share of the pixels. One workgroup a view and level.
@group(0) @binding(0) var coverage: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> scales: array<f32>;

const THRESHOLD: f32 = 0.5;
const STEPS: i32 = 16;
const LARGEST: f32 = 64.0;

var<workgroup> passing: atomic<u32>;

// The share of view `cell`'s texels at `level` whose coverage, times
// `scale`, passes the threshold. Every thread returns the same.
fn passed(cell: vec2<u32>, side: u32, level: i32, scale: f32, thread: u32) -> f32 {
    if thread == 0u {
        atomicStore(&passing, 0u);
    }
    workgroupBarrier();
    var count = 0u;
    for (var i = thread; i < side * side; i += 64u) {
        let at = cell * side + vec2<u32>(i % side, i / side);
        let a = textureLoad(coverage, at, level).a;
        count += select(0u, 1u, a * scale > THRESHOLD);
    }
    atomicAdd(&passing, count);
    workgroupBarrier();
    let share = f32(atomicLoad(&passing)) / f32(side * side);
    workgroupBarrier();
    return share;
}

@compute @workgroup_size(64)
fn main(
    @builtin(workgroup_id) group: vec3<u32>,
    @builtin(num_workgroups) groups: vec3<u32>,
    @builtin(local_invocation_index) thread: u32,
) {
    let frames = groups.x;
    let level = group.z + 1u;
    let full = textureDimensions(coverage, 0).x / frames;
    let side = max(full >> level, 1u);
    let wanted = passed(group.xy, full, 0, 1.0, thread);
    // The scale whose share comes nearest, searched by halves: the share
    // only grows with the scale.
    var low = 0.0;
    var high = LARGEST;
    var best = 1.0;
    var miss = 2.0;
    for (var step = 0; step < STEPS; step++) {
        let scale = (low + high) * 0.5;
        let share = passed(group.xy, side, i32(level), scale, thread);
        let off = abs(share - wanted);
        if off < miss || (off == miss && share > wanted) {
            miss = off;
            best = scale;
        }
        let under = share < wanted;
        low = select(low, scale, under);
        high = select(scale, high, under);
    }
    if thread == 0u {
        scales[(level - 1u) * frames * frames + group.y * frames + group.x] =
            select(best, 1.0, wanted == 0.0);
    }
}
