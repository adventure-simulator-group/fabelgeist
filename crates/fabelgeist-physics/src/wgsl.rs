//! The collision kernels.
//!
//! Both of them are XPBD substep hooks: they run after the prediction and
//! before the material constraints, and they resolve by moving positions. A
//! position moved here becomes a velocity change for free when the substep
//! finalises, which is why there is no impulse arithmetic anywhere.

/// Signed distance to each analytic shape, and the outward direction. Mirrors
/// `Collider::signed_distance` on the host.
use fabelgeist_gpu::prelude::ShaderSource;
pub const SHAPES: &str = r#"
struct Collider {
    data0: vec4<f32>,
    data1: vec4<f32>,
    data2: vec4<f32>,
    // x = kind (bitcast u32), y = friction, z = thickness
    settings: vec4<f32>,
};

struct Surface {
    distance: f32,
    // Outward, away from the solid.
    normal: vec3<f32>,
};

fn quaternion_rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let u = q.xyz;
    return u * (2.0 * dot(u, v)) + v * (q.w * q.w - dot(u, u)) + cross(u, v) * (2.0 * q.w);
}

fn quaternion_conjugate(q: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(-q.xyz, q.w);
}

// A direction to fall back on when the point sits exactly on the medial axis
// and there is no well-defined outward. Any fixed choice will do; it only has
// to be finite.
const DEGENERATE_NORMAL = vec3<f32>(0.0, 1.0, 0.0);

fn collider_surface(collider: Collider, point: vec3<f32>) -> Surface {
    let kind = bitcast<u32>(collider.settings.x);

    if (kind == 0u) {
        // Plane.
        return Surface(dot(point, collider.data0.xyz) - collider.data0.w, collider.data0.xyz);
    }

    if (kind == 1u) {
        // Sphere.
        let delta = point - collider.data0.xyz;
        let length_ = length(delta);
        var normal = DEGENERATE_NORMAL;
        if (length_ > 1e-9) {
            normal = delta / length_;
        }
        return Surface(length_ - collider.data0.w, normal);
    }

    if (kind == 2u) {
        // Capsule: the segment a-b thickened by a radius.
        let a = collider.data0.xyz;
        let b = collider.data1.xyz;
        let axis = b - a;
        let length_squared = dot(axis, axis);
        var t = 0.0;
        if (length_squared > 1e-12) {
            t = clamp(dot(point - a, axis) / length_squared, 0.0, 1.0);
        }
        let delta = point - (a + axis * t);
        let length_ = length(delta);
        var normal = DEGENERATE_NORMAL;
        if (length_ > 1e-9) {
            normal = delta / length_;
        }
        return Surface(length_ - collider.data0.w, normal);
    }

    // Box.
    let rotation = collider.data2;
    let local = quaternion_rotate(quaternion_conjugate(rotation), point - collider.data0.xyz);
    let q = abs(local) - collider.data1.xyz;
    let outside = max(q, vec3<f32>(0.0));
    let outside_length = length(outside);
    let inside = min(max(q.x, max(q.y, q.z)), 0.0);

    var local_normal: vec3<f32>;
    if (outside_length > 1e-9) {
        local_normal = (outside / outside_length) * sign(local);
    } else {
        // Inside: the nearest face is the least negative component.
        if (q.x >= q.y && q.x >= q.z) {
            local_normal = vec3<f32>(sign(local.x), 0.0, 0.0);
        } else if (q.y >= q.z) {
            local_normal = vec3<f32>(0.0, sign(local.y), 0.0);
        } else {
            local_normal = vec3<f32>(0.0, 0.0, sign(local.z));
        }
    }
    return Surface(outside_length + inside, quaternion_rotate(rotation, local_normal));
}
"#;

