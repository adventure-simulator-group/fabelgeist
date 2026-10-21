//! The grooming kernels: every operation of a groom's stack after
//! interpolation, one invocation a strand (or a point), reading one
//! position buffer (`src`) and writing the other (`dst`).
//!
//! Each mirrors the host operation of the same name in [`crate::ops`]; see
//! there for what the arithmetic means.

use super::common::{HEAD, INDEX, MATH, NOISE, RANDOM, STRAND, WALK};

fn kernel(bindings: &str, helpers: &[&str], body: &str) -> String {
    let mut source = String::from(bindings);
    for h in helpers {
        source.push_str(h);
    }
    source.push_str(INDEX);
    source.push_str(body);
    source
}

/// `GrowthField::direction`, from the parameters `growth`, `whorls`,
/// `center_*`, `w0_*` and `w1_*`.
pub(super) const GROWTH: &str = r#"
fn whorl_direction(point: vec3<f32>, spiral: f32, hand: f32, reach: f32, p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let offset = p - point;
    let away_raw = in_plane(offset, n);
    if (dot(away_raw, away_raw) < 1e-10) {
        return safe_normalize(in_plane(vec3<f32>(0.0, 0.0, -1.0), n), vec3<f32>(0.0, 0.0, -1.0));
    }
    let away = normalize(away_raw);
    let around = cross(n, away) * hand;
    let turn = spiral * (1.0 - smoothstep1(0.0, reach, length(offset)));
    return away * cos(turn) + around * sin(turn);
}

// `GrowthField::direction`.
fn growth_direction(p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let down = in_plane(vec3<f32>(0.0, -1.0, 0.0), n);
    let fallback = safe_normalize(down, vec3<f32>(0.0, 0.0, -1.0));
    let w0 = vec3<f32>(params.w0_x, params.w0_y, params.w0_z);
    let w1 = vec3<f32>(params.w1_x, params.w1_y, params.w1_z);
    var v = down;
    switch params.growth {
        case 0u, 3u: {
            if (params.whorls >= 1u) {
                v = whorl_direction(w0, params.w0_spiral, params.w0_hand, params.w0_reach, p, n);
            }
        }
        case 1u: {
            if (params.whorls >= 2u) {
                let t = smoothstep1(-0.004, 0.004, distance(p, w0) - distance(p, w1));
                v = whorl_direction(w0, params.w0_spiral, params.w0_hand, params.w0_reach, p, n) * (1.0 - t)
                    + whorl_direction(w1, params.w1_spiral, params.w1_hand, params.w1_reach, p, n) * t;
            }
        }
        case 2u: {
            if (params.whorls >= 2u) {
                let t = 1.0 - smoothstep1(0.5 * params.w1_reach, params.w1_reach, distance(p, w1));
                v = whorl_direction(w0, params.w0_spiral, params.w0_hand, params.w0_reach, p, n) * (1.0 - t)
                    + whorl_direction(w1, params.w1_spiral, params.w1_hand, params.w1_reach, p, n) * t;
            }
        }
        case 4u: {
            let over = smoothstep1(0.35, 0.75, n.y);
            v = in_plane(vec3<f32>(0.0, -0.15, 1.0), n) * over + down * (1.0 - over);
        }
        case 5u: {
            let over = smoothstep1(0.2, 0.6, n.y + 0.5 * n.z);
            v = in_plane(vec3<f32>(0.0, -0.15, -1.0), n) * over + down * (1.0 - over);
        }
        case 6u: {
            let q = p - vec3<f32>(params.center_x, params.center_y, params.center_z);
            let out = vec3<f32>(q.x, 0.0, q.z * 0.3);
            v = in_plane(vec3<f32>(0.0, -1.0, 0.0) + safe_normalize(out, vec3<f32>(1.0, 0.0, 0.0)) * 0.35, n);
        }
        default: {}
    }
    return safe_normalize(in_plane(v, n), fallback);
}
"#;

/// `Parting::group`, from the parameters `parting`, `lanes`, `plane_*`,
/// `until` and `fringe_*`.
pub(super) const PARTING: &str = r#"
fn parting_group(p: vec3<f32>) -> i32 {
    switch params.parting {
        case 1u: {
            if (p.z < params.until) {
                return -1;
            }
            let side = dot(vec3<f32>(params.plane_x, params.plane_y, params.plane_z), p) > params.plane_offset;
            return select(0, 1, side);
        }
        case 2u: {
            let a = atan2(p.x, p.z + 0.02);
            let t = (a + PI) / TAU;
            return clamp(i32(t * f32(params.lanes)), 0, i32(params.lanes) - 1);
        }
        case 3u: {
            if (p.z > params.fringe_front && abs(p.x) < params.fringe_half_width) {
                return 1;
            }
            if (p.z > -0.03) {
                return select(0, 2, p.x >= 0.0);
            }
            return -1;
        }
        default: {
            return -1;
        }
    }
}
"#;

/// Candidate roots, stratified over the scalp by area, each kept or not by
/// the density masks: the balding pattern, uneven cover, the hairline's
/// thinning. Writes the candidate and a 0/1 flag for the scan.
pub fn roots() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> tri_cdf: array<f32>;
@group(0) @binding(1) var<storage, read> tri_index: array<vec4<u32>>;
@group(0) @binding(2) var<storage, read> scalp_points: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> scalp_normals: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> scalp_extra: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> cand_position: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read_write> cand_normal: array<vec4<f32>>;
@group(0) @binding(7) var<storage, read_write> keep: array<u32>;

