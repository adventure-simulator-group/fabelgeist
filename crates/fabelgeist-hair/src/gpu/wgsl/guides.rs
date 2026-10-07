//! The guides' kernels: [`crate::hairdo::Hairdo::guides`] on the GPU.
//!
//! Roots are drawn as the host draws them -- candidates hashed by index,
//! kept greedily in index order while none is nearer another than the
//! spacing, then a row along the hairline -- so the same roots come out.
//! The greedy pass is sequential by nature; one workgroup runs it, a chunk
//! of candidates at a time checked in parallel against the roots kept so
//! far, the few survivors of a chunk settled among themselves in order.
//! Then every guide is combed by an invocation of its own, drawing from a
//! [`crate::rng::Stream`] as the host does: its path into a buffer, then
//! resampled.

use super::common::{HEAD, INDEX, MATH, RANDOM, STRAND};
use super::groom::{GROWTH, PARTING};

fn kernel(bindings: &str, helpers: &[&str], body: &str) -> String {
    let mut source = String::from(bindings);
    for h in helpers {
        source.push_str(h);
    }
    source.push_str(INDEX);
    source.push_str(body);
    source
}

/// Scalp candidates for guide roots: candidate `i` at the shares
/// [`crate::surface::Surface::sample_at`] takes, hashed from its index.
/// Three words a candidate: position, normal, uv.
pub fn guide_candidates() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> tri_cdf: array<f32>;
@group(0) @binding(1) var<storage, read> tri_index: array<vec4<u32>>;
@group(0) @binding(2) var<storage, read> scalp_points: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> scalp_normals: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> scalp_uvs: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> candidates: array<vec4<f32>>;
struct Params {
    count: u32,
    triangles: u32,
    seed_share: u32,
    seed_u: u32,
    seed_v: u32,
    area: f32,
    pad0: u32,
    pad1: u32,
};
@group(0) @binding(6) var<uniform> params: Params;
"#,
        &[RANDOM, MATH],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    if (i >= params.count) {
        return;
    }
    let target_area = unit(i, params.seed_share) * params.area;
    // The first triangle whose running area reaches the target.
    var lo = 0u;
    var hi = params.triangles;
    while (lo < hi) {
        let mid = (lo + hi) / 2u;
        if (tri_cdf[mid] < target_area) {
            lo = mid + 1u;
        } else {
            hi = mid;
        }
    }
    let t = min(lo, params.triangles - 1u);
    var u = unit(i, params.seed_u);
    var v = unit(i, params.seed_v);
    if (u + v > 1.0) {
        u = 1.0 - u;
        v = 1.0 - v;
    }
    let w = vec3<f32>(1.0 - u - v, u, v);
    let tri = tri_index[t];
    let position = scalp_points[tri.x].xyz * w.x + scalp_points[tri.y].xyz * w.y + scalp_points[tri.z].xyz * w.z;
    let normal = normalize(scalp_normals[tri.x].xyz * w.x + scalp_normals[tri.y].xyz * w.y + scalp_normals[tri.z].xyz * w.z);
    let uv = scalp_uvs[tri.x].xy * w.x + scalp_uvs[tri.y].xy * w.y + scalp_uvs[tri.z].xy * w.z;
    candidates[i * 3u] = vec4<f32>(position, 0.0);
    candidates[i * 3u + 1u] = vec4<f32>(normal, 0.0);
    candidates[i * 3u + 2u] = vec4<f32>(uv, 0.0, 0.0);
}
"#,
    )
}