/// The response shared by both kernels: push out along the normal, then apply
/// Coulomb friction against the motion made so far this substep.
///
/// Friction in XPBD is positional too. The tangential part of the particle's
/// own movement since the substep began is what friction opposes, and the
/// Coulomb limit scales with the penetration depth -- which is standing in for
/// the normal force, exactly as it does in the rigid-body formulation.
pub const RESPONSE: &str = r#"
fn collision_response(
    position: vec3<f32>,
    previous_position: vec3<f32>,
    normal: vec3<f32>,
    penetration: f32,
    friction: f32,
) -> vec3<f32> {
    if (penetration <= 0.0) {
        return vec3<f32>(0.0);
    }

    var correction = normal * penetration;

    if (friction > 0.0) {
        // Where the particle would sit once pushed out, against where it was
        // when the substep started.
        let travelled = (position + correction) - previous_position;
        let tangential = travelled - normal * dot(travelled, normal);
        let magnitude = length(tangential);
        if (magnitude > 1e-9) {
            // Static below the limit -- cancel the slide outright; dynamic
            // above it -- cancel only as much as the limit allows.
            let limit = friction * penetration;
            let scale = min(limit / magnitude, 1.0);
            correction = correction - tangential * scale;
        }
    }

    return correction;
}
"#;

/// Resolve every particle against every analytic collider.
///
/// One thread per particle, looping over the colliders. There is no broad
/// phase because there is nothing to gain from one: the list is a handful of
/// shapes, and reading it is cheaper than culling it.
pub fn analytic_source() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read_write> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> previous: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> colliders: array<Collider>;

struct Params {{
    count: u32,
    collider_count: u32,
    particle_radius: f32,
    pad: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
{SHAPES}
{RESPONSE}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let index = global_id.x;
    if (index >= params.count) {{
        return;
    }}

    let entry = positions[index];
    if (entry.w == 0.0) {{
        return;
    }}

    var position = entry.xyz;
    let previous_position = previous[index].xyz;

    for (var i = 0u; i < params.collider_count; i = i + 1u) {{
        let collider = colliders[i];
        let surface = collider_surface(collider, position);
        let separation = collider.settings.z + params.particle_radius;
        let penetration = separation - surface.distance;
        position = position + collision_response(
            position,
            previous_position,
            surface.normal,
            penetration,
            collider.settings.y,
        );
    }}

    positions[index] = vec4<f32>(position, entry.w);
}}
"#
    ))
}

/// Resolve every particle against a triangle mesh, through the mesh's own BVH.
///
/// `traversal` is the source generated by `fabelgeist_bvh::gpu::traversal_source`,
/// which calls back into `bvh_hit` for every candidate triangle. WGSL has no
/// closures, so the callback keeps its result in module-scope `var`s -- the
/// kernel is one particle per thread, so there is nothing to share them with.
pub fn mesh_source(traversal: &str) -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read_write> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> previous: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> mesh_positions: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> mesh_triangles: array<u32>;
@group(0) @binding(4) var<storage, read> bvh_nodes: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> bvh_right_children: array<u32>;
@group(0) @binding(6) var<storage, read> bvh_indices: array<u32>;

struct Params {{
    count: u32,
    thickness: f32,
    friction: f32,
    // Widens the search beyond the shell. Zero for a normal substep, where the
    // swept box is enough; large for the recovery pass that has to find the
    // surface from deep inside the mesh.
    search_radius: f32,
}};
@group(0) @binding(7) var<uniform> params: Params;
{RESPONSE}

// The query, and the best candidate so far. One particle per thread, so these
// stand in for the closure the traversal would otherwise take.
var<private> query_point: vec3<f32>;
var<private> best_distance_squared: f32;
var<private> best_point: vec3<f32>;
var<private> best_normal: vec3<f32>;
var<private> found: bool;