struct Params {
    count: u32,
    triangles: u32,
    seed: u32,
    balding: u32,
    total_area: f32,
    crown: f32,
    temples: f32,
    top: f32,
    patchiness: f32,
    edge: f32,
    offset_x: f32,
    offset_y: f32,
    offset_z: f32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(8) var<uniform> params: Params;
"#,
        &[RANDOM, NOISE, MATH],
        r#"
fn balding_keep(uv: vec2<f32>) -> f32 {
    let x = uv.x - 0.5;
    let y = uv.y - 0.5;
    let r = sqrt(x * x + y * y) * 2.0;
    let cy = y + 0.08;
    let crown_r = sqrt(x * x + cy * cy) * 2.0;
    let crown_loss = params.crown * (1.0 - smoothstep1(0.1, 0.1 + 0.35 * params.crown, crown_r));
    let angle = atan2(x, y);
    let temple = exp(-pow((abs(angle) - 0.55) / 0.3, 2.0));
    let temple_loss = params.temples * temple * smoothstep1(0.62 - 0.35 * params.temples, 0.8, r);
    let band = exp(-pow(x / 0.16, 2.0)) * smoothstep1(-0.2, 0.3, y);
    let top_loss = params.top * band * smoothstep1(0.0, 0.5, r);
    return clamp(1.0 - max(crown_loss, max(temple_loss, top_loss)), 0.0, 1.0);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let c = item(id, groups);
    if (c >= params.count) {
        return;
    }
    // Systematic sampling along the triangles' cumulative area: each
    // triangle gets its share of candidates, spread over it by a
    // low-discrepancy sequence.
    let target_area = (f32(c) + 0.5) / f32(params.count) * params.total_area;
    var lo = 0u;
    var hi = params.triangles - 1u;
    while (lo < hi) {
        let mid = (lo + hi) / 2u;
        if (tri_cdf[mid] < target_area) {
            lo = mid + 1u;
        } else {
            hi = mid;
        }
    }
    let t = lo;
    var before = 0.0;
    if (t > 0u) {
        before = tri_cdf[t - 1u];
    }
    let area = max(tri_cdf[t] - before, 1e-12);
    let k = (target_area - before) / area * (area / params.total_area * f32(params.count));
    var r1 = fract(k * 0.7548776662 + unit(t, params.seed));
    var r2 = fract(k * 0.5698402910 + unit(t, params.seed ^ 0x51u));
    if (r1 + r2 > 1.0) {
        r1 = 1.0 - r1;
        r2 = 1.0 - r2;
    }
    let w = vec3<f32>(1.0 - r1 - r2, r1, r2);
    let tri = tri_index[t];
    let position = scalp_points[tri.x].xyz * w.x + scalp_points[tri.y].xyz * w.y + scalp_points[tri.z].xyz * w.z;
    let normal = normalize(scalp_normals[tri.x].xyz * w.x + scalp_normals[tri.y].xyz * w.y + scalp_normals[tri.z].xyz * w.z);
    let extra = scalp_extra[tri.x] * w.x + scalp_extra[tri.y] * w.y + scalp_extra[tri.z] * w.z;

    var keep_here = 1.0;
    if (params.balding != 0u) {
        keep_here = balding_keep(extra.xy);
    }
    if (params.patchiness > 0.0) {
        let n = fractal(position * 30.0 + vec3<f32>(params.offset_x, params.offset_y, params.offset_z), 3u, 0.5);
        keep_here *= 1.0 - params.patchiness * smoothstep1(0.45, 0.7, n);
    }
    var inset = 1.0;
    if (params.edge > 0.0) {
        inset = smoothstep1(0.0, params.edge, extra.z);
    }
    keep_here *= 0.08 + 0.92 * pow(inset, 1.5);
    let kept = keep_here >= 1.0 || unit(c, params.seed ^ 0xD35Eu) < keep_here;
    cand_position[c] = vec4<f32>(position, inset);
    cand_normal[c] = vec4<f32>(normal, 0.0);
    cand_position[c + params.count] = vec4<f32>(extra.xy, 0.0, 0.0);
    keep[c] = select(0u, 1u, kept);
}
"#,
    )
}

/// The kept candidates into their strand slots (the flags scanned into
/// offsets), with their parting group, id and radius.
pub fn roots_compact() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
@group(0) @binding(0) var<storage, read> cand_position: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> cand_normal: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> keep: array<u32>;
@group(0) @binding(3) var<storage, read_write> roots: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> strands: array<StrandGpu>;

struct Params {{
    count: u32,
    id_seed: u32,
    parting: u32,
    lanes: u32,
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
    let c = item(id, groups);
    if (c >= params.count || keep[c + 1u] == keep[c]) {
        return;
    }
    let s = keep[c];
    let p = cand_position[c];
    roots[s] = p;
    var strand: StrandGpu;
    strand.normal = cand_normal[c].xyz;
    strand.id = hash2(s, params.id_seed);
    strand.uv = cand_position[c + params.count].xy;
    strand.group = parting_group(p.xyz);
    // Hair at the hairline is finer.
    strand.radius = params.radius * (0.45 + 0.55 * p.w);
    strands[s] = strand;
}
"#,
    )
}

/// For each strand, the guides it takes its shape from: the nearest
/// `neighbors` in its parting group, those whose courses stray from the
/// nearest one's left out, weighted by inverse square distance.
pub fn interp_select() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
@group(0) @binding(0) var<storage, read> roots: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> strands: array<StrandGpu>;
@group(0) @binding(2) var<storage, read> guide_points: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> guide_info: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> neighbors: array<vec4<u32>>;
@group(0) @binding(5) var<storage, read_write> weights: array<vec4<f32>>;

