//! Kernels for [`super::GpuSurfaceContacts`].
//!
//! The narrow phase is a port of [`crate::ccd`] and the projection of
//! `surface_contact::solve`, in `f32`. Every pair is evaluated relative to its
//! first start position, so the geometry stays within millimetres of the
//! origin and single precision loses nothing that the contact shell can see.

use fabelgeist_bvh::gpu::{TraversalConfig, traversal_source};

const PARAMS: &str = r#"
struct Params {
    first: u32,
    count: u32,
    particle_count: u32,
    budget: u32,
    thickness: f32,
    static_clearance: f32,
    search_radius: f32,
    primitive_count: u32,
    edges_offset: u32,
    groups_offset: u32,
    static_faces_offset: u32,
    static_edges_offset: u32,
};
"#;

/// Cloth against cloth: seven storage buffers.
const CLOTH_BINDINGS: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
// [0, n): positions at the start of the swept interval. [n, 2n): velocities.
@group(0) @binding(1) var<storage, read> motion: array<vec4<f32>>;
// Twelve words per particle -- the largest positive then largest negative
// position correction per axis, the same for velocity -- then one contact
// counter.
@group(0) @binding(2) var<storage, read_write> accumulator: array<atomic<i32>>;
@group(0) @binding(3) var<storage, read> topology: array<u32>;
@group(0) @binding(4) var<storage, read> bvh_nodes: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> bvh_right_children: array<u32>;
@group(0) @binding(6) var<storage, read> bvh_indices: array<u32>;
@group(0) @binding(7) var<uniform> params: Params;
"#;

/// Cloth against the fixed obstacle: eight storage buffers, the device minimum.
const OBSTACLE_BINDINGS: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> motion: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> accumulator: array<atomic<i32>>;
@group(0) @binding(3) var<storage, read> topology: array<u32>;
@group(0) @binding(4) var<storage, read> static_positions: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> bvh_nodes: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read> bvh_right_children: array<u32>;
@group(0) @binding(7) var<storage, read> bvh_indices: array<u32>;
@group(0) @binding(8) var<uniform> params: Params;

fn static_point(index: u32) -> vec3<f32> {
    return static_positions[index].xyz;
}
"#;

/// Particle access, accumulation, conservative advancement and projection.
const NARROW_PHASE: &str = r#"
const VERTEX_TRIANGLE: u32 = 0u;
const EDGE_EDGE: u32 = 1u;
// Marks an obstacle vertex, which receives no correction.
const STATIC_ID: u32 = 0xFFFFFFFFu;
// Fixed point, because WGSL has no atomic float. Positions resolve to about a
// micrometre, velocities to about fifteen micrometres per second.
const POSITION_SCALE: f32 = 1048576.0;
const VELOCITY_SCALE: f32 = 65536.0;
// Lengths below this have no usable direction.
const TINY: f32 = 1e-10;

fn dynamic_end(index: u32) -> vec3<f32> {
    return positions[index].xyz;
}

fn dynamic_start(index: u32) -> vec3<f32> {
    return motion[index].xyz;
}

fn dynamic_velocity(index: u32) -> vec3<f32> {
    return motion[params.particle_count + index].xyz;
}

fn dynamic_mass(index: u32) -> f32 {
    return positions[index].w;
}

fn seam_group(index: u32) -> u32 {
    return topology[params.groups_offset + index];
}

// Keep the extremes rather than a sum or an average. Pairs sharing a particle
// were all measured before any of them moved it: summing over-corrects, and
// averaging lets a shallow pair dilute the deepest one until the particle
// stays through the surface. The largest push along each axis in each
// direction resolves every pair at once, and opposing pushes still cancel.
fn accumulate(base: u32, delta: vec3<f32>, scale: f32) {
    let fixed = vec3<i32>(round(delta * scale));
    atomicMax(&accumulator[base], fixed.x);
    atomicMax(&accumulator[base + 1u], fixed.y);
    atomicMax(&accumulator[base + 2u], fixed.z);
    atomicMin(&accumulator[base + 3u], fixed.x);
    atomicMin(&accumulator[base + 4u], fixed.y);
    atomicMin(&accumulator[base + 5u], fixed.z);
}

