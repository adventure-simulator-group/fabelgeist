//! The query kernels.
//!
//! Every kernel stages its targets through workgroup memory a tile at a time:
//! all 256 invocations of a workgroup load one target each, then each tests
//! its own query against the whole tile. The targets are read from global
//! memory once per workgroup rather than once per query.
//!
//! Ties go to the lowest target index. Targets are visited in ascending order
//! and only a strictly better candidate replaces the incumbent, so a query
//! gets the same answer as a host loop taking the first minimum.

/// Workgroup size, and targets per tile.
pub(super) const TILE: u32 = 256;

const TARGETS: &str = r#"
// A target set is either every element, or the listed elements.
fn target_id(slot: u32) -> u32 {
    if (params.use_candidates != 0u) {
        return candidates[slot];
    }
    return slot;
}

fn load_point(buffer_index: u32) -> vec3<f32> {
    return vec3<f32>(
        positions[buffer_index * 3u],
        positions[buffer_index * 3u + 1u],
        positions[buffer_index * 3u + 2u],
    );
}

fn load_query(index: u32) -> vec3<f32> {
    return vec3<f32>(queries[index * 3u], queries[index * 3u + 1u], queries[index * 3u + 2u]);
}

// Summed in the order a host loop over x, y, z sums it.
fn squared_length(v: vec3<f32>) -> f32 {
    return (v.x * v.x + v.y * v.y) + v.z * v.z;
}
"#;

const PARAMS: &str = r#"
struct Params {
    query_count: u32,
    target_count: u32,
    use_candidates: u32,
    mode: u32,
};
"#;

/// Nearest point of a point set, per query.
pub(super) fn nearest_points() -> fabelgeist_gpu::prelude::ShaderSource {
    fabelgeist_gpu::prelude::ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> queries: array<f32>;
@group(0) @binding(1) var<storage, read> positions: array<f32>;
@group(0) @binding(2) var<storage, read> candidates: array<u32>;
@group(0) @binding(3) var<storage, read_write> nearest: array<u32>;
@group(0) @binding(4) var<storage, read_write> distances: array<f32>;
{PARAMS}
@group(0) @binding(5) var<uniform> params: Params;
{TARGETS}

var<workgroup> tile_points: array<vec3<f32>, {TILE}u>;
var<workgroup> tile_ids: array<u32, {TILE}u>;

@compute @workgroup_size({TILE}u)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {{
    let query = global_id.x;
    let answering = query < params.query_count;
    var point = vec3<f32>(0.0);
    if (answering) {{
        point = load_query(query);
    }}
    var best = 0xffffffffu;
    var best_distance = 3.4e38;
    let tiles = (params.target_count + {TILE}u - 1u) / {TILE}u;
    for (var tile = 0u; tile < tiles; tile = tile + 1u) {{
        let slot = tile * {TILE}u + local_id.x;
        if (slot < params.target_count) {{
            let id = target_id(slot);
            tile_ids[local_id.x] = id;
            tile_points[local_id.x] = load_point(id);
        }}
        workgroupBarrier();
        let loaded = min({TILE}u, params.target_count - tile * {TILE}u);
        if (answering) {{
            for (var i = 0u; i < loaded; i = i + 1u) {{
                let distance = squared_length(point - tile_points[i]);
                let id = tile_ids[i];
                if (distance < best_distance || (distance == best_distance && id < best)) {{
                    best_distance = distance;
                    best = id;
                }}
            }}
        }}
        workgroupBarrier();
    }}
    if (answering) {{
        nearest[query] = best;
        distances[query] = best_distance;
    }}
}}
"#
    ))
}

const TRIANGLES: &str = r#"
struct Triangle {
    a: vec3<f32>,
    b: vec3<f32>,
    c: vec3<f32>,
};

fn load_triangle(id: u32) -> Triangle {
    return Triangle(
        load_point(triangles[id * 3u]),
        load_point(triangles[id * 3u + 1u]),
        load_point(triangles[id * 3u + 2u]),
    );
}
"#;

