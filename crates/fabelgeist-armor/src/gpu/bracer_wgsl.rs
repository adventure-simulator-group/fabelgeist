//! The bracer's kernels.
//!
//! Each rounds in a fixed order of operations with exact device arithmetic,
//! so the bracer is deterministic: sums fold in surface-vertex order, samples
//! keep their vertices sorted by index, and sorts are stable, so ties keep
//! their earlier order.

use super::anatomy::SURFACE_HEADER;
use super::bracer::{ALONG, STATUS_EMPTY_CONTOUR};
use super::wgsl;

/// Floats of the measured axis: axis and weld distance, then the two
/// directions angles around it are measured in, each padded to four.
pub(crate) const AXIS_WORDS: u32 = 12;
/// Words of a sample: vertex count, four surface vertices, four weights.
pub(crate) const SAMPLE_WORDS: u32 = 9;
/// Floats of a ring crossing: vertex count, two vertices, two weights, the
/// crossing, its angle.
pub(crate) const POINT_WORDS: u32 = 9;
/// Where the design's columns begin among its words: clearance, wall,
/// elbow and wrist flare, ridge, axial start and end, fluted, then the
/// eight flute words.
const DESIGN_COLUMNS: u32 = 16;

/// Everything the kernels share besides their bindings.
fn prelude() -> String {
    format!(
        r#"
{math}
const HEADER: u32 = {header}u;
const ALONG: u32 = {along}u;
const SAMPLE_WORDS: u32 = {sample_words}u;
const POINT_WORDS: u32 = {point_words}u;
const DESIGN_COLUMNS: u32 = {design_columns}u;
const EMPTY_CONTOUR: u32 = {empty_contour}u;

fn host_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {{
    return (a.x * b.x + a.y * b.y) + a.z * b.z;
}}

fn host_cross(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {{
    return vec3<f32>(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x);
}}

// The unit vector along `v`, in xyz; w is one when `v` was longer than `minimum`.
fn unit_above(v: vec3<f32>, minimum: f32) -> vec4<f32> {{
    let magnitude = sqrt(host_dot(v, v));
    if (magnitude > minimum) {{
        return vec4<f32>(v * (1.0 / magnitude), 1.0);
    }}
    return vec4<f32>(v, 0.0);
}}

fn pow6(x: f32) -> f32 {{
    let s = x * x;
    return s * (s * s);
}}
"#,
        math = wgsl::MATH,
        header = SURFACE_HEADER,
        along = ALONG,
        sample_words = SAMPLE_WORDS,
        point_words = POINT_WORDS,
        design_columns = DESIGN_COLUMNS,
        empty_contour = STATUS_EMPTY_CONTOUR,
    )
}

const PARAMS: &str = r#"
struct Params {
    count: u32,
    around: u32,
    faces_at: u32,
    record_body_normals: u32,
};
"#;

/// A kernel's full source: its bindings and entry after the prelude. A
/// kernel that binds `status` also gets `fail`.
pub(crate) fn source(entry: &str, binds_status: bool) -> String {
    let status = if binds_status { wgsl::STATUS } else { "" };
    format!("{PARAMS}\n{}\n{entry}\n{status}", prelude())
}

/// The design's relief and flute fan, from its words.
pub(crate) const DESIGN: &str = r#"
var<private> flute: array<f32, 8>;

fn load_flute() {
    for (var k = 0u; k < 8u; k = k + 1u) {
        flute[k] = design[8u + k];
    }
}

fn fluted() -> bool {
    return design[7] != 0.0;
}

// `BracerDesign::relief`: end flares, the centre ridge, and the flutes.
fn relief(u: f32, v: f32) -> f32 {
    let edge = design[2] * pow6(1.0 - v) + design[3] * pow6(v);
    let ridge = pow8(max(cos(TAU * (u - 0.5)), 0.0)) * pow2(sin(PI * v)) * design[4];
    var total = edge + ridge;
    if (fluted()) {
        total = total + flute_relief(u, 1.0 - v);
    }
    return total;
}
"#;

/// One invocation: the regression axis of the surface, its angular frame,
/// and the weld distance of its crossings.
pub(crate) const AXIS: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> surface: array<u32>;
@group(0) @binding(2) var<storage, read> axial: array<f32>;
@group(0) @binding(3) var<storage, read> joint_weights: array<f32>;
@group(0) @binding(4) var<storage, read_write> axis: array<f32>;
@group(0) @binding(5) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(6) var<uniform> params: Params;

fn point(i: u32) -> vec3<f32> {
    return vec3<f32>(positions[i * 3u], positions[i * 3u + 1u], positions[i * 3u + 2u]);
}

@compute @workgroup_size(1)
fn main() {
    let n = surface[0];
    if (n == 0u) {
        fail(STATUS_INVALID_SURFACE);
        return;
    }
    let infinity = MAX_FINITE;
    var axial_sum = 0.0;
    var position_sum = vec3<f32>(0.0);
    var lowest = vec3<f32>(infinity);
    var highest = vec3<f32>(-infinity);
    for (var s = 0u; s < n; s = s + 1u) {
        let body = surface[HEADER + s * 2u];
        let p = point(body);
        axial_sum = axial_sum + axial[body];
        position_sum = position_sum + p;
        lowest = min(lowest, p);
        highest = max(highest, p);
        var weight = 0.0;
        for (var k = 0u; k < 8u; k = k + 1u) {
            weight = weight + joint_weights[body * 8u + k];
        }
        if (abs(weight - 1.0) > 1e-4 || !is_finite(p.x) || !is_finite(p.y) || !is_finite(p.z)) {
            fail(STATUS_INVALID_SURFACE);
        }
    }
    let count = f32(n);
    let mean_axial = axial_sum / count;
    let mean_position = position_sum * (1.0 / count);
    var regression = vec3<f32>(0.0);
    for (var s = 0u; s < n; s = s + 1u) {
        let body = surface[HEADER + s * 2u];
        regression = regression + (point(body) - mean_position) * (axial[body] - mean_axial);
    }
    let direction = unit_above(regression, 1e-8);
    if (direction.w == 0.0) {
        fail(STATUS_DEGENERATE);
        return;
    }
    let d = direction.xyz;
    var reference = vec3<f32>(0.0, 1.0, 0.0);
    if (abs(d.x) < 0.8) {
        reference = vec3<f32>(1.0, 0.0, 0.0);
    }
    let u = unit_above(host_cross(d, reference), 1e-8);
    if (u.w == 0.0) {
        fail(STATUS_DEGENERATE);
        return;
    }
    let v = host_cross(d, u.xyz);
    let extent = highest - lowest;
    let weld = max(sqrt(host_dot(extent, extent)) * 1e-5, 1e-7);
    var words = array<f32, 12>(d.x, d.y, d.z, weld, u.x, u.y, u.z, 0.0, v.x, v.y, v.z, 0.0);
    for (var k = 0u; k < 12u; k = k + 1u) {
        axis[k] = words[k];
    }
}
"#;

/// One invocation per bracer vertex: its sample displaced off the bound
/// body realization, outer wall first.
pub(crate) fn displace() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> body_positions: array<f32>;
@group(0) @binding(1) var<storage, read> body_normals_in: array<f32>;
@group(0) @binding(2) var<storage, read> surface: array<u32>;
@group(0) @binding(3) var<storage, read> design: array<f32>;
@group(0) @binding(4) var<storage, read> samples: array<u32>;
@group(0) @binding(5) var<storage, read_write> positions: array<f32>;
@group(0) @binding(6) var<storage, read_write> body_normals: array<f32>;
@group(0) @binding(7) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(8) var<uniform> params: Params;
{fluting}
{design}
{positions}
{body_normals}
{body}
"#,
        fluting = super::chart::FLUTING,
        design = DESIGN,
        positions = wgsl::points("positions"),
        body_normals = wgsl::points("body_normals"),
        body = DISPLACE_BODY,
    )
}

