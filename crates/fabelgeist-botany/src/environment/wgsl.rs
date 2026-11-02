//! The environment kernels. Each mirrors a host function in `super` -- see
//! there for what the arithmetic means.

use fabelgeist_buildings::SOLID_WGSL;
use fabelgeist_spatial_hash::SPHERE_OVERLAP_WGSL;
use fabelgeist_spatial_hash::gpu::{HASH_WGSL, RANGE_WGSL};

/// `crowding()` from `super`: the overlap of a sphere with its neighbours as
/// a fraction of its own volume. Expects `points`, `parents`, `hash_indices`
/// and `hash_ranges` bound, and `params.table_size`/`params.inverse_cell`.
const CROWDING: &str = r#"
// The overlap volume with the entries of one cell.
fn crowding_cell(index: u32, parent: u32, center: vec3<f32>, radius: f32, cell: vec3<i32>) -> f32 {
    var sum = 0.0;
    let range = spatial_hash_range(cell, params.table_size);
    for (var k = range.x; k < range.y; k++) {
        let j = hash_indices[k];
        let pj = parents[j];
        if (j == index || j == parent || pj == index || (parent != 0xFFFFFFFFu && pj == parent)) {
            continue;
        }
        let s = points[j];
        if (any(spatial_hash_cell(s.xyz, params.inverse_cell) != cell)) {
            continue;
        }
        sum += sphere_overlap_volume(radius, s.w, distance(s.xyz, center));
    }
    return sum;
}

fn neighbour_cell(base: vec3<i32>, n: u32) -> vec3<i32> {
    return base + vec3<i32>(i32(n % 3u) - 1, i32((n / 3u) % 3u) - 1, i32(n / 9u) - 1);
}

fn crowding(index: u32, parent: u32, center: vec3<f32>, radius: f32) -> f32 {
    let base = spatial_hash_cell(center, params.inverse_cell);
    var sum = 0.0;
    for (var n = 0u; n < 27u; n++) {
        sum += crowding_cell(index, parent, center, radius, neighbour_cell(base, n));
    }
    return sum / max(sphere_volume(radius), 1e-9);
}
"#;

/// `Obstacles::crowding()`: the cap of a sphere past the nearest solid's
/// surface, as a fraction of the sphere. Expects `solids` bound.
const OBSTACLES: &str = r#"
fn cap_fraction(d: f32, r: f32) -> f32 {
    let h = clamp(r - d, 0.0, 2.0 * r);
    return h * h * (3.0 * r - h) / max(4.0 * r * r * r, 1e-12);
}

fn obstacle_crowding(center: vec3<f32>, radius: f32) -> f32 {
    var d = 1e30;
    let n = arrayLength(&solids);
    for (var i = 0u; i < n; i++) {
        let s = solids[i];
        // Only a solid nearer than the radius can take any of the sphere.
        let beside = length(center.xz - s.base.xz) - solid_reach(s);
        if (beside > radius || center.y < s.base.y - radius || center.y > s.base.y + s.size.y + radius) {
            continue;
        }
        d = min(d, solid_distance(s, center));
    }
    return cap_fraction(d, radius);
}
"#;

const NEIGHBOUR_BINDINGS: &str = r#"
@group(0) @binding(0) var<storage, read> points: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> parents: array<u32>;
@group(0) @binding(2) var<storage, read> hash_indices: array<u32>;
@group(0) @binding(3) var<storage, read> hash_ranges: array<u32>;
"#;

/// Quaternion composition and vector rotation for the orientation search.
const ROTATIONS: &str = r#"
fn qmul(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
        a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
        a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
        a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    );
}

fn qrot(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let c = cross(q.xyz, v);
    return v + (c * q.w + cross(q.xyz, c)) * 2.0;
}
"#;

