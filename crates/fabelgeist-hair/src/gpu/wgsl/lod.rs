//! The kernels that prepare a groom for drawing: its shuffled order, the
//! occlusion volume, every strand level, the cards and the scalp's cover.
//! Each mirrors the host code of [`crate::lod`], [`crate::occlusion`],
//! [`crate::cards`] and [`crate::cover`].

use super::common::{HEAD, INDEX, MATH, RANDOM, STRAND};

/// Fixed point for atomically summed densities.
pub const DENSITY_FIXED: f32 = 16384.0;
/// Fixed point for card spines (10 micrometres).
pub const SPINE_FIXED: f32 = 1e5;
/// Fixed point for squared lateral spreads.
pub const SPREAD_FIXED: f32 = 1e8;
/// Fixed point for how far a lock's roots reach behind its middle.
pub const BEHIND_FIXED: f32 = 1e6;

fn kernel(bindings: &str, helpers: &[&str], body: &str) -> String {
    let mut source = String::from(bindings);
    for h in helpers {
        source.push_str(h);
    }
    source.push_str(INDEX);
    source.push_str(body);
    source
}

/// Sort keys: each strand's place in the shuffled order.
pub fn order_keys() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
@group(0) @binding(0) var<storage, read> strands: array<StrandGpu>;
@group(0) @binding(1) var<storage, read_write> keys: array<u32>;
@group(0) @binding(2) var<storage, read_write> order: array<u32>;
struct Params {{
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
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
    keys[s] = hash2(strands[s].id, 0x0D3Eu);
    order[s] = s;
}
"#,
    )
}

const GRID: &str = r#"
fn grid_index(c: vec3<i32>) -> u32 {
    return (u32(c.z) * params.ny + u32(c.y)) * params.nx + u32(c.x);
}

fn grid_inside(c: vec3<i32>) -> bool {
    return all(c >= vec3<i32>(0)) && c.x < i32(params.nx) && c.y < i32(params.ny) && c.z < i32(params.nz);
}

fn grid_cell(p: vec3<f32>) -> vec3<i32> {
    return vec3<i32>(floor((p - vec3<f32>(params.lo_x, params.lo_y, params.lo_z)) / params.cell));
}

fn grid_center(i: u32) -> vec3<f32> {
    let c = vec3<u32>(i % params.nx, (i / params.nx) % params.ny, i / (params.nx * params.ny));
    return vec3<f32>(params.lo_x, params.lo_y, params.lo_z) + (vec3<f32>(c) + vec3<f32>(0.5)) * params.cell;
}
"#;

const GRID_PARAMS: &str = r#"
    nx: u32,
    ny: u32,
    nz: u32,
    count: u32,
    lo_x: f32,
    lo_y: f32,
    lo_z: f32,
    cell: f32,
"#;

/// Every segment's projected area into its cell (optical density, per
/// metre), summed in fixed point.
pub fn splat() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> strands: array<StrandGpu>;
@group(0) @binding(2) var<storage, read_write> deposits: array<atomic<u32>>;
struct Params {{
{GRID_PARAMS}
    points: u32,
    strength: f32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
"#
        ),
        &[GRID],
        &format!(
            r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let i = item(id, groups);
    let segments = params.points - 1u;
    if (i >= params.count * segments) {{
        return;
    }}
    let s = i / segments;
    let j = i % segments;
    let a = positions[s * params.points + j].xyz;
    let b = positions[s * params.points + j + 1u].xyz;
    let c = grid_cell((a + b) * 0.5);
    if (!grid_inside(c)) {{
        return;
    }}
    let volume = params.cell * params.cell * params.cell;
    let diameter = 2.0 * max(strands[s].radius, 1e-6);
    let d = distance(a, b) * diameter / volume * params.strength;
    atomicAdd(&deposits[grid_index(c)], u32(d * {DENSITY_FIXED:.1}));
}}
"#
        ),
    )
}

/// Deposits into densities; with the body, its cells opaque.
pub fn densify() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read> deposits: array<u32>;
@group(0) @binding(1) var<storage, read> head_shape: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> density: array<f32>;
struct Params {{
{GRID_PARAMS}
    body: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
"#
        ),
        &[MATH, HEAD, GRID],
        &format!(
            r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let i = item(id, groups);
    if (i >= params.nx * params.ny * params.nz) {{
        return;
    }}
    var d = f32(deposits[i]) / {DENSITY_FIXED:.1};
    if (params.body != 0u && head_distance(grid_center(i)) < 0.0) {{
        d = 1e4;
    }}
    density[i] = d;
}}
"#
        ),
    )
}

