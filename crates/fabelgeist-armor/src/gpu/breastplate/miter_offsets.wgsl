@group(0) @binding(0) var<storage, read> normals: array<f32>;
@group(0) @binding(1) var<storage, read> initial: array<f32>;
@group(0) @binding(2) var<storage, read> ranges: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read> incident: array<u32>;
@group(0) @binding(4) var<storage, read_write> offsets: array<f32>;
@group(0) @binding(5) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(6) var<uniform> params: Params;
const MITER_PROJECTION_PASSES: u32 = 32u;
const MITER_SUPPORT_TOLERANCE: f32 = 1e-4;
// The sheet retains its carrier gauge. Only correct trim vectors that point
// back through an incident facet; imposing full normal gauge on a concave
// cut would reshape the neighboring outer wall. Keep a numerical outward
// reserve above the support comparison tolerance.
const MIN_RIM_SUPPORT_RATIO: f32 = 1e-3;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) { return; }
    let range = ranges[i];
    var offset = initial_at(i);
    for (var iteration = 0u; iteration < MITER_PROJECTION_PASSES; iteration += 1u) {
        var corrected = false;
        for (var j = 0u; j < range.y; j += 1u) {
            let n = normals_at(incident[range.x + j]);
            let missing = host_sub(MIN_RIM_SUPPORT_RATIO, host_dot(n, offset));
            if (missing > 0.0) { offset = host_add3(offset, host_scale3(n, missing)); corrected = true; }
        }
        if (!corrected) { break; }
    }
    for (var j = 0u; j < range.y; j += 1u) {
        let n = normals_at(incident[range.x + j]);
        if (!(host_dot(n, offset) >= host_sub(MIN_RIM_SUPPORT_RATIO, MITER_SUPPORT_TOLERANCE))) { fail(STATUS_INVALID_SURFACE); }
    }
    offsets_set(i, offset);
}