struct Params {{
    count: u32,
    guides: u32,
    points: u32,
    wanted: u32,
    divergence: f32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(6) var<uniform> params: Params;
"#
        ),
        &[],
        r#"
fn compatible(own: i32, guide: i32) -> bool {
    return own < 0 || guide < 0 || own == guide;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let root = roots[s].xyz;
    let own = strands[s].group;
    let k = min(params.wanted, 8u);
    var best_d: array<f32, 8>;
    var best_g: array<u32, 8>;
    var found = 0u;
    // Brute force: a few hundred guides, a hundred thousand strands.
    for (var sweep = 0u; sweep < 2u && found == 0u; sweep++) {
        for (var g = 0u; g < params.guides; g++) {
            let group = i32(guide_info[g].w);
            if (sweep == 0u && !compatible(own, group)) {
                continue;
            }
            let d = distance(guide_points[g * params.points].xyz, root);
            if (found < k || d < best_d[found - 1u]) {
                var at = min(found, k - 1u);
                if (found < k) {
                    found += 1u;
                }
                while (at > 0u && best_d[at - 1u] > d) {
                    best_d[at] = best_d[at - 1u];
                    best_g[at] = best_g[at - 1u];
                    at -= 1u;
                }
                best_d[at] = d;
                best_g[at] = g;
            }
        }
    }
    // Guides that agree with the nearest one.
    let p = params.points;
    let reference = best_g[0] * p;
    var kept = 0u;
    var ids: array<u32, 8>;
    var w: array<f32, 8>;
    var total = 0.0;
    for (var n = 0u; n < found; n++) {
        let g = best_g[n];
        var agrees = true;
        if (params.divergence > 0.0 && p >= 3u) {
            let samples = array<u32, 3>(p / 3u, 2u * p / 3u, p - 1u);
            for (var q = 0u; q < 3u; q++) {
                let j = samples[q];
                let own_offset = guide_points[reference + j].xyz - guide_points[reference].xyz;
                let theirs = guide_points[g * p + j].xyz - guide_points[g * p].xyz;
                if (length(own_offset - theirs) > params.divergence * max(length(own_offset), 0.02)) {
                    agrees = false;
                }
            }
        }
        if (agrees) {
            ids[kept] = g;
            w[kept] = 1.0 / (best_d[n] * best_d[n] + 1e-10);
            total += w[kept];
            kept += 1u;
        }
    }
    var a = vec4<u32>(0u);
    var b = vec4<u32>(0u);
    var wa = vec4<f32>(0.0);
    var wb = vec4<f32>(0.0);
    for (var n = 0u; n < kept; n++) {
        let weight = w[n] / total;
        if (n < 4u) {
            a[n] = ids[n];
            wa[n] = weight;
        } else {
            b[n - 4u] = ids[n];
            wb[n - 4u] = weight;
        }
    }
    neighbors[s * 2u] = a;
    neighbors[s * 2u + 1u] = b;
    weights[s * 2u] = wa;
    weights[s * 2u + 1u] = wb;
}
"#,
    )
}

/// Every point of every strand: its guides' offsets from their roots,
/// turned from their root's normal to this root's near the root, blended.
pub fn interp_points() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
@group(0) @binding(0) var<storage, read> roots: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> strands: array<StrandGpu>;
@group(0) @binding(2) var<storage, read> guide_points: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> guide_info: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> neighbors: array<vec4<u32>>;
@group(0) @binding(5) var<storage, read> weights: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read_write> dst: array<vec4<f32>>;

struct Params {{
    count: u32,
    points: u32,
    follow: u32,
    pad0: u32,
    follow_length: f32,
    pad1: u32,
    pad2: u32,
    pad3: u32,
}};
@group(0) @binding(7) var<uniform> params: Params;
"#
        ),
        &[MATH],
        r#"
fn turn_between(v: vec3<f32>, source_raw: vec3<f32>, target_raw: vec3<f32>, share: f32) -> vec3<f32> {
    let source = safe_normalize(source_raw, vec3<f32>(0.0, 1.0, 0.0));
    let target_dir = safe_normalize(target_raw, vec3<f32>(0.0, 1.0, 0.0));
    let axis = cross(source, target_dir);
    let s = length(axis);
    if (s < 1e-6) {
        return v;
    }
    return rotate(v, axis / s, atan2(s, dot(source, target_dir)) * share);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    let p = params.points;
    if (i >= params.count * p) {
        return;
    }
    let s = i / p;
    let j = i % p;
    let root = roots[s].xyz;
    let normal = strands[s].normal;
    var shape = vec3<f32>(0.0);
    for (var n = 0u; n < 8u; n++) {
        let half = n / 4u;
        let weight = weights[s * 2u + half][n % 4u];
        if (weight <= 0.0) {
            continue;
        }
        let g = neighbors[s * 2u + half][n % 4u];
        let here = guide_points[g * p + j];
        var offset = here.xyz - guide_points[g * p].xyz;
        if (params.follow != 0u) {
            // `here.w` is the guide's arc length at this point.
            let share = 1.0 - smoothstep1(0.0, params.follow_length, here.w);
            offset = turn_between(offset, guide_info[g].xyz, normal, share);
        }
        shape += offset * weight;
    }
    dst[i] = vec4<f32>(root + shape, 0.0);
}
"#,
    )
}

const PER_STRAND: &str = r#"
@group(0) @binding(0) var<storage, read> src: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> dst: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> strands: array<StrandGpu>;
"#;

/// Each strand's root turned the way its own follicle points.
pub fn orient() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
{PER_STRAND}
struct Params {{
    count: u32,
    points: u32,
    seed: u32,
    growth: u32,
    memory: f32,
    direction_spread: f32,
    angle_spread: f32,
    whorls: u32,
    center_x: f32,
    center_y: f32,
    center_z: f32,
    pad0: u32,
    w0_x: f32,
    w0_y: f32,
    w0_z: f32,
    w0_spiral: f32,
    w0_hand: f32,
    w0_reach: f32,
    w1_hand: f32,
    w1_reach: f32,
    w1_x: f32,
    w1_y: f32,
    w1_z: f32,
    w1_spiral: f32,
}};
@group(0) @binding(3) var<uniform> params: Params;
"#
        ),
        &[RANDOM, MATH, WALK, GROWTH],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let points = params.points;
    let base = s * points;
    let root = src[base].xyz;
    let n = safe_normalize(strands[s].normal, vec3<f32>(0.0, 1.0, 0.0));
    var ahead = -1;
    for (var j = 1u; j < points; j++) {
        if (distance(src[base + j].xyz, root) > 0.002) {
            ahead = i32(j);
            break;
        }
    }
    if (ahead < 0) {
        copy_strand(base, points);
        return;
    }
    let leaves = src[base + u32(ahead)].xyz - root;
    let flat_raw = in_plane(leaves, n);
    let grows = growth_direction(root, n);
    let sid = strands[s].id;
    var turn = params.direction_spread * signed1(sid, params.seed);
    if (length(flat_raw) > 0.25 * length(leaves)) {
        let flat = normalize(flat_raw);
        turn += atan2(dot(n, cross(flat, grows)), dot(flat, grows));
    }
    let tilt = params.angle_spread * signed1(sid, params.seed ^ 0x7117u);
    let tilt_axis = safe_normalize(cross(grows, n), orthogonal(n));
    let lift = (1.0 - cos(turn)) * 0.5 * params.memory * 0.35;
    dst[base] = src[base];
    var along = 0.0;
    var previous = root;
    for (var j = 1u; j < points; j++) {
        let p = src[base + j].xyz;
        along += distance(p, previous);
        previous = p;
        let share = 1.0 - smoothstep1(0.0, params.memory, along);
        if (share <= 0.0) {
            dst[base + j] = src[base + j];
            continue;
        }
        var offset = rotate(p - root, n, turn * share);
        offset = rotate(offset, tilt_axis, -tilt * share);
        let rise = lift * smoothstep1(0.0, params.memory, along) * share * 4.0;
        dst[base + j] = vec4<f32>(root + offset + n * rise, src[base + j].w);
    }
}
"#,
    )
}

