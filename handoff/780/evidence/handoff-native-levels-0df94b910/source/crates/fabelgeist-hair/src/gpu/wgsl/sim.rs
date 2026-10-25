//! Real-time physics' kernels: [`crate::sim::HairSim`] on the GPU.

use super::common::{HEAD, INDEX, MATH, NOISE, RANDOM};

fn kernel(bindings: &str, helpers: &[&str], body: &str) -> String {
    let mut source = String::from(bindings);
    for h in helpers {
        source.push_str(h);
    }
    source.push_str(INDEX);
    source.push_str(body);
    source
}

/// Each drawn strand's four simulated strands, nearest its root, weighted
/// by inverse square distance. With a shared source, the simulated prefix
/// follows itself; transfers between sources compare their actual roots.
/// The nearest four are kept sorted in registers, not a local array.
pub fn sim_bind() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> level: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> guides: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> ids: array<vec4<u32>>;
@group(0) @binding(3) var<storage, read_write> weights: array<vec4<f32>>;
struct Params {
    strands: u32,
    points: u32,
    count: u32,
    guide_points: u32,
    shared_prefix: u32,
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
    let s = item(id, groups);
    if (s >= params.strands) {
        return;
    }
    if (params.shared_prefix != 0u && s < params.count) {
        ids[s] = vec4<u32>(s, 0u, 0u, 0u);
        weights[s] = vec4<f32>(1.0, 0.0, 0.0, 0.0);
        return;
    }
    let root = level[s * params.points].xyz;
    let far = 1e30;
    var best = vec4<f32>(far);
    var near = vec4<u32>(0u);
    for (var g = 0u; g < params.count; g++) {
        let d = distance(root, guides[g * params.guide_points].xyz);
        if (d >= best.w) {
            continue;
        }
        if (d < best.z) {
            best.w = best.z;
            near.w = near.z;
            if (d < best.y) {
                best.z = best.y;
                near.z = near.y;
                if (d < best.x) {
                    best.y = best.x;
                    near.y = near.x;
                    best.x = d;
                    near.x = g;
                } else {
                    best.y = d;
                    near.y = g;
                }
            } else {
                best.z = d;
                near.z = g;
            }
        } else {
            best.w = d;
            near.w = g;
        }
    }
    var w = vec4<f32>(1.0) / (best * best + vec4<f32>(1e-8));
    w = select(vec4<f32>(0.0), w, best < vec4<f32>(far));
    ids[s] = near;
    weights[s] = w / max(w.x + w.y + w.z + w.w, 1e-12);
}
"#,
    )
}

const RIGID: &str = r#"
fn head_apply(p: vec3<f32>) -> vec3<f32> {
    return head_rotate(p) + vec3<f32>(params.tx, params.ty, params.tz);
}

fn head_rotate(v: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(params.ax_x, params.ax_y, params.ax_z) * v.x
        + vec3<f32>(params.ay_x, params.ay_y, params.ay_z) * v.y
        + vec3<f32>(params.az_x, params.az_y, params.az_z) * v.z;
}

// From the world into the head's frame.
fn head_unapply(p: vec3<f32>) -> vec3<f32> {
    let q = p - vec3<f32>(params.tx, params.ty, params.tz);
    return vec3<f32>(
        dot(vec3<f32>(params.ax_x, params.ax_y, params.ax_z), q),
        dot(vec3<f32>(params.ay_x, params.ay_y, params.ay_z), q),
        dot(vec3<f32>(params.az_x, params.az_y, params.az_z), q),
    );
}
"#;

const RIGID_PARAMS: &str = r#"
    ax_x: f32,
    ax_y: f32,
    ax_z: f32,
    ay_x: f32,
    ay_y: f32,
    ay_z: f32,
    az_x: f32,
    az_y: f32,
    az_z: f32,
    tx: f32,
    ty: f32,
    tz: f32,
"#;

/// The simulated strands' rest: each of the first `count` strands of the
/// groom's finest level (`level`, `level_points` points a strand)
/// resampled evenly by arc length to `points`, in the head's frame (the
/// groom's), `w` one; and the longest segment, as its bits, in `longest`.
pub fn sim_rest() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> level: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> rest: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> longest: array<atomic<u32>>;
struct Params {
    count: u32,
    points: u32,
    level_points: u32,
    pad0: u32,
};
@group(0) @binding(3) var<uniform> params: Params;
"#,
        &[MATH],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let g = item(id, groups);
    if (g >= params.count) {
        return;
    }
    let from_points = params.level_points;
    let base = g * from_points;
    var total = 0.0;
    for (var j = 1u; j < from_points; j++) {
        total += distance(level[base + j].xyz, level[base + j - 1u].xyz);
    }
    let points = params.points;
    var segment = 0u;
    var segment_start = 0.0;
    var segment_end = distance(level[base + 1u].xyz, level[base].xyz);
    for (var j = 0u; j < points; j++) {
        let target_length = total * f32(j) / f32(max(points - 1u, 1u));
        while (segment + 2u < from_points && segment_end < target_length) {
            segment += 1u;
            segment_start = segment_end;
            segment_end += distance(level[base + segment + 1u].xyz, level[base + segment].xyz);
        }
        let span = segment_end - segment_start;
        var t = 0.0;
        if (span > 1e-12) {
            t = clamp((target_length - segment_start) / span, 0.0, 1.0);
        }
        rest[g * points + j] = vec4<f32>(lerp3(level[base + segment].xyz, level[base + segment + 1u].xyz, t), 1.0);
    }
    // Evenly resampled: every segment is as long.
    atomicMax(&longest[0], bitcast<u32>(total / f32(max(points - 1u, 1u))));
}
"#,
    )
}

/// Back to the groom, the head where it is: both positions from the rest.
pub fn sim_reset() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read> rest: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> position: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> previous: array<vec4<f32>>;
struct Params {{
{RIGID_PARAMS}
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
"#
        ),
        &[RIGID],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    if (i >= params.count) {
        return;
    }
    let p = vec4<f32>(head_apply(rest[i].xyz), 1.0);
    position[i] = p;
    previous[i] = p;
}
"#,
    )
}

/// A new level's simulation from the old one's: each new strand's points
/// at its rest, moved as the four old strands nearest its root have moved
/// at the same share of their length (a strand the old level simulated
/// too follows only itself), its previous points likewise, so it carries
/// on as it was moving.
pub fn sim_transfer() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read> rest: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> position: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> previous: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> old_rest: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> old_position: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> old_previous: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read> ids: array<vec4<u32>>;
@group(0) @binding(7) var<storage, read> weights: array<vec4<f32>>;
struct Params {{
{RIGID_PARAMS}
    count: u32,
    points: u32,
    old_points: u32,
    pad0: u32,
}};
@group(0) @binding(8) var<uniform> params: Params;
"#
        ),
        &[MATH, RIGID],
        r#"