const DISPLACE_BODY: &str = r#"
fn body_point(i: u32) -> vec3<f32> {
    return vec3<f32>(body_positions[i * 3u], body_positions[i * 3u + 1u], body_positions[i * 3u + 2u]);
}

fn body_normal(i: u32) -> vec3<f32> {
    return vec3<f32>(body_normals_in[i * 3u], body_normals_in[i * 3u + 1u], body_normals_in[i * 3u + 2u]);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    load_flute();
    let sample_count = params.count / 2u;
    let inner = i >= sample_count;
    let s = i % sample_count;
    let column = s % params.around;
    let ring = s / params.around;
    var offset = design[0] + design[1];
    if (inner) {
        offset = design[0];
    }
    offset = offset + relief(design[DESIGN_COLUMNS + column], f32(ring) / f32(ALONG));
    let at = s * SAMPLE_WORDS;
    let n = samples[at];
    var position = vec3<f32>(0.0);
    var normal = vec3<f32>(0.0);
    for (var e = 0u; e < n; e = e + 1u) {
        let body = surface[HEADER + samples[at + 1u + e] * 2u];
        let weight = bitcast<f32>(samples[at + 5u + e]);
        position = position + body_point(body) * weight;
        normal = normal + body_normal(body) * weight;
    }
    let unit = unit_above(normal, 1e-8);
    if (unit.w == 0.0) {
        fail(STATUS_DEGENERATE);
        return;
    }
    positions_set(i, position + unit.xyz * offset);
    if (params.record_body_normals != 0u && !inner) {
        body_normals_set(s, unit.xyz);
    }
}
"#;