/// Each strand cut to a share of its length, and resampled.
pub fn trim() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
{PER_STRAND}
struct Params {{
    count: u32,
    points: u32,
    seed: u32,
    pad0: u32,
    keep_share: f32,
    random: f32,
    max_length: f32,
    pad1: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
"#
        ),
        &[RANDOM, MATH, WALK],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let points = params.points;
    let base = s * points;
    let total = strand_length(base, points);
    var keep = total * clamp(params.keep_share * (1.0 - params.random * unit(strands[s].id, params.seed)), 0.02, 1.0);
    if (params.max_length > 0.0) {
        keep = min(keep, params.max_length);
    }
    if (total <= 0.0 || keep >= total * 0.9999) {
        copy_strand(base, points);
        return;
    }
    // Resampled evenly along the first `keep` of the arc.
    var segment = 0u;
    var segment_start = 0.0;
    var segment_end = distance(point_at(base, 1u), point_at(base, 0u));
    for (var j = 0u; j < points; j++) {
        let target_length = keep * f32(j) / f32(points - 1u);
        while (segment + 2u < points && segment_end < target_length) {
            segment += 1u;
            segment_start = segment_end;
            segment_end += distance(point_at(base, segment + 1u), point_at(base, segment));
        }
        let span = segment_end - segment_start;
        var t = 0.0;
        if (span > 1e-12) {
            t = clamp((target_length - segment_start) / span, 0.0, 1.0);
        }
        dst[base + j] = vec4<f32>(mix(point_at(base, segment), point_at(base, segment + 1u), t), 0.0);
    }
}
"#,
    )
}

/// `Create Guide Index Map`, on a grid: in every cell of side `distance`,
/// for every parting group, the candidate whose hash is lowest leads (a
/// packed `!(priority << 20 | strand)` kept by `atomicMax`, so an empty
/// cell is zero).
pub fn map_candidates() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
@group(0) @binding(0) var<storage, read> roots: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> strands: array<StrandGpu>;
@group(0) @binding(2) var<storage, read_write> leaders: array<atomic<u32>>;

struct Params {{
    count: u32,
    nx: u32,
    ny: u32,
    nz: u32,
    lo_x: f32,
    lo_y: f32,
    lo_z: f32,
    inverse_cell: f32,
    mask: f32,
    seed: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
"#
        ),
        &[RANDOM],
        MAP_CELL,
    ) + r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let sid = strands[s].id;
    if (unit(sid, params.seed ^ 0x2F0B1A3Du) >= params.mask) {
        return;
    }
    let cell = map_cell(roots[s].xyz);
    let slot = map_slot(cell, strands[s].group);
    let priority = hash2(sid, params.seed) >> 20u;
    atomicMax(&leaders[slot], ~((priority << 20u) | s));
}
"#
}

const MAP_CELL: &str = r#"
fn map_cell(p: vec3<f32>) -> vec3<i32> {
    let c = vec3<i32>(floor((p - vec3<f32>(params.lo_x, params.lo_y, params.lo_z)) * params.inverse_cell));
    return clamp(c, vec3<i32>(0), vec3<i32>(i32(params.nx) - 1, i32(params.ny) - 1, i32(params.nz) - 1));
}

fn map_slot(c: vec3<i32>, group: i32) -> u32 {
    let cell = (u32(c.z) * params.ny + u32(c.y)) * params.nx + u32(c.x);
    return cell * 5u + u32(clamp(group + 1, 0, 4));
}
"#;

/// Every strand to its nearest leader in the cells around it, its own
/// parting group's first.
pub fn map_assign() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
@group(0) @binding(0) var<storage, read> roots: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> strands: array<StrandGpu>;
@group(0) @binding(2) var<storage, read> leaders: array<u32>;
@group(0) @binding(3) var<storage, read_write> guide_of: array<u32>;

struct Params {{
    count: u32,
    nx: u32,
    ny: u32,
    nz: u32,
    lo_x: f32,
    lo_y: f32,
    lo_z: f32,
    inverse_cell: f32,
    mask: f32,
    seed: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(4) var<uniform> params: Params;
"#
        ),
        &[],
        MAP_CELL,
    ) + r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let p = roots[s].xyz;
    let base = map_cell(p);
    let own = strands[s].group;
    var best = s;
    var best_d = 1e30;
    for (var sweep = 0u; sweep < 2u && best_d >= 1e30; sweep++) {
        for (var n = 0u; n < 27u; n++) {
            let c = base + vec3<i32>(i32(n % 3u) - 1, i32((n / 3u) % 3u) - 1, i32(n / 9u) - 1);
            if (any(c < vec3<i32>(0)) || c.x >= i32(params.nx) || c.y >= i32(params.ny) || c.z >= i32(params.nz)) {
                continue;
            }
            for (var g = 0; g < 5; g++) {
                if (sweep == 0u && g != clamp(own + 1, 0, 4)) {
                    continue;
                }
                let stored = leaders[map_slot(c, g - 1)];
                if (stored == 0u) {
                    continue;
                }
                let leader = (~stored) & 0xFFFFFu;
                let d = distance(roots[leader].xyz, p);
                if (d < best_d) {
                    best_d = d;
                    best = leader;
                }
            }
        }
    }
    guide_of[s] = best;
}
"#
}

const WITH_GUIDES: &str = r#"
@group(0) @binding(3) var<storage, read> guide_of: array<u32>;
"#;