// Strand `g` of the old level at `share` of its points.
fn old_at(buffer: u32, g: u32, share: f32) -> vec3<f32> {
    let f = share * f32(params.old_points - 1u);
    let i = min(u32(floor(f)), params.old_points - 2u);
    let t = f - f32(i);
    let a = g * params.old_points + i;
    switch buffer {
        case 0u: { return lerp3(old_rest[a].xyz, old_rest[a + 1u].xyz, t); }
        case 1u: { return lerp3(old_position[a].xyz, old_position[a + 1u].xyz, t); }
        default: { return lerp3(old_previous[a].xyz, old_previous[a + 1u].xyz, t); }
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    if (i >= params.count * params.points) {
        return;
    }
    let g = i / params.points;
    let share = f32(i % params.points) / f32(max(params.points - 1u, 1u));
    let here = head_apply(rest[i].xyz);
    var moved = vec3<f32>(0.0);
    var before = vec3<f32>(0.0);
    let near = ids[g];
    let w = weights[g];
    for (var k = 0u; k < 4u; k++) {
        if (w[k] <= 0.0) {
            continue;
        }
        let old_rest_here = head_apply(old_at(0u, near[k], share));
        moved += (old_at(1u, near[k], share) - old_rest_here) * w[k];
        before += (old_at(2u, near[k], share) - old_rest_here) * w[k];
    }
    position[i] = vec4<f32>(here + moved, 1.0);
    previous[i] = vec4<f32>(here + before, 1.0);
}
"#,
    )
}

/// The density grid's parameters: `lo` and `cell` of an `nx` by `ny` by
/// `nz` grid of cells in the head's frame.
const GRID_PARAMS: &str = r#"
    lo_x: f32,
    lo_y: f32,
    lo_z: f32,
    cell: f32,
    nx: u32,
    ny: u32,
    nz: u32,
"#;

/// Trilinear weights over cell centres: the eight cells around a point.
const GRID: &str = r#"
const DENSITY_FIXED: f32 = 65536.0;
const MOMENTUM_FIXED: f32 = 10000000.0;

fn grid_base(local: vec3<f32>) -> vec3<f32> {
    return (local - vec3<f32>(params.lo_x, params.lo_y, params.lo_z)) / params.cell - vec3<f32>(0.5);
}

// The `corner`th of the eight cells around `f` (from `grid_base`), its
// index (or -1 outside the grid) and its weight.
fn grid_corner(f: vec3<f32>, corner: u32) -> vec2<f32> {
    let i0 = floor(f);
    let t = f - i0;
    let o = vec3<f32>(f32(corner & 1u), f32((corner >> 1u) & 1u), f32((corner >> 2u) & 1u));
    let c = vec3<i32>(i0 + o);
    let w = select(1.0 - t, t, o > vec3<f32>(0.5));
    let n = vec3<i32>(i32(params.nx), i32(params.ny), i32(params.nz));
    if (any(c < vec3<i32>(0)) || any(c >= n)) {
        return vec2<f32>(-1.0, 0.0);
    }
    let index = (c.z * n.y + c.y) * n.x + c.x;
    return vec2<f32>(f32(index), w.x * w.y * w.z);
}
"#;

/// The groom's own density at rest, splatted into `rest_density` (fixed
/// point) from the rest points, which are in the head's frame already.
pub fn sim_rest_density() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read> rest: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> rest_density: array<atomic<u32>>;
struct Params {{
{GRID_PARAMS}
    count: u32,
}};
@group(0) @binding(2) var<uniform> params: Params;
"#
        ),
        &[GRID],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    if (i >= params.count) {
        return;
    }
    let f = grid_base(rest[i].xyz);
    for (var corner = 0u; corner < 8u; corner++) {
        let c = grid_corner(f, corner);
        if (c.x >= 0.0) {
            atomicAdd(&rest_density[u32(c.x)], u32(c.y * DENSITY_FIXED));
        }
    }
}
"#,
    )
}

/// The rest density's sum (in 256ths of a point) and how many cells hold
/// any: their mean is the density hair has where there is hair.
pub fn sim_rest_stats() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> rest_density: array<u32>;
@group(0) @binding(1) var<storage, read_write> stats: array<atomic<u32>>;
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
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let c = item(id, groups);
    if (c >= params.count || rest_density[c] == 0u) {
        return;
    }
    atomicAdd(&stats[0], rest_density[c] >> 8u);
    atomicAdd(&stats[1], 1u);
}
"#,
    )
}

/// Every point's density and momentum (its velocity a substep) splatted
/// into the grid, in the head's frame: four words a cell, density then
/// momentum, in fixed point.
pub fn sim_splat() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read> position: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> previous: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> grid_words: array<atomic<i32>>;
struct Params {{
{RIGID_PARAMS}
{GRID_PARAMS}
    count: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
"#
        ),
        &[RIGID, GRID],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    if (i >= params.count) {
        return;
    }
    let x = position[i].xyz;
    let v = x - previous[i].xyz;
    let f = grid_base(head_unapply(x));
    for (var corner = 0u; corner < 8u; corner++) {
        let c = grid_corner(f, corner);
        if (c.x < 0.0) {
            continue;
        }
        let at = u32(c.x) * 4u;
        atomicAdd(&grid_words[at], i32(c.y * DENSITY_FIXED));
        let m = v * (c.y * MOMENTUM_FIXED);
        atomicAdd(&grid_words[at + 1u], i32(m.x));
        atomicAdd(&grid_words[at + 2u], i32(m.y));
        atomicAdd(&grid_words[at + 3u], i32(m.z));
    }
}
"#,
    )
}

/// Volume and friction, a point an invocation (roots stay): pushed down
/// the gradient of how much denser the hair is than it may be -- than at
/// rest, or than hair's typical density where there was none, and a
/// tolerance more -- and its velocity drawn towards the grid's there.
pub fn sim_volume() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read_write> position: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> previous: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> grid_words: array<i32>;
@group(0) @binding(3) var<storage, read> rest_density: array<u32>;
@group(0) @binding(4) var<storage, read> stats: array<u32>;
struct Params {{
{RIGID_PARAMS}
{GRID_PARAMS}
    count: u32,
    points: u32,
    volume: f32,
    tolerance: f32,
    friction: f32,
}};
@group(0) @binding(5) var<uniform> params: Params;
"#
        ),
        &[MATH, RIGID, GRID],
        r#"
// The density at `local`, and what it may be there.
fn densities(local: vec3<f32>, typical: f32) -> vec2<f32> {
    let f = grid_base(local);
    var now = 0.0;
    var rest = 0.0;
    for (var corner = 0u; corner < 8u; corner++) {
        let c = grid_corner(f, corner);
        if (c.x < 0.0) {
            continue;
        }
        let index = u32(c.x);
        now += f32(grid_words[index * 4u]) * c.y;
        rest += f32(rest_density[index]) * c.y;
    }
    let allowed = max(rest / DENSITY_FIXED, typical) * (1.0 + params.tolerance);
    return vec2<f32>(now / DENSITY_FIXED, allowed);
}

