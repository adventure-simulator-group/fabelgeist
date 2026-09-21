//! The plate armor surface, and every output triangle mapped onto it.
//!
//! One invocation per output triangle: it finds its instance, reads (or, for
//! the breastplate grid, evaluates) its source triangle, walks the midpoint
//! subdivision down to its own leaf, maps the corners onto the armor surface
//! and writes three unwelded vertices -- position, normal and texture
//! coordinate. A face-on triangle takes smooth normals from finite
//! differences of the surface and a wall its flat normal. A triangle the
//! mapping collapses is marked invalid rather than removed, so that every
//! triangle keeps its slot and the weld can compact them in order.

use super::plan::{GRID_COLUMNS, GRID_ROWS, INSTANCE_WORDS};
use super::wgsl;
use crate::{Armor, material::Metal};

/// Floats of an unwelded vertex: position, normal, texture coordinate.
pub(super) const VERTEX_FLOATS: usize = 8;

/// The bit of a triangle's info word, beside its part, that marks it as not
/// collapsed by the mapping.
pub(super) const VALID_TRIANGLE: u32 = 1 << 31;

/// The design as the surface kernel reads it.
pub(super) fn design(a: &Armor) -> [f32; 13] {
    [
        a.width,
        a.height,
        a.depth,
        a.waist,
        a.neck,
        a.arm_cut,
        a.thickness,
        a.ridge,
        a.ridge_sharpness,
        a.center_point,
        a.translation[0],
        a.translation[1],
        a.translation[2],
    ]
}

/// The emission kernel's source.
pub(super) fn emit_source() -> String {
    format!(
        "{BINDINGS}{math}{constants}{SURFACE}{EMIT}",
        math = wgsl::math(),
        constants = constants(),
    )
}

/// The constants the kernel shares with the planner.
fn constants() -> String {
    format!(
        r#"
const INSTANCE_WORDS: u32 = {instance_words}u;
const VERTEX_FLOATS: u32 = {vertex_floats}u;
const GRID_COLUMNS: f32 = {grid_columns:.1};
const GRID_ROWS: f32 = {grid_rows:.1};
const TILES_PER_METRE: f32 = {tiles_per_metre:?};
// How far across the half width the neckline curve of the grid reaches.
const NECKLINE_REACH: f32 = 0.65;
// Step of the finite differences that give a face its smooth normal.
const NORMAL_STEP: f32 = 0.00001;
// A mapped triangle with a smaller squared doubled area is dropped.
const COLLAPSED_AREA: f32 = 1e-16;
// A source face this close to the plate's own plane gets a smooth normal.
const FACE_ON: f32 = 0.99;
const VALID: u32 = {valid}u;
"#,
        instance_words = INSTANCE_WORDS,
        valid = VALID_TRIANGLE,
        vertex_floats = VERTEX_FLOATS,
        grid_columns = GRID_COLUMNS as f32,
        grid_rows = GRID_ROWS as f32,
        tiles_per_metre = Metal::TILES_PER_METRE,
    )
}

const BINDINGS: &str = r#"
@group(0) @binding(0) var<storage, read> design: array<f32>;
@group(0) @binding(1) var<storage, read> sources: array<f32>;
@group(0) @binding(2) var<storage, read> instances: array<u32>;
@group(0) @binding(3) var<storage, read_write> vertices: array<f32>;
@group(0) @binding(4) var<storage, read_write> triangle_info: array<u32>;

struct Params {
    count: u32,
    instance_count: u32,
    zero: u32,
    pad0: u32,
};
@group(0) @binding(5) var<uniform> params: Params;
"#;

/// The armor surface, and the breastplate grid's cell corners on it.
const SURFACE: &str = r#"
struct Armor {
    width: f32,
    height: f32,
    depth: f32,
    waist: f32,
    neck: f32,
    arm_cut: f32,
    thickness: f32,
    ridge: f32,
    ridge_sharpness: f32,
    center_point: f32,
    translation: vec3<f32>,
};

fn armor() -> Armor {
    return Armor(
        design[0], design[1], design[2], design[3], design[4], design[5], design[6],
        design[7], design[8], design[9], vec3<f32>(design[10], design[11], design[12]),
    );
}

