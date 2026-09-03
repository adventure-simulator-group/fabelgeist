//! The cloth-specific kernels: bending, and cloth against itself.

/// Quadratic (isometric) bending.
///
/// The obvious constraint on a hinge is its dihedral angle, and it is the
/// wrong one for cloth: `acos` is singular exactly where its derivative is
/// needed most, and a flat hinge -- which is every hinge in a flat panel, at
/// rest -- sits precisely on that singularity. A garment built that way gets
/// no bending resistance at all until it has already creased.
///
/// So this uses Bergou's quadratic model instead. For four points that are
/// coplanar at rest there is exactly one affine dependency between them:
/// weights `k` with `sum(k) = 0` and `sum(k_i * x_i) = 0`. That combination
/// stays zero under *any* affine deformation -- translation, rotation,
/// stretch, shear -- and becomes non-zero only when the patch stops being
/// flat. Which makes it a pure bending measure:
///
/// ```text
/// s = sum(k_i * p_i)      C = |s| - rest      grad_i C = k_i * s / |s|
/// ```
///
/// Linear in the positions, with full stiffness at the flat state and no
/// singularity anywhere the fabric actually goes. The weights are computed per
/// hinge when the garment is built, from the panel's own flat layout.
pub const BEND: &str = r#"
@group(0) @binding(1) var<storage, read_write> lambdas: array<f32>;
// Four particle indices per constraint.
@group(0) @binding(2) var<storage, read> particles: array<u32>;
// The four affine weights, and the rest value in the fifth slot. Non-zero rest
// lets a garment be built with a crease already in it.
@group(0) @binding(3) var<storage, read> weights: array<f32>;
@group(0) @binding(4) var<uniform> params: SolveParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let local = global_id.x;
    if (local >= params.count) {
        return;
    }
    let constraint = params.first + local;

    let base = constraint * 8u;
    let k0 = weights[base];
    let k1 = weights[base + 1u];
    let k2 = weights[base + 2u];
    let k3 = weights[base + 3u];
    let rest = weights[base + 4u];

    let i0 = particles[constraint * 4u];
    let i1 = particles[constraint * 4u + 1u];
    let i2 = particles[constraint * 4u + 2u];
    let i3 = particles[constraint * 4u + 3u];

    let w0 = particle_inverse_mass(i0);
    let w1 = particle_inverse_mass(i1);
    let w2 = particle_inverse_mass(i2);
    let w3 = particle_inverse_mass(i3);

    let s = particle_position(i0) * k0
          + particle_position(i1) * k1
          + particle_position(i2) * k2
          + particle_position(i3) * k3;

    let magnitude = length(s);
    // Exactly flat is exactly at rest, so there is nothing to correct -- and
    // no direction to correct along either. Unlike the dihedral form, this is
    // a genuinely empty case rather than a singularity being dodged.
    if (magnitude < 1e-12) {
        return;
    }
    let normal = s / magnitude;

    // |grad_i C| is |k_i|, so the weighted denominator is a plain sum.
    let denominator = w0 * k0 * k0 + w1 * k1 * k1 + w2 * k2 * k2 + w3 * k3 * k3;
    let alpha_tilde = xpbd_alpha_tilde(params.compliance, params.substep);
    let solved = xpbd_solve(magnitude - rest, denominator, lambdas[constraint], alpha_tilde);
    if (!solved.valid) {
        return;
    }

    lambdas[constraint] = lambdas[constraint] + solved.delta_lambda;
    particle_move(i0, normal * (w0 * k0 * solved.delta_lambda));
    particle_move(i1, normal * (w1 * k1 * solved.delta_lambda));
    particle_move(i2, normal * (w2 * k2 * solved.delta_lambda));
    particle_move(i3, normal * (w3 * k3 * solved.delta_lambda));
}
"#;

// ----- self-collision -----
//
// A uniform grid rather than a hierarchy. Cloth particles are all the same
// size and roughly evenly spread, which is exactly the case a grid is best at
// and a BVH is worst at -- and the grid is rebuilt from scratch every substep
// for the cost of a sort.