fn excess(local: vec3<f32>, typical: f32) -> f32 {
    let d = densities(local, typical);
    return max(d.x - d.y, 0.0);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    if (i >= params.count || i % params.points == 0u) {
        return;
    }
    let x = position[i].xyz;
    let v = x - previous[i].xyz;
    let local = head_unapply(x);
    // Points a cell where there is hair, from the rest.
    let typical = f32(stats[0]) * 256.0 / DENSITY_FIXED / max(f32(stats[1]), 1.0);
    let here = densities(local, typical);
    var push = vec3<f32>(0.0);
    if (params.volume > 0.0 && here.x > here.y) {
        // The excess's gradient across a cell, as a share of what is
        // allowed: how far, in cells, to move.
        let h = params.cell * 0.5;
        let g = vec3<f32>(
            excess(local + vec3<f32>(h, 0.0, 0.0), typical) - excess(local - vec3<f32>(h, 0.0, 0.0), typical),
            excess(local + vec3<f32>(0.0, h, 0.0), typical) - excess(local - vec3<f32>(0.0, h, 0.0), typical),
            excess(local + vec3<f32>(0.0, 0.0, h), typical) - excess(local - vec3<f32>(0.0, 0.0, h), typical),
        ) / max(here.y, 1e-3);
        push = -g * (params.volume * params.cell);
        let reach = 0.5 * params.cell;
        let l = length(push);
        if (l > reach) {
            push *= reach / l;
        }
        push = head_rotate(push);
    }
    var velocity = v;
    if (params.friction > 0.0) {
        // The hair's mean velocity here.
        let f = grid_base(local);
        var m = vec3<f32>(0.0);
        var d = 0.0;
        for (var corner = 0u; corner < 8u; corner++) {
            let c = grid_corner(f, corner);
            if (c.x < 0.0) {
                continue;
            }
            let at = u32(c.x) * 4u;
            d += f32(grid_words[at]) * c.y;
            m += vec3<f32>(f32(grid_words[at + 1u]), f32(grid_words[at + 2u]), f32(grid_words[at + 3u])) * c.y;
        }
        if (d > 0.0) {
            let mean = m / MOMENTUM_FIXED / (d / DENSITY_FIXED);
            velocity = lerp3(v, mean, params.friction);
        }
    }
    // The velocity drawn to the hair's; the push moves the point, and so
    // carries on as velocity, a pressure.
    position[i] = vec4<f32>(x + push, 1.0);
    previous[i] = vec4<f32>(x - velocity, 1.0);
}
"#,
    )
}

/// The collision grid's parameters: `lo` and `cell` of an `nx` by `ny` by
/// `nz` grid in the head's frame.
const CELL_PARAMS: &str = r#"
    cell_lo_x: f32,
    cell_lo_y: f32,
    cell_lo_z: f32,
    cell_size: f32,
    cell_nx: u32,
    cell_ny: u32,
    cell_nz: u32,
"#;

/// Points along the colliding strands' segments, `per_segment` a segment,
/// to find which segments are near which: each sample's position (`w` its
/// strand), what the narrow phase needs of it -- its segment's ends now
/// (`w` the strand) and at rest, its velocity a substep -- and its cell in
/// the collision grid (in the head's frame, clamped to it) with its place
/// among the cell's samples, counted.
pub fn sim_samples() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read> position: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> previous: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> rest: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> samples: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> sample_data: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> cell_counts: array<atomic<u32>>;
@group(0) @binding(6) var<storage, read_write> sample_slots: array<u32>;
struct Params {{
{RIGID_PARAMS}
{CELL_PARAMS}
    count: u32,
    points: u32,
    per_segment: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(7) var<uniform> params: Params;
"#
        ),
        &[MATH, RIGID],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let k = item(id, groups);
    if (k >= params.count) {
        return;
    }
    let segment = k / params.per_segment;
    let segments = params.points - 1u;
    let g = segment / segments;
    let a = g * params.points + segment % segments;
    let t = (f32(k % params.per_segment) + 0.5) / f32(params.per_segment);
    let strand = f32(g);
    let here = lerp3(position[a].xyz, position[a + 1u].xyz, t);
    samples[k] = vec4<f32>(here, strand);
    let va = position[a].xyz - previous[a].xyz;
    let vb = position[a + 1u].xyz - previous[a + 1u].xyz;
    let local = head_unapply(here);
    let n = vec3<i32>(i32(params.cell_nx), i32(params.cell_ny), i32(params.cell_nz));
    let c = clamp(
        vec3<i32>(floor((local - vec3<f32>(params.cell_lo_x, params.cell_lo_y, params.cell_lo_z)) / params.cell_size)),
        vec3<i32>(0),
        n - vec3<i32>(1),
    );
    let cell = u32((c.z * n.y + c.y) * n.x + c.x);
    sample_data[k * 5u] = vec4<f32>(position[a].xyz, strand);
    sample_data[k * 5u + 1u] = vec4<f32>(position[a + 1u].xyz, 0.0);
    sample_data[k * 5u + 2u] = vec4<f32>(rest[a].xyz, 0.0);
    sample_data[k * 5u + 3u] = vec4<f32>(rest[a + 1u].xyz, 0.0);
    sample_data[k * 5u + 4u] = vec4<f32>(lerp3(va, vb, t), f32(cell));
    sample_slots[k * 2u] = cell;
    sample_slots[k * 2u + 1u] = atomicAdd(&cell_counts[cell], 1u);
}
"#,
    )
}

/// The samples scattered into the collision grid's order (`cell_offsets`,
/// the counts scanned), so that a cell's samples lie together in memory and
/// neighbouring invocations look about the same place: eight words a
/// sample -- where it is (`w` its strand), where it was at the last pass,
/// its segment's ends now and at rest, its velocity (`w` its cell) -- and
/// which sample it is.
pub fn sim_sort_samples() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> samples: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> samples_before: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> sample_data: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> sample_slots: array<u32>;
@group(0) @binding(4) var<storage, read> cell_offsets: array<u32>;
@group(0) @binding(5) var<storage, read_write> sorted: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read_write> sorted_index: array<u32>;
struct Params {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(7) var<uniform> params: Params;
"#,
        &[],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let k = item(id, groups);
    if (k >= params.count) {
        return;
    }
    let r = cell_offsets[sample_slots[k * 2u]] + sample_slots[k * 2u + 1u];
    sorted_index[r] = k;
    sorted[r * 8u] = samples[k];
    sorted[r * 8u + 1u] = samples_before[k];
    for (var w = 0u; w < 5u; w++) {
        sorted[r * 8u + 2u + w] = sample_data[k * 5u + w];
    }
}
"#,
    )
}

/// The strands' collisions, a sample an invocation (in the grid's order).
/// Samples find which segments are near which; each pair of segments
/// within the radius is then taken as it is: its closest points, how far
/// apart, and the normal between them. A pair may come as near as the
/// radius, or as near as the groom had the two segments at rest if nearer
/// (hair lying on hair stays as it lies); nearer than that, each side is to
/// move half the shortfall apart -- to the side its samples were on at the
/// last pass, so a pair that crossed in between goes back -- without
/// gaining speed, and lose its half of the closing velocity, the sliding
/// slowed by Coulomb friction. Each sample writes the mean of its asks, two
/// words: the move and whether it asks, then the velocity change. No
/// atomics: a point's samples are gathered after.
pub fn sim_collide() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> sorted: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> sorted_index: array<u32>;
@group(0) @binding(2) var<storage, read> cell_offsets: array<u32>;
@group(0) @binding(3) var<storage, read_write> asks: array<vec4<f32>>;
struct Params {
    count: u32,
    cell_nx: u32,
    cell_ny: u32,
    cell_nz: u32,
    radius: f32,
    reach: f32,
    friction: f32,
    have_before: u32,
};
@group(0) @binding(4) var<uniform> params: Params;
"#,
        &[MATH],
        r#"
// The closest points of segments `p1 q1` and `p2 q2` (Ericson's), as the
// two parameters along them.
fn closest_parameters(p1: vec3<f32>, q1: vec3<f32>, p2: vec3<f32>, q2: vec3<f32>) -> vec2<f32> {
    let d1 = q1 - p1;
    let d2 = q2 - p2;
    let r = p1 - p2;
    let a = dot(d1, d1);
    let e = dot(d2, d2);
    let f = dot(d2, r);
    if (a <= 1e-12 && e <= 1e-12) {
        return vec2<f32>(0.0);
    }
    if (a <= 1e-12) {
        return vec2<f32>(0.0, clamp(f / e, 0.0, 1.0));
    }
    let c = dot(d1, r);
    if (e <= 1e-12) {
        return vec2<f32>(clamp(-c / a, 0.0, 1.0), 0.0);
    }
    let b = dot(d1, d2);
    let denom = a * e - b * b;
    var s = 0.0;
    if (denom > 1e-12) {
        s = clamp((b * f - c * e) / denom, 0.0, 1.0);
    }
    var t = (b * s + f) / e;
    if (t < 0.0) {
        t = 0.0;
        s = clamp(-c / a, 0.0, 1.0);
    } else if (t > 1.0) {
        t = 1.0;
        s = clamp((b - c) / a, 0.0, 1.0);
    }
    return vec2<f32>(s, t);
}