/// Closest point on a triangle set, per query, with barycentric weights.
pub(super) fn closest_triangles() -> fabelgeist_gpu::prelude::ShaderSource {
    fabelgeist_gpu::prelude::ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> queries: array<f32>;
@group(0) @binding(1) var<storage, read> positions: array<f32>;
@group(0) @binding(2) var<storage, read> triangles: array<u32>;
@group(0) @binding(3) var<storage, read> candidates: array<u32>;
@group(0) @binding(4) var<storage, read_write> nearest: array<u32>;
@group(0) @binding(5) var<storage, read_write> weights: array<f32>;
@group(0) @binding(6) var<storage, read_write> distances: array<f32>;
{PARAMS}
@group(0) @binding(7) var<uniform> params: Params;
{TARGETS}
{TRIANGLES}

var<workgroup> tile_triangles: array<Triangle, {TILE}u>;
var<workgroup> tile_ids: array<u32, {TILE}u>;

// Barycentric weights of the closest point on a triangle.
// After Ericson, "Real-Time Collision Detection", 5.1.5.
fn closest_weights(p: vec3<f32>, t: Triangle) -> vec3<f32> {{
    let ab = t.b - t.a;
    let ac = t.c - t.a;
    let ap = p - t.a;
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if (d1 <= 0.0 && d2 <= 0.0) {{
        return vec3<f32>(1.0, 0.0, 0.0);
    }}
    let bp = p - t.b;
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if (d3 >= 0.0 && d4 <= d3) {{
        return vec3<f32>(0.0, 1.0, 0.0);
    }}
    let vc = d1 * d4 - d3 * d2;
    if (vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0) {{
        let v = d1 / (d1 - d3);
        return vec3<f32>(1.0 - v, v, 0.0);
    }}
    let cp = p - t.c;
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if (d6 >= 0.0 && d5 <= d6) {{
        return vec3<f32>(0.0, 0.0, 1.0);
    }}
    let vb = d5 * d2 - d1 * d6;
    if (vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0) {{
        let w = d2 / (d2 - d6);
        return vec3<f32>(1.0 - w, 0.0, w);
    }}
    let va = d3 * d6 - d5 * d4;
    if (va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0) {{
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return vec3<f32>(0.0, 1.0 - w, w);
    }}
    let denominator = 1.0 / (va + vb + vc);
    let v = vb * denominator;
    let w = vc * denominator;
    return vec3<f32>(1.0 - v - w, v, w);
}}

@compute @workgroup_size({TILE}u)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {{
    let query = global_id.x;
    let answering = query < params.query_count;
    var point = vec3<f32>(0.0);
    if (answering) {{
        point = load_query(query);
    }}
    var best = 0xffffffffu;
    var best_distance = 3.4e38;
    var best_weights = vec3<f32>(0.0);
    let tiles = (params.target_count + {TILE}u - 1u) / {TILE}u;
    for (var tile = 0u; tile < tiles; tile = tile + 1u) {{
        let slot = tile * {TILE}u + local_id.x;
        if (slot < params.target_count) {{
            let id = target_id(slot);
            tile_ids[local_id.x] = id;
            tile_triangles[local_id.x] = load_triangle(id);
        }}
        workgroupBarrier();
        let loaded = min({TILE}u, params.target_count - tile * {TILE}u);
        if (answering) {{
            for (var i = 0u; i < loaded; i = i + 1u) {{
                let triangle = tile_triangles[i];
                let w = closest_weights(point, triangle);
                let closest = triangle.a * w.x + triangle.b * w.y + triangle.c * w.z;
                let distance = squared_length(point - closest);
                let id = tile_ids[i];
                if (distance < best_distance || (distance == best_distance && id < best)) {{
                    best_distance = distance;
                    best = id;
                    best_weights = w;
                }}
            }}
        }}
        workgroupBarrier();
    }}
    if (answering) {{
        nearest[query] = best;
        weights[query * 3u] = best_weights.x;
        weights[query * 3u + 1u] = best_weights.y;
        weights[query * 3u + 2u] = best_weights.z;
        distances[query] = best_distance;
    }}
}}
"#
    ))
}