fn triangle_closest_point(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec3<f32> {{
    // Ericson, Real-Time Collision Detection 5.1.5: the three vertex regions
    // and the three edge regions come first, so a point outside the triangle's
    // prism gets the right answer rather than a projection onto the plane.
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;

    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if (d1 <= 0.0 && d2 <= 0.0) {{ return a; }}

    let bp = p - b;
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if (d3 >= 0.0 && d4 <= d3) {{ return b; }}

    let vc = d1 * d4 - d3 * d2;
    if (vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0) {{
        let denominator = d1 - d3;
        if (abs(denominator) < 1e-20) {{ return a; }}
        return a + ab * (d1 / denominator);
    }}

    let cp = p - c;
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if (d6 >= 0.0 && d5 <= d6) {{ return c; }}

    let vb = d5 * d2 - d1 * d6;
    if (vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0) {{
        let denominator = d2 - d6;
        if (abs(denominator) < 1e-20) {{ return a; }}
        return a + ac * (d2 / denominator);
    }}

    let va = d3 * d6 - d5 * d4;
    if (va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0) {{
        let denominator = (d4 - d3) + (d5 - d6);
        if (abs(denominator) < 1e-20) {{ return b; }}
        return b + (c - b) * ((d4 - d3) / denominator);
    }}

    let denominator = va + vb + vc;
    if (abs(denominator) < 1e-20) {{ return a; }}
    return a + ab * (vb / denominator) + ac * (vc / denominator);
}}

fn bvh_hit(triangle: u32) {{
    let ia = mesh_triangles[triangle * 3u];
    let ib = mesh_triangles[triangle * 3u + 1u];
    let ic = mesh_triangles[triangle * 3u + 2u];
    let a = mesh_positions[ia].xyz;
    let b = mesh_positions[ib].xyz;
    let c = mesh_positions[ic].xyz;

    let closest = triangle_closest_point(query_point, a, b, c);
    let delta = query_point - closest;
    let distance_squared = dot(delta, delta);
    if (found && distance_squared >= best_distance_squared) {{
        return;
    }}

    found = true;
    best_distance_squared = distance_squared;
    best_point = closest;

    let face_normal = cross(b - a, c - a);
    let face_length = length(face_normal);
    if (face_length > 1e-20) {{
        best_normal = face_normal / face_length;
    }} else {{
        best_normal = vec3<f32>(0.0, 1.0, 0.0);
    }}
}}
{traversal}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {{
    let index = global_id.x;
    if (index >= params.count) {{
        return;
    }}

    let entry = positions[index];
    if (entry.w == 0.0) {{
        return;
    }}

    let position = entry.xyz;
    let previous_position = previous[index].xyz;
    query_point = position;
    found = false;
    best_distance_squared = 0.0;

    // The box covers where the particle was as well as where it is, grown by
    // the shell. Querying only around the current position would miss a
    // particle that crossed a thin surface within one substep -- it would be
    // out the other side, past the shell, and look perfectly free.
    let margin = vec3<f32>(max(params.thickness, params.search_radius));
    let lower = min(position, previous_position) - margin;
    let upper = max(position, previous_position) + margin;
    bvh_query_aabb(lower, upper);
    if (!found) {{
        return;
    }}

    let delta = position - best_point;
    let distance = sqrt(best_distance_squared);
    // Which side of the surface the particle is on. The face normal decides,
    // rather than the direction to the closest point, because that direction
    // says nothing at all once the particle is exactly on the surface.
    let side = dot(delta, best_normal);

    var normal: vec3<f32>;
    var penetration: f32;
    if (side < 0.0) {{
        // Already through the surface: come back out the way the face faces,
        // by the whole depth plus the shell.
        normal = best_normal;
        penetration = distance + params.thickness;
    }} else {{
        if (distance >= params.thickness) {{
            return;
        }}
        // Outside but inside the shell. Push along the direction to the
        // particle where that is meaningful, and along the face otherwise.
        if (distance > 1e-7) {{
            normal = delta / distance;
        }} else {{
            normal = best_normal;
        }}
        penetration = params.thickness - distance;
    }}

    let correction = collision_response(
        position,
        previous_position,
        normal,
        penetration,
        params.friction,
    );
    positions[index] = vec4<f32>(position + correction, entry.w);
}}
"#
    ))
}

/// Triangle bounds for the mesh BVH, expanded by the collision shell so that a
/// query for a point in the shell does not have to expand its own box.
pub const TRIANGLE_BOUNDS: &str = r#"
@group(0) @binding(0) var<storage, read> mesh_positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> mesh_triangles: array<u32>;
@group(0) @binding(2) var<storage, read_write> primitive_bounds: array<vec4<f32>>;

struct Params {
    count: u32,
    margin: f32,
    pad0: u32,
    pad1: u32,
};
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.count) {
        return;
    }
    let a = mesh_positions[mesh_triangles[index * 3u]].xyz;
    let b = mesh_positions[mesh_triangles[index * 3u + 1u]].xyz;
    let c = mesh_positions[mesh_triangles[index * 3u + 2u]].xyz;

    let margin = vec3<f32>(params.margin);
    primitive_bounds[index * 2u] = vec4<f32>(min(a, min(b, c)) - margin, 0.0);
    primitive_bounds[index * 2u + 1u] = vec4<f32>(max(a, max(b, c)) + margin, 0.0);
}
"#;