fn segment_distance(p1: vec3<f32>, q1: vec3<f32>, p2: vec3<f32>, q2: vec3<f32>) -> f32 {
    let st = closest_parameters(p1, q1, p2, q2);
    return distance(lerp3(p1, q1, st.x), lerp3(p2, q2, st.y));
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let r = item(id, groups);
    if (r >= params.count) {
        return;
    }
    let own = sorted[r * 8u];
    let own_before = sorted[r * 8u + 1u].xyz;
    let a1 = sorted[r * 8u + 2u].xyz;
    let b1 = sorted[r * 8u + 3u].xyz;
    let rest_a1 = sorted[r * 8u + 4u].xyz;
    let rest_b1 = sorted[r * 8u + 5u].xyz;
    let own_velocity = sorted[r * 8u + 6u].xyz;
    let p = own.xyz;
    let reach2 = params.reach * params.reach;
    let n = vec3<i32>(i32(params.cell_nx), i32(params.cell_ny), i32(params.cell_nz));
    let own_cell = i32(sorted[r * 8u + 6u].w);
    let base = vec3<i32>(own_cell % n.x, (own_cell / n.x) % n.y, own_cell / (n.x * n.y));
    var shift = vec3<f32>(0.0);
    var change = vec3<f32>(0.0);
    var contacts = 0u;
    for (var m = 0u; m < 27u; m++) {
        let c = base + vec3<i32>(i32(m % 3u) - 1, i32((m / 3u) % 3u) - 1, i32(m / 9u) - 1);
        if (any(c < vec3<i32>(0)) || any(c >= n)) {
            continue;
        }
        let cell = u32((c.z * n.y + c.y) * n.x + c.x);
        let last = cell_offsets[cell + 1u];
        for (var o = cell_offsets[cell]; o < last; o++) {
            let q = sorted[o * 8u];
            let near = p - q.xyz;
            if (dot(near, near) >= reach2 || q.w == own.w) {
                continue;
            }
            // The segments themselves.
            let a2 = sorted[o * 8u + 2u].xyz;
            let b2 = sorted[o * 8u + 3u].xyz;
            let st = closest_parameters(a1, b1, a2, b2);
            let apart = lerp3(a1, b1, st.x) - lerp3(a2, b2, st.y);
            let d = length(apart);
            if (d >= params.radius) {
                continue;
            }
            let allowed = min(
                params.radius,
                segment_distance(rest_a1, rest_b1, sorted[o * 8u + 4u].xyz, sorted[o * 8u + 5u].xyz),
            );
            // Between the segments, to the side the samples were on.
            var normal = safe_normalize(apart, safe_normalize(near, vec3<f32>(0.0, 1.0, 0.0)));
            if (params.have_before != 0u && dot(normal, own_before - sorted[o * 8u + 1u].xyz) < 0.0) {
                normal = -normal;
            }
            let gap = dot(apart, normal);
            if (gap >= allowed) {
                continue;
            }
            let relative = own_velocity - sorted[o * 8u + 6u].xyz;
            let closing = dot(relative, normal);
            // Hair lying on hair as at rest, to rounding, and still: nothing
            // to ask.
            let short = allowed - gap;
            if (short < 0.02 * params.radius && closing >= 0.0) {
                continue;
            }
            shift += normal * (0.5 * short);
            if (closing < 0.0) {
                var dv = -0.5 * closing * normal;
                let sliding = relative - closing * normal;
                let s = length(sliding);
                if (s > 1e-9) {
                    dv -= sliding * min(params.friction * (-0.5 * closing) / s, 0.5);
                }
                change += dv;
            }
            contacts += 1u;
        }
    }
    let k = sorted_index[r];
    if (contacts == 0u) {
        asks[k * 2u] = vec4<f32>(0.0);
        return;
    }
    let mean = 1.0 / f32(contacts);
    asks[k * 2u] = vec4<f32>(shift * mean, 1.0);
    asks[k * 2u + 1u] = vec4<f32>(change * mean, 0.0);
}
"#,
    )
}

/// The collisions applied, a point an invocation (roots stay): the asks of
/// the samples on its two segments, weighted by how near each is, averaged
/// over those that ask; it moves, with the point it was a substep ago so
/// the move is not speed, and its velocity changes.
pub fn sim_collide_apply() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read_write> position: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> previous: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> asks: array<vec4<f32>>;
struct Params {
    count: u32,
    points: u32,
    per_segment: u32,
    pad0: u32,
};
@group(0) @binding(3) var<uniform> params: Params;
"#,
        &[],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let i = item(id, groups);
    let points = params.points;
    let j = i % points;
    if (i >= params.count || j == 0u) {
        return;
    }
    let segments = points - 1u;
    let g = i / points;
    var shift = vec3<f32>(0.0);
    var change = vec3<f32>(0.0);
    var weight = 0.0;
    // The segment before (this point its far end) and after (its near end).
    for (var side = 0u; side < 2u; side++) {
        if (side == 1u && j >= segments) {
            continue;
        }
        let segment = g * segments + j - 1u + side;
        for (var s = 0u; s < params.per_segment; s++) {
            let k = segment * params.per_segment + s;
            let a = asks[k * 2u];
            if (a.w == 0.0) {
                continue;
            }
            let t = (f32(s) + 0.5) / f32(params.per_segment);
            let w = select(t, 1.0 - t, side == 1u);
            shift += a.xyz * w;
            change += asks[k * 2u + 1u].xyz * w;
            weight += w;
        }
    }
    if (weight <= 0.0) {
        return;
    }
    shift /= weight;
    change /= weight;
    position[i] = vec4<f32>(position[i].xyz + shift + change, 1.0);
    previous[i] = vec4<f32>(previous[i].xyz + shift, 1.0);
}
"#,
    )
}

/// A pose as a kernel holds it, and the host's [`crate::sim::blend`].
const POSE: &str = r#"
struct Pose {
    x: vec3<f32>,
    y: vec3<f32>,
    z: vec3<f32>,
    t: vec3<f32>,
};

fn pose_rotate(q: Pose, v: vec3<f32>) -> vec3<f32> {
    return q.x * v.x + q.y * v.y + q.z * v.z;
}

fn pose_apply(q: Pose, p: vec3<f32>) -> vec3<f32> {
    return pose_rotate(q, p) + q.t;
}

fn pose_unapply(q: Pose, p: vec3<f32>) -> vec3<f32> {
    let d = p - q.t;
    return vec3<f32>(dot(q.x, d), dot(q.y, d), dot(q.z, d));
}

fn pose_blend(a: Pose, b: Pose, s: f32) -> Pose {
    let x = safe_normalize(lerp3(a.x, b.x, s), b.x);
    let y0 = lerp3(a.y, b.y, s);
    let y = safe_normalize(y0 - x * dot(y0, x), b.y);
    return Pose(x, y, cross(x, y), lerp3(a.t, b.t, s));
}
"#;