pub fn light() -> String {
    format!(
        r#"
{NEIGHBOUR_BINDINGS}
@group(0) @binding(4) var<storage, read> shadow: array<f32>;
@group(0) @binding(5) var<storage, read_write> light: array<vec2<f32>>;

struct LightParams {{
    count: u32,
    table_size: u32,
    inverse_cell: f32,
    sensitivity: f32,
    origin_x: f32,
    origin_y: f32,
    origin_z: f32,
    inverse_shadow_cell: f32,
    nx: u32,
    ny: u32,
    nz: u32,
    shadow_enabled: u32,
}};
@group(0) @binding(6) var<uniform> params: LightParams;

{HASH_WGSL}
{RANGE_WGSL}
{SPHERE_OVERLAP_WGSL}
{CROWDING}

fn global_light(p: vec3<f32>) -> f32 {{
    if (params.shadow_enabled == 0u) {{
        return 1.0;
    }}
    let c = floor((p - vec3<f32>(params.origin_x, params.origin_y, params.origin_z)) * params.inverse_shadow_cell);
    if (c.x < 0.0 || c.z < 0.0 || c.x >= f32(params.nx) || c.z >= f32(params.nz) || c.y >= f32(params.ny)) {{
        return 1.0;
    }}
    let y = u32(max(c.y, 0.0));
    let index = (y * params.nz + u32(c.z)) * params.nx + u32(c.x);
    return exp(-shadow[index]);
}}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let i = id.y * groups.x * 64u + id.x;
    if (i >= params.count) {{
        return;
    }}
    let s = points[i];
    let f = crowding(i, parents[i], s.xyz, s.w);
    light[i] = vec2<f32>(exp(-params.sensitivity * f), global_light(s.xyz));
}}
"#
    )
}

pub fn splat() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> points: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> density: array<atomic<u32>>;

struct SplatParams {{
    count: u32,
    nx: u32,
    ny: u32,
    nz: u32,
    origin_x: f32,
    origin_y: f32,
    origin_z: f32,
    cell: f32,
    strength: f32,
    fixed_point: f32,
    inverse_cell: f32,
    pad1: u32,
}};
@group(0) @binding(2) var<uniform> params: SplatParams;

{SPHERE_OVERLAP_WGSL}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) groups: vec3<u32>) {{
    let i = id.y * groups.x * 64u + id.x;
    if (i >= params.count) {{
        return;
    }}
    let s = points[i];
    let c = floor((s.xyz - vec3<f32>(params.origin_x, params.origin_y, params.origin_z)) * params.inverse_cell);
    if (any(c < vec3<f32>(0.0)) || c.x >= f32(params.nx) || c.y >= f32(params.ny) || c.z >= f32(params.nz)) {{
        return;
    }}
    let volume = sphere_volume(s.w) / (params.cell * params.cell * params.cell);
    let amount = u32(params.strength * min(volume, 1.0) * params.fixed_point);
    let index = (u32(c.y) * params.nz + u32(c.z)) * params.nx + u32(c.x);
    atomicAdd(&density[index], amount);
}}
"#
    )
}

pub fn propagate() -> String {
    let [center, edge, corner] = super::shadow::KERNEL;
    format!(
        r#"
@group(0) @binding(0) var<storage, read> density: array<u32>;
@group(0) @binding(1) var<storage, read_write> shadow: array<f32>;

struct PropagateParams {{
    nx: u32,
    nz: u32,
    layer: u32,
    pad: u32,
    decay: f32,
    fixed_point: f32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(2) var<uniform> params: PropagateParams;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let layer_size = params.nx * params.nz;
    if (id.x >= layer_size) {{
        return;
    }}
    let x = i32(id.x % params.nx);
    let z = i32(id.x / params.nx);
    let above = (params.layer + 1u) * layer_size;
    var sum = 0.0;
    for (var dz = -1; dz <= 1; dz++) {{
        for (var dx = -1; dx <= 1; dx++) {{
            let sx = x + dx;
            let sz = z + dz;
            if (sx < 0 || sz < 0 || sx >= i32(params.nx) || sz >= i32(params.nz)) {{
                continue;
            }}
            let i = above + u32(sz) * params.nx + u32(sx);
            var w = {center:.9};
            let ring = abs(dx) + abs(dz);
            if (ring == 1) {{ w = {edge:.9}; }}
            if (ring == 2) {{ w = {corner:.9}; }}
            sum += w * (shadow[i] + f32(density[i]) / params.fixed_point);
        }}
    }}
    shadow[params.layer * layer_size + id.x] = params.decay * sum;
}}
"#
    )
}