/// Sky visibility per cell: the transmittance averaged over directions,
/// weighted towards the sky, for cells next to hair.
pub fn occlusion() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read> density: array<f32>;
@group(0) @binding(1) var<storage, read_write> visibility: array<f32>;
struct Params {{
{GRID_PARAMS}
    directions: u32,
    steps: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(2) var<uniform> params: Params;
"#
        ),
        &[GRID],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    if (i >= params.nx * params.ny * params.nz) {
        return;
    }
    let c = vec3<i32>(vec3<u32>(i % params.nx, (i / params.nx) % params.ny, i / (params.nx * params.ny)));
    // Only cells next to hair are ever looked up.
    var needed = false;
    for (var n = 0u; n < 27u && !needed; n++) {
        let q = c + vec3<i32>(i32(n % 3u) - 1, i32((n / 3u) % 3u) - 1, i32(n / 9u) - 1);
        if (grid_inside(q)) {
            let d = density[grid_index(q)];
            needed = d > 0.0 && d < 1e4;
        }
    }
    if (!needed) {
        visibility[i] = 1.0;
        return;
    }
    let start = grid_center(i);
    let golden = 3.14159265 * (3.0 - sqrt(5.0));
    var sum = 0.0;
    var norm = 0.0;
    for (var k = 0u; k < params.directions; k++) {
        let y = 1.0 - 2.0 * (f32(k) + 0.5) / f32(params.directions);
        let r = sqrt(1.0 - y * y);
        let a = golden * f32(k);
        let dir = vec3<f32>(r * cos(a), y, r * sin(a));
        var depth = 0.0;
        for (var step = 1u; step < params.steps; step++) {
            let q = grid_cell(start + dir * (f32(step) * params.cell));
            if (!grid_inside(q)) {
                break;
            }
            depth += density[grid_index(q)] * params.cell;
            if (depth > 8.0) {
                break;
            }
        }
        let weight = 0.5 + 0.5 * max(dir.y, 0.0);
        sum += exp(-depth) * weight;
        norm += weight;
    }
    visibility[i] = sum / norm;
}
"#,
    )
}

/// Trilinear sampling of a grid of values, between cell centres.
const SAMPLE: &str = r#"
fn grid_sample(p: vec3<f32>) -> f32 {
    let q = (p - vec3<f32>(params.lo_x, params.lo_y, params.lo_z)) / params.cell - vec3<f32>(0.5);
    let base = floor(q);
    let f = q - base;
    var sum = 0.0;
    for (var corner = 0u; corner < 8u; corner++) {
        let o = vec3<u32>(corner & 1u, (corner >> 1u) & 1u, (corner >> 2u) & 1u);
        let c = clamp(vec3<i32>(base) + vec3<i32>(o), vec3<i32>(0), vec3<i32>(i32(params.nx) - 1, i32(params.ny) - 1, i32(params.nz) - 1));
        var w = 1.0;
        w *= select(1.0 - f.x, f.x, o.x == 1u);
        w *= select(1.0 - f.y, f.y, o.y == 1u);
        w *= select(1.0 - f.z, f.z, o.z == 1u);
        sum += w * grid_values[grid_index(c)];
    }
    return sum;
}
"#;

/// Pushing a coarse polyline's points out wherever the straight segments
/// between them come near the head: [`crate::collider::keep_clear`], in
/// place on the kernel's own output through its `clear_get` and `clear_set`.
///
/// Registers only, deliberately: the same sweep over a local
/// `array<vec3<f32>, 32>` (the points, and a second for the pushes) came out
/// differently from run to run on the same input -- the driver's handling of
/// large, dynamically indexed local arrays is not to be trusted. A point's
/// push is the longer of its two segments' (as the host's running maximum
/// has it), so a sweep needs only the segment behind and the one ahead.
const KEEP_CLEAR: &str = r#"
// The push segment `i`, from `a` to `b`, asks for, and whether it asks at all.
fn segment_push(i: u32, a: vec3<f32>, b: vec3<f32>, clearance: f32) -> vec4<f32> {
    // A distance field changes no faster than distance: a segment whose ends
    // are well clear cannot come near in between.
    if (min(head_distance(a), head_distance(b)) - 0.5 * distance(a, b) >= clearance) {
        return vec4<f32>(0.0);
    }
    var push = vec3<f32>(0.0);
    var asked = 0.0;
    for (var k = 1u; k <= 3u; k++) {
        let t = f32(k) * 0.25;
        let q = lerp3(a, b, t);
        let d = head_distance(q);
        if (d >= clearance) {
            continue;
        }
        asked = 1.0;
        let out = head_gradient(q);
        let need = clearance - d;
        // The root stays: a first segment is cleared by lifting its far end.
        var v = out * need;
        if (i == 0u) {
            v = out * min(need / t, 0.01);
        }
        if (dot(v, v) > dot(push, push)) {
            push = v;
        }
    }
    return vec4<f32>(push, asked);
}