/// The other pose a kernel takes, where the head starts: [`RIGID_PARAMS`]
/// is where it ends.
const FROM_PARAMS: &str = r#"
    from_xx: f32,
    from_xy: f32,
    from_xz: f32,
    from_yx: f32,
    from_yy: f32,
    from_yz: f32,
    from_zx: f32,
    from_zy: f32,
    from_zz: f32,
    from_tx: f32,
    from_ty: f32,
    from_tz: f32,
"#;

/// [`crate::rod`]'s matrices and quaternions (`xyz` the vector part, `w`
/// the scalar), and the rod's lengths and masses from the strand's groomed
/// points in `rest`.
const ROD: &str = r#"
fn identity3() -> mat3x3<f32> {
    return mat3x3<f32>(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 0.0, 1.0));
}

fn scaling3(s: vec3<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(vec3<f32>(s.x, 0.0, 0.0), vec3<f32>(0.0, s.y, 0.0), vec3<f32>(0.0, 0.0, s.z));
}

fn outer3(a: vec3<f32>, b: vec3<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(a * b.x, a * b.y, a * b.z);
}

fn skew3(a: vec3<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(vec3<f32>(0.0, a.z, -a.y), vec3<f32>(-a.z, 0.0, a.x), vec3<f32>(a.y, -a.x, 0.0));
}

fn inverse3(m: mat3x3<f32>) -> mat3x3<f32> {
    let r0 = cross(m[1], m[2]);
    let r1 = cross(m[2], m[0]);
    let r2 = cross(m[0], m[1]);
    let det = dot(m[0], r0);
    var inv = 0.0;
    if (abs(det) > 1e-30) {
        inv = 1.0 / det;
    }
    return transpose(mat3x3<f32>(r0 * inv, r1 * inv, r2 * inv));
}

fn qmul(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(b.xyz * a.w + a.xyz * b.w + cross(a.xyz, b.xyz), a.w * b.w - dot(a.xyz, b.xyz));
}

fn qconj(a: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(-a.xyz, a.w);
}

fn qrotate(q: vec4<f32>, p: vec3<f32>) -> vec3<f32> {
    let t = cross(q.xyz, p) * 2.0;
    return p + t * q.w + cross(q.xyz, t);
}

fn qmatrix(q: vec4<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(
        qrotate(q, vec3<f32>(1.0, 0.0, 0.0)),
        qrotate(q, vec3<f32>(0.0, 1.0, 0.0)),
        qrotate(q, vec3<f32>(0.0, 0.0, 1.0)),
    );
}

fn qnormalize(q: vec4<f32>) -> vec4<f32> {
    let l = sqrt(dot(q.xyz, q.xyz) + q.w * q.w);
    if (l > 1e-12) {
        return vec4<f32>(q.xyz * (1.0 / l), q.w / l);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

fn qexp(theta: vec3<f32>) -> vec4<f32> {
    let angle = length(theta);
    if (angle < 1e-12) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    let half_angle = 0.5 * angle;
    return vec4<f32>(theta * (sin(half_angle) / angle), cos(half_angle));
}

// Shepperd's method.
fn qfrom_matrix(m: mat3x3<f32>) -> vec4<f32> {
    let c0 = m[0];
    let c1 = m[1];
    let c2 = m[2];
    let trace = c0.x + c1.y + c2.z;
    var q: vec4<f32>;
    if (trace > 0.0) {
        let s = sqrt(trace + 1.0) * 2.0;
        q = vec4<f32>(vec3<f32>(c1.z - c2.y, c2.x - c0.z, c0.y - c1.x) * (1.0 / s), 0.25 * s);
    } else if (c0.x > c1.y && c0.x > c2.z) {
        let s = sqrt(1.0 + c0.x - c1.y - c2.z) * 2.0;
        q = vec4<f32>(0.25 * s, (c1.x + c0.y) / s, (c2.x + c0.z) / s, (c1.z - c2.y) / s);
    } else if (c1.y > c2.z) {
        let s = sqrt(1.0 + c1.y - c0.x - c2.z) * 2.0;
        q = vec4<f32>((c1.x + c0.y) / s, 0.25 * s, (c2.y + c1.z) / s, (c2.x - c0.z) / s);
    } else {
        let s = sqrt(1.0 + c2.z - c0.x - c1.y) * 2.0;
        q = vec4<f32>((c2.x + c0.z) / s, (c2.y + c1.z) / s, 0.25 * s, (c0.y - c1.x) / s);
    }
    return qnormalize(q);
}

fn qfrom_frame(d1: vec3<f32>, d3: vec3<f32>) -> vec4<f32> {
    return qfrom_matrix(mat3x3<f32>(d1, cross(d3, d1), d3));
}

// The least rotation taking unit `a` to unit `b`.
fn least_turn(a: vec3<f32>, b: vec3<f32>) -> vec4<f32> {
    let c = dot(a, b);
    if (c < -0.999999) {
        return vec4<f32>(orthogonal(a), 0.0);
    }
    return qnormalize(vec4<f32>(cross(a, b), 1.0 + c));
}

// The turn from frame `a` to `b`, `rod::darboux`.
struct Turn {
    v: vec3<f32>,
    change: mat3x3<f32>,
    matrix: mat3x3<f32>,
};

fn darboux(a: vec4<f32>, b: vec4<f32>, groomed: vec4<f32>) -> Turn {
    var q = qmul(qconj(a), b);
    // The groom's turn the short way round, as the rest was solved for.
    let side = dot(q, groomed) * groomed.w;
    if (side < 0.0 || (side == 0.0 && q.w < 0.0)) {
        q = -q;
    }
    return Turn(q.xyz, (skew3(q.xyz) - identity3() * q.w) * 0.5, qmatrix(q));
}

// Groomed segment `j` of the strand from `base`.
fn rest_segment(base: u32, j: u32) -> vec3<f32> {
    return rest[base + j + 1u].xyz - rest[base + j].xyz;
}

fn rest_length(base: u32, j: u32) -> f32 {
    return max(length(rest_segment(base, j)), 1e-6);
}

fn rest_tangent(base: u32, j: u32) -> vec3<f32> {
    return safe_normalize(rest_segment(base, j), vec3<f32>(0.0, -1.0, 0.0));
}

// Point `i`'s mass over mass per length.
fn point_mass(base: u32, i: u32, segs: u32) -> f32 {
    var next = 0.0;
    if (i < segs) {
        next = rest_length(base, i);
    }
    return 0.5 * (rest_length(base, i - 1u) + next);
}

// The stiffness of the joint before segment `s`, `Rod::joint`.
fn joint_stiffness(base: u32, s: u32) -> vec3<f32> {
    var span = 0.5 * rest_length(base, 0u);
    if (s > 0u) {
        span = 0.5 * (rest_length(base, s - 1u) + rest_length(base, s));
    }
    return vec3<f32>(params.bend, params.bend, params.twist) * (4.0 / span);
}
"#;

/// Each simulated strand's rod's rest, a strand an invocation:
/// [`crate::rod::rest_shape`] into `shape`, two slots a point -- which
/// holds each segment's strain and length while the strand is walked back
/// from the tip.
pub fn sim_shape() -> String {
    kernel(
        r#"
@group(0) @binding(0) var<storage, read> rest: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> shape: array<vec4<f32>>;
struct Params {
    count: u32,
    points: u32,
    sag_free: u32,
    pad0: u32,
    stretch: f32,
    bend: f32,
    twist: f32,
    gravity: f32,
};
@group(0) @binding(2) var<uniform> params: Params;
"#,
        &[MATH, ROD],
        r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let g = item(id, groups);
    if (g >= params.count) {
        return;
    }
    let points = params.points;
    let base = g * points;
    let segs = points - 1u;
    let gravity = vec3<f32>(0.0, -params.gravity, 0.0);
    // What each segment carries: the weight below it.
    var below = vec3<f32>(0.0);
    if (params.sag_free != 0u) {
        for (var i = segs; i >= 1u; i--) {
            below += gravity * point_mass(base, i, segs);
        }
    }
    let normal = orthogonal(rest_tangent(base, 0u));
    let follicle = qfrom_frame(normal, rest_tangent(base, 0u));
    var t = rest_tangent(base, 0u);
    var u = normal;
    for (var j = 0u; j < segs; j++) {
        if (j > 0u && params.sag_free != 0u) {
            below -= gravity * point_mass(base, j, segs);
        }
        let tangent = rest_tangent(base, j);
        u = transport(u, t, tangent);
        t = tangent;
        let lj = rest_length(base, j);
        let strain = below * (lj / params.stretch);
        let chord = tangent * lj - strain;
        let len = max(length(chord), 1e-9);
        let axis = chord * (1.0 / len);
        let d1 = safe_normalize(transport(u, t, axis), u);
        shape[base * 2u + 2u * j] = qfrom_frame(d1, axis);
        shape[base * 2u + 2u * j + 1u] = vec4<f32>(strain, len);
    }
    // Each joint rests bent and twisted by the moment it carries.
    var moment = vec3<f32>(0.0);
    for (var s = segs; s >= 1u; s--) {
        let k = s - 1u;
        var a = follicle;
        if (k > 0u) {
            a = shape[base * 2u + 2u * (k - 1u)];
        }
        let b = shape[base * 2u + 2u * k];
        let held = shape[base * 2u + 2u * k + 1u];
        let turn = darboux(a, b, vec4<f32>(0.0, 0.0, 0.0, 1.0));
        var rest_turn = turn.v;
        if (params.sag_free != 0u) {
            let axis = qrotate(b, vec3<f32>(0.0, 0.0, 1.0));
            let stretch = params.stretch / rest_length(base, k);
            moment += cross(axis, held.xyz) * (stretch * held.w);
            let stiffness = joint_stiffness(base, k);
            let wanted = inverse3(transpose(turn.change)) * (qrotate(qconj(a), moment) * -1.0);
            rest_turn = turn.v - wanted / stiffness;
        }
        shape[base * 2u + 2u * k + 1u] = vec4<f32>(rest_turn, held.w);
    }
    shape[base * 2u + 2u * segs] = follicle;
    shape[base * 2u + 2u * segs + 1u] = vec4<f32>(0.0);
}
"#,
    )
}

/// Each simulated segment's frame onto where its strand now is, the head
/// at the pose given, not spinning: [`crate::rod::frames_along`], a point
/// an invocation (the last of each strand has no segment).
pub fn sim_orient() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read> rest: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> shape: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> position: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> frames: array<vec4<f32>>;
struct Params {{
{RIGID_PARAMS}
    count: u32,
    points: u32,
    pad0: u32,
    pad1: u32,
}};
@group(0) @binding(4) var<uniform> params: Params;
"#
        ),
        &[MATH, RIGID],
        &format!(
            "{}{}",
            ROD_QUATERNIONS,
            r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    let k = item(id, groups);
    if (k >= params.count || k % params.points == params.points - 1u) {
        return;
    }
    let head = qfrom_matrix(mat3x3<f32>(
        vec3<f32>(params.ax_x, params.ax_y, params.ax_z),
        vec3<f32>(params.ay_x, params.ay_y, params.ay_z),
        vec3<f32>(params.az_x, params.az_y, params.az_z),
    ));
    let carried = qmul(head, shape[2u * k]);
    let groomed = head_rotate(safe_normalize(rest[k + 1u].xyz - rest[k].xyz, vec3<f32>(0.0, -1.0, 0.0)));
    let along = safe_normalize(position[k + 1u].xyz - position[k].xyz, groomed);
    frames[2u * k] = qnormalize(qmul(least_turn(groomed, along), carried));
    frames[2u * k + 1u] = vec4<f32>(0.0);
}
"#
        ),
    )
}