/// The closest point on a triangle (Ericson's region test).
const CLOSEST: &str = r#"
fn closest_on_triangle(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if (d1 <= 0.0 && d2 <= 0.0) {
        return a;
    }
    let bp = p - b;
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if (d3 >= 0.0 && d4 <= d3) {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if (vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0) {
        return a + ab * (d1 / (d1 - d3));
    }
    let cp = p - c;
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if (d6 >= 0.0 && d5 <= d6) {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if (vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0) {
        return a + ac * (d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if (va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0) {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let denom = 1.0 / (va + vb + vc);
    return a + ab * (vb * denom) + ac * (vc * denom);
}

// Barycentric weights of `p` in the triangle, as `surface::barycentric`.
fn barycentric3(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let d00 = dot(v0, v0);
    let d01 = dot(v0, v1);
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);
    let denom = d00 * d11 - d01 * d01;
    if (abs(denom) < 1e-20) {
        return vec3<f32>(1.0, 0.0, 0.0);
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    return vec3<f32>(1.0 - v - w, v, w);
}
"#;

/// Each point along the scalp's hairline, moved to the closest point of the
/// scalp within a centimetre: three words (position with `w` one if there
/// is one, normal, uv).
pub fn edge_closest() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> boundary: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> tri_index: array<vec4<u32>>;
@group(0) @binding(2) var<storage, read> scalp_points: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> scalp_normals: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> scalp_uvs: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> closest: array<vec4<f32>>;
struct Params {
    count: u32,
    triangles: u32,
    max_distance: f32,
    pad0: u32,
};
@group(0) @binding(6) var<uniform> params: Params;
"#,
        &[CLOSEST],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    if (i >= params.count) {
        return;
    }
    let p = boundary[i].xyz;
    var best = params.max_distance;
    var found = false;
    var best_point = p;
    var best_triangle = 0u;
    for (var t = 0u; t < params.triangles; t++) {
        let tri = tri_index[t];
        let a = scalp_points[tri.x].xyz;
        // A triangle is a few millimetres across: one whose corner is far
        // cannot be near.
        if (distance(p, a) > best + 0.03) {
            continue;
        }
        let q = closest_on_triangle(p, a, scalp_points[tri.y].xyz, scalp_points[tri.z].xyz);
        let d = distance(p, q);
        if (d <= best) {
            best = d;
            best_point = q;
            best_triangle = t;
            found = true;
        }
    }
    if (!found) {
        closest[i * 3u] = vec4<f32>(p, 0.0);
        return;
    }
    let tri = tri_index[best_triangle];
    let w = barycentric3(best_point, scalp_points[tri.x].xyz, scalp_points[tri.y].xyz, scalp_points[tri.z].xyz);
    let position = scalp_points[tri.x].xyz * w.x + scalp_points[tri.y].xyz * w.y + scalp_points[tri.z].xyz * w.z;
    let normal = normalize(scalp_normals[tri.x].xyz * w.x + scalp_normals[tri.y].xyz * w.y + scalp_normals[tri.z].xyz * w.z);
    let uv = scalp_uvs[tri.x].xy * w.x + scalp_uvs[tri.y].xy * w.y + scalp_uvs[tri.z].xy * w.z;
    closest[i * 3u] = vec4<f32>(position, 1.0);
    closest[i * 3u + 1u] = vec4<f32>(normal, 0.0);
    closest[i * 3u + 2u] = vec4<f32>(uv, 0.0, 0.0);
}
"#,
    )
}

/// The guides' roots, greedily in index order, one workgroup: candidates
/// kept while no kept root is nearer than `spacing`, up to `wanted`; then
/// the hairline's points, each moved onto the scalp, kept while no kept
/// hairline point is nearer than `edge_spacing` and no root nearer than
/// half the spacing. `counts` gets how many of each.
pub fn guide_select() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> candidates: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> boundary: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> closest: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> root_position: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> root_normal: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> root_uv: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read_write> counts: array<u32>;
struct Params {
    candidate_count: u32,
    wanted: u32,
    boundary_count: u32,
    pad0: u32,
    spacing: f32,
    edge_spacing: f32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(7) var<uniform> params: Params;

var<workgroup> kept: u32;
var<workgroup> survive: array<u32, 256>;
"#,
        &[],
        r#"
fn keep_root(slot: u32, position: vec3<f32>, normal: vec3<f32>, uv: vec2<f32>, from_edge: f32) {
    root_position[slot] = vec4<f32>(position, from_edge);
    root_normal[slot] = vec4<f32>(normal, 0.0);
    root_uv[slot] = vec4<f32>(uv, 0.0, 0.0);
}

// Whether `p` is nearer than `radius` to any kept root in `first..last`
// (by the hairline point it came from, for the row: `w` holds its index
// plus one).
fn near_kept(p: vec3<f32>, first: u32, last: u32, radius: f32, by_edge: bool) -> bool {
    let r2 = radius * radius;
    for (var k = first; k < last; k++) {
        var q = root_position[k].xyz;
        if (by_edge) {
            q = boundary[u32(root_position[k].w) - 1u].xyz;
        }
        let o = q - p;
        if (dot(o, o) < r2) {
            return true;
        }
    }
    return false;
}

@compute @workgroup_size(256)
fn main(@builtin(local_invocation_index) lid: u32) {
    if (lid == 0u) {
        kept = 0u;
    }
    // The roots.
    var chunk = 0u;
    loop {
        storageBarrier();
        let before = workgroupUniformLoad(&kept);
        if (before >= params.wanted || chunk >= params.candidate_count) {
            break;
        }
        let i = chunk + lid;
        var ok = 0u;
        if (i < params.candidate_count && !near_kept(candidates[i * 3u].xyz, 0u, before, params.spacing, false)) {
            ok = 1u;
        }
        survive[lid] = ok;
        workgroupBarrier();
        if (lid == 0u) {
            var n = before;
            for (var k = 0u; k < 256u; k++) {
                if (n >= params.wanted) {
                    break;
                }
                if (survive[k] == 0u) {
                    continue;
                }
                let c = (chunk + k) * 3u;
                let p = candidates[c].xyz;
                if (near_kept(p, before, n, params.spacing, false)) {
                    continue;
                }
                keep_root(n, p, candidates[c + 1u].xyz, candidates[c + 2u].xy, 0.0);
                n += 1u;
            }
            kept = n;
        }
        chunk += 256u;
    }
    storageBarrier();
    let roots = workgroupUniformLoad(&kept);
    // The row along the hairline, after them.
    chunk = 0u;
    loop {
        storageBarrier();
        let before = workgroupUniformLoad(&kept);
        if (chunk >= params.boundary_count) {
            break;
        }
        let i = chunk + lid;
        var ok = 0u;
        if (i < params.boundary_count && closest[i * 3u].w > 0.5) {
            let p = boundary[i].xyz;
            if (!near_kept(p, roots, before, params.edge_spacing, true)
                && !near_kept(p, 0u, roots, 0.5 * params.spacing, false)) {
                ok = 1u;
            }
        }
        survive[lid] = ok;
        workgroupBarrier();
        if (lid == 0u) {
            var n = before;
            for (var k = 0u; k < 256u; k++) {
                if (survive[k] == 0u) {
                    continue;
                }
                let e = chunk + k;
                if (near_kept(boundary[e].xyz, before, n, params.edge_spacing, true)) {
                    continue;
                }
                let c = e * 3u;
                keep_root(n, closest[c].xyz, closest[c + 1u].xyz, closest[c + 2u].xy, f32(e + 1u));
                n += 1u;
            }
            kept = n;
        }
        chunk += 256u;
    }
    storageBarrier();
    let all = workgroupUniformLoad(&kept);
    if (lid == 0u) {
        counts[0] = roots;
        counts[1] = all;
    }
}
"#,
    )
}

/// The hairdo, as the comb reads it; [`crate::gpu::guides::HairdoGpu`]
/// mirrors it field for field.
const HAIRDO: &str = r#"
struct Hairdo {
    guides: u32,
    points: u32,
    max_path: u32,
    key: u32,
    variation: f32,
    angle: f32,
    angle_spread: f32,
    direction_spread: f32,
    kink: f32,
    growth: u32,
    whorls: u32,
    fringe_falls: u32,
    center_x: f32,
    center_y: f32,
    center_z: f32,
    lanes: u32,
    w0_x: f32,
    w0_y: f32,
    w0_z: f32,
    w0_spiral: f32,
    w0_hand: f32,
    w0_reach: f32,
    w1_x: f32,
    w1_y: f32,
    w1_z: f32,
    w1_spiral: f32,
    w1_hand: f32,
    w1_reach: f32,
    top: f32,
    front: f32,
    sides: f32,
    back: f32,
    has_hemline: u32,
    hemline: f32,
    slope: f32,
    clip: f32,
    clipper: f32,
    fade: f32,
    strip: f32,
    has_fringe: u32,
    fringe_width: f32,
    fringe_depth: f32,
    fringe_drop: f32,
    dome: f32,
    flow: u32,
    flow_value: f32,
    part: f32,
    hold: f32,
    lift: f32,
    volume: f32,
    cling: f32,
    tuck: f32,
    hairline_front: f32,
    hairline_side: f32,
    hairline_back: f32,
    recession: f32,
    peak: f32,
    tied: u32,
    tie_x: f32,
    tie_y: f32,
    tie_z: f32,
    out_x: f32,
    out_y: f32,
    out_z: f32,
    axis_len: u32,
};
"#;

/// The head's hairline, from `hairdo` (or, replaced, any struct with the
/// same fields): `Head`'s queries.
pub(super) const HAIRLINE: &str = r#"
fn cranium_center() -> vec3<f32> {
    return head_shape[0].xyz;
}

fn azimuth(p: vec3<f32>) -> f32 {
    let q = p - cranium_center();
    return atan2(q.x, q.z);
}

fn around(front: f32, side: f32, back: f32, a: f32) -> f32 {
    let a1 = (front - back) * 0.5;
    let a0_plus_a2 = (front + back) * 0.5;
    let a0 = (a0_plus_a2 + side) * 0.5;
    let a2 = a0_plus_a2 - a0;
    return a0 + a1 * cos(a) + a2 * cos(2.0 * a);
}

fn hairline_height(a: f32) -> f32 {
    let t = (abs(a) - 0.62) / 0.3;
    let temples = exp(-(t * t));
    let m = a / 0.13;
    let middle = exp(-(m * m));
    return around(hairdo.hairline_front, hairdo.hairline_side, hairdo.hairline_back, a)
        + hairdo.recession * temples - hairdo.peak * middle;
}

fn above_hairline(p: vec3<f32>) -> f32 {
    return p.y - hairline_height(azimuth(p));
}

fn polar(p: vec3<f32>) -> f32 {
    let d = normalize(p - cranium_center());
    return acos(clamp(d.y, -1.0, 1.0));
}

fn elevation(p: vec3<f32>) -> f32 {
    let a = azimuth(p);
    let h = hairline_height(a);
    let c = cranium_center();
    let edge = c + vec3<f32>(sin(a) * 0.09, h - c.y, cos(a) * 0.09);
    let hairline = clamp(polar(edge), 0.1, 2.6);
    return clamp(1.0 - polar(p) / hairline, 0.0, 1.0);
}

"#;

/// The haircut's queries, from the hairdo: `Haircut`'s.
const CUT: &str = r#"
fn fringe_holds(root: vec3<f32>) -> bool {
    return hairdo.has_fringe != 0u && root.z > 0.1 - hairdo.fringe_depth && abs(root.x) < hairdo.fringe_width;
}

fn cut_clipped(root: vec3<f32>) -> bool {
    return (hairdo.clip > 0.0 && elevation(root) < hairdo.clip - 0.5 * hairdo.fade)
        || (hairdo.strip > 0.0 && abs(root.x) > hairdo.strip);
}

fn cut_length(root: vec3<f32>) -> f32 {
    if (fringe_holds(root)) {
        return 0.1 + hairdo.fringe_drop;
    }
    let e = elevation(root);
    let ring = around(hairdo.front, hairdo.sides, hairdo.back, azimuth(root));
    var cut = ring + (hairdo.top - ring) * smoothstep1(0.25, 0.85, e);
    if (hairdo.clip > 0.0) {
        let half = 0.5 * hairdo.fade + 1e-3;
        let t = smoothstep1(hairdo.clip - half, hairdo.clip + half, e);
        cut = hairdo.clipper + (cut - hairdo.clipper) * t;
    }
    if (hairdo.strip > 0.0) {
        let t = 1.0 - smoothstep1(hairdo.strip - 0.006, hairdo.strip, abs(root.x));
        cut = hairdo.clipper + (cut - hairdo.clipper) * t;
    }
    return max(cut, 0.0005);
}

// Whether the strand from `root` is cut where it is at `p`.
fn is_cut(root: vec3<f32>, p: vec3<f32>) -> bool {
    if (fringe_holds(root)) {
        return p.y < hairline_height(0.0) - hairdo.fringe_drop;
    }
    if (hairdo.has_hemline != 0u) {
        return p.y < hairdo.hemline - hairdo.slope * p.z;
    }
    return false;
}
"#;

/// A guide's own draws, as [`crate::rng::Stream`].
const STREAM: &str = r#"
var<private> stream_key: u32;
var<private> stream_counter: u32;

fn next_u32() -> u32 {
    let value = hash2(stream_key, stream_counter);
    stream_counter += 1u;
    return value;
}

fn next_f32() -> f32 {
    return f32(next_u32() >> 8u) / 16777216.0;
}

fn gaussian() -> f32 {
    let u = max(next_f32(), 1e-7);
    let v = next_f32();
    return sqrt(-2.0 * log(u)) * cos(TAU * v);
}
"#;

/// The tie: its axis from `tie_axis`, and where a strand is in its bundle.
const TIE: &str = r#"
fn tie_point() -> vec3<f32> {
    return vec3<f32>(hairdo.tie_x, hairdo.tie_y, hairdo.tie_z);
}

fn tie_out() -> vec3<f32> {
    return vec3<f32>(hairdo.out_x, hairdo.out_y, hairdo.out_z);
}

struct TieFrame {
    p: vec3<f32>,
    across: vec3<f32>,
    depth: vec3<f32>,
};

fn tie_frame(s: f32) -> TieFrame {
    let f = max(s / 0.002, 0.0);
    let last = max(hairdo.axis_len, 2u) - 2u;
    let i = min(u32(f), last);
    let t = clamp(f - f32(i), 0.0, 1.0);
    let a = tie_axis[i].xyz;
    let b = tie_axis[i + 1u].xyz;
    let d = safe_normalize(b - a, vec3<f32>(0.0, -1.0, 0.0));
    let across = safe_normalize(cross(d, tie_out()), vec3<f32>(1.0, 0.0, 0.0));
    return TieFrame(lerp3(a, b, t), across, cross(across, d));
}

fn bundled(along: f32, bundle: vec3<f32>, lane: f32, phase: f32) -> vec3<f32> {
    let kind = u32(hairdo.flow_value);
    if (kind == 1u) {
        // The bun: coiled round the tie, out from the head as it winds.
        let turns = along / 0.11;
        let a = phase + turns * TAU;
        let r = 0.018 + 0.01 * min(turns * 0.5, 1.0) + bundle.x;
        let out = tie_out();
        let u = safe_normalize(in_plane(vec3<f32>(0.0, 1.0, 0.0), out), vec3<f32>(0.0, 1.0, 0.0));
        let v = cross(out, u);
        return tie_point() + (u * cos(a) + v * sin(a)) * r
            + out * (0.004 + 0.012 * min(turns * 0.4, 1.0) + bundle.y);
    }
    let frame = tie_frame(along);
    if (kind == 2u) {
        // The braid: three lanes plaited.
        let phase_b = along / 0.035 * TAU + lane * TAU / 3.0;
        let width = 0.016 * (1.0 - 0.45 * smoothstep1(0.0, 0.45, along));
        let taper = 1.0 - smoothstep1(0.38, 0.5, along);
        return frame.p + frame.across * (sin(phase_b) * width * taper + bundle.x * 0.6)
            + frame.depth * (sin(2.0 * phase_b) * width * 0.35 * taper + bundle.y * 0.6);
    }
    // The tail.
    let spread = 1.0 + 0.6 * smoothstep1(0.1, 0.4, along);
    return frame.p + (frame.across * bundle.x + frame.depth * bundle.y) * spread;
}
"#;

/// Every guide combed from its root, as `Hairdo::comb` combs it: a
/// workgroup a guide, its lanes sampling the head's field together -- the
/// distance at a point and the six around it the gradient is taken from,
/// at once -- and running the rest of the comb in step, each drawing the
/// same numbers. The path is kept in `path` and resampled to `points` into
/// `positions`.
pub fn guide_comb() -> String {
    kernel(
        &format!(
            r#"
{HAIRDO}
@group(0) @binding(0) var<storage, read> root_position: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> root_normal: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> hairdo: Hairdo;
@group(0) @binding(3) var<storage, read> head_shape: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> tie_axis: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> path: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read_write> positions: array<vec4<f32>>;

var<workgroup> probes: array<f32, 8>;
"#
        ),
        &[
            RANDOM,
            MATH,
            HEAD,
            &GROWTH.replace("params.", "hairdo."),
            HAIRLINE,
            CUT,
            STREAM,
            TIE,
        ],
        r#"
const NATURAL: u32 = 0u;
const PART: u32 = 1u;
const SWEPT: u32 = 2u;
const BACK: u32 = 3u;
const FORWARD: u32 = 4u;
const RADIAL: u32 = 5u;
const DOWN: u32 = 6u;
const TIE: u32 = 7u;

// The head's field at `p`, the workgroup together: the outward normal (the
// gradient by central differences, as `Head::gradient`) and the distance.
// Every lane gets the same, so the comb stays in step.
fn field(p: vec3<f32>, lane: u32) -> vec4<f32> {
    workgroupBarrier();
    if (lane < 7u) {
        let e = 2e-4;
        var o = vec3<f32>(0.0);
        switch lane {
            case 0u: { o.x = e; }
            case 1u: { o.x = -e; }
            case 2u: { o.y = e; }
            case 3u: { o.y = -e; }
            case 4u: { o.z = e; }
            case 5u: { o.z = -e; }
            default: {}
        }
        probes[lane] = head_distance(p + o);
    }
    let s = workgroupUniformLoad(&probes);
    let g = vec3<f32>(s[0] - s[1], s[2] - s[3], s[4] - s[5]);
    return vec4<f32>(safe_normalize(g, vec3<f32>(0.0, 1.0, 0.0)), s[6]);
}

// What a strand is combed by, for the helpers below.
struct Comb {
    root: vec3<f32>,
    flow: u32,
    falls: f32,
    frontness: f32,
    elevation: f32,
    layer: f32,
    rise: f32,
    floor: f32,
    fringe_gap: f32,
};

// Skin below the hairline that faces forward, near enough to lie on: `f`
// is the field at `p`.
fn over_face(p: vec3<f32>, f: vec4<f32>) -> bool {
    return above_hairline(p) < 0.0 && f.w < 0.03 && f.z > 0.35;
}

fn layer_height(c: Comb, p: vec3<f32>, s: f32, f: vec4<f32>) -> f32 {
    let own = c.layer * smoothstep1(0.0, c.rise, s) + c.floor;
    if (over_face(p, f)) {
        return min(own, c.fringe_gap);
    }
    return own;
}

fn down_at(c: Comb, p: vec3<f32>) -> vec3<f32> {
    let outward = safe_normalize(p - cranium_center(), vec3<f32>(0.0, 1.0, 0.0));
    if (p.z > 0.0) {
        let side = select(-1.0, 1.0, c.root.x >= 0.0);
        return vec3<f32>(side, -0.2, -0.6);
    }
    return vec3<f32>(outward.x, -0.6, outward.z - 0.3);
}

fn flow_at(c: Comb, p: vec3<f32>, f: vec4<f32>) -> vec3<f32> {
    let down = vec3<f32>(0.0, -1.0, 0.0);
    let fall = safe_normalize(down_at(c, p), down) * c.falls;
    switch c.flow {
        case NATURAL: {
            return safe_normalize(growth_direction(p, f.xyz), down) * (1.0 - c.falls) + fall;
        }
        case PART: {
            let side = select(-1.0, 1.0, c.root.x - hairdo.flow_value >= 0.0);
            return vec3<f32>(side * (0.8 + 0.6 * c.frontness), -0.1, -0.35 * (1.0 - c.frontness));
        }
        case SWEPT: {
            var side = hairdo.flow_value;
            if ((c.root.x - hairdo.part) * side < 0.0) {
                side = -side;
            }
            return vec3<f32>(side, -0.05, 0.1 * (1.0 - c.elevation));
        }
        case BACK: {
            return vec3<f32>(0.0, -0.05, -1.0);
        }
        case FORWARD: {
            return safe_normalize(vec3<f32>(0.0, -0.3, 1.0), down) * (1.0 - c.falls) + fall;
        }
        case RADIAL: {
            return safe_normalize(p - cranium_center(), vec3<f32>(0.0, 1.0, 0.0));
        }
        case DOWN: {
            return down_at(c, p);
        }
        default: {
            if (hairdo.tied != 0u) {
                return tie_point() - p;
            }
            return down;
        }
    }
}

// Bend `d` towards `force` as a cantilever with `remaining` metres still to
// hang off it.
fn cantilever(d: vec3<f32>, force: vec3<f32>, remaining: f32, stiffness: f32, ds: f32) -> vec3<f32> {
    let axis = cross(d, force);
    let s = length(axis);
    if (s < 1e-6) {
        return d;
    }
    let angle = atan2(s, dot(d, force));
    let r = max(remaining, 0.0);
    let curvature = r * r / (2.0 * stiffness * stiffness * stiffness) * s;
    return rotate(d, axis / s, min(curvature * ds, angle));
}

// The braid's lane of a root: `Parting::Lanes`' group.
fn lane_of(root: vec3<f32>) -> f32 {
    if (hairdo.lanes == 0u) {
        return 0.0;
    }
    let a = atan2(root.x, root.z + 0.02);
    let t = (a + PI) / TAU;
    return f32(clamp(i32(t * f32(hairdo.lanes)), 0, i32(hairdo.lanes) - 1));
}

@compute @workgroup_size(8)
fn main(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_index) lane: u32) {
    let g = wid.x;
    if (g >= hairdo.guides) {
        return;
    }
    let root = root_position[g].xyz;
    let normal = root_normal[g].xyz;
    stream_key = hash2(hairdo.key ^ 0x0C0Bu, g);
    stream_counter = 0u;
    let elev = elevation(root);
    let az = azimuth(root);
    let frontness = smoothstep1(0.3, 0.95, cos(az)) * pow(1.0 - elev, 0.5);

    // The follicle: which way it points and at what angle.
    let spread = hairdo.direction_spread * clamp(gaussian(), -2.5, 2.5);
    let grows = rotate(growth_direction(root, normal), normal, spread);
    let follicle_angle = hairdo.angle + hairdo.angle_spread * clamp(gaussian(), -2.5, 2.5);

    var flow = hairdo.flow;
    var stiffness = hairdo.hold;
    var angle = follicle_angle + hairdo.lift;
    var strand_length = cut_length(root);
    var tuck = hairdo.tuck;
    let fringed = fringe_holds(root) && hairdo.fringe_falls != 0u;
    if (fringed) {
        flow = FORWARD;
        stiffness = 0.06;
        tuck = 0.0;
    } else if (cut_clipped(root)) {
        flow = NATURAL;
        angle = follicle_angle;
        stiffness = 1.0;
        tuck = 0.0;
    }
    angle = clamp(angle, 0.05, 1.5);
    strand_length *= 1.0 + hairdo.variation * 0.12 * clamp(gaussian(), -2.0, 2.0);
    let wx = gaussian();
    let wy = gaussian() * 0.3;
    let wz = gaussian();
    let wander = vec3<f32>(wx, wy, wz) * (0.12 * hairdo.variation);

    var c: Comb;
    c.root = root;
    c.flow = flow;
    c.frontness = frontness;
    c.elevation = elev;
    c.layer = select(hairdo.volume * pow(elev, 0.7), 0.0, flow == RADIAL);
    c.floor = 0.0008;
    c.rise = max(c.layer / max(tan(angle), 0.05), 0.01);
    c.fringe_gap = 0.005 + 0.004 * next_f32();
    c.falls = 0.0;
    if (!fringed && (flow == NATURAL || flow == FORWARD)) {
        c.falls = 0.85 * smoothstep1(0.12, 0.3, strand_length);
    }

    let at_root = field(root, lane);
    let first = rotate(
        safe_normalize(in_plane(flow_at(c, root, at_root) + wander, normal), grows),
        normal,
        spread,
    );
    let leaving = first * cos(angle) + normal * sin(angle);
    var p = root;
    var d = safe_normalize(leaving, normal);
    if (hairdo.dome > 0.0) {
        let centre = cranium_center() + vec3<f32>(0.0, 0.04, 0.012);
        let o = root - centre;
        let b = dot(d, o);
        let cc = dot(o, o) - hairdo.dome * hairdo.dome;
        let reach = -b + sqrt(max(b * b - cc, 0.0));
        strand_length = max(reach, 0.02) * (1.2 + 0.1 * clamp(gaussian(), -2.0, 2.0));
    }
    let ds = clamp(strand_length / 300.0, 0.0004, 0.002);
    let steps = min(u32(ceil(strand_length / ds)), hairdo.max_path - 1u);
    var tied_at = -1.0;
    let bx = gaussian();
    let by = gaussian();
    let bundle = vec3<f32>(bx, by, 0.0) * 0.004;
    let lane_in_braid = lane_of(root);
    let tie = hairdo.tied != 0u;
    var to_tie = 0.0;
    if (tie) {
        to_tie = distance(tie_point(), root);
    }
    let down = vec3<f32>(0.0, -1.0, 0.0);

    let base = g * hairdo.max_path;
    if (lane == 0u) {
        path[base] = vec4<f32>(root, 0.0);
    }
    var n = 1u;
    for (var k = 1u; k <= steps; k++) {
        let s = f32(k) * ds;
        let remaining = strand_length - s;

        // Past the tie: the tail, the coil or the braid.
        if (tie && tied_at >= 0.0) {
            p = bundled(s - tied_at, bundle, lane_in_braid, az);
            if (lane == 0u) {
                path[base + n] = vec4<f32>(p, 0.0);
            }
            n += 1u;
            if (is_cut(root, p)) {
                break;
            }
            continue;
        }

        let here = field(p, lane);
        let nrm = here.xyz;
        let gap = here.w - layer_height(c, p, s, here);
        // Hair over the forehead rests on it, however high it came from.
        let on_face = over_face(p, here);
        let resting = on_face && gap < 0.03;
        let supported = (gap < hairdo.cling || resting) && nrm.y > -0.3 && flow != RADIAL;

        if (flow == TIE && tie) {
            if (distance(tie_point(), p) < 0.02 || s > 1.6 * to_tie + 0.03) {
                tied_at = s;
                continue;
            }
            // Over the scalp to the tie, held there.
            let want = safe_normalize(in_plane(tie_point() - p, nrm), d);
            d = safe_normalize(lerp3(d, want, min(ds / 0.01, 1.0)), want);
        } else if (supported) {
            // Lying on the hair beneath: the comb, and gravity down the
            // slope as the hair still to come weighs.
            var comb_to = flow_at(c, p, here) + wander;
            if (on_face && fringed) {
                comb_to = down;
            }
            let combed = safe_normalize(in_plane(comb_to, nrm), d);
            let downhill = in_plane(down, nrm);
            var slide = smoothstep1(0.0, 4.0 * stiffness, remaining);
            if (!fringed) {
                let ahead = p + safe_normalize(downhill, d) * 0.02;
                // Only below the hairline is there a face to slide onto.
                if (above_hairline(ahead) < 0.0 && over_face(ahead, field(ahead, lane))) {
                    slide *= 0.05;
                }
            }
            var want = safe_normalize(combed * (1.0 - slide) + downhill * slide, combed);
            // Rising to its layer near the root, the strand keeps its angle.
            let rising = 1.0 - smoothstep1(0.0, c.rise, s);
            want = safe_normalize(want * (1.0 - rising) + leaving * rising, want);
            d = safe_normalize(lerp3(d, want, min(ds / 0.012, 1.0)), want);
        } else {
            // Free: a cantilever under its own weight.
            d = cantilever(d, down, remaining, stiffness, ds);
        }
        // Coily hair wanders: a random turn a step.
        if (hairdo.kink > 0.0) {
            let k0 = gaussian();
            let k1 = gaussian();
            let k2 = gaussian();
            let axis = safe_normalize(cross(d, vec3<f32>(k0, k1, k2)), orthogonal(d));
            d = rotate(d, axis, gaussian() * hairdo.kink * sqrt(ds));
            let outward = safe_normalize(p - cranium_center(), d);
            d = safe_normalize(d + outward * 0.05, d);
        }
        // The last few centimetres of a tucked cut turn in towards the neck.
        if (tuck > 0.0 && remaining < 0.05) {
            let neck = vec3<f32>(0.0, p.y, head_shape[10].z);
            let inward = safe_normalize(in_plane(neck - p, d), d);
            d = safe_normalize(d + inward * (tuck * ds / 0.05), d);
        }

        var next = p + d * ds;
        let there = field(next, lane);
        let floor_here = layer_height(c, next, s + ds, there);
        if (there.w < floor_here) {
            next += there.xyz * (floor_here - there.w);
            next = p + safe_normalize(next - p, d) * ds;
        } else if (supported && there.w > floor_here) {
            // Lying on the layer below: settle onto it where it drops away.
            next -= there.xyz * ((there.w - floor_here) * 0.15);
            next = p + safe_normalize(next - p, d) * ds;
        }
        d = safe_normalize(next - p, d);
        p = next;
        if (lane == 0u) {
            path[base + n] = vec4<f32>(p, 0.0);
        }
        n += 1u;
        if (is_cut(root, p)) {
            break;
        }
    }
    if (lane != 0u) {
        return;
    }

    // Resampled evenly by arc length, as `resample_into`.
    if (n < 2u) {
        path[base + 1u] = path[base];
        n = 2u;
    }
    var total = 0.0;
    for (var j = 1u; j < n; j++) {
        total += distance(path[base + j].xyz, path[base + j - 1u].xyz);
        path[base + j].w = total;
    }
    let points = hairdo.points;
    let out = g * points;
    var segment = 0u;
    for (var j = 0u; j < points; j++) {
        let target_length = total * f32(j) / f32(max(points - 1u, 1u));
        while (segment + 2u < n && path[base + segment + 1u].w < target_length) {
            segment += 1u;
        }
        let a = path[base + segment];
        let b = path[base + segment + 1u];
        let span = b.w - a.w;
        var t = 0.0;
        if (span > 1e-12) {
            t = clamp((target_length - a.w) / span, 0.0, 1.0);
        }
        positions[out + j] = vec4<f32>(lerp3(a.xyz, b.xyz, t), 0.0);
    }
}
"#,
    )
}

