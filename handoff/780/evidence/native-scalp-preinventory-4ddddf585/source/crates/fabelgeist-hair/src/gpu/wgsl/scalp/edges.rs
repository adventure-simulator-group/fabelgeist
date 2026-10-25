//! Packed scalp-edge identity, guarded production, and boundary decoding.

use super::{MATH, kernel};

// The native sort key stores two vertex indices in one word.
const EDGE_VERTEX_BITS: u32 = u16::BITS;
pub(in crate::gpu) const EDGE_VERTEX_LIMIT: u32 = 1 << EDGE_VERTEX_BITS;

/// The triangles by vertex number, their areas, their edges as sort keys
/// (the lesser vertex's number above the greater's), and the box the
/// vertices are in, as ordered words ([`crate::gpu::wgsl::lod::bounds`]).
pub fn nets_triangles() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> triangles: array<vec4<u32>>;
@group(0) @binding(1) var<storage, read> vertex_of: array<u32>;
@group(0) @binding(2) var<storage, read> scalp_points: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> tri_index: array<vec4<u32>>;
@group(0) @binding(4) var<storage, read_write> areas: array<f32>;
@group(0) @binding(5) var<storage, read_write> edge_keys: array<u32>;
@group(0) @binding(6) var<storage, read_write> edge_ids: array<u32>;
@group(0) @binding(7) var<storage, read_write> box_words: array<atomic<u32>>;
struct Params {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(8) var<uniform> params: Params;
@group(0) @binding(9) var<storage, read> flags: array<u32>;
"#,
        &[&edge_keys()],
        r#"
fn ordered(x: f32) -> u32 {
    let bits = bitcast<u32>(x);
    return select(bits | 0x80000000u, ~bits, (bits & 0x80000000u) != 0u);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let t = item(id, groups);
    if (t >= params.count) {
        return;
    }
    // Invalid populations are rejected by the host after this batch.
    // Initialize every sorted key so no stale packed identity is decoded.
    if (flags[params.count * 3u] >= EDGE_VERTEX_LIMIT) {
        tri_index[t] = vec4<u32>(0u);
        areas[t] = 0.0;
        for (var e = 0u; e < 3u; e++) {
            edge_keys[t * 3u + e] = 0u;
            edge_ids[t * 3u + e] = t * 3u + e;
        }
        return;
    }
    let cells = triangles[t];
    let v = vec3<u32>(vertex_of[cells.x], vertex_of[cells.y], vertex_of[cells.z]);
    tri_index[t] = vec4<u32>(v, 0u);
    let a = scalp_points[v.x].xyz;
    let b = scalp_points[v.y].xyz;
    let c = scalp_points[v.z].xyz;
    areas[t] = length(cross(b - a, c - a)) * 0.5;
    // The box: every vertex is some triangle's.
    for (var e = 0u; e < 3u; e++) {
        let p = scalp_points[v[e]].xyz;
        for (var k = 0u; k < 3u; k++) {
            let key = ordered(p[k]);
            atomicMax(&box_words[k], ~key);
            atomicMax(&box_words[3u + k], key);
        }
    }
    for (var e = 0u; e < 3u; e++) {
        let p = v[e];
        let q = v[(e + 1u) % 3u];
        edge_keys[t * 3u + e] = packed_edge_key(p, q);
        edge_ids[t * 3u + e] = t * 3u + e;
    }
}
"#,
    )
}

/// Along the sorted edges, which are open (used by one triangle only) and
/// how many points each gets at two spacings (`steps` then `fine`), for
/// the scans to place them.
pub fn nets_open() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> edge_keys: array<u32>;
@group(0) @binding(1) var<storage, read> scalp_points: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> steps: array<u32>;
@group(0) @binding(3) var<storage, read_write> fine: array<u32>;
struct Params {
    count: u32,
    pad0: u32,
    step: f32,
    fine_step: f32,
};
@group(0) @binding(4) var<uniform> params: Params;
"#,
        &[&edge_keys()],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let k = item(id, groups);
    if (k >= params.count) {
        return;
    }
    let key = edge_keys[k];
    let open = (k == 0u || edge_keys[k - 1u] != key) && (k + 1u >= params.count || edge_keys[k + 1u] != key);
    steps[k] = 0u;
    fine[k] = 0u;
    if (!open) {
        return;
    }
    let length_ab = distance(scalp_points[edge_begin(key)].xyz, scalp_points[edge_end(key)].xyz);
    steps[k] = u32(max(ceil(length_ab / max(params.step, 1e-5)), 1.0));
    fine[k] = u32(max(ceil(length_ab / max(params.fine_step, 1e-5)), 1.0));
}
"#,
    )
}

/// The points along the open edges, each edge's from its lesser vertex.
pub fn nets_boundary() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> edge_keys: array<u32>;
@group(0) @binding(1) var<storage, read> scalp_points: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> offsets: array<u32>;
@group(0) @binding(3) var<storage, read_write> boundary: array<vec4<f32>>;
struct Params {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(4) var<uniform> params: Params;
"#,
        &[MATH, &edge_keys()],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let k = item(id, groups);
    if (k >= params.count) {
        return;
    }
    let first = offsets[k];
    let steps = offsets[k + 1u] - first;
    let key = edge_keys[k];
    let a = scalp_points[edge_begin(key)].xyz;
    let b = scalp_points[edge_end(key)].xyz;
    for (var i = 0u; i < steps; i++) {
        boundary[first + i] = vec4<f32>(lerp3(a, b, f32(i) / f32(steps)), 0.0);
    }
}
"#,
    )
}

fn edge_keys() -> String {
    format!(
        "const EDGE_VERTEX_BITS: u32 = {EDGE_VERTEX_BITS}u;\n\
         const EDGE_VERTEX_LIMIT: u32 = {EDGE_VERTEX_LIMIT}u;\n"
    ) + r#"
fn packed_edge_key(a: u32, b: u32) -> u32 {
    return (min(a, b) << EDGE_VERTEX_BITS) | max(a, b);
}

fn edge_begin(key: u32) -> u32 {
    return key >> EDGE_VERTEX_BITS;
}

fn edge_end(key: u32) -> u32 {
    return key & (EDGE_VERTEX_LIMIT - 1u);
}
"#
}