/// `Clump Hair Curves`: strands gathered to their clump guide.
pub fn clump() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
{PER_STRAND}
{WITH_GUIDES}
struct Params {{
    count: u32,
    points: u32,
    seed: u32,
    preserve: u32,
    factor: f32,
    shape: f32,
    twist: f32,
    twist_random: f32,
    twist_factor: f32,
    tip_spread: f32,
    clump_offset: f32,
    distance_falloff: f32,
    distance_threshold: f32,
    min_length: f32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(4) var<uniform> params: Params;
"#
        ),
        &[RANDOM, NOISE, MATH, WALK],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let points = params.points;
    let base = s * points;
    let g = guide_of[s];
    let guide_base = g * points;

    // The guide's root frame, and its tip's.
    var root_walk = walker_new(guide_base, points);
    let guide_root = walker_at(&root_walk, 0.0);
    var tip_walk = walker_new(guide_base, points);
    let tip = walker_at(&tip_walk, tip_walk.total);

    let root = src[base].xyz;
    let offset = root - guide_root.position;
    let scalp = strands[g].normal;
    let t0 = guide_root.tangent;
    let across = safe_normalize(cross(t0, scalp), guide_root.normal);
    let along = safe_normalize(t0 - scalp * dot(t0, scalp), t0);
    let depth = safe_normalize(cross(across, t0), guide_root.binormal);
    let placed = across * dot(offset, across) + depth * dot(offset, along);
    let x = dot(guide_root.normal, placed);
    let y = dot(guide_root.binormal, placed);

    var gather = vec3<f32>(0.0);
    if (params.tip_spread != 0.0) {
        let r = signed3(strands[s].id, params.seed) * params.tip_spread;
        gather += tip.normal * r.x + tip.binormal * r.y + tip.tangent * r.z;
    }
    if (params.clump_offset != 0.0) {
        let r = signed3(strands[g].id, params.seed ^ 0x0FF5u) * params.clump_offset;
        gather += tip.normal * r.x + tip.binormal * r.y;
    }

    let d = length(offset);
    var weight = 1.0 - smoothstep1(params.distance_threshold, params.distance_threshold + params.distance_falloff, d);
    if (params.distance_threshold != 0.0 && params.distance_falloff == 0.0) {
        weight = select(0.0, 1.0, d <= params.distance_threshold);
    }
    let clump_random = white(vec3<f32>(f32(g), 0.0, 0.0));
    let total = strand_length(base, points);
    if (params.min_length > 0.0) {
        weight *= smoothstep1(params.min_length, 2.0 * params.min_length, total);
    }

    var walk = walker_new(guide_base, points);
    dst[base] = src[base];
    var travelled = 0.0;
    for (var j = 1u; j < points; j++) {
        travelled += distance(point_at(base, j), point_at(base, j - 1u));
        let t = travelled / max(total, 1e-12);
        let f = shape_range(t, 0.0, 1.0, params.shape);
        let at = walker_at(&walk, t * walk.total);
        let parallel = at.normal * x + at.binormal * y;
        let target_point = at.position + parallel * (1.0 - f) + gather * f;
        var p = mix(point_at(base, j), target_point, f * params.factor * weight);
        let twist_base = travelled * params.twist;
        let angle = params.twist_factor * (twist_base + (twist_base * clump_random - twist_base) * params.twist_random);
        if (angle != 0.0) {
            p = at.position + rotate(p - at.position, at.tangent, angle);
        }
        dst[base + j] = vec4<f32>(p, src[base + j].w);
    }
    if (params.preserve != 0u) {
        restore_lengths(base, points, 1.0);
    }
}
"#,
    )
}

/// `Curl Hair Curves`: ringlets around the clump guide.
pub fn curl() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
{PER_STRAND}
{WITH_GUIDES}
struct Params {{
    count: u32,
    points: u32,
    seed: u32,
    pad0: u32,
    factor: f32,
    curl_start: f32,
    radius: f32,
    factor_start: f32,
    factor_end: f32,
    turns_per_metre: f32,
    random_offset: f32,
    pad1: u32,
}};
@group(0) @binding(4) var<uniform> params: Params;
"#
        ),
        &[RANDOM, MATH, WALK],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let points = params.points;
    let base = s * points;
    let g = guide_of[s];
    let total = strand_length(base, points);
    let phase0 = unit(strands[g].id, params.seed) * 3.9 + unit(strands[s].id, params.seed ^ 0xC0FFu) * params.random_offset;
    var walk = walker_new(g * points, points);
    dst[base] = src[base];
    var travelled = 0.0;
    for (var j = 1u; j < points; j++) {
        travelled += distance(point_at(base, j), point_at(base, j - 1u));
        let t = travelled / max(total, 1e-12);
        let phase = (travelled * params.turns_per_metre + phase0) * TAU;
        let r = params.radius * (params.factor_start + (params.factor_end - params.factor_start) * t);
        let at = walker_at(&walk, travelled);
        let circle = at.normal * (sin(phase) * r) + at.binormal * (cos(phase) * r);
        let ease = sqrt(smootherstep1(-1e-5, params.curl_start, t));
        let p = point_at(base, j);
        dst[base + j] = vec4<f32>(p + (at.position - p + circle) * (ease * params.factor), src[base + j].w);
    }
}
"#,
    )
}

/// Waves across each strand, in the plane it lies in, a phase a lock.
pub fn wave() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
{PER_STRAND}
{WITH_GUIDES}
struct Params {{
    count: u32,
    points: u32,
    seed: u32,
    pad0: u32,
    amplitude: f32,
    wavelength: f32,
    start: f32,
    random_phase: f32,
}};
@group(0) @binding(4) var<uniform> params: Params;
"#
        ),
        &[RANDOM, MATH, WALK],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let points = params.points;
    let base = s * points;
    let total = strand_length(base, points);
    let lock = strands[guide_of[s]].id;
    let phase = (unit(lock, params.seed) + signed1(strands[s].id, params.seed ^ 0x51u) * params.random_phase) * TAU;
    let out = strands[s].normal;
    dst[base] = src[base];
    var travelled = 0.0;
    for (var j = 1u; j < points; j++) {
        travelled += distance(point_at(base, j), point_at(base, j - 1u));
        let t = tangent_at(base, points, j);
        let across = safe_normalize(cross(t, out), orthogonal(t));
        let ease = smoothstep1(params.start * 0.5, params.start + 0.1, travelled / max(total, 1e-12));
        let w = sin(travelled / max(params.wavelength, 1e-3) * TAU + phase);
        dst[base + j] = vec4<f32>(point_at(base, j) + across * (params.amplitude * ease * w), src[base + j].w);
    }
    restore_lengths(base, points, 1.0);
}
"#,
    )
}