/// The quaternion helpers of [`ROD`] alone, for kernels without its rod.
const ROD_QUATERNIONS: &str = r#"
fn qmul(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(b.xyz * a.w + a.xyz * b.w + cross(a.xyz, b.xyz), a.w * b.w - dot(a.xyz, b.xyz));
}

fn qnormalize(q: vec4<f32>) -> vec4<f32> {
    let l = sqrt(dot(q.xyz, q.xyz) + q.w * q.w);
    if (l > 1e-12) {
        return vec4<f32>(q.xyz * (1.0 / l), q.w / l);
    }
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

fn qfrom_matrix(m: mat3x3<f32>) -> vec4<f32> {
    let c0 = m[0];
    let c1 = m[1];
    let c2 = m[2];
    let trace = c0.x + c1.y + c2.z;
    var q: vec4<f32>;
    if (trace > 0.0) {
        let s = sqrt(trace + 1.0) * 2.0;
        q = vec4<f32>(vec3<f32>(c1.z - c2.y, c2.x - c0.z, c0.y - c1.x) * (1.0 / s), 0.25 * s);
    } else if (c0.x > c1.y && c0.x > c2.z) {
        let s = sqrt(1.0 + c0.x - c1.y - c2.z) * 2.0;
        q = vec4<f32>(0.25 * s, (c1.x + c0.y) / s, (c2.x + c0.z) / s, (c1.z - c2.y) / s);
    } else if (c1.y > c2.z) {
        let s = sqrt(1.0 + c1.y - c0.x - c2.z) * 2.0;
        q = vec4<f32>((c1.x + c0.y) / s, 0.25 * s, (c2.y + c1.z) / s, (c2.x - c0.z) / s);
    } else {
        let s = sqrt(1.0 + c2.z - c0.x - c1.y) * 2.0;
        q = vec4<f32>((c2.x + c0.z) / s, (c2.y + c1.z) / s, 0.25 * s, (c0.y - c1.x) / s);
    }
    return qnormalize(q);
}

fn least_turn(a: vec3<f32>, b: vec3<f32>) -> vec4<f32> {
    let c = dot(a, b);
    if (c < -0.999999) {
        return vec4<f32>(orthogonal(a), 0.0);
    }
    return qnormalize(vec4<f32>(cross(a, b), 1.0 + c));
}
"#;

/// The simulated strands' substeps of a step, a strand an invocation:
/// [`crate::rod::Substep::strand`] `inner` times, the head going from
/// `from` to the end pose, after turning each frame back onto its segment
/// (the strands meeting each other move points, not frames). Each solves its strand's implicit step by block
/// LDLᵀ, its factors in `factors` (fourteen words a point: the 6x6 block of
/// L left of the diagonal, and the forward solve's solution), and leaves
/// `previous` a substep's velocity behind `position` and each segment's
/// frame and angular velocity in `frames`.
pub fn sim_step() -> String {
    kernel(
        &format!(
            r#"
@group(0) @binding(0) var<storage, read> rest: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> shape: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> position: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> previous: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> frames: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> head_shape: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read_write> factors: array<vec4<f32>>;
struct Params {{
{RIGID_PARAMS}
{FROM_PARAMS}
    count: u32,
    points: u32,
    body: u32,
    inner: u32,
    h: f32,
    h_last: f32,
    time: f32,
    gravity: f32,
    wind_x: f32,
    wind_y: f32,
    wind_z: f32,
    turbulence: f32,
    collision_offset: f32,
    head_friction: f32,
    stretch: f32,
    bend: f32,
    twist: f32,
    gyration: f32,
    damping: f32,
    drag: f32,
    drag_viscous: f32,
    pad0: f32,
    pad1: f32,
    pad2: f32,
}};
@group(0) @binding(7) var<uniform> params: Params;
"#
        ),
        &[RANDOM, NOISE, MATH, HEAD, POSE, ROD],
        r#"
struct M6 {
    a: mat3x3<f32>,
    b: mat3x3<f32>,
    c: mat3x3<f32>,
    d: mat3x3<f32>,
};

struct V6 {
    w: vec3<f32>,
    v: vec3<f32>,
};

fn m6_zero() -> M6 {
    return M6(mat3x3<f32>(), mat3x3<f32>(), mat3x3<f32>(), mat3x3<f32>());
}

fn m6_mul(p: M6, o: M6) -> M6 {
    return M6(p.a * o.a + p.b * o.c, p.a * o.b + p.b * o.d, p.c * o.a + p.d * o.c, p.c * o.b + p.d * o.d);
}

fn m6_apply(p: M6, u: V6) -> V6 {
    return V6(p.a * u.w + p.b * u.v, p.c * u.w + p.d * u.v);
}

fn m6_transpose(p: M6) -> M6 {
    return M6(transpose(p.a), transpose(p.c), transpose(p.b), transpose(p.d));
}

fn m6_inverse(p: M6) -> M6 {
    let ai = inverse3(p.a);
    let aib = ai * p.b;
    let cai = p.c * ai;
    let si = inverse3(p.d - p.c * aib);
    return M6(ai + aib * si * cai, (aib * si) * -1.0, (si * cai) * -1.0, si);
}

// A segment as a substep sees it, `rod::Segment`.
struct Segment {
    axes: mat3x3<f32>,
    strain: vec3<f32>,
    stiffness: f32,
    len: f32,
};

// A joint as a substep sees it, `rod::Joint`.
struct Joint {
    stiffness: mat3x3<f32>,
    torque: vec3<f32>,
    turn: mat3x3<f32>,
    curving: f32,
};

fn no_segment() -> Segment {
    return Segment(mat3x3<f32>(), vec3<f32>(0.0), 0.0, 0.0);
}

fn no_joint() -> Joint {
    return Joint(mat3x3<f32>(), vec3<f32>(0.0), mat3x3<f32>(), 0.0);
}

fn frame_at(base: u32, j: u32) -> vec4<f32> {
    return frames[2u * (base + j)];
}

fn segment_at(base: u32, j: u32) -> Segment {
    let axes = qmatrix(frame_at(base, j));
    let len = shape[2u * (base + j) + 1u].w;
    return Segment(
        axes,
        position[base + j + 1u].xyz - position[base + j].xyz - axes[2] * len,
        params.stretch / rest_length(base, j),
        len,
    );
}

fn joint_at(base: u32, s: u32, before: vec4<f32>) -> Joint {
    var groomed_before = shape[2u * (base + params.points - 1u)];
    if (s > 0u) {
        groomed_before = shape[2u * (base + s - 1u)];
    }
    let groomed = qmul(qconj(groomed_before), shape[2u * (base + s)]);
    let turn = darboux(before, frame_at(base, s), groomed);
    let r = shape[2u * (base + s) + 1u];
    let k = joint_stiffness(base, s);
    let gk = transpose(turn.change) * scaling3(k);
    let curving = max(-0.25 * dot(k * (turn.v - r.xyz), turn.v), 0.0);
    return Joint(gk * turn.change, gk * (turn.v - r.xyz), turn.matrix, curving);
}

fn factor_l(row: u32) -> M6 {
    let f = row * 14u;
    return M6(
        mat3x3<f32>(factors[f].xyz, factors[f + 1u].xyz, factors[f + 2u].xyz),
        mat3x3<f32>(factors[f + 3u].xyz, factors[f + 4u].xyz, factors[f + 5u].xyz),
        mat3x3<f32>(factors[f + 6u].xyz, factors[f + 7u].xyz, factors[f + 8u].xyz),
        mat3x3<f32>(factors[f + 9u].xyz, factors[f + 10u].xyz, factors[f + 11u].xyz),
    );
}

fn store_factor(row: u32, l: M6, z: V6) {
    let f = row * 14u;
    factors[f] = vec4<f32>(l.a[0], 0.0);
    factors[f + 1u] = vec4<f32>(l.a[1], 0.0);
    factors[f + 2u] = vec4<f32>(l.a[2], 0.0);
    factors[f + 3u] = vec4<f32>(l.b[0], 0.0);
    factors[f + 4u] = vec4<f32>(l.b[1], 0.0);
    factors[f + 5u] = vec4<f32>(l.b[2], 0.0);
    factors[f + 6u] = vec4<f32>(l.c[0], 0.0);
    factors[f + 7u] = vec4<f32>(l.c[1], 0.0);
    factors[f + 8u] = vec4<f32>(l.c[2], 0.0);
    factors[f + 9u] = vec4<f32>(l.d[0], 0.0);
    factors[f + 10u] = vec4<f32>(l.d[1], 0.0);
    factors[f + 11u] = vec4<f32>(l.d[2], 0.0);
    factors[f + 12u] = vec4<f32>(z.w, 0.0);
    factors[f + 13u] = vec4<f32>(z.v, 0.0);
}

// Off the head: out to the offset, or as near as the groom has the point,
// the velocity into it stopped and the sliding slowed by Coulomb friction,
// relative to the head's surface.
fn contact(p: vec3<f32>, v: vec3<f32>, groomed: vec3<f32>, from_pose: Pose, to_pose: Pose, h: f32) -> array<vec3<f32>, 2> {
    if (params.body == 0u) {
        return array<vec3<f32>, 2>(p, v);
    }
    let local = pose_unapply(to_pose, p);
    let distance_now = head_distance(local);
    if (distance_now >= params.collision_offset) {
        return array<vec3<f32>, 2>(p, v);
    }
    let offset = min(params.collision_offset, head_distance(groomed));
    if (distance_now >= offset) {
        return array<vec3<f32>, 2>(p, v);
    }
    let out = push_out(local, offset);
    let normal = pose_rotate(to_pose, head_gradient(out));
    let q = pose_apply(to_pose, out);
    let surface = (q - pose_apply(from_pose, pose_unapply(to_pose, q))) * (1.0 / h);
    var relative = v - surface;
    let into = dot(relative, normal);
    if (into < 0.0) {
        relative = relative - normal * into;
        let sliding = length(relative);
        if (sliding > 1e-9) {
            relative = relative * max(1.0 - params.head_friction * -into / sliding, 0.0);
        }
    }
    return array<vec3<f32>, 2>(q, relative + surface);
}

// The angular velocity turning `from_q` to `to_q` in `h` seconds.
fn spin(from_q: vec4<f32>, to_q: vec4<f32>, h: f32) -> vec3<f32> {
    var q = qmul(to_q, qconj(from_q));
    if (q.w < 0.0) {
        q = -q;
    }
    let s = length(q.xyz);
    if (s < 1e-9) {
        return q.xyz * (2.0 / h);
    }
    return q.xyz * (2.0 * atan2(s, q.w) / (s * h));
}

fn pose_rotation(q: Pose) -> vec4<f32> {
    return qfrom_matrix(mat3x3<f32>(q.x, q.y, q.z));
}

@compute @workgroup_size(32)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {
    // Thirty-two strands a workgroup: a strand's solve is long and
    // serial, so the workgroups are spread over as many cores as can be.
    let g = id.y * groups.x * 32u + id.x;
    if (g >= params.count) {
        return;
    }
    let points = params.points;
    let segs = points - 1u;
    let base = g * points;
    let h = params.h;
    let start = Pose(
        vec3<f32>(params.from_xx, params.from_xy, params.from_xz),
        vec3<f32>(params.from_yx, params.from_yy, params.from_yz),
        vec3<f32>(params.from_zx, params.from_zy, params.from_zz),
        vec3<f32>(params.from_tx, params.from_ty, params.from_tz),
    );
    let end = Pose(
        vec3<f32>(params.ax_x, params.ax_y, params.ax_z),
        vec3<f32>(params.ay_x, params.ay_y, params.ay_z),
        vec3<f32>(params.az_x, params.az_y, params.az_z),
        vec3<f32>(params.tx, params.ty, params.tz),
    );
    let gravity = vec3<f32>(0.0, -params.gravity, 0.0);
    let elastic = h * h;
    let viscous = h * h + h * params.damping;
    let identity = identity3();
    let unbent = scaling3(vec3<f32>(1.0, 1.0, 0.0));
    let e3 = vec3<f32>(0.0, 0.0, 1.0);
    // The strands meeting each other moved their points, not their frames:
    // each frame turned the least way back onto its segment.
    for (var j = 0u; j < segs; j++) {
        let q = frame_at(base, j);
        let along = safe_normalize(position[base + j + 1u].xyz - position[base + j].xyz, qrotate(q, e3));
        frames[2u * (base + j)] = qnormalize(qmul(least_turn(qrotate(q, e3), along), q));
    }
    var h_last = params.h_last;
    for (var m = 0u; m < params.inner; m++) {
        let from_pose = pose_blend(start, end, f32(m) / f32(params.inner));
        let to_pose = pose_blend(start, end, f32(m + 1u) / f32(params.inner));
        let root_from = pose_apply(from_pose, rest[base].xyz);
        let root_to = pose_apply(to_pose, rest[base].xyz);
        let root_velocity = (root_to - root_from) * (1.0 / h);
        position[base] = vec4<f32>(root_from, 1.0);
        let head_from = pose_rotation(from_pose);
        let follicle = qmul(head_from, shape[2u * (base + segs)]);
        let follicle_spin = qrotate(qconj(follicle), spin(head_from, pose_rotation(to_pose), h));
        var gust = 1.0;
        if (params.turbulence > 0.0) {
            let time = params.time + f32(m) * h;
            let q = rest[base].xyz * 3.0 + vec3<f32>(time * 0.9, time * 0.4, 0.0);
            gust = 1.0 + params.turbulence * perlin(q) * 2.0;
        }
        let air = vec3<f32>(params.wind_x, params.wind_y, params.wind_z) * gust;

        // Segments g and g + 1, the joints before them, about row g.
        var seg = segment_at(base, 0u);
        var joint = joint_at(base, 0u, follicle);
        var next = no_segment();
        var after = no_joint();
        if (segs > 1u) {
            next = segment_at(base, 1u);
            after = joint_at(base, 1u, frame_at(base, 0u));
        }
        var d_last = m6_zero();
        var y_last = V6(vec3<f32>(0.0), vec3<f32>(0.0));
        for (var r = 0u; r < segs; r++) {
            let i = r + 1u;
            let mass = point_mass(base, i, segs);
            let velocity = (position[base + i].xyz - previous[base + i].xyz) * (1.0 / h_last);
            let spinning = transpose(seg.axes) * frames[2u * (base + r) + 1u].xyz;
            // The air, across the point's tangent.
            let tangent = safe_normalize(seg.strain + seg.axes[2] * seg.len + next.strain + next.axes[2] * next.len, seg.axes[2]);
            let relative = velocity - air;
            let across = relative - tangent * dot(relative, tangent);
            let speed = length(across);
            let c = params.drag * (speed + params.drag_viscous * pow(speed, 1.0 / 3.0));
            let drag = (identity - outer3(tangent, tangent)) * (mass * c);

            let inertia = scaling3(vec3<f32>(0.25, 0.25, 0.5)) * (rest_length(base, r) * params.gyration);
            let lever = transpose(skew3(e3)) * transpose(seg.axes) * (seg.len * seg.stiffness);
            let local_strain = transpose(seg.axes) * seg.strain;
            let bend = transpose(joint.turn) * joint.stiffness * joint.turn + identity * joint.curving
                + after.stiffness + identity * after.curving;
            let torque = cross(e3, local_strain) * (seg.stiffness * seg.len)
                + transpose(joint.turn) * joint.torque - after.torque;
            let force = gravity * mass - seg.strain * seg.stiffness + next.strain * next.stiffness;
            // Turning the frame across its axis: the shear's stiffness and
            // the tension's along it.
            let across_axis = seg.len * (seg.len + max(local_strain.z, 0.0));
            let diagonal = M6(
                inertia + unbent * (elastic * seg.stiffness * across_axis) + bend * viscous,
                lever * elastic,
                transpose(lever) * elastic,
                identity * (mass + elastic * (seg.stiffness + next.stiffness)) + drag * h,
            );
            var left = M6(
                transpose(joint.turn) * joint.stiffness * -viscous,
                lever * -elastic,
                mat3x3<f32>(),
                identity * -(elastic * seg.stiffness),
            );
            var rhs = V6(inertia * spinning + torque * h, velocity * mass + force * h + drag * air * h);
            if (r == 0u) {
                let known = m6_apply(left, V6(follicle_spin, root_velocity));
                rhs = V6(rhs.w - known.w, rhs.v - known.v);
                left = m6_zero();
            }
            let l = m6_mul(left, d_last);
            let lt = m6_mul(l, m6_transpose(left));
            let d = M6(diagonal.a - lt.a, diagonal.b - lt.b, diagonal.c - lt.c, diagonal.d - lt.d);
            let d_inverse = m6_inverse(d);
            let ly = m6_apply(l, y_last);
            let y = V6(rhs.w - ly.w, rhs.v - ly.v);
            store_factor(base + r, l, m6_apply(d_inverse, y));
            d_last = d_inverse;
            y_last = y;

            // Along by one.
            seg = next;
            joint = after;
            next = no_segment();
            after = no_joint();
            if (r + 2u < segs) {
                next = segment_at(base, r + 2u);
                after = joint_at(base, r + 2u, frame_at(base, r + 1u));
            }
        }

        // Back up the strand.
        var later = V6(vec3<f32>(0.0), vec3<f32>(0.0));
        for (var r = segs; r >= 1u; r--) {
            let row = r - 1u;
            let f = (base + row) * 14u;
            var u = V6(factors[f + 12u].xyz, factors[f + 13u].xyz);
            if (row + 1u < segs) {
                let lu = m6_apply(m6_transpose(factor_l(base + row + 1u)), later);
                u = V6(u.w - lu.w, u.v - lu.v);
            }
            later = u;
            let i = row + 1u;
            let moved = contact(position[base + i].xyz + u.v * h, u.v, rest[base + i].xyz, from_pose, to_pose, h);
            position[base + i] = vec4<f32>(moved[0], 1.0);
            previous[base + i] = vec4<f32>(moved[0] - moved[1] * h, 1.0);
            let old = frame_at(base, row);
            let w = qmatrix(old) * u.w;
            frames[2u * (base + row)] = qnormalize(qmul(qexp(w * h), old));
            frames[2u * (base + row) + 1u] = vec4<f32>(w, 0.0);
        }
        position[base] = vec4<f32>(root_to, 1.0);
        previous[base] = vec4<f32>(root_from, 1.0);
        h_last = h;
    }
}
"#,
    )
}