fn keep_clear(n: u32, clearance: f32) {
    if (n < 2u) {
        return;
    }
    for (var sweep = 0u; sweep < 4u; sweep++) {
        // Point j's old position, its successor's, and what the segment
        // behind it asks: each sweep moves every point from the old ones.
        var here = clear_get(1u);
        let behind = segment_push(0u, clear_get(0u), here, clearance);
        var asked = behind.w;
        var push_behind = behind.xyz;
        for (var j = 1u; j < n; j++) {
            var ahead = vec4<f32>(0.0);
            var next = here;
            if (j + 1u < n) {
                next = clear_get(j + 1u);
                ahead = segment_push(j, here, next, clearance);
                asked = max(asked, ahead.w);
            }
            var push = push_behind;
            if (dot(ahead.xyz, ahead.xyz) > dot(push, push)) {
                push = ahead.xyz;
            }
            clear_set(j, here + push);
            push_behind = ahead.xyz;
            here = next;
        }
        if (asked == 0.0) {
            break;
        }
    }
}
"#;

/// One strand level, in the shuffled order: resampled to its point count,
/// kept off the head where coarse, with each point's sky visibility.
pub fn level() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> strands: array<StrandGpu>;
@group(0) @binding(2) var<storage, read> order: array<u32>;
@group(0) @binding(3) var<storage, read> grid_values: array<f32>;
@group(0) @binding(4) var<storage, read> head_shape: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> level_positions: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read_write> infos: array<vec4<f32>>;
struct Params {{
{GRID_PARAMS}
    points: u32,
    level_points: u32,
    body: u32,
    pad0: u32,
    clearance: f32,
    pad1: u32,
    pad2: u32,
    pad3: u32,
}};
@group(0) @binding(7) var<uniform> params: Params;

// This strand's first point in the output, for `keep_clear`.
var<private> clear_first: u32;
fn clear_get(k: u32) -> vec3<f32> {{
    return level_positions[clear_first + k].xyz;
}}
fn clear_set(k: u32, v: vec3<f32>) {{
    level_positions[clear_first + k] = vec4<f32>(v, 0.0);
}}
"#
        ),
        &[RANDOM, MATH, HEAD, GRID, SAMPLE, KEEP_CLEAR],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    if (i >= params.count) {
        return;
    }
    let s = order[i];
    let points = params.points;
    let n = params.level_points;
    let base = s * points;
    let strand = strands[s];
    infos[i] = vec4<f32>(strand.radius, unit(strand.id, 0x5EEDu), strand.uv);
    if (n == points) {
        for (var j = 0u; j < n; j++) {
            let p = positions[base + j].xyz;
            level_positions[i * n + j] = vec4<f32>(p, grid_sample(p));
        }
        return;
    }
    // Resampled evenly by arc length.
    var total = 0.0;
    for (var j = 1u; j < points; j++) {
        total += distance(positions[base + j].xyz, positions[base + j - 1u].xyz);
    }
    var segment = 0u;
    var segment_start = 0.0;
    var segment_end = distance(positions[base + 1u].xyz, positions[base].xyz);
    for (var j = 0u; j < n; j++) {
        let target_length = total * f32(j) / f32(max(n - 1u, 1u));
        while (segment + 2u < points && segment_end < target_length) {
            segment += 1u;
            segment_start = segment_end;
            segment_end += distance(positions[base + segment + 1u].xyz, positions[base + segment].xyz);
        }
        let span = segment_end - segment_start;
        var t = 0.0;
        if (span > 1e-12) {
            t = clamp((target_length - segment_start) / span, 0.0, 1.0);
        }
        let p = lerp3(positions[base + segment].xyz, positions[base + segment + 1u].xyz, t);
        level_positions[i * n + j] = vec4<f32>(p, 0.0);
    }
    clear_first = i * n;
    if (params.body != 0u) {
        keep_clear(n, params.clearance);
    }
    for (var j = 0u; j < n; j++) {
        let p = level_positions[i * n + j].xyz;
        level_positions[i * n + j] = vec4<f32>(p, grid_sample(p));
    }
}
"#,
    )
}