struct Contact {
    time: f32,
    distance: f32,
    weights: vec4<f32>,
    normal: vec3<f32>,
    hit: bool,
};

fn segment_parameter(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>) -> f32 {
    let edge = b - a;
    let length_squared = dot(edge, edge);
    if (length_squared > 0.0) {
        return clamp(dot(p - a, edge) / length_squared, 0.0, 1.0);
    }
    return 0.0;
}

// Barycentric weights of the closest point, and in `w` whether it lies inside
// the face. The edge fallback also covers degenerate triangles without
// dividing by their area.
fn triangle_weights(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec4<f32> {
    let ab = b - a;
    let ac = c - a;
    let normal = cross(ab, ac);
    let area2 = dot(normal, normal);
    if (area2 > 0.0) {
        let projected = p - normal * (dot(p - a, normal) / area2);
        let ap = projected - a;
        let v = dot(cross(ap, ac), normal) / area2;
        let w = dot(cross(ab, ap), normal) / area2;
        let u = 1.0 - v - w;
        if (u >= 0.0 && v >= 0.0 && w >= 0.0) {
            return vec4<f32>(u, v, w, 1.0);
        }
    }
    let t0 = segment_parameter(p, a, b);
    var best = vec3<f32>(1.0 - t0, t0, 0.0);
    var best_distance = length(p - (a + (b - a) * t0));
    let t1 = segment_parameter(p, b, c);
    let d1 = length(p - (b + (c - b) * t1));
    if (d1 < best_distance) {
        best = vec3<f32>(0.0, 1.0 - t1, t1);
        best_distance = d1;
    }
    let t2 = segment_parameter(p, c, a);
    let d2 = length(p - (c + (a - c) * t2));
    if (d2 < best_distance) {
        best = vec3<f32>(t2, 0.0, 1.0 - t2);
    }
    return vec4<f32>(best, 0.0);
}

// Closest parameters on both segments, and in `z` whether both lie strictly
// inside their segments.
fn edge_parameters(a: vec3<f32>, b: vec3<f32>, c: vec3<f32>, d: vec3<f32>) -> vec3<f32> {
    let u = b - a;
    let v = d - c;
    let r = a - c;
    let aa = dot(u, u);
    let bb = dot(u, v);
    let cc = dot(v, v);
    let dd = dot(u, r);
    let ee = dot(v, r);
    if (aa == 0.0 && cc == 0.0) {
        return vec3<f32>(0.0, 0.0, 0.0);
    }
    if (aa == 0.0) {
        return vec3<f32>(0.0, clamp(ee / cc, 0.0, 1.0), 0.0);
    }
    if (cc == 0.0) {
        return vec3<f32>(clamp(-dd / aa, 0.0, 1.0), 0.0, 0.0);
    }
    let normal = cross(u, v);
    let determinant = dot(normal, normal);
    var interior = determinant > 1e-10 * aa * cc;
    var s = 0.0;
    if (interior) {
        s = clamp((bb * ee - cc * dd) / determinant, 0.0, 1.0);
    }
    var t = (bb * s + ee) / cc;
    if (t < 0.0) {
        t = 0.0;
        s = clamp(-dd / aa, 0.0, 1.0);
        interior = false;
    } else if (t > 1.0) {
        t = 1.0;
        s = clamp((bb - dd) / aa, 0.0, 1.0);
        interior = false;
    }
    if (s <= 0.0 || s >= 1.0) {
        interior = false;
    }
    return vec3<f32>(s, t, select(0.0, 1.0, interior));
}

fn ccd_sample(pair: u32, p: array<vec3<f32>, 4>, time: f32, fallback: vec3<f32>) -> Contact {
    var weights: vec4<f32>;
    var interior: bool;
    var geometric: vec3<f32>;
    if (pair == VERTEX_TRIANGLE) {
        let bary = triangle_weights(p[0], p[1], p[2], p[3]);
        weights = vec4<f32>(1.0, -bary.x, -bary.y, -bary.z);
        interior = bary.w > 0.5;
        geometric = cross(p[2] - p[1], p[3] - p[1]);
    } else {
        let st = edge_parameters(p[0], p[1], p[2], p[3]);
        weights = vec4<f32>(1.0 - st.x, st.x, -(1.0 - st.y), -st.y);
        interior = st.z > 0.5;
        geometric = cross(p[1] - p[0], p[3] - p[2]);
    }
    let delta = p[0] * weights.x + p[1] * weights.y + p[2] * weights.z + p[3] * weights.w;
    var distance = length(delta);
    let geometric_length = length(geometric);
    var direction = vec3<f32>(1.0, 0.0, 0.0);
    if (interior && geometric_length > TINY) {
        // Inside the face, or between both edges' ends, the separation is
        // along the primitives' own normal. Taking that direction from `delta`
        // instead would lose it in single precision: `delta` is a millimetre
        // gap reconstructed from points a metre apart.
        let unit = geometric / geometric_length;
        let along = dot(delta, unit);
        distance = abs(along);
        if (distance > TINY) {
            direction = unit * sign(along);
        } else if (dot(fallback, unit) < 0.0) {
            direction = -unit;
        } else {
            direction = unit;
        }
    } else if (distance > TINY) {
        direction = delta / distance;
    } else if (length(fallback) > TINY) {
        direction = normalize(fallback);
    } else if (geometric_length > TINY) {
        direction = geometric / geometric_length;
    }
    return Contact(time, distance, weights, direction, true);
}

// Conservative advancement over linear paths. An exhausted budget returns the
// last safe time as a contact, never a miss.
fn ccd_sweep(pair: u32, start: array<vec3<f32>, 4>, end: array<vec3<f32>, 4>, clearance: f32) -> Contact {
    var velocity: array<vec3<f32>, 4>;
    for (var i = 0u; i < 4u; i = i + 1u) {
        velocity[i] = end[i] - start[i];
    }
    var speed = 0.0;
    if (pair == VERTEX_TRIANGLE) {
        for (var i = 1u; i < 4u; i = i + 1u) {
            speed = max(speed, length(velocity[0] - velocity[i]));
        }
    } else {
        for (var i = 0u; i < 2u; i = i + 1u) {
            for (var j = 2u; j < 4u; j = j + 1u) {
                speed = max(speed, length(velocity[i] - velocity[j]));
            }
        }
    }
    var contact = ccd_sample(pair, start, 0.0, vec3<f32>(0.0));
    let initial = contact.distance;
    if (initial <= clearance) {
        return contact;
    }
    if (speed <= TINY) {
        contact.hit = false;
        return contact;
    }
    // Stop a small fraction of the gap short of contact, which keeps the
    // lower bound safe and the step count finite.
    let goal = clearance + (initial - clearance) * 0.01;
    var time = 0.0;
    for (var iteration = 0u; iteration < params.budget; iteration = iteration + 1u) {
        if (contact.distance <= goal) {
            return contact;
        }
        let safe_step = 0.9 * (contact.distance - clearance) / speed;
        if (safe_step >= 1.0 - time) {
            contact.hit = false;
            return contact;
        }
        let advanced = time + safe_step;
        if (advanced <= time) {
            return contact;
        }
        time = advanced;
        var p: array<vec3<f32>, 4>;
        for (var i = 0u; i < 4u; i = i + 1u) {
            p[i] = start[i] + velocity[i] * time;
        }
        contact = ccd_sample(pair, p, time, contact.normal);
    }
    return contact;
}

// Project one pair, as `surface_contact::solve::resolve` does, into the
// accumulator rather than the positions.
fn ccd_resolve(
    pair: u32,
    ids: array<u32, 4>,
    start: array<vec3<f32>, 4>,
    end: array<vec3<f32>, 4>,
    velocity: array<vec3<f32>, 4>,
    masses: vec4<f32>,
    thickness: f32,
) {
    let origin = start[0];
    var local_start: array<vec3<f32>, 4>;
    var local_end: array<vec3<f32>, 4>;
    for (var i = 0u; i < 4u; i = i + 1u) {
        local_start[i] = start[i] - origin;
        local_end[i] = end[i] - origin;
    }
    // The swept guard sits inside the resting shell, so touching cloth can
    // slide without reporting time zero every substep.
    var contact = ccd_sweep(pair, local_start, local_end, thickness * 0.5);
    if (!contact.hit) {
        let previous = ccd_sample(pair, local_start, 0.0, vec3<f32>(0.0));
        contact = ccd_sample(pair, local_end, 1.0, previous.normal);
    }
    let w = contact.weights;
    let closest = local_end[0] * w.x + local_end[1] * w.y + local_end[2] * w.z + local_end[3] * w.w;
    let depth = thickness - dot(closest, contact.normal);
    if (depth <= 0.0) {
        return;
    }
    let denominator = masses.x * w.x * w.x + masses.y * w.y * w.y
        + masses.z * w.z * w.z + masses.w * w.w * w.w;
    if (denominator <= 1e-12) {
        return;
    }
    let relative = dot(
        velocity[0] * w.x + velocity[1] * w.y + velocity[2] * w.z + velocity[3] * w.w,
        contact.normal,
    );
    for (var i = 0u; i < 4u; i = i + 1u) {
        let share = w[i] * masses[i] / denominator;
        if (ids[i] == STATIC_ID || share == 0.0) {
            continue;
        }
        accumulate(ids[i] * 12u, contact.normal * (depth * share), POSITION_SCALE);
        if (relative < 0.0) {
            accumulate(ids[i] * 12u + 6u, contact.normal * (-relative * share), VELOCITY_SCALE);
        }
    }
    atomicAdd(&accumulator[params.particle_count * 12u], 1);
}
"#;

/// Every cloth vertex against every cloth face.
const VERTEX_FACE: &str = r#"
var<private> query_particle: u32;

fn bvh_hit(face: u32) {
    let point = query_particle;
    let a = topology[face * 3u];
    let b = topology[face * 3u + 1u];
    let c = topology[face * 3u + 2u];
    // A face sharing the vertex, or a sewn copy of it, is the same surface.
    let seam = seam_group(point);
    if (seam_group(a) == seam || seam_group(b) == seam || seam_group(c) == seam) {
        return;
    }
    let masses = vec4<f32>(dynamic_mass(point), dynamic_mass(a), dynamic_mass(b), dynamic_mass(c));
    if (all(masses == vec4<f32>(0.0))) {
        return;
    }
    ccd_resolve(
        VERTEX_TRIANGLE,
        array<u32, 4>(point, a, b, c),
        array<vec3<f32>, 4>(dynamic_start(point), dynamic_start(a), dynamic_start(b), dynamic_start(c)),
        array<vec3<f32>, 4>(dynamic_end(point), dynamic_end(a), dynamic_end(b), dynamic_end(c)),
        array<vec3<f32>, 4>(dynamic_velocity(point), dynamic_velocity(a), dynamic_velocity(b), dynamic_velocity(c)),
        masses,
        params.thickness,
    );
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.count || params.primitive_count == 0u) {
        return;
    }
    let point = params.first + global_id.x;
    query_particle = point;
    let start = dynamic_start(point);
    let end = dynamic_end(point);
    // Face bounds already carry one search radius.
    let reach = vec3<f32>(params.search_radius);
    bvh_query_aabb(min(start, end) - reach, max(start, end) + reach);
}
"#;