/// One invocation per quad of the bracer's rings, then per quad of its end
/// walls: each triangle is flipped when its normal opposes its reference --
/// the blended skin normal on the outer wall, its reverse on the inner wall,
/// and the axis pointing out of the bracer on an end wall.
pub(crate) const INDICES: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> body_normals: array<f32>;
@group(0) @binding(2) var<storage, read> axis: array<f32>;
@group(0) @binding(3) var<storage, read_write> indices: array<u32>;
@group(0) @binding(4) var<uniform> params: Params;

fn point(i: u32) -> vec3<f32> {
    return vec3<f32>(positions[i * 3u], positions[i * 3u + 1u], positions[i * 3u + 2u]);
}

fn body_normal(i: u32) -> vec3<f32> {
    return vec3<f32>(body_normals[i * 3u], body_normals[i * 3u + 1u], body_normals[i * 3u + 2u]);
}

fn vertex(ring: u32, segment: u32) -> u32 {
    return ring * params.around + segment % params.around;
}

fn push(at: u32, face: vec3<u32>, reference: vec3<f32>) {
    let a = point(face.x);
    let normal = host_cross(point(face.y) - a, point(face.z) - a);
    var wound = face;
    if (host_dot(normal, reference) < 0.0) {
        wound = face.xzy;
    }
    indices[at] = wound.x;
    indices[at + 1u] = wound.y;
    indices[at + 2u] = wound.z;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let t = id.x;
    if (t >= params.count) {
        return;
    }
    let layer = (ALONG + 1u) * params.around;
    let quads = ALONG * params.around;
    if (t < quads) {
        let ring = t / params.around;
        let segment = t % params.around;
        let a = vertex(ring, segment);
        let b = vertex(ring, segment + 1u);
        let c = vertex(ring + 1u, segment + 1u);
        let d = vertex(ring + 1u, segment);
        let blended = unit_above(body_normal(a) + body_normal(c), 1e-8);
        var reference = body_normal(a);
        if (blended.w != 0.0) {
            reference = blended.xyz;
        }
        let at = t * 12u;
        push(at, vec3<u32>(a, d, c), reference);
        push(at + 3u, vec3<u32>(a, c, b), reference);
        push(at + 6u, vec3<u32>(a + layer, c + layer, d + layer), -reference);
        push(at + 9u, vec3<u32>(a + layer, b + layer, c + layer), -reference);
        return;
    }
    let wall = t - quads;
    let end = wall / params.around;
    let segment = wall % params.around;
    var ring = 0u;
    var reference = -vec3<f32>(axis[0], axis[1], axis[2]);
    if (end == 1u) {
        ring = ALONG;
        reference = vec3<f32>(axis[0], axis[1], axis[2]);
    }
    let a = vertex(ring, segment);
    let b = vertex(ring, segment + 1u);
    let at = quads * 12u + wall * 6u;
    push(at, vec3<u32>(a, b, b + layer), reference);
    push(at + 3u, vec3<u32>(a, b + layer, a + layer), reference);
}
"#;