// The breastplate's curved front, widening from the
// waist, with a centre ridge.
fn surface(a: Armor, x: f32, y: f32, z: f32, flare: f32) -> vec3<f32> {
    // Every intermediate is fenced, so that it rounds the same on any device.
    let t = clamp(host_div(y, a.height), 0.0, 1.0);
    let rise = host_mul(host_fence(1.0 - a.waist), host_sin(host_fence(t * FRAC_PI_2)));
    let width = host_fence(host_mul(a.width, host_fence(a.waist + rise)) + host_fence(flare * 2.0));
    let u = clamp(host_div(x, host_fence(a.width * 0.5)), -1.0, 1.0);
    let across = host_fence(1.0 - host_mul(u, u));
    let bulge = host_mul(a.depth, host_sqrt(max(across, 0.0)));
    let crest = host_mul(a.ridge, host_pow(host_fence(1.0 - abs(u)), a.ridge_sharpness));
    let ridge = host_mul(crest, host_sin(host_fence(PI * t)));
    return vec3<f32>(
        host_fence(host_fence(host_mul(u, width) * 0.5) + a.translation.x),
        host_fence(y + a.translation.y),
        host_fence(host_fence(host_fence(host_fence(bulge + ridge) + z) + a.translation.z) + flare * 0.5),
    );
}

// A breastplate grid corner, following the neckline and arm openings.
fn grid_point(a: Armor, corner: vec3<f32>) -> vec3<f32> {
    let u = host_fence(host_fence(host_div(corner.x, GRID_COLUMNS) * 2.0) - 1.0);
    let t = host_div(corner.y, GRID_ROWS);
    let reach = min(host_div(abs(u), NECKLINE_REACH), 1.0);
    let top = host_fence(a.height - host_mul(a.neck, host_fence(1.0 - host_mul(reach, reach))));
    let bottom = host_mul(-a.center_point, host_fence(1.0 - abs(u)));
    let t2 = host_mul(t, t);
    let cut = host_mul(a.arm_cut, host_mul(t2, t2));
    let xx = host_mul(u, host_fence(host_fence(a.width * 0.5) - cut));
    let y = host_fence(bottom + host_mul(host_fence(top - bottom), t));
    return vec3<f32>(xx, y, host_fence(corner.z * host_fence(a.thickness * 0.5)));
}

struct Placement {
    offset: vec3<f32>,
    flare: f32,
};

// A source point on the armor surface. Its result is fenced: the compiler
// could otherwise cancel terms shared by two nearby points, and a difference
// of two placed points would not round consistently.
fn place(a: Armor, at: Placement, v: vec3<f32>) -> vec3<f32> {
    let moved = host_fence3(v + at.offset);
    return host_fence3(surface(a, moved.x, moved.y, moved.z, at.flare));
}
"#;

/// One output triangle: its instance, source, subdivision leaf, mapping,
/// normals and texture coordinates.
const EMIT: &str = r#"
fn instance_word(instance: u32, word: u32) -> u32 {
    return instances[instance * INSTANCE_WORDS + word];
}

// The last instance whose outputs start at or before `triangle`.
fn find_instance(triangle: u32) -> u32 {
    var low = 0u;
    var high = params.instance_count - 1u;
    loop {
        if (low >= high) {
            break;
        }
        let middle = (low + high + 1u) / 2u;
        if (instance_word(middle, 3u) <= triangle) {
            low = middle;
        } else {
            high = middle - 1u;
        }
    }
    return low;
}

fn source_corner(triangle: u32, corner: u32) -> vec3<f32> {
    let base = triangle * 9u + corner * 3u;
    return vec3<f32>(sources[base], sources[base + 1u], sources[base + 2u]);
}