/// Free cloth vertices against obstacle faces.
const VERTEX_STATIC_FACE: &str = r#"
var<private> query_particle: u32;

fn bvh_hit(face: u32) {
    let point = query_particle;
    let base = params.static_faces_offset + face * 3u;
    let a = static_point(topology[base]);
    let b = static_point(topology[base + 1u]);
    let c = static_point(topology[base + 2u]);
    let zero = vec3<f32>(0.0);
    ccd_resolve(
        VERTEX_TRIANGLE,
        array<u32, 4>(point, STATIC_ID, STATIC_ID, STATIC_ID),
        array<vec3<f32>, 4>(dynamic_start(point), a, b, c),
        array<vec3<f32>, 4>(dynamic_end(point), a, b, c),
        array<vec3<f32>, 4>(dynamic_velocity(point), zero, zero, zero),
        vec4<f32>(dynamic_mass(point), 0.0, 0.0, 0.0),
        params.static_clearance,
    );
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.count || params.primitive_count == 0u) {
        return;
    }
    let point = params.first + global_id.x;
    if (dynamic_mass(point) == 0.0) {
        return;
    }
    query_particle = point;
    let start = dynamic_start(point);
    let end = dynamic_end(point);
    // Obstacle bounds are tight, so the query carries both radii.
    let reach = vec3<f32>(params.search_radius * 2.0);
    bvh_query_aabb(min(start, end) - reach, max(start, end) + reach);
}
"#;