const CARD_PARAMS: &str = r#"
    count: u32,
    points: u32,
    segments: u32,
    min_strands: u32,
"#;

/// A lock's statistics, `4 + rows * 4` words a lock: its strand count, how
/// far its roots reach behind the card's first row, its thickest strand's
/// radius (its bits), a pad, then each row's position sum (three words, in
/// fixed point) and then each row's summed squared spread across the card.
const LOCKS: &str = r#"
fn lock_stride() -> u32 {
    return 4u + (params.segments + 1u) * 4u;
}

fn lock_spine_word(g: u32, j: u32, k: u32) -> u32 {
    return g * lock_stride() + 4u + j * 3u + k;
}

fn lock_spread_word(g: u32, j: u32) -> u32 {
    return g * lock_stride() + 4u + (params.segments + 1u) * 3u + j;
}
"#;

/// Each lock's strand count, and the sum of its strands' positions at
/// each row of its card (by share of the points, as the host samples).
pub fn card_accumulate() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> guide_of: array<u32>;
@group(0) @binding(2) var<storage, read_write> locks: array<atomic<i32>>;
struct Params {{
{CARD_PARAMS}
}};
@group(0) @binding(3) var<uniform> params: Params;
"#
        ),
        &[MATH, LOCKS],
        &format!(
            r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let s = item(id, groups);
    if (s >= params.count) {{
        return;
    }}
    let g = guide_of[s];
    atomicAdd(&locks[g * lock_stride()], 1);
    for (var j = 0u; j <= params.segments; j++) {{
        let x = f32(j) / f32(params.segments) * f32(params.points - 1u);
        let k = min(u32(x), params.points - 2u);
        let p = lerp3(positions[s * params.points + k].xyz, positions[s * params.points + k + 1u].xyz, x - f32(k));
        atomicAdd(&locks[lock_spine_word(g, j, 0u)], i32(round(p.x * {SPINE_FIXED:.1})));
        atomicAdd(&locks[lock_spine_word(g, j, 1u)], i32(round(p.y * {SPINE_FIXED:.1})));
        atomicAdd(&locks[lock_spine_word(g, j, 2u)], i32(round(p.z * {SPINE_FIXED:.1})));
    }}
}}
"#
        ),
    )
}

/// Which locks become cards: those of enough strands, flagged for a scan.
pub fn card_flags() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read> locks: array<i32>;
@group(0) @binding(1) var<storage, read_write> flags: array<u32>;
struct Params {{
{CARD_PARAMS}
}};
@group(0) @binding(2) var<uniform> params: Params;
"#
        ),
        &[MATH, LOCKS],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let s = item(id, groups);
    if (s >= params.count) {
        return;
    }
    flags[s] = select(0u, 1u, locks[s * lock_stride()] >= i32(max(params.min_strands, 1u)));
}
"#,
    )
}

const SPINE: &str = r#"
fn lock_count(g: u32) -> u32 {
    return u32(max(lock_value(g * lock_stride()), 1));
}

fn spine_at(g: u32, j: u32) -> vec3<f32> {
    let n = f32(lock_count(g));
    return vec3<f32>(
        f32(lock_value(lock_spine_word(g, j, 0u))),
        f32(lock_value(lock_spine_word(g, j, 1u))),
        f32(lock_value(lock_spine_word(g, j, 2u))),
    ) / (SPINE_FIXED * n);
}
"#;