/// Offsets in each strand's own frame -- noise or frizz -- walked from the
/// root if cumulative, faded in by the shape.
const DISPLACE: &str = r#"
fn displace(s: u32, scale: f32, cumulative: bool, shape: f32, preserve: bool) {
    let points = params.points;
    let base = s * points;
    let total = strand_length(base, points);
    let first_segment = distance(point_at(base, 1u), point_at(base, 0u));
    var t = tangent_at(base, points, 0u);
    var n = orthogonal(t);
    var root_offset = vec3<f32>(0.0);
    var trailing = vec3<f32>(0.0);
    var travelled = 0.0;
    for (var j = 0u; j < points; j++) {
        if (j > 0u) {
            travelled += distance(point_at(base, j), point_at(base, j - 1u));
            let next = tangent_at(base, points, j);
            n = transport(n, t, next);
            t = next;
        }
        let r = offset_at(s, j, travelled, first_segment, scale);
        let b = cross(t, n);
        let v = t * r.x + b * r.y + n * r.z;
        if (j == 0u) {
            root_offset = v;
        }
        var o: vec3<f32>;
        if (cumulative) {
            let leading = trailing + v;
            o = (leading + trailing - root_offset) * 0.5;
            trailing = leading;
        } else {
            o = v - root_offset;
        }
        let fade = shape_range(travelled / max(total, 1e-12), 0.0, 1.0, shape);
        dst[base + j] = vec4<f32>(point_at(base, j) + o * fade, src[base + j].w);
    }
    if (preserve) {
        restore_lengths(base, points, 1.0);
    }
}
"#;

/// `Hair Curves Noise`: coherent Perlin waviness.
pub fn noise() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
{PER_STRAND}
struct Params {{
    count: u32,
    points: u32,
    seed: u32,
    cumulative: u32,
    scale_total: f32,
    shape: f32,
    scale: f32,
    scale_along: f32,
    offset_per_curve: f32,
    preserve: u32,
    origin_x: f32,
    origin_y: f32,
    origin_z: f32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
"#
        ),
        &[RANDOM, NOISE, MATH, WALK],
        r#"
fn offset_at(s: u32, j: u32, travelled: f32, first_segment: f32, scale: f32) -> vec3<f32> {
    let base = src[s * params.points].xyz * params.scale + vec3<f32>(params.origin_x, params.origin_y, params.origin_z);
    let along = signed1(strands[s].id, params.seed) * params.offset_per_curve;
    let z = travelled * params.scale_along + along;
    let c = noise_color(base + vec3<f32>(0.0, 0.0, z), 5.0);
    return (c - vec3<f32>(0.5)) * (2.0 * scale);
}
"#,
    ) + DISPLACE
        + r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    displace(s, params.scale_total, params.cumulative != 0u, params.shape, params.preserve != 0u);
}
"#
}

/// `Frizz Hair Curves`: a few flyaways wander, the rest barely.
pub fn frizz() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
{PER_STRAND}
struct Params {{
    count: u32,
    points: u32,
    seed: u32,
    cumulative: u32,
    scale_total: f32,
    shape: f32,
    share: f32,
    calm: f32,
    frequency: f32,
    preserve: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
"#
        ),
        &[RANDOM, NOISE, MATH, WALK],
        r#"
fn offset_at(s: u32, j: u32, travelled: f32, first_segment: f32, scale: f32) -> vec3<f32> {
    let sid = strands[s].id;
    if (params.frequency > 0.0) {
        let origin = signed3(sid, params.seed ^ 0x0F75u) * 50.0;
        let q = origin + vec3<f32>(0.0, 0.0, travelled * params.frequency);
        let c = noise_color(q, 1.0) - vec3<f32>(0.5);
        return c * (2.0 * scale * max(first_segment, 1e-6) * params.frequency);
    }
    return signed3(hash2(sid, j), params.seed) * scale;
}
"#,
    ) + DISPLACE
        + r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let wild = unit(strands[s].id, params.seed ^ 0xF1A7u) < params.share;
    let scale = select(params.scale_total * params.calm, params.scale_total, wild);
    if (scale == 0.0) {
        copy_strand(s * params.points, params.points);
        return;
    }
    displace(s, scale, params.cumulative != 0u, params.shape, params.preserve != 0u);
}
"#
}

/// `Smooth Hair Curves`: each point averaged with its neighbours, the root
/// held, the tip smoothed against its own extension.
pub fn smooth() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
{PER_STRAND}
struct Params {{
    count: u32,
    points: u32,
    iterations: u32,
    preserve: u32,
    factor: f32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
"#
        ),
        &[MATH, WALK],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let points = params.points;
    let base = s * points;
    copy_strand(base, points);
    let tip = points - 1u;
    let extended = point_at(base, tip) * 2.0 - point_at(base, tip - 1u);
    let w = clamp(params.factor, 0.0, 1.0);
    for (var k = 0u; k < params.iterations; k++) {
        // Jacobi in place: the old value of the point before is carried.
        var before = dst[base].xyz;
        for (var j = 1u; j < points; j++) {
            let here = dst[base + j].xyz;
            var after = extended;
            if (j + 1u < points) {
                after = dst[base + j + 1u].xyz;
            }
            let average = (before + here + after) * (1.0 / 3.0);
            dst[base + j] = vec4<f32>(mix(here, average, w), dst[base + j].w);
            before = here;
        }
    }
    if (params.preserve != 0u) {
        restore_lengths(base, points, 1.0);
    }
}
"#,
    )
}

