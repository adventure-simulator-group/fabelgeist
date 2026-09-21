//! The torso's measurement in WGSL: the torso frame and the front torso's
//! support, rounded in a fixed order of operations with exact device
//! arithmetic (see [`fabelgeist_compute::host_float`]), so that the
//! breastplate's wearer is measured deterministically, independent of the
//! device's fused operations.

use adventuresim_armor_model::gpu::wgsl;
use fabelgeist_compute::host_float;

use crate::device_torso::{STATUS_COINCIDENT_LANDMARKS, STATUS_DEGENERATE_WIDTH, STATUS_NO_WIDTH};

/// A torso kernel's full source.
pub(crate) fn torso_source(entry: &str) -> String {
    format!(
        r#"
struct Params {{
    count: u32,
    joint0: u32,
    joint1: u32,
    joint2: u32,
    joint3: u32,
    joint4: u32,
    joint5: u32,
    joint6: u32,
    joint7: u32,
    joint8: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
    pad3: u32,
    pad4: u32,
    pad5: u32,
}};
const COINCIDENT_LANDMARKS: u32 = {coincident}u;
const NO_WIDTH: u32 = {no_width}u;
const DEGENERATE_WIDTH: u32 = {degenerate_width}u;

{zero_hook}{host_float}

// The unit vector along `a`: w is one when `a` is long enough to normalize.
fn unit(a: vec3<f32>) -> vec4<f32> {{
    let magnitude = host_length(a);
    if (magnitude > 1e-8) {{
        return vec4<f32>(host_scale3(a, host_div(1.0, magnitude)), 1.0);
    }}
    return vec4<f32>(a, 0.0);
}}
{entry}
"#,
        coincident = STATUS_COINCIDENT_LANDMARKS,
        no_width = STATUS_NO_WIDTH,
        degenerate_width = STATUS_DEGENERATE_WIDTH,
        zero_hook = host_float::zero_hook("params.pad0"),
        host_float = host_float::wgsl(),
        entry = entry
            .replace("{status}", wgsl::STATUS)
            .replace("{readers}", FRAME_READERS),
    )
}

/// What the per-vertex torso kernels read of the frame.
const FRAME_READERS: &str = r#"
fn bottom() -> vec3<f32> { return vec3<f32>(frame[0], frame[1], frame[2]); }
fn vertical_axis() -> vec3<f32> { return vec3<f32>(frame[3], frame[4], frame[5]); }
fn vertical_extent() -> f32 { return frame[6]; }
fn lateral_axis() -> vec3<f32> { return vec3<f32>(frame[7], frame[8], frame[9]); }
fn front_axis() -> vec3<f32> { return vec3<f32>(frame[10], frame[11], frame[12]); }

fn point(i: u32) -> vec3<f32> {
    return vec3<f32>(positions[i * 3u], positions[i * 3u + 1u], positions[i * 3u + 2u]);
}

fn normal(i: u32) -> vec3<f32> {
    return vec3<f32>(normals[i * 3u], normals[i * 3u + 1u], normals[i * 3u + 2u]);
}

fn raw_coordinates(p: vec3<f32>) -> vec2<f32> {
    let relative = host_fence3(p - bottom());
    return vec2<f32>(
        host_dot(relative, lateral_axis()),
        host_div(host_dot(relative, vertical_axis()), vertical_extent()),
    );
}

fn support_weight(v: u32) -> f32 {
    var weight = 0.0;
    for (var k = 0u; k < 8u; k = k + 1u) {
        if (supports[joint_indices[v * 8u + k]] != 0u) {
            weight = host_fence(weight + joint_weights[v * 8u + k]);
        }
    }
    return weight;
}
"#;

/// One invocation: the torso frame from the rig -- rising from the lower
/// spine to the neck, lateral from the right clavicle to the left, and facing
/// the eyes square to the rise -- and the rig anchors the breastplate reads.
pub(crate) const FRAME: &str = r#"
@group(0) @binding(0) var<storage, read> joints: array<f32>;
@group(0) @binding(1) var<storage, read_write> frame: array<f32>;
@group(0) @binding(2) var<storage, read_write> rig: array<f32>;
@group(0) @binding(3) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(4) var<uniform> params: Params;
{status}

fn joint(index: u32) -> vec3<f32> {
    return vec3<f32>(joints[index * 8u], joints[index * 8u + 1u], joints[index * 8u + 2u]);
}