/// How far each lock's strands spread across its card at each row, how far
/// its roots reach behind the card's first row, and its thickest strand.
pub fn card_spread() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> guide_of: array<u32>;
@group(0) @binding(2) var<storage, read_write> locks: array<atomic<i32>>;
@group(0) @binding(3) var<storage, read> strands: array<StrandGpu>;
struct Params {{
{CARD_PARAMS}
    center_x: f32,
    center_y: f32,
    center_z: f32,
    pad0: u32,
}};
@group(0) @binding(4) var<uniform> params: Params;
const SPINE_FIXED: f32 = {SPINE_FIXED:.1};
fn lock_value(i: u32) -> i32 {{
    return atomicLoad(&locks[i]);
}}
"#
        ),
        &[MATH, LOCKS, SPINE],
        &format!(
            r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let s = item(id, groups);
    if (s >= params.count) {{
        return;
    }}
    let g = guide_of[s];
    if (lock_count(g) < max(params.min_strands, 1u)) {{
        return;
    }}
    let center = vec3<f32>(params.center_x, params.center_y, params.center_z);
    for (var j = 0u; j <= params.segments; j++) {{
        let here = spine_at(g, j);
        let tangent = safe_normalize(spine_at(g, min(j + 1u, params.segments)) - spine_at(g, select(j - 1u, 0u, j == 0u)), vec3<f32>(0.0, -1.0, 0.0));
        let out = safe_normalize(here - center, vec3<f32>(0.0, 1.0, 0.0));
        let normal = safe_normalize(out - tangent * dot(out, tangent), vec3<f32>(0.0, 0.0, 1.0));
        let side = cross(tangent, normal);
        let x = f32(j) / f32(params.segments) * f32(params.points - 1u);
        let k = min(u32(x), params.points - 2u);
        let p = lerp3(positions[s * params.points + k].xyz, positions[s * params.points + k + 1u].xyz, x - f32(k));
        let lateral = dot(p - here, side);
        atomicAdd(&locks[lock_spread_word(g, j)], i32(min(lateral * lateral * {SPREAD_FIXED:.1}, 2e9)));
    }}
    let ahead = safe_normalize(spine_at(g, 1u) - spine_at(g, 0u), vec3<f32>(0.0, -1.0, 0.0));
    let back = max(dot(spine_at(g, 0u) - positions[s * params.points].xyz, ahead), 0.0);
    atomicMax(&locks[g * lock_stride() + 1u], i32(min(back * {BEHIND_FIXED:.1}, 2e9)));
    atomicMax(&locks[g * lock_stride() + 2u], bitcast<i32>(max(strands[s].radius, 0.0)));
}}
"#
        ),
    )
}