/// First or last crossing of a ray with a triangle set.
///
/// `mode` 0 keeps the nearest crossing, 1 the farthest; both only within
/// `[minimum, maximum]` along the ray, in units of the direction's length.
pub(super) fn ray_triangles() -> fabelgeist_gpu::prelude::ShaderSource {
    fabelgeist_gpu::prelude::ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> queries: array<f32>;
@group(0) @binding(1) var<storage, read> directions: array<f32>;
@group(0) @binding(2) var<storage, read> positions: array<f32>;
@group(0) @binding(3) var<storage, read> triangles: array<u32>;
@group(0) @binding(4) var<storage, read> candidates: array<u32>;
@group(0) @binding(5) var<storage, read_write> nearest: array<u32>;
@group(0) @binding(6) var<storage, read_write> weights: array<f32>;
@group(0) @binding(7) var<storage, read_write> distances: array<f32>;
struct Params {{
    query_count: u32,
    target_count: u32,
    use_candidates: u32,
    mode: u32,
    minimum: f32,
    maximum: f32,
    epsilon: f32,
    pad: u32,
}};
@group(0) @binding(8) var<uniform> params: Params;
{TARGETS}
{TRIANGLES}

var<workgroup> tile_triangles: array<Triangle, {TILE}u>;
var<workgroup> tile_ids: array<u32, {TILE}u>;

// Moller-Trumbore, two-sided. Returns (t, u, v), or t < 0 for a miss.
fn crossing(origin: vec3<f32>, direction: vec3<f32>, t: Triangle) -> vec3<f32> {{
    let e1 = t.b - t.a;
    let e2 = t.c - t.a;
    let p = cross(direction, e2);
    let determinant = dot(e1, p);
    if (abs(determinant) <= params.epsilon) {{
        return vec3<f32>(-1.0, 0.0, 0.0);
    }}
    let inverse = 1.0 / determinant;
    let s = origin - t.a;
    let u = dot(s, p) * inverse;
    if (u < 0.0 || u > 1.0) {{
        return vec3<f32>(-1.0, 0.0, 0.0);
    }}
    let q = cross(s, e1);
    let v = dot(direction, q) * inverse;
    if (v < 0.0 || u + v > 1.0) {{
        return vec3<f32>(-1.0, 0.0, 0.0);
    }}
    return vec3<f32>(dot(e2, q) * inverse, u, v);
}}

@compute @workgroup_size({TILE}u)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
) {{
    let query = global_id.x;
    let answering = query < params.query_count;
    var origin = vec3<f32>(0.0);
    var direction = vec3<f32>(0.0);
    if (answering) {{
        origin = load_query(query);
        direction = vec3<f32>(
            directions[query * 3u],
            directions[query * 3u + 1u],
            directions[query * 3u + 2u],
        );
    }}
    let farthest = params.mode == 1u;
    var best = 0xffffffffu;
    var best_t = select(3.4e38, -3.4e38, farthest);
    var best_uv = vec2<f32>(0.0);
    let tiles = (params.target_count + {TILE}u - 1u) / {TILE}u;
    for (var tile = 0u; tile < tiles; tile = tile + 1u) {{
        let slot = tile * {TILE}u + local_id.x;
        if (slot < params.target_count) {{
            let id = target_id(slot);
            tile_ids[local_id.x] = id;
            tile_triangles[local_id.x] = load_triangle(id);
        }}
        workgroupBarrier();
        let loaded = min({TILE}u, params.target_count - tile * {TILE}u);
        if (answering) {{
            for (var i = 0u; i < loaded; i = i + 1u) {{
                let hit = crossing(origin, direction, tile_triangles[i]);
                let id = tile_ids[i];
                if (hit.x < params.minimum || hit.x > params.maximum) {{
                    continue;
                }}
                let better = select((hit.x < best_t), (hit.x > best_t), farthest);
                if (better || (hit.x == best_t && id < best)) {{
                    best_t = hit.x;
                    best = id;
                    best_uv = hit.yz;
                }}
            }}
        }}
        workgroupBarrier();
    }}
    if (answering) {{
        nearest[query] = best;
        weights[query * 3u] = 1.0 - best_uv.x - best_uv.y;
        weights[query * 3u + 1u] = best_uv.x;
        weights[query * 3u + 2u] = best_uv.y;
        distances[query] = best_t;
    }}
}}
"#
    ))
}