@compute @workgroup_size(1)
fn main() {
    let bottom = joint(params.joint0);
    let neck = joint(params.joint1);
    let left_clavicle = joint(params.joint2);
    let right_clavicle = joint(params.joint3);
    let head = joint(params.joint6);
    let eyes = host_fence3(host_fence3(joint(params.joint7) + joint(params.joint8)) * 0.5);
    let rise = host_fence3(neck - bottom);
    let vertical = unit(rise);
    let lateral = unit(host_fence3(left_clavicle - right_clavicle));
    let eye_direction = host_fence3(eyes - head);
    let front = unit(host_fence3(
        eye_direction - host_fence3(vertical.xyz * host_dot(eye_direction, vertical.xyz)),
    ));
    if (vertical.w == 0.0 || lateral.w == 0.0 || front.w == 0.0) {
        fail(COINCIDENT_LANDMARKS);
        return;
    }
    var words = array<f32, 13>(
        bottom.x, bottom.y, bottom.z,
        vertical.x, vertical.y, vertical.z,
        host_length(rise),
        lateral.x, lateral.y, lateral.z,
        front.x, front.y, front.z,
    );
    for (var k = 0u; k < 13u; k = k + 1u) {
        frame[k] = words[k];
    }
    var anchors = array<vec3<f32>, 4>(front.xyz, neck, joint(params.joint4), joint(params.joint5));
    for (var k = 0u; k < 12u; k = k + 1u) {
        rig[k] = anchors[k / 3u][k % 3u];
    }
}
"#;

/// One invocation per body vertex: its lateral extent, if it measures the
/// front's width, or a negative mark.
pub(crate) const WIDTHS: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> normals: array<f32>;
@group(0) @binding(2) var<storage, read> joint_indices: array<u32>;
@group(0) @binding(3) var<storage, read> joint_weights: array<f32>;
@group(0) @binding(4) var<storage, read> supports: array<u32>;
@group(0) @binding(5) var<storage, read> frame: array<f32>;
@group(0) @binding(6) var<storage, read_write> widths: array<f32>;
@group(0) @binding(7) var<storage, read_write> counter: array<atomic<u32>>;
@group(0) @binding(8) var<uniform> params: Params;
{readers}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let v = id.x;
    if (v >= params.count) {
        return;
    }
    let raw = raw_coordinates(point(v));
    let measures = raw.y >= 0.05 && raw.y <= 0.95
        && host_dot(normal(v), front_axis()) > 0.02
        && support_weight(v) >= 0.2;
    widths[v] = select(-1.0, abs(raw.x), measures);
    if (measures) {
        atomicAdd(&counter[0], 1u);
    }
}
"#;

/// One invocation per body vertex: the width at the ninetieth percentile,
/// found by the one width ranked there.
pub(crate) const RANK: &str = r#"
@group(0) @binding(0) var<storage, read> widths: array<f32>;
@group(0) @binding(1) var<storage, read> counter: array<u32>;
@group(0) @binding(2) var<storage, read_write> frame: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let v = id.x;
    if (v >= params.count) {
        return;
    }
    let width = widths[v];
    if (width < 0.0) {
        return;
    }
    var rank = 0u;
    for (var other = 0u; other < params.count; other = other + 1u) {
        let w = widths[other];
        if (w >= 0.0 && (w < width || (w == width && other < v))) {
            rank = rank + 1u;
        }
    }
    if (rank == counter[0] * 9u / 10u) {
        frame[13] = width;
    }
}
"#;

/// One invocation: whether the half width measures a torso.
pub(crate) const HALF_WIDTH: &str = r#"
@group(0) @binding(0) var<storage, read> counter: array<u32>;
@group(0) @binding(1) var<storage, read> frame: array<f32>;
@group(0) @binding(2) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(3) var<uniform> params: Params;
{status}

@compute @workgroup_size(1)
fn main() {
    if (counter[0] == 0u) {
        fail(NO_WIDTH);
    } else if (frame[13] <= 1e-4) {
        fail(DEGENERATE_WIDTH);
    }
}
"#;

/// One invocation per body vertex: its semantic coordinates, and whether
/// the front torso supports it.
pub(crate) const SUPPORT: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> normals: array<f32>;
@group(0) @binding(2) var<storage, read> joint_indices: array<u32>;
@group(0) @binding(3) var<storage, read> joint_weights: array<f32>;
@group(0) @binding(4) var<storage, read> supports: array<u32>;
@group(0) @binding(5) var<storage, read> frame: array<f32>;
@group(0) @binding(6) var<storage, read_write> semantic: array<f32>;
@group(0) @binding(7) var<storage, read_write> support: array<u32>;
@group(0) @binding(8) var<uniform> params: Params;
{readers}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let v = id.x;
    if (v >= params.count) {
        return;
    }
    let raw = raw_coordinates(point(v));
    let lateral = host_div(raw.x, frame[13]);
    let vertical = raw.y;
    semantic[v * 2u] = lateral;
    semantic[v * 2u + 1u] = vertical;
    let supported = abs(lateral) <= 1.35
        && vertical >= -0.30 && vertical <= 1.08
        && host_dot(normal(v), front_axis()) > 0.02
        && support_weight(v) >= 0.2;
    support[v] = select(0u, 1u, supported);
}
"#;