/// Each guide's attributes, as the host's guides have them: its root's
/// normal and UV, a stable ID, its parting group, the follicles' radius.
pub fn guide_attributes() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
@group(0) @binding(0) var<storage, read> root_position: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> root_normal: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> root_uv: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> strands: array<StrandGpu>;
@group(0) @binding(4) var<storage, read_write> roots: array<vec4<f32>>;
struct Params {{
    count: u32,
    lanes: u32,
    parting: u32,
    pad0: u32,
    plane_x: f32,
    plane_y: f32,
    plane_z: f32,
    plane_offset: f32,
    until: f32,
    fringe_front: f32,
    fringe_half_width: f32,
    radius: f32,
}};
@group(0) @binding(5) var<uniform> params: Params;
"#
        ),
        &[RANDOM, MATH, PARTING],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let g = item(id, groups);
    if (g >= params.count) {
        return;
    }
    let root = root_position[g].xyz;
    roots[g] = vec4<f32>(root, 0.0);
    strands[g] = StrandGpu(
        root_normal[g].xyz,
        hash2(g, 0x006D1DE5u),
        root_uv[g].xy,
        parting_group(root),
        params.radius,
    );
}
"#,
    )
}

/// The guides as interpolation reads them: each point with its arc length,
/// and each guide's root normal and parting group.
pub fn guide_pack() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> strands: array<StrandGpu>;
@group(0) @binding(2) var<storage, read_write> guide_points: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> guide_info: array<vec4<f32>>;
struct Params {{
    count: u32,
    points: u32,
    lanes: u32,
    parting: u32,
    plane_x: f32,
    plane_y: f32,
    plane_z: f32,
    plane_offset: f32,
    until: f32,
    fringe_front: f32,
    fringe_half_width: f32,
    pad0: u32,
}};
@group(0) @binding(4) var<uniform> params: Params;
"#
        ),
        &[MATH, PARTING],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let g = item(id, groups);
    if (g >= params.count) {
        return;
    }
    let base = g * params.points;
    var along = 0.0;
    var previous = positions[base].xyz;
    for (var j = 0u; j < params.points; j++) {
        let p = positions[base + j].xyz;
        along += distance(p, previous);
        previous = p;
        guide_points[base + j] = vec4<f32>(p, along);
    }
    let n = strands[g].normal;
    guide_info[g] = vec4<f32>(n, f32(parting_group(positions[base].xyz)));
}
"#,
    )
}

/// How far each scalp vertex is from the scalp's open edge, up to `edge`
/// (1 everywhere when there is no edge), beside its UV.
pub fn edge_distances() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> scalp_points: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> scalp_uvs: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> boundary: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> extra: array<vec4<f32>>;
struct Params {
    count: u32,
    boundary_count: u32,
    edge: f32,
    pad0: u32,
};
@group(0) @binding(4) var<uniform> params: Params;
"#,
        &[],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let v = item(id, groups);
    if (v >= params.count) {
        return;
    }
    var e = 1.0;
    if (params.edge > 0.0) {
        let p = scalp_points[v].xyz;
        e = params.edge;
        for (var b = 0u; b < params.boundary_count; b++) {
            e = min(e, distance(p, boundary[b].xyz));
        }
    }
    extra[v] = vec4<f32>(scalp_uvs[v].xy, e, 0.0);
}
"#,
    )
}