fn write_vertex(slot: u32, p: vec3<f32>, n: vec3<f32>, uv: vec2<f32>) {
    let base = slot * VERTEX_FLOATS;
    vertices[base] = p.x;
    vertices[base + 1u] = p.y;
    vertices[base + 2u] = p.z;
    vertices[base + 3u] = n.x;
    vertices[base + 4u] = n.y;
    vertices[base + 5u] = n.z;
    vertices[base + 6u] = uv.x;
    vertices[base + 7u] = uv.y;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let triangle = id.x;
    if (triangle >= params.count) {
        return;
    }
    let a = armor();
    let instance = find_instance(triangle);
    let levels = instance_word(instance, 2u);
    let local = triangle - instance_word(instance, 3u);
    let source_index = instance_word(instance, 0u) + (local >> (2u * levels));
    let part = instance_word(instance, 4u);
    let at = Placement(
        vec3<f32>(
            bitcast<f32>(instance_word(instance, 8u)),
            bitcast<f32>(instance_word(instance, 9u)),
            bitcast<f32>(instance_word(instance, 10u)),
        ),
        bitcast<f32>(instance_word(instance, 11u)),
    );
    var tri = array<vec3<f32>, 3>(
        source_corner(source_index, 0u),
        source_corner(source_index, 1u),
        source_corner(source_index, 2u),
    );
    if (instance_word(instance, 5u) != 0u) {
        for (var k = 0u; k < 3u; k = k + 1u) {
            tri[k] = grid_point(a, tri[k]);
        }
    }
    // Walk the midpoint subdivision down to this leaf, most significant level
    // first: corner a, corner b, corner c, then the middle triangle.
    for (var level = 0u; level < levels; level = level + 1u) {
        let digit = (local >> (2u * (levels - 1u - level))) & 3u;
        let p0 = tri[0];
        let p1 = tri[1];
        let p2 = tri[2];
        let ab = host_fence3((p0 + p1) * 0.5);
        let bc = host_fence3((p1 + p2) * 0.5);
        let ca = host_fence3((p2 + p0) * 0.5);
        if (digit == 0u) {
            tri = array<vec3<f32>, 3>(p0, ab, ca);
        } else if (digit == 1u) {
            tri = array<vec3<f32>, 3>(ab, p1, bc);
        } else if (digit == 2u) {
            tri = array<vec3<f32>, 3>(ca, bc, p2);
        } else {
            tri = array<vec3<f32>, 3>(ab, bc, ca);
        }
    }
    var p = array<vec3<f32>, 3>(place(a, at, tri[0]), place(a, at, tri[1]), place(a, at, tri[2]));
    let mapped = host_cross(p[1] - p[0], p[2] - p[0]);
    if (host_dot(mapped, mapped) < COLLAPSED_AREA) {
        // Dropped, but kept in place so the triangle can still be told apart.
        for (var k = 0u; k < 3u; k = k + 1u) {
            write_vertex(triangle * 3u + k, p[k], vec3<f32>(0.0), vec2<f32>(0.0));
        }
        triangle_info[triangle] = part;
        return;
    }
    let flat_normal = normalized(mapped);
    let source_normal = normalized(host_cross(tri[1] - tri[0], tri[2] - tri[0]));
    let face_on = abs(source_normal.z) > FACE_ON;
    for (var k = 0u; k < 3u; k = k + 1u) {
        let v = tri[k];
        var normal = flat_normal;
        var uv: vec2<f32>;
        if (face_on) {
            let ex = vec3<f32>(NORMAL_STEP, 0.0, 0.0);
            let ey = vec3<f32>(0.0, NORMAL_STEP, 0.0);
            let dx = place(a, at, v + ex) - place(a, at, v - ex);
            let dy = place(a, at, v + ey) - place(a, at, v - ey);
            normal = normalized(host_cross(dx, dy)) * sign(source_normal.z);
            uv = vec2<f32>(v.x * TILES_PER_METRE, v.y * TILES_PER_METRE);
        } else if (abs(source_normal.x) > abs(source_normal.y)) {
            uv = vec2<f32>(v.y * TILES_PER_METRE, v.z * TILES_PER_METRE);
        } else {
            uv = vec2<f32>(v.x * TILES_PER_METRE, v.z * TILES_PER_METRE);
        }
        write_vertex(triangle * 3u + k, p[k], normal, uv);
    }
    triangle_info[triangle] = part | VALID;
}
"#;