/// One invocation per sample: its atlas coordinate and its eight strongest
/// joints, for both walls.
pub(crate) const SKIN: &str = r#"
@group(0) @binding(0) var<storage, read> surface: array<u32>;
@group(0) @binding(1) var<storage, read> samples: array<u32>;
@group(0) @binding(2) var<storage, read> atlas: array<f32>;
@group(0) @binding(3) var<storage, read> body_joint_indices: array<u32>;
@group(0) @binding(4) var<storage, read> body_joint_weights: array<f32>;
@group(0) @binding(5) var<storage, read_write> texcoords: array<f32>;
@group(0) @binding(6) var<storage, read_write> joint_indices: array<u32>;
@group(0) @binding(7) var<storage, read_write> joint_weights: array<f32>;
@group(0) @binding(8) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let s = id.x;
    if (s >= params.count) {
        return;
    }
    let at = s * SAMPLE_WORDS;
    let n = samples[at];
    var uv = vec2<f32>(0.0);
    var joints: array<u32, 32>;
    var weights: array<f32, 32>;
    var m = 0u;
    for (var e = 0u; e < n; e = e + 1u) {
        let vertex = samples[at + 1u + e];
        let share = bitcast<f32>(samples[at + 5u + e]);
        let coordinate = surface[HEADER + vertex * 2u + 1u];
        uv.x = uv.x + atlas[coordinate * 2u] * share;
        uv.y = uv.y + atlas[coordinate * 2u + 1u] * share;
        let body = surface[HEADER + vertex * 2u];
        for (var k = 0u; k < 8u; k = k + 1u) {
            let joint = body_joint_indices[body * 8u + k];
            let weight = body_joint_weights[body * 8u + k] * share;
            var found = false;
            for (var i = 0u; i < m; i = i + 1u) {
                if (joints[i] == joint) {
                    weights[i] = weights[i] + weight;
                    found = true;
                }
            }
            if (!found) {
                joints[m] = joint;
                weights[m] = weight;
                m = m + 1u;
            }
        }
    }
    // The eight strongest, strongest first, ties to the lower joint.
    let kept = min(m, 8u);
    for (var slot = 0u; slot < kept; slot = slot + 1u) {
        var best = slot;
        for (var i = slot + 1u; i < m; i = i + 1u) {
            if (weights[i] > weights[best] || (weights[i] == weights[best] && joints[i] < joints[best])) {
                best = i;
            }
        }
        let joint = joints[best];
        joints[best] = joints[slot];
        joints[slot] = joint;
        let weight = weights[best];
        weights[best] = weights[slot];
        weights[slot] = weight;
    }
    var total = 0.0;
    for (var slot = 0u; slot < kept; slot = slot + 1u) {
        total = total + weights[slot];
    }
    for (var wall = 0u; wall < 2u; wall = wall + 1u) {
        let vertex = s + wall * params.count;
        texcoords[vertex * 2u] = uv.x;
        texcoords[vertex * 2u + 1u] = uv.y;
        for (var slot = 0u; slot < 8u; slot = slot + 1u) {
            var joint = 0u;
            var weight = 0.0;
            if (slot < kept) {
                joint = joints[slot];
                weight = weights[slot] / total;
            }
            joint_indices[vertex * 8u + slot] = joint;
            joint_weights[vertex * 8u + slot] = weight;
        }
    }
}
"#;