/// `Shrinkwrap Hair Curves`: out of the head, the move blurred along the
/// strand, then anything still in pushed straight out.
pub fn shrinkwrap() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
{PER_STRAND}
@group(0) @binding(3) var<storage, read> head_shape: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> moves: array<vec4<f32>>;
struct Params {{
    count: u32,
    points: u32,
    smoothing: u32,
    lock_roots: u32,
    factor: f32,
    offset: f32,
    above_surface: f32,
    preserve: u32,
}};
@group(0) @binding(5) var<uniform> params: Params;
"#
        ),
        &[MATH, HEAD, WALK],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let points = params.points;
    let base = s * points;
    let offset = max(params.offset, 1e-10);
    var any_move = false;
    for (var j = 0u; j < points; j++) {
        let p = point_at(base, j);
        let locked = j == 0u && params.lock_roots != 0u;
        let d = head_distance(p);
        var weight = 0.0;
        if (!locked && d < offset) {
            if (d < 0.0) {
                weight = 1.0;
            } else {
                weight = 1.0 + (params.above_surface - 1.0) * (d / offset);
            }
        }
        var m = vec3<f32>(0.0);
        if (weight > 0.0) {
            m = head_gradient(p) * (offset - d) * weight;
            any_move = true;
        }
        moves[base + j] = vec4<f32>(m, select(1.0, 0.0, locked));
    }
    if (!any_move) {
        copy_strand(base, points);
        return;
    }
    // Blur the moves along the strand, root and tip held.
    for (var k = 0u; k < params.smoothing; k++) {
        var before = moves[base].xyz;
        for (var j = 1u; j + 1u < points; j++) {
            let here = moves[base + j];
            let after = moves[base + j + 1u].xyz;
            let average = (before + here.xyz + after) * (1.0 / 3.0);
            moves[base + j] = vec4<f32>(mix(here.xyz, average, clamp(here.w, 0.0, 1.0)), here.w);
            before = here.xyz;
        }
    }
    for (var j = 0u; j < points; j++) {
        dst[base + j] = vec4<f32>(point_at(base, j) + moves[base + j].xyz * params.factor, src[base + j].w);
    }
    if (params.preserve != 0u) {
        restore_lengths(base, points, 1.0);
    }
    if (params.factor >= 1.0) {
        for (var j = 1u; j < points; j++) {
            dst[base + j] = vec4<f32>(push_out(dst[base + j].xyz, params.offset * 0.5), dst[base + j].w);
        }
    }
}
"#,
    )
}

/// `Set Hair Curve Profile`: each strand's root radius.
pub fn profile() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
@group(0) @binding(0) var<storage, read_write> strands: array<StrandGpu>;
struct Params {{
    count: u32,
    seed: u32,
    relative: u32,
    pad0: u32,
    radius: f32,
    random: f32,
    grown: f32,
    pad1: u32,
}};
@group(0) @binding(1) var<uniform> params: Params;
"#
        ),
        &[RANDOM],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    var base = params.radius;
    if (params.relative != 0u && params.grown > 0.0) {
        base = params.radius * strands[s].radius / params.grown;
    }
    strands[s].radius = base * max(1.0 + params.random * signed1(strands[s].id, params.seed), 0.05);
}
"#,
    )
}

/// One solve of the drape ([`crate::ops::Settle`]): each strand's
/// equilibrium as an elastic rod under `load`, from its groomed shape in
/// `src`, into `solved`. The suffix sums of a sweep and the joints'
/// directions are kept in `moments` and `dirs`, a strand's own stretch of
/// each, never in a local array.
pub fn settle_solve() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> src: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> solved: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> moments: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> dirs: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> head_shape: array<vec4<f32>>;
struct Params {
    count: u32,
    points: u32,
    iterations: u32,
    pad0: u32,
    load_x: f32,
    load_y: f32,
    load_z: f32,
    compliance: f32,
    collision_offset: f32,
    pad1: u32,
    pad2: u32,
    pad3: u32,
};
@group(0) @binding(5) var<uniform> params: Params;
"#,
        &[MATH, HEAD],
        r#"
fn rest_dir(base: u32, i: u32) -> vec3<f32> {
    return safe_normalize(src[base + i + 1u].xyz - src[base + i].xyz, vec3<f32>(0.0, -1.0, 0.0));
}

fn rest_length(base: u32, i: u32) -> f32 {
    return distance(src[base + i + 1u].xyz, src[base + i].xyz);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let points = params.points;
    let segments = points - 1u;
    let base = s * points;
    let load = vec3<f32>(params.load_x, params.load_y, params.load_z);
    for (var j = 0u; j < points; j++) {
        solved[base + j] = src[base + j];
    }
    let magnitude = length(load);
    if (magnitude < 1e-6) {
        return;
    }
    let force = load / magnitude;
    for (var i = 0u; i < segments; i++) {
        dirs[base + i] = vec4<f32>(rest_dir(base, i), 0.0);
    }
    for (var sweep = 0u; sweep < params.iterations; sweep++) {
        // Suffix sums of each segment's centre times its length, and of the
        // lengths: the moment about point i of the load on all past it.
        var arm = vec3<f32>(0.0);
        var total = 0.0;
        moments[base + segments] = vec4<f32>(0.0);
        for (var k = 0u; k < segments; k++) {
            let j = segments - 1u - k;
            let l = rest_length(base, j);
            arm += (solved[base + j].xyz + solved[base + j + 1u].xyz) * 0.5 * l;
            total += l;
            moments[base + j] = vec4<f32>(arm, total);
        }
        // Root to tip: each joint bends from its groomed angle -- carried
        // along as its parent turned -- towards the load.
        var position = solved[base].xyz;
        for (var i = 0u; i < segments; i++) {
            var groomed = rest_dir(base, 0u);
            if (i > 0u) {
                groomed = transport(rest_dir(base, i), rest_dir(base, i - 1u), dirs[base + i - 1u].xyz);
            }
            let m = moments[base + i];
            let moment = cross(m.xyz - solved[base + i].xyz * m.w, load);
            let torque = length(moment);
            var target_dir = groomed;
            if (torque > 1e-12) {
                let axis = cross(groomed, force);
                let sin_angle = length(axis);
                if (sin_angle > 1e-6) {
                    let limit = atan2(sin_angle, dot(groomed, force));
                    let bend = min(torque * params.compliance * rest_length(base, i), limit);
                    target_dir = rotate(groomed, axis / sin_angle, bend);
                }
            }
            let d = safe_normalize(lerp3(dirs[base + i].xyz, target_dir, 0.5), target_dir);
            let l = rest_length(base, i);
            var next = position + d * l;
            // The body holds the hair up.
            if (head_distance(next) < params.collision_offset) {
                next = push_out(next, params.collision_offset);
                next = position + safe_normalize(next - position, d) * l;
            }
            dirs[base + i] = vec4<f32>(safe_normalize(next - position, d), 0.0);
            solved[base + i + 1u] = vec4<f32>(next, solved[base + i + 1u].w);
            position = next;
        }
    }
}
"#,
    )
}

