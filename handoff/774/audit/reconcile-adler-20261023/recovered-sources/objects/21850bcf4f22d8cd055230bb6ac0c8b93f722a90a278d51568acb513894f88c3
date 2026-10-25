//! The placing kernels' WGSL: carrying strands onto a character and lifting
//! their roots, gathering the guides, the guides' moves, and each strand
//! following its nearest guide.

/// The flat index of an invocation of a folded dispatch, 64 wide.
pub(super) const FOLDED: &str = r#"
fn item(id: vec3<u32>, groups: vec3<u32>) -> u32 {
    return id.y * groups.x * 64u + id.x;
}
"#;

/// The placement, as the kernels take it.
const FRAME: &str = r#"
    origin_x: f32,
    origin_y: f32,
    origin_z: f32,
    x_x: f32,
    x_y: f32,
    x_z: f32,
    y_x: f32,
    y_y: f32,
    y_z: f32,
    z_x: f32,
    z_y: f32,
    z_z: f32,
    inverse_x: f32,
    inverse_y: f32,
    inverse_z: f32,
"#;

const FRAME_FNS: &str = r#"
fn axis_x() -> vec3<f32> { return vec3<f32>(params.x_x, params.x_y, params.x_z); }
fn axis_y() -> vec3<f32> { return vec3<f32>(params.y_x, params.y_y, params.y_z); }
fn axis_z() -> vec3<f32> { return vec3<f32>(params.z_x, params.z_y, params.z_z); }

// A move of the bust's frame, carried.
fn carry_move(v: vec3<f32>) -> vec3<f32> {
    return axis_x() * v.x + axis_y() * v.y + axis_z() * v.z;
}

fn carry(p: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(params.origin_x, params.origin_y, params.origin_z) + carry_move(p);
}
"#;

/// `source` with the placement's parameters and helpers in it.
pub(super) fn framed(source: &str) -> String {
    source
        .replace("FRAME_FNS", FRAME_FNS)
        .replace("FRAME\n", FRAME)
}

pub(super) const CARRY: &str = r#"
struct StrandGpu {
    normal: vec3<f32>,
    id: u32,
    uv: vec2<f32>,
    group: i32,
    radius: f32,
};
@group(0) @binding(0) var<storage, read> src: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> dst: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> strands_in: array<StrandGpu>;
@group(0) @binding(3) var<storage, read_write> strands_out: array<StrandGpu>;
@group(0) @binding(4) var<storage, read> roots_in: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> roots_out: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read> head_shape: array<vec4<f32>>;
struct Params {
FRAME
    count: u32,
    points: u32,
    clearance: f32,
    tolerance: f32,
    max_shift: f32,
    min_incidence: f32,
    steps: u32,
    pad0: u32,
    pad1: u32,
};
@group(0) @binding(7) var<uniform> params: Params;
FRAME_FNS

fn carry_normal(n: vec3<f32>) -> vec3<f32> {
    let carried = axis_x() * (n.x * params.inverse_x)
        + axis_y() * (n.y * params.inverse_y)
        + axis_z() * (n.z * params.inverse_z);
    let length = sqrt(dot(carried, carried));
    if (length < 1e-12) {
        return n;
    }
    return carried / length;
}

// How far to move `root` along `normal` to sit `clearance` outside the
// body: out of it, or down onto it; pushed straight out where the normal
// runs too nearly along the skin to reach it.
fn root_lift(root: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    var along = 0.0;
    for (var k = 0u; k < params.steps; k++) {
        let at = root + normal * along;
        let error = params.clearance - head_distance(at);
        if (abs(error) < params.tolerance) {
            return normal * along;
        }
        // The distance changes along the normal as fast as the normal
        // meets the skin's.
        along += error / max(dot(normal, head_gradient(at)), params.min_incidence);
        if (abs(along) > params.max_shift) {
            break;
        }
    }
    return push_out(root, params.clearance) - root;
}
"#;