/// Obstacle vertices against cloth faces with a free vertex. Visited from the
/// faces, so a dense body is never walked vertex by vertex.
const STATIC_VERTEX_FACE: &str = r#"
var<private> query_face: u32;

fn bvh_hit(obstacle_vertex: u32) {
    let a = topology[query_face * 3u];
    let b = topology[query_face * 3u + 1u];
    let c = topology[query_face * 3u + 2u];
    let p = static_point(obstacle_vertex);
    let zero = vec3<f32>(0.0);
    ccd_resolve(
        VERTEX_TRIANGLE,
        array<u32, 4>(STATIC_ID, a, b, c),
        array<vec3<f32>, 4>(p, dynamic_start(a), dynamic_start(b), dynamic_start(c)),
        array<vec3<f32>, 4>(p, dynamic_end(a), dynamic_end(b), dynamic_end(c)),
        array<vec3<f32>, 4>(zero, dynamic_velocity(a), dynamic_velocity(b), dynamic_velocity(c)),
        vec4<f32>(0.0, dynamic_mass(a), dynamic_mass(b), dynamic_mass(c)),
        params.static_clearance,
    );
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.count || params.primitive_count == 0u) {
        return;
    }
    let face = params.first + global_id.x;
    let a = topology[face * 3u];
    let b = topology[face * 3u + 1u];
    let c = topology[face * 3u + 2u];
    if (dynamic_mass(a) == 0.0 && dynamic_mass(b) == 0.0 && dynamic_mass(c) == 0.0) {
        return;
    }
    query_face = face;
    let lower = min(min(dynamic_start(a), dynamic_end(a)), min(min(dynamic_start(b), dynamic_end(b)), min(dynamic_start(c), dynamic_end(c))));
    let upper = max(max(dynamic_start(a), dynamic_end(a)), max(max(dynamic_start(b), dynamic_end(b)), max(dynamic_start(c), dynamic_end(c))));
    let reach = vec3<f32>(params.search_radius * 2.0);
    bvh_query_aabb(lower - reach, upper + reach);
}
"#;