/// The drape applied: the groom moved by what changed between the solve
/// under the asked conditions and under standard ones, lengths restored
/// and kept off the body (or, not relative, the solve itself).
pub fn settle_apply() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
{PER_STRAND}
@group(0) @binding(3) var<storage, read> under: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> standard: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> head_shape: array<vec4<f32>>;
struct Params {{
    count: u32,
    points: u32,
    relative: u32,
    pad0: u32,
    collision_offset: f32,
    pad1: u32,
    pad2: u32,
    pad3: u32,
}};
@group(0) @binding(6) var<uniform> params: Params;
"#
        ),
        &[MATH, HEAD, WALK],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    let points = params.points;
    let base = s * points;
    if (params.relative == 0u) {
        for (var j = 0u; j < points; j++) {
            dst[base + j] = under[base + j];
        }
        return;
    }
    for (var j = 0u; j < points; j++) {
        dst[base + j] = vec4<f32>(src[base + j].xyz + under[base + j].xyz - standard[base + j].xyz, src[base + j].w);
    }
    restore_lengths(base, points, 1.0);
    for (var j = 1u; j < points; j++) {
        dst[base + j] = vec4<f32>(push_out(dst[base + j].xyz, params.collision_offset * 0.5), dst[base + j].w);
    }
}
"#,
    )
}

/// The most points a strand may have for [`settle_solve_few`].
pub const SETTLE_FEW_POINTS: u32 = 192;

/// Both solves of the drape for a few strands -- the guides -- a workgroup
/// a strand and a solve (`y` 0 under the asked load, 1 under the reference
/// one). A few hundred invocations cannot hide storage's latency from
/// each other, so each strand's joints live in workgroup memory while one
/// lane runs its sweeps; the others load and store it.
pub fn settle_solve_few() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read> src: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> under: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> standard: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> head_shape: array<vec4<f32>>;
struct Params {{
    count: u32,
    points: u32,
    iterations: u32,
    pad0: u32,
    load_x: f32,
    load_y: f32,
    load_z: f32,
    compliance: f32,
    reference_x: f32,
    reference_y: f32,
    reference_z: f32,
    collision_offset: f32,
}};
@group(0) @binding(4) var<uniform> params: Params;

const FEW: u32 = {SETTLE_FEW_POINTS}u;
// Each segment's groomed direction and length.
var<workgroup> segments_rest: array<vec4<f32>, FEW>;
var<workgroup> solved: array<vec4<f32>, FEW>;
var<workgroup> dirs: array<vec4<f32>, FEW>;
// Suffix sums: each segment's centre times its length, and the lengths.
var<workgroup> moments: array<vec4<f32>, FEW>;
"#
        ),
        &[MATH, HEAD],
        r#"
@compute @workgroup_size(32)
fn main(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_index) lane: u32) {
    let s = wid.x;
    let which = wid.y;
    if (s >= params.count) {
        return;
    }
    let points = params.points;
    let segments = points - 1u;
    let base = s * points;
    var load = vec3<f32>(params.load_x, params.load_y, params.load_z);
    if (which == 1u) {
        load = vec3<f32>(params.reference_x, params.reference_y, params.reference_z);
    }
    for (var j = lane; j < points; j += 32u) {
        solved[j] = src[base + j];
        if (j < segments) {
            let d = src[base + j + 1u].xyz - src[base + j].xyz;
            let rest = safe_normalize(d, vec3<f32>(0.0, -1.0, 0.0));
            segments_rest[j] = vec4<f32>(rest, length(d));
            dirs[j] = vec4<f32>(rest, 0.0);
        }
    }
    workgroupBarrier();
    let magnitude = length(load);
    if (lane == 0u && magnitude >= 1e-6) {
        let force = load / magnitude;
        for (var sweep = 0u; sweep < params.iterations; sweep++) {
            var arm = vec3<f32>(0.0);
            var total = 0.0;
            moments[segments] = vec4<f32>(0.0);
            for (var k = 0u; k < segments; k++) {
                let j = segments - 1u - k;
                let l = segments_rest[j].w;
                arm += (solved[j].xyz + solved[j + 1u].xyz) * 0.5 * l;
                total += l;
                moments[j] = vec4<f32>(arm, total);
            }
            var position = solved[0].xyz;
            for (var i = 0u; i < segments; i++) {
                let rest = segments_rest[i];
                var groomed = segments_rest[0].xyz;
                if (i > 0u) {
                    groomed = transport(rest.xyz, segments_rest[i - 1u].xyz, dirs[i - 1u].xyz);
                }
                let m = moments[i];
                let torque = length(cross(m.xyz - solved[i].xyz * m.w, load));
                var target_dir = groomed;
                if (torque > 1e-12) {
                    let axis = cross(groomed, force);
                    let sin_angle = length(axis);
                    if (sin_angle > 1e-6) {
                        let limit = atan2(sin_angle, dot(groomed, force));
                        let bend = min(torque * params.compliance * rest.w, limit);
                        target_dir = rotate(groomed, axis / sin_angle, bend);
                    }
                }
                let d = safe_normalize(lerp3(dirs[i].xyz, target_dir, 0.5), target_dir);
                var next = position + d * rest.w;
                // The body holds the hair up.
                if (head_distance(next) < params.collision_offset) {
                    next = push_out(next, params.collision_offset);
                    next = position + safe_normalize(next - position, d) * rest.w;
                }
                dirs[i] = vec4<f32>(safe_normalize(next - position, d), 0.0);
                solved[i + 1u] = vec4<f32>(next, solved[i + 1u].w);
                position = next;
            }
        }
    }
    workgroupBarrier();
    for (var j = lane; j < points; j += 32u) {
        if (which == 0u) {
            under[base + j] = solved[j];
        } else {
            standard[base + j] = solved[j];
        }
    }
}
"#,
    )
}