pub(super) const CARRY_BODY: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let base = s * params.points;
    var strand = strands_in[s];
    strand.normal = carry_normal(strand.normal);
    strands_out[s] = strand;
    let lift = root_lift(carry(src[base].xyz), strand.normal);
    for (var j = 0u; j < params.points; j++) {
        dst[base + j] = vec4<f32>(carry(src[base + j].xyz) + lift, src[base + j].w);
    }
    roots_out[s] = vec4<f32>(carry(roots_in[s].xyz) + lift, roots_in[s].w);
}
"#;

pub(super) const GATHER: &str = r#"
@group(0) @binding(0) var<storage, read> placed: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> groomed: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> placed_guides: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> groomed_guides: array<vec4<f32>>;
struct Params {
    guides: u32,
    points: u32,
    step: u32,
    pad0: u32,
};
@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    if (i >= params.guides * params.points) {
        return;
    }
    let guide = i / params.points;
    let source_index = guide * params.step * params.points + i % params.points;
    placed_guides[i] = placed[source_index];
    groomed_guides[i] = groomed[source_index];
}
"#;

pub(super) const MOVES: &str = r#"
@group(0) @binding(0) var<storage, read> by_body: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> placed_free: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> by_bust: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> groomed_free: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> moves: array<vec4<f32>>;
struct Params {
FRAME
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
    pad3: u32,
};
@group(0) @binding(5) var<uniform> params: Params;
FRAME_FNS

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    if (i >= params.count) {
        return;
    }
    // How much more the character's body moves the point than the bust's,
    // the bust's push carried as the strand was.
    let character = by_body[i].xyz - placed_free[i].xyz;
    let bust = carry_move(by_bust[i].xyz - groomed_free[i].xyz);
    moves[i] = vec4<f32>(character - bust, 0.0);
}
"#;

pub(super) const FOLLOW: &str = r#"
@group(0) @binding(0) var<storage, read> src: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> dst: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> moves: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> guides_at: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> head_shape: array<vec4<f32>>;
struct Params {
    count: u32,
    points: u32,
    guides: u32,
    pad0: u32,
    sink_start: f32,
    sink_full: f32,
    hold_start: f32,
    hold_full: f32,
};
@group(0) @binding(5) var<uniform> params: Params;

fn smooth01(t: f32) -> f32 {
    let c = clamp(t, 0.0, 1.0);
    return c * c * (3.0 - 2.0 * c);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let points = params.points;
    let base = s * points;
    // How deep the strand sinks into the body as carried.
    var sunk = 0.0;
    for (var j = 0u; j < points; j++) {
        sunk = max(sunk, -head_distance(src[base + j].xyz));
    }
    let sinking = smooth01((sunk - params.sink_start) / (params.sink_full - params.sink_start));
    if (sinking <= 0.0) {
        for (var j = 0u; j < points; j++) {
            dst[base + j] = src[base + j];
        }
        return;
    }
    // The guide whose root is nearest.
    let root = src[base].xyz;
    var nearest = 0u;
    var nearest_distance = 3.4e38;
    for (var g = 0u; g < params.guides; g++) {
        let offset = guides_at[g * points].xyz - root;
        let d = dot(offset, offset);
        if (d < nearest_distance) {
            nearest = g;
            nearest_distance = d;
        }
    }
    let guide = nearest * points;
    dst[base] = src[base];
    for (var j = 1u; j < points; j++) {
        let before = src[base + j];
        let hold = smooth01((params.hold_start - before.y) / (params.hold_start - params.hold_full));
        dst[base + j] = vec4<f32>(before.xyz + moves[guide + j].xyz * (sinking * hold), before.w);
    }
    // Each segment back to its length before, root to tip.
    var previous = dst[base].xyz;
    var previous_moved = dst[base].xyz;
    for (var j = 1u; j < points; j++) {
        let moved = dst[base + j].xyz;
        let rest = distance(src[base + j].xyz, src[base + j - 1u].xyz);
        let direction = moved - previous_moved;
        let length = sqrt(dot(direction, direction));
        var along = src[base + j].xyz - src[base + j - 1u].xyz;
        if (length > 1e-12) {
            along = direction / length;
        } else {
            along = along / max(rest, 1e-12);
        }
        previous_moved = moved;
        let placed = previous + along * rest;
        dst[base + j] = vec4<f32>(placed, dst[base + j].w);
        previous = placed;
    }
}
"#;