/// Cloth edges against cloth edges, each pair once.
const EDGE_EDGE: &str = r#"
var<private> query_edge: u32;

fn bvh_hit(other: u32) {
    let edge = query_edge;
    let a = topology[params.edges_offset + edge * 2u];
    let b = topology[params.edges_offset + edge * 2u + 1u];
    let c = topology[params.edges_offset + other * 2u];
    let d = topology[params.edges_offset + other * 2u + 1u];
    // The lower edge owns a pair, unless the other edge is pinned and so
    // never runs a query of its own.
    let other_pinned = dynamic_mass(c) == 0.0 && dynamic_mass(d) == 0.0;
    if (other <= edge && !other_pinned) {
        return;
    }
    let ga = seam_group(a);
    let gb = seam_group(b);
    let gc = seam_group(c);
    let gd = seam_group(d);
    if (ga == gc || ga == gd || gb == gc || gb == gd) {
        return;
    }
    ccd_resolve(
        EDGE_EDGE,
        array<u32, 4>(a, b, c, d),
        array<vec3<f32>, 4>(dynamic_start(a), dynamic_start(b), dynamic_start(c), dynamic_start(d)),
        array<vec3<f32>, 4>(dynamic_end(a), dynamic_end(b), dynamic_end(c), dynamic_end(d)),
        array<vec3<f32>, 4>(dynamic_velocity(a), dynamic_velocity(b), dynamic_velocity(c), dynamic_velocity(d)),
        vec4<f32>(dynamic_mass(a), dynamic_mass(b), dynamic_mass(c), dynamic_mass(d)),
        params.thickness,
    );
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.count || params.primitive_count == 0u) {
        return;
    }
    let edge = params.first + global_id.x;
    let a = topology[params.edges_offset + edge * 2u];
    let b = topology[params.edges_offset + edge * 2u + 1u];
    if (dynamic_mass(a) == 0.0 && dynamic_mass(b) == 0.0) {
        return;
    }
    query_edge = edge;
    let lower = min(min(dynamic_start(a), dynamic_end(a)), min(dynamic_start(b), dynamic_end(b)));
    let upper = max(max(dynamic_start(a), dynamic_end(a)), max(dynamic_start(b), dynamic_end(b)));
    let reach = vec3<f32>(params.search_radius);
    bvh_query_aabb(lower - reach, upper + reach);
}
"#;