/// The cards: for each lock of enough strands, a strip along its mean path,
/// as wide as its strands spread, started behind its middle root, kept off
/// the head, lying flat against it -- vertices and indices at its slot.
pub fn card_build() -> String {
    kernel(
        &format!(
            r#"
{STRAND}
@group(0) @binding(0) var<storage, read> locks: array<i32>;
@group(0) @binding(1) var<storage, read> slots: array<u32>;
@group(0) @binding(2) var<storage, read> strands: array<StrandGpu>;
@group(0) @binding(3) var<storage, read> grid_values: array<f32>;
@group(0) @binding(4) var<storage, read> head_shape: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> card_vertices: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read_write> card_indices: array<u32>;
struct Params {{
    count: u32,
    points: u32,
    segments: u32,
    min_strands: u32,
    nx: u32,
    ny: u32,
    nz: u32,
    body: u32,
    lo_x: f32,
    lo_y: f32,
    lo_z: f32,
    cell: f32,
    center_x: f32,
    center_y: f32,
    center_z: f32,
    size: f32,
    width_scale: f32,
    variants: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(7) var<uniform> params: Params;
const SPINE_FIXED: f32 = {SPINE_FIXED:.1};
fn lock_value(i: u32) -> i32 {{
    return locks[i];
}}

// The spine, kept in each row's first vertex until the row is built.
var<private> clear_first: u32;
fn clear_get(k: u32) -> vec3<f32> {{
    return card_vertices[(clear_first + k * 2u) * 4u].xyz;
}}
fn clear_set(k: u32, v: vec3<f32>) {{
    card_vertices[(clear_first + k * 2u) * 4u] = vec4<f32>(v, 0.0);
}}

// Row `j`'s width: the 85th percentile of an even spread is 1.47 of its RMS.
fn row_width(g: u32, j: u32, radius: f32, n: f32) -> f32 {{
    let rms = sqrt(f32(locks[lock_spread_word(g, j)]) / {SPREAD_FIXED:.1} / n);
    return clamp(2.0 * 1.47 * rms + 4.0 * radius, 0.0015, params.size * 3.0);
}}
"#
        ),
        &[RANDOM, MATH, HEAD, GRID, SAMPLE, LOCKS, SPINE, KEEP_CLEAR],
        &format!(
            r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let g = item(id, groups);
    if (g >= params.count || lock_count(g) < max(params.min_strands, 1u) || locks[g * lock_stride()] <= 0) {{
        return;
    }}
    let slot = slots[g];
    let segments = params.segments;
    let rows = segments + 1u;
    let first = slot * rows * 2u;
    clear_first = first;
    for (var j = 0u; j < rows; j++) {{
        clear_set(j, spine_at(g, j));
    }}
    // Back from the middle root over the roots behind it, on the scalp.
    var root = clear_get(0u);
    let ahead = safe_normalize(clear_get(1u) - root, vec3<f32>(0.0, -1.0, 0.0));
    let back = min(f32(locks[g * lock_stride() + 1u]) / {BEHIND_FIXED:.1}, params.size);
    root -= ahead * back;
    if (params.body != 0u) {{
        root = push_out(root, 0.0005);
    }}
    clear_set(0u, root);
    if (params.body != 0u) {{
        keep_clear(rows, 0.0015);
    }}
    let radius = bitcast<f32>(locks[g * lock_stride() + 2u]);
    let n = f32(lock_count(g));
    let center = vec3<f32>(params.center_x, params.center_y, params.center_z);
    let sid = strands[g].id;
    let variant = f32(hash2(sid, 0x7Au) % max(params.variants, 1u));
    let random = unit(sid, 0x7Bu);
    // Each row overwrites its spine point: the one behind is carried along.
    var behind = clear_get(0u);
    var here = behind;
    var width_behind = row_width(g, 0u, radius, n);
    var width_here = width_behind;
    for (var j = 0u; j < rows; j++) {{
        let next = clear_get(min(j + 1u, segments));
        var width_next = width_here;
        if (j < segments) {{
            width_next = row_width(g, j + 1u, radius, n);
        }}
        var width = width_here;
        if (j > 0u && j < segments) {{
            width = (width_behind + 2.0 * width_here + width_next) * 0.25;
        }}
        let tangent = safe_normalize(next - behind, vec3<f32>(0.0, -1.0, 0.0));
        let out = safe_normalize(here - center, vec3<f32>(0.0, 1.0, 0.0));
        let normal = safe_normalize(out - tangent * dot(out, tangent), vec3<f32>(0.0, 0.0, 1.0));
        let side = cross(tangent, normal);
        let half_width = width * 0.5 * params.width_scale;
        let occlusion = grid_sample(here + normal * 0.002);
        let v = f32(j) / f32(segments);
        for (var e = 0u; e < 2u; e++) {{
            let sign = select(-1.0, 1.0, e == 1u);
            let vertex = (first + j * 2u + e) * 4u;
            card_vertices[vertex] = vec4<f32>(here + side * (half_width * sign), f32(e));
            card_vertices[vertex + 1u] = vec4<f32>(tangent, v);
            card_vertices[vertex + 2u] = vec4<f32>(normal, variant);
            card_vertices[vertex + 3u] = vec4<f32>(random, occlusion, 0.0, 0.0);
        }}
        behind = here;
        here = next;
        width_behind = width_here;
        width_here = width_next;
    }}
    for (var j = 0u; j < segments; j++) {{
        let a = first + 2u * j;
        let index = (slot * segments + j) * 6u;
        card_indices[index] = a;
        card_indices[index + 1u] = a + 2u;
        card_indices[index + 2u] = a + 1u;
        card_indices[index + 3u] = a + 1u;
        card_indices[index + 4u] = a + 2u;
        card_indices[index + 5u] = a + 3u;
    }}
}}
"#
        ),
    )
}