/// Which cell each particle is in, hashed into a fixed table.
pub const HASH: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> cells: array<u32>;
@group(0) @binding(2) var<storage, read_write> indices: array<u32>;

struct HashParams {
    count: u32,
    table_size: u32,
    inverse_spacing: f32,
    pad: u32,
};
@group(0) @binding(3) var<uniform> params: HashParams;

fn cell_of(position: vec3<f32>, inverse_spacing: f32) -> vec3<i32> {
    return vec3<i32>(floor(position * inverse_spacing));
}

fn hash_cell(cell: vec3<i32>, table_size: u32) -> u32 {
    // Three large odd primes: the usual spatial hash, and good enough that
    // collisions are rare and harmless when they happen (a collision just
    // means two distant cells share a bucket, and the exact distance test
    // rejects the false neighbours).
    let h = (u32(cell.x) * 73856093u) ^ (u32(cell.y) * 19349663u) ^ (u32(cell.z) * 83492791u);
    return h % table_size;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.count) {
        return;
    }
    cells[index] = hash_cell(
        cell_of(positions[index].xyz, params.inverse_spacing),
        params.table_size,
    );
    indices[index] = index;
}
"#;

/// Where each bucket begins in the sorted list.
///
/// After the sort the particles are grouped by bucket, so a bucket is a
/// contiguous range and only its start has to be recorded -- the next
/// non-empty bucket's start is this one's end.
pub const CELL_RANGES: &str = r#"
@group(0) @binding(0) var<storage, read> cells: array<u32>;
@group(0) @binding(1) var<storage, read_write> starts: array<u32>;

struct RangeParams {
    count: u32,
    table_size: u32,
    pad0: u32,
    pad1: u32,
};
@group(0) @binding(2) var<uniform> params: RangeParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.count) {
        return;
    }
    let cell = cells[index];
    // The first entry of a run, and every run start, writes its own position.
    if (index == 0u || cells[index - 1u] != cell) {
        starts[cell] = index;
    }
    // The last entry closes the table's end marker.
    if (index + 1u == params.count) {
        starts[params.table_size] = params.count;
    }
}
"#;

/// Clear the bucket table to "empty".
pub const CLEAR_RANGES: &str = r#"
@group(0) @binding(0) var<storage, read_write> starts: array<u32>;

struct ClearParams {
    table_size: u32,
    count: u32,
    pad0: u32,
    pad1: u32,
};
@group(0) @binding(1) var<uniform> params: ClearParams;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index > params.table_size) {
        return;
    }
    // `count` marks an empty bucket: a range that starts past the end of the
    // list is empty whatever comes after it.
    starts[index] = params.count;
}
"#;

/// Push apart any two particles closer than the fabric's own thickness.
///
/// The response is symmetric and mass-weighted, and it is applied with
/// `atomicAdd` on a separate accumulator rather than written into the
/// positions directly: a particle has many neighbours, they are spread across
/// workgroups, and letting them all read-modify-write one position would lose
/// most of the corrections. Accumulating and then applying costs one extra
/// pass and is exact.
pub const SELF_COLLIDE: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> cells: array<u32>;
@group(0) @binding(2) var<storage, read> sorted: array<u32>;
@group(0) @binding(3) var<storage, read> starts: array<u32>;
@group(0) @binding(4) var<storage, read_write> corrections: array<atomic<i32>>;
// Vertices that already share a mesh edge are held by a stretch constraint and
// must not also be pushed apart. Sorted per particle, as a start/end range
// into a flat neighbour list.
@group(0) @binding(5) var<storage, read> neighbour_starts: array<u32>;
@group(0) @binding(6) var<storage, read> neighbours: array<u32>;

struct CollideParams {
    count: u32,
    table_size: u32,
    inverse_spacing: f32,
    radius: f32,
};
@group(0) @binding(7) var<uniform> params: CollideParams;

fn cell_of(position: vec3<f32>, inverse_spacing: f32) -> vec3<i32> {
    return vec3<i32>(floor(position * inverse_spacing));
}