/// Cloth edges against obstacle edges.
const EDGE_STATIC_EDGE: &str = r#"
var<private> query_edge: u32;

fn bvh_hit(other: u32) {
    let edge = query_edge;
    let a = topology[params.edges_offset + edge * 2u];
    let b = topology[params.edges_offset + edge * 2u + 1u];
    let base = params.static_edges_offset + other * 2u;
    let c = static_point(topology[base]);
    let d = static_point(topology[base + 1u]);
    let zero = vec3<f32>(0.0);
    ccd_resolve(
        EDGE_EDGE,
        array<u32, 4>(a, b, STATIC_ID, STATIC_ID),
        array<vec3<f32>, 4>(dynamic_start(a), dynamic_start(b), c, d),
        array<vec3<f32>, 4>(dynamic_end(a), dynamic_end(b), c, d),
        array<vec3<f32>, 4>(dynamic_velocity(a), dynamic_velocity(b), zero, zero),
        vec4<f32>(dynamic_mass(a), dynamic_mass(b), 0.0, 0.0),
        params.static_clearance,
    );
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x >= params.count || params.primitive_count == 0u) {
        return;
    }
    let edge = params.first + global_id.x;
    let a = topology[params.edges_offset + edge * 2u];
    let b = topology[params.edges_offset + edge * 2u + 1u];
    if (dynamic_mass(a) == 0.0 && dynamic_mass(b) == 0.0) {
        return;
    }
    query_edge = edge;
    let lower = min(min(dynamic_start(a), dynamic_end(a)), min(dynamic_start(b), dynamic_end(b)));
    let upper = max(max(dynamic_start(a), dynamic_end(a)), max(dynamic_start(b), dynamic_end(b)));
    let reach = vec3<f32>(params.search_radius * 2.0);
    bvh_query_aabb(lower - reach, upper + reach);
}
"#;

/// Bounds of cloth faces or edges over the swept interval, grown by a margin.
pub const SWEPT_BOUNDS: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> motion: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> topology: array<u32>;
@group(0) @binding(3) var<storage, read_write> primitive_bounds: array<vec4<f32>>;

struct Params {
    count: u32,
    offset: u32,
    arity: u32,
    margin: f32,
};
@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.count) {
        return;
    }
    var lower = vec3<f32>(3.0e38);
    var upper = vec3<f32>(-3.0e38);
    for (var k = 0u; k < params.arity; k = k + 1u) {
        let particle = topology[params.offset + index * params.arity + k];
        let end = positions[particle].xyz;
        let start = motion[particle].xyz;
        lower = min(lower, min(start, end));
        upper = max(upper, max(start, end));
    }
    let margin = vec3<f32>(params.margin);
    primitive_bounds[index * 2u] = vec4<f32>(lower - margin, 0.0);
    primitive_bounds[index * 2u + 1u] = vec4<f32>(upper + margin, 0.0);
}
"#;