/// All layers in one dispatch: a single workgroup walks the grid top-down,
/// with a storage barrier between layers standing in for the dispatch
/// boundary. For grids whose layers are small, where a dispatch per layer
/// would cost more in overhead than in work.
pub fn propagate_all() -> String {
    let [center, edge, corner] = super::shadow::KERNEL;
    format!(
        r#"
@group(0) @binding(0) var<storage, read> density: array<u32>;
@group(0) @binding(1) var<storage, read_write> shadow: array<f32>;

struct PropagateParams {{
    nx: u32,
    nz: u32,
    ny: u32,
    pad: u32,
    decay: f32,
    fixed_point: f32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(2) var<uniform> params: PropagateParams;

@compute @workgroup_size(256)
fn main(@builtin(local_invocation_index) t: u32) {{
    let layer_size = params.nx * params.nz;
    for (var layer = i32(params.ny) - 2; layer >= 0; layer--) {{
        let above = u32(layer + 1) * layer_size;
        for (var cell = t; cell < layer_size; cell += 256u) {{
            let x = i32(cell % params.nx);
            let z = i32(cell / params.nx);
            var sum = 0.0;
            for (var dz = -1; dz <= 1; dz++) {{
                for (var dx = -1; dx <= 1; dx++) {{
                    let sx = x + dx;
                    let sz = z + dz;
                    if (sx < 0 || sz < 0 || sx >= i32(params.nx) || sz >= i32(params.nz)) {{
                        continue;
                    }}
                    let i = above + u32(sz) * params.nx + u32(sx);
                    var w = {center:.9};
                    let ring = abs(dx) + abs(dz);
                    if (ring == 1) {{ w = {edge:.9}; }}
                    if (ring == 2) {{ w = {corner:.9}; }}
                    sum += w * (shadow[i] + f32(density[i]) / params.fixed_point);
                }}
            }}
            shadow[u32(layer) * layer_size + cell] = params.decay * sum;
        }}
        storageBarrier();
        workgroupBarrier();
    }}
}}
"#
    )
}

/// One workgroup per request. Every step evaluates the current rotation and
/// its four neighbours in the pattern at once: the 5 x 27 (candidate, cell)
/// pairs are spread over the workgroup's threads, their partial overlaps
/// summed in shared memory, and one thread picks the best and publishes it.
/// A request is a few thousand sphere tests per candidate in a dense canopy,
/// so a thread per request -- a few thousand threads in all -- would leave
/// most of the GPU idle while each thread walked its neighbourhood alone.
pub fn orient() -> String {
    format!(
        r#"
{NEIGHBOUR_BINDINGS}
@group(0) @binding(4) var<storage, read> requests: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> rotations: array<vec4<f32>>;
@group(0) @binding(7) var<storage, read_write> crowding_out: array<f32>;
@group(0) @binding(8) var<storage, read> solids: array<Solid>;

struct OrientParams {{
    count: u32,
    table_size: u32,
    inverse_cell: f32,
    epsilon: f32,
    steps: u32,
    groups_x: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(6) var<uniform> params: OrientParams;

{HASH_WGSL}
{RANGE_WGSL}
{SPHERE_OVERLAP_WGSL}
{CROWDING}
{SOLID_WGSL}
{OBSTACLES}

{ROTATIONS}

const THREADS: u32 = 64u;
const CANDIDATES: u32 = 5u;

var<workgroup> partial: array<array<f32, 5>, 64>;
var<workgroup> obstacle: array<f32, 5>;
var<workgroup> rotation: vec4<f32>;
var<workgroup> candidates: array<vec4<f32>, 5>;
var<workgroup> centers: array<vec3<f32>, 5>;
var<workgroup> done: u32;
var<workgroup> crowded: f32;

@compute @workgroup_size(64)
fn main(
    @builtin(workgroup_id) group: vec3<u32>,
    @builtin(local_invocation_index) t: u32,
) {{
    let k = group.y * params.groups_x + group.x;
    if (k >= params.count) {{
        return;
    }}
    let a = requests[k * 5u];
    let b = requests[k * 5u + 1u];
    let d = requests[k * 5u + 3u];
    let e = requests[k * 5u + 4u];
    let root = a.xyz;
    let radius = a.w;
    let local_center = b.xyz;
    let index = bitcast<u32>(b.w);
    let axis = d.xyz;
    let cos_target = d.w;
    let w_collision = e.x;
    let w_tropism = e.y;
    let parent = bitcast<u32>(e.z);
    let near_solids = bitcast<u32>(e.w) != 0u;
    let volume = max(sphere_volume(radius), 1e-9);

    let s = sin(params.epsilon * 0.5);
    let c = cos(params.epsilon * 0.5);

    if (t == 0u) {{
        rotation = requests[k * 5u + 2u];
        done = 0u;
        crowded = 0.0;
    }}

    for (var step = 0u; step < params.steps; step++) {{
        let q = workgroupUniformLoad(&rotation);
        // All lanes search the same candidates. Publish one copy before
        // distributing their neighbourhood queries across the workgroup.
        if (t < CANDIDATES) {{
            if (t == 0u) {{
                candidates[t] = q;
            }} else {{
                var perturbation = vec4<f32>(s, 0.0, 0.0, c);
                switch t {{
                    case 2u: {{ perturbation = vec4<f32>(-s, 0.0, 0.0, c); }}
                    case 3u: {{ perturbation = vec4<f32>(0.0, 0.0, s, c); }}
                    case 4u: {{ perturbation = vec4<f32>(0.0, 0.0, -s, c); }}
                    default: {{}}
                }}
                candidates[t] = normalize(qmul(q, perturbation));
            }}
            centers[t] = root + qrot(candidates[t], local_center);
        }}
        workgroupBarrier();

        for (var m = 0u; m < CANDIDATES; m++) {{
            partial[t][m] = 0.0;
        }}
        for (var task = t; task < CANDIDATES * 27u; task += THREADS) {{
            let m = task / 27u;
            let base = spatial_hash_cell(centers[m], params.inverse_cell);
            let cell = neighbour_cell(base, task % 27u);
            partial[t][m] += crowding_cell(index, parent, centers[m], radius, cell);
        }}
        // Each candidate's share inside solids, a thread apiece.
        if (t < CANDIDATES) {{
            obstacle[t] = select(0.0, obstacle_crowding(centers[t], radius), near_solids);
        }}
        workgroupBarrier();

        if (t == 0u) {{
            var best = 0u;
            var best_cost = 0.0;
            var best_crowding = 0.0;
            for (var m = 0u; m < CANDIDATES; m++) {{
                var total = 0.0;
                for (var i = 0u; i < THREADS; i++) {{
                    total += partial[i][m];
                }}
                let turned = qrot(candidates[m], axis);
                let crowd = total / volume + obstacle[m];
                let cost = w_collision * crowd + w_tropism * abs(cos_target - turned.y);
                if (m == 0u || cost < best_cost) {{
                    best = m;
                    best_cost = cost;
                    best_crowding = crowd;
                }}
            }}
            crowded = best_crowding;
            if (best == 0u) {{
                done = 1u;
            }} else {{
                rotation = candidates[best];
            }}
        }}
        if (workgroupUniformLoad(&done) == 1u) {{
            break;
        }}
    }}

    if (t == 0u) {{
        rotations[k] = rotation;
        crowding_out[k] = crowded;
    }}
}}
"#
    )
}