/// Roots around each vertex of the head (from the hash grid over roots),
/// and their totals for the mean.
pub fn cover_count() -> String {
    use fabelgeist_spatial_hash::gpu::{HASH_WGSL, RANGE_WGSL};
    format!(
        r#"
@group(0) @binding(0) var<storage, read> points: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> hash_indices: array<u32>;
@group(0) @binding(2) var<storage, read> hash_ranges: array<u32>;
@group(0) @binding(3) var<storage, read> head_points: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> root_counts: array<f32>;
@group(0) @binding(5) var<storage, read_write> totals: array<atomic<u32>>;
struct Params {{
    count: u32,
    table_size: u32,
    inverse_cell: f32,
    pad: u32,
    vertices: u32,
    radius: f32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(6) var<uniform> params: Params;
{HASH_WGSL}
{RANGE_WGSL}
{INDEX}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let v = item(id, groups);
    if (v >= params.vertices) {{
        return;
    }}
    let p = head_points[v].xyz;
    let base = spatial_hash_cell(p, params.inverse_cell);
    var count = 0u;
    for (var n = 0u; n < 27u; n++) {{
        let cell = base + vec3<i32>(i32(n % 3u) - 1, i32((n / 3u) % 3u) - 1, i32(n / 9u) - 1);
        let range = spatial_hash_range(cell, params.table_size);
        for (var k = range.x; k < range.y; k++) {{
            let q = points[hash_indices[k]].xyz;
            if (any(spatial_hash_cell(q, params.inverse_cell) != cell)) {{
                continue;
            }}
            if (distance(q, p) <= params.radius) {{
                count += 1u;
            }}
        }}
    }}
    root_counts[v] = f32(count);
    if (count > 0u) {{
        atomicAdd(&totals[0], count);
        atomicAdd(&totals[1], 1u);
    }}
}}
"#
    )
}

/// How much hair covers each vertex: its roots against the mean of the
/// growing scalp, and the hair over it (a line straight out through the
/// density grid).
pub fn cover_finish() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read> head_points: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> head_normals: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> root_counts: array<f32>;
@group(0) @binding(3) var<storage, read> totals: array<u32>;
@group(0) @binding(4) var<storage, read> grid_values: array<f32>;
@group(0) @binding(5) var<storage, read_write> cover: array<vec2<f32>>;
struct Params {{
{GRID_PARAMS}
    vertices: u32,
    reach: f32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(6) var<uniform> params: Params;
"#
        ),
        &[GRID, SAMPLE],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let v = item(id, groups);
    if (v >= params.vertices) {
        return;
    }
    // Full growth: a little over the mean where roots grow.
    let mean = f32(totals[0]) / f32(max(totals[1], 1u));
    let full = max(1.25 * mean, 1.0);
    let grown = min(root_counts[v] / full, 1.0);
    let p = head_points[v].xyz;
    let n = head_normals[v].xyz;
    let steps = 8u;
    let ds = params.reach / f32(steps);
    var depth = 0.0;
    for (var k = 0u; k < steps; k++) {
        depth += grid_sample(p + n * (0.002 + (f32(k) + 0.5) * ds)) * ds;
    }
    let over = 1.0 - exp(-depth);
    cover[v] = vec2<f32>(grown, max(grown, over));
}
"#,
    )
}

/// One sweep of smoothing the cover over the head's mesh: each vertex the
/// mean of itself and its neighbours.
pub fn cover_smooth() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> adjacency_start: array<u32>;
@group(0) @binding(1) var<storage, read> adjacency: array<u32>;
@group(0) @binding(2) var<storage, read> cover_in: array<vec2<f32>>;
@group(0) @binding(3) var<storage, read_write> cover_out: array<vec2<f32>>;
struct Params {
    vertices: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(4) var<uniform> params: Params;
"#,
        &[],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let v = item(id, groups);
    if (v >= params.vertices) {
        return;
    }
    var sum = cover_in[v];
    var n = 1.0;
    for (var k = adjacency_start[v]; k < adjacency_start[v + 1u]; k++) {
        sum += cover_in[adjacency[k]];
        n += 1.0;
    }
    cover_out[v] = sum / n;
}
"#,
    )
}

/// The box every point is in: each coordinate's least and greatest, as
/// ordered words (a float's bits made to sort as unsigned integers), the
/// least inverted so that a cleared buffer starts every slot empty.
pub fn bounds() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> box_words: array<atomic<u32>>;
struct Params {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(2) var<uniform> params: Params;
"#,
        &[],
        r#"
fn ordered(x: f32) -> u32 {
    let bits = bitcast<u32>(x);
    return select(bits | 0x80000000u, ~bits, (bits & 0x80000000u) != 0u);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    if (i >= params.count) {
        return;
    }
    let p = positions[i].xyz;
    for (var k = 0u; k < 3u; k++) {
        let key = ordered(p[k]);
        atomicMax(&box_words[k], ~key);
        atomicMax(&box_words[3u + k], key);
    }
}
"#,
    )
}