/// Bounds of obstacle faces or edges.
pub const STATIC_BOUNDS: &str = r#"
@group(0) @binding(0) var<storage, read> static_positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> topology: array<u32>;
@group(0) @binding(2) var<storage, read_write> primitive_bounds: array<vec4<f32>>;

struct Params {
    count: u32,
    offset: u32,
    arity: u32,
    margin: f32,
};
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.count) {
        return;
    }
    var lower = vec3<f32>(3.0e38);
    var upper = vec3<f32>(-3.0e38);
    for (var k = 0u; k < params.arity; k = k + 1u) {
        let point = static_positions[topology[params.offset + index * params.arity + k]].xyz;
        lower = min(lower, point);
        upper = max(upper, point);
    }
    let margin = vec3<f32>(params.margin);
    primitive_bounds[index * 2u] = vec4<f32>(lower - margin, 0.0);
    primitive_bounds[index * 2u + 1u] = vec4<f32>(upper + margin, 0.0);
}
"#;

/// Point bounds of obstacle vertices.
pub const VERTEX_BOUNDS: &str = r#"
@group(0) @binding(0) var<storage, read> static_positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> primitive_bounds: array<vec4<f32>>;

struct Params {
    count: u32,
    margin: f32,
    pad0: u32,
    pad1: u32,
};
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.count) {
        return;
    }
    let point = static_positions[index].xyz;
    let margin = vec3<f32>(params.margin);
    primitive_bounds[index * 2u] = vec4<f32>(point - margin, 0.0);
    primitive_bounds[index * 2u + 1u] = vec4<f32>(point + margin, 0.0);
}
"#;

/// Move each particle by its extreme corrections, then reset them. The contact
/// counter past the last particle is left alone.
pub const APPLY: &str = r#"
@group(0) @binding(0) var<storage, read_write> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> accumulator: array<atomic<i32>>;

struct Params {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(3) var<uniform> params: Params;

const POSITION_SCALE: f32 = 1048576.0;
const VELOCITY_SCALE: f32 = 65536.0;

// The largest positive plus the largest negative push per axis, reset to zero.
fn take(base: u32) -> vec3<i32> {
    let high = vec3<i32>(
        atomicExchange(&accumulator[base], 0),
        atomicExchange(&accumulator[base + 1u], 0),
        atomicExchange(&accumulator[base + 2u], 0),
    );
    let low = vec3<i32>(
        atomicExchange(&accumulator[base + 3u], 0),
        atomicExchange(&accumulator[base + 4u], 0),
        atomicExchange(&accumulator[base + 5u], 0),
    );
    return high + low;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.count) {
        return;
    }
    let base = index * 12u;
    let moved = take(base);
    if (any(moved != vec3<i32>(0))) {
        let delta = vec3<f32>(moved) / POSITION_SCALE;
        positions[index] = vec4<f32>(positions[index].xyz + delta, positions[index].w);
    }
    let pushed = take(base + 6u);
    if (any(pushed != vec3<i32>(0))) {
        let delta = vec3<f32>(pushed) / VELOCITY_SCALE;
        velocities[index] = vec4<f32>(velocities[index].xyz + delta, velocities[index].w);
    }
}
"#;

fn contact_kernel(bindings: &str, body: &str) -> String {
    let traversal = traversal_source(&TraversalConfig::default());
    [PARAMS, bindings, NARROW_PHASE, body, &traversal].join("\n")
}

pub fn vertex_face() -> String {
    contact_kernel(CLOTH_BINDINGS, VERTEX_FACE)
}

pub fn vertex_static_face() -> String {
    contact_kernel(OBSTACLE_BINDINGS, VERTEX_STATIC_FACE)
}

pub fn static_vertex_face() -> String {
    contact_kernel(OBSTACLE_BINDINGS, STATIC_VERTEX_FACE)
}

pub fn edge_edge() -> String {
    contact_kernel(CLOTH_BINDINGS, EDGE_EDGE)
}

pub fn edge_static_edge() -> String {
    contact_kernel(OBSTACLE_BINDINGS, EDGE_STATIC_EDGE)
}