fn hash_cell(cell: vec3<i32>, table_size: u32) -> u32 {
    let h = (u32(cell.x) * 73856093u) ^ (u32(cell.y) * 19349663u) ^ (u32(cell.z) * 83492791u);
    return h % table_size;
}

// Corrections accumulate as fixed point, because WGSL has no atomic float.
// A metre is 2^20 units, so the resolution is about a micrometre -- three
// orders of magnitude finer than the thickness being resolved -- and the
// range runs to a kilometre before it could overflow.
const FIXED_SCALE = 1048576.0;

fn accumulate(index: u32, delta: vec3<f32>) {
    atomicAdd(&corrections[index * 4u], i32(delta.x * FIXED_SCALE));
    atomicAdd(&corrections[index * 4u + 1u], i32(delta.y * FIXED_SCALE));
    atomicAdd(&corrections[index * 4u + 2u], i32(delta.z * FIXED_SCALE));
    // The fourth lane counts contributors, so the apply pass can average
    // rather than sum -- summing would over-correct a particle wedged between
    // several others.
    atomicAdd(&corrections[index * 4u + 3u], 1);
}

fn is_connected(a: u32, b: u32) -> bool {
    let start = neighbour_starts[a];
    let end = neighbour_starts[a + 1u];
    for (var i = start; i < end; i = i + 1u) {
        if (neighbours[i] == b) {
            return true;
        }
    }
    return false;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.count) {
        return;
    }

    let entry = positions[index];
    if (entry.w == 0.0) {
        return;
    }
    let position = entry.xyz;
    let diameter = params.radius * 2.0;
    let base = cell_of(position, params.inverse_spacing);

    for (var dz = -1; dz <= 1; dz = dz + 1) {
    for (var dy = -1; dy <= 1; dy = dy + 1) {
    for (var dx = -1; dx <= 1; dx = dx + 1) {
        let bucket = hash_cell(base + vec3<i32>(dx, dy, dz), params.table_size);
        let start = starts[bucket];
        if (start >= params.count) {
            continue;
        }
        // The bucket runs until the hash changes.
        for (var slot = start; slot < params.count; slot = slot + 1u) {
            if (cells[slot] != bucket) {
                break;
            }
            let other = sorted[slot];
            if (other == index) {
                continue;
            }

            let other_entry = positions[other];
            let delta = position - other_entry.xyz;
            let distance = length(delta);
            if (distance >= diameter || distance < 1e-9) {
                continue;
            }
            // Mesh neighbours are already held at the right distance by a
            // stretch constraint. Pushing them apart as well would fight it,
            // and at any sensible resolution the fabric would inflate.
            if (is_connected(index, other)) {
                continue;
            }

            let total = entry.w + other_entry.w;
            if (total <= 0.0) {
                continue;
            }
            // Only this particle's share; the other particle's own thread
            // applies the opposite half.
            let correction = (delta / distance) * ((diameter - distance) * (entry.w / total));
            accumulate(index, correction);
        }
    }
    }
    }
}
"#;

/// Apply the accumulated corrections and reset the accumulator.
pub const APPLY_CORRECTIONS: &str = r#"
@group(0) @binding(0) var<storage, read_write> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> corrections: array<atomic<i32>>;

struct ApplyParams {
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};
@group(0) @binding(2) var<uniform> params: ApplyParams;

const FIXED_SCALE = 1048576.0;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.count) {
        return;
    }

    let contributors = atomicExchange(&corrections[index * 4u + 3u], 0);
    let x = atomicExchange(&corrections[index * 4u], 0);
    let y = atomicExchange(&corrections[index * 4u + 1u], 0);
    let z = atomicExchange(&corrections[index * 4u + 2u], 0);
    if (contributors == 0) {
        return;
    }

    let delta = vec3<f32>(f32(x), f32(y), f32(z)) / (FIXED_SCALE * f32(contributors));
    let entry = positions[index];
    positions[index] = vec4<f32>(entry.xyz + delta, entry.w);
}
"#;
