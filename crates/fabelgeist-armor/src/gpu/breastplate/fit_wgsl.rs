//! The breastplate's section fit in WGSL, and the rear plate's lap onto
//! the front.
//!
//! The fit moves every carrier vertex horizontally, so each keeps its
//! height: the torso section it is measured against is found once per
//! vertex. Each of the fit's up to twenty-five iterations is four
//! dispatches -- apply the profile, measure every vertex's residual with a
//! ray against the torso, pick the first largest, update the profile -- and
//! a converged fit turns the rest into no-ops.

/// Floats of a torso section centre per carrier vertex: lateral and depth
/// centre, and whether the section had enough crossings.
pub(crate) const CENTER_WORDS: u32 = 4;
/// Floats prepared per carrier vertex at the start of a fit: its original
/// position, its radial direction (lateral, depth), side blend, height
/// weights, and whether it has a section.
pub(crate) const PREPARED_WORDS: u32 = 10;
/// Words per carrier vertex a fit iteration measures: residual (as an
/// ordered float, zero when unconstrained) and side blend.
pub(crate) const RESIDUAL_WORDS: u32 = 2;

/// What every fit kernel shares: the profile's evaluation.
pub(crate) const FIT_COMMON: &str = r#"
fn reference_height(point: vec3<f32>) -> f32 {
    return host_add(
        REFERENCE_RIG_NECK_HEIGHT,
        host_div(host_sub(local(point).y, neck_local().y), max(host_mul(y_scale(), plate_length()), 1e-6)),
    );
}

fn height_weights(reference_y: f32, bottom: f32) -> vec3<f32> {
    let t = clamp(
        host_div(host_sub(reference_y, bottom), host_sub(REFERENCE_CARRIER_TOP_HEIGHT, bottom)), 0.0, 1.0
    );
    let rest = host_sub(1.0, t);
    return vec3<f32>(host_mul(rest, rest), host_mul(host_mul(2.0, t), rest), host_mul(t, t));
}

fn weighted_value(weights: vec3<f32>, controls: vec3<f32>) -> f32 {
    return host_dot(weights, controls);
}

// A point's horizontal radial direction about a known section centre: direction
// (lateral, depth), radius, side blend; radius zero when there is none.
fn radial(point: vec3<f32>, center: vec3<f32>) -> vec4<f32> {
    let p = local(point);
    let delta = vec3<f32>(host_sub(p.x, center.x), 0.0, host_sub(p.z, center.y));
    let radius = host_length(delta);
    if (!(radius > 1e-6)) {
        return vec4<f32>(0.0);
    }
    let direction = host_scale3(delta, host_div(1.0, radius));
    let side_blend = host_smoothstep(host_div(host_sub(abs(direction.x), 0.45), 0.45));
    return vec4<f32>(direction.x, direction.z, radius, side_blend);
}
"#;

/// One invocation per carrier vertex: remember where it started, and what
/// of the fit depends only on that.
pub(crate) const PREPARE: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> positions: array<f32>;
@group(0) @binding(2) var<storage, read> centers: array<f32>;
@group(0) @binding(3) var<storage, read_write> prepared: array<f32>;
@group(0) @binding(4) var<uniform> params: Params;

fn center_of(i: u32) -> vec3<f32> {
    let at = i * CENTER_WORDS;
    return vec3<f32>(centers[at], centers[at + 1u], centers[at + 2u]);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let original = positions_at(i);
    let center = center_of(i);
    let at = i * PREPARED_WORDS;
    prepared[at] = original.x;
    prepared[at + 1u] = original.y;
    prepared[at + 2u] = original.z;
    var direction = vec4<f32>(0.0);
    if (center.z != 0.0) {
        direction = radial(original, center);
    }
    let weights = height_weights(reference_height(original), bottom_height(params.rear != 0u));
    prepared[at + 3u] = direction.x;
    prepared[at + 4u] = direction.y;
    prepared[at + 5u] = direction.w;
    prepared[at + 6u] = weights.x;
    prepared[at + 7u] = weights.y;
    prepared[at + 8u] = weights.z;
    prepared[at + 9u] = select(0.0, 1.0, direction.z != 0.0);
}
"#;

/// One invocation per carrier vertex: move its original position along its
/// radial direction by the current profile's offset there.
pub(crate) const APPLY: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> prepared: array<f32>;
@group(0) @binding(2) var<storage, read> profile: array<f32>;
@group(0) @binding(3) var<storage, read_write> positions: array<f32>;
@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let at = i * PREPARED_WORDS;
    let original = vec3<f32>(prepared[at], prepared[at + 1u], prepared[at + 2u]);
    if (prepared[at + 9u] == 0.0) {
        positions_set(i, original);
        return;
    }
    let side_blend = prepared[at + 5u];
    let weights = vec3<f32>(prepared[at + 6u], prepared[at + 7u], prepared[at + 8u]);
    let center_controls = vec3<f32>(profile[0], profile[1], profile[2]);
    let side_controls = vec3<f32>(profile[3], profile[4], profile[5]);
    let offset = host_add(
        host_mul(weighted_value(weights, center_controls), host_sub(1.0, side_blend)),
        host_mul(weighted_value(weights, side_controls), side_blend),
    );
    let direction = vec3<f32>(prepared[at + 3u], 0.0, prepared[at + 4u]);
    positions_set(i, world(host_add3(local(original), host_scale3(direction, offset))));
}
"#;

/// One invocation per carrier vertex: how far inside the clearance it is,
/// measured along its radial ray against the torso; the largest bounds the
/// iteration's witness.
pub(crate) const RESIDUAL: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> positions: array<f32>;
@group(0) @binding(2) var<storage, read> centers: array<f32>;
@group(0) @binding(3) var<storage, read> torso_faces: array<u32>;
@group(0) @binding(4) var<storage, read> body_local: array<f32>;
@group(0) @binding(5) var<storage, read_write> residuals: array<u32>;
@group(0) @binding(6) var<storage, read_write> control: array<atomic<u32>>;
@group(0) @binding(7) var<uniform> params: Params;

fn center_of(i: u32) -> vec3<f32> {
    let at = i * CENTER_WORDS;
    return vec3<f32>(centers[at], centers[at + 1u], centers[at + 2u]);
}

// The torso's radial extent: the nearest torso crossing along a horizontal ray from
// the section centre.
fn body_radius(origin: vec3<f32>, direction: vec3<f32>) -> f32 {
    var nearest = MAX_FINITE;
    for (var f = 0u; f < params.torso_count; f = f + 1u) {
        let a = body_local_at(torso_faces[f * 3u]);
        let b = body_local_at(torso_faces[f * 3u + 1u]);
        let c = body_local_at(torso_faces[f * 3u + 2u]);
        // A quick unrounded test first: most faces are nowhere near the
        // ray, and only a clear miss may skip the exactly rounded test.
        let quick_ac = c - a;
        let quick_h = cross(direction, quick_ac);
        let quick_determinant = dot(b - a, quick_h);
        if (abs(quick_determinant) > 1e-6) {
            let quick_u = dot(origin - a, quick_h) / quick_determinant;
            if (quick_u < -0.01 || quick_u > 1.01) {
                continue;
            }
        }
        let edge_ab = host_sub3(b, a);
        let edge_ac = host_sub3(c, a);
        let h = host_cross(direction, edge_ac);
        let determinant = host_dot(edge_ab, h);
        if (abs(determinant) <= 1e-9) {
            continue;
        }
        let inverse = host_div(1.0, determinant);
        let from_a = host_sub3(origin, a);
        let u = host_mul(inverse, host_dot(from_a, h));
        if (!(u >= -1e-5 && u <= 1.0 + 1e-5)) {
            continue;
        }
        let q = host_cross(from_a, edge_ab);
        let v = host_mul(inverse, host_dot(direction, q));
        if (v < -1e-5 || host_add(u, v) > 1.0 + 1e-5) {
            continue;
        }
        let distance = host_mul(inverse, host_dot(edge_ac, q));
        if (distance >= 1e-6) {
            nearest = min(nearest, distance);
        }
    }
    return nearest;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count || atomicLoad(&control[2]) != 0u) {
        return;
    }
    residuals[i * RESIDUAL_WORDS] = 0u;
    let center = center_of(i);
    if (center.z == 0.0) {
        return;
    }
    let point = positions_at(i);
    let radial_state = radial(point, center);
    if (radial_state.z == 0.0) {
        return;
    }
    let rear = params.rear != 0u;
    if ((rear && radial_state.y >= 0.0) || (!rear && radial_state.y <= 0.0)) {
        return;
    }
    let direction = vec3<f32>(radial_state.x, 0.0, radial_state.y);
    let origin = vec3<f32>(center.x, local(point).y, center.y);
    let extent = body_radius(origin, direction);
    if (!is_finite(extent)) {
        return;
    }
    let clearance = max(plate_clearance(rear), FIT_SURFACE_MARGIN);
    let residual = host_sub(host_add(extent, clearance), radial_state.z);
    if (residual > 1e-5) {
        let ordered = ordered_from_float(residual);
        residuals[i * RESIDUAL_WORDS] = ordered;
        residuals[i * RESIDUAL_WORDS + 1u] = bitcast<u32>(radial_state.w);
        atomicMax(&control[0], ordered);
    }
}
"#;

/// One invocation per carrier vertex: the first vertex with the largest
/// residual is the witness; ties go to the lowest index.
pub(crate) const WITNESS: &str = r#"
@group(0) @binding(0) var<storage, read> residuals: array<u32>;
@group(0) @binding(1) var<storage, read_write> control: array<atomic<u32>>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count || atomicLoad(&control[2]) != 0u) {
        return;
    }
    let largest = atomicLoad(&control[0]);
    if (largest != 0u && residuals[i * RESIDUAL_WORDS] == largest) {
        atomicMin(&control[1], i);
    }
}
"#;

/// One invocation: grow the radial profile to clear the witness's
/// residual, spread over its height weights and side blend; or end the fit.
pub(crate) const UPDATE: &str = r#"
@group(0) @binding(0) var<storage, read> prepared: array<f32>;
@group(0) @binding(1) var<storage, read> residuals: array<u32>;
@group(0) @binding(2) var<storage, read_write> profile: array<f32>;
@group(0) @binding(3) var<storage, read_write> control: array<u32>;
@group(0) @binding(4) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(5) var<uniform> params: Params;

@compute @workgroup_size(1)
fn main() {
    if (control[2] != 0u) {
        return;
    }
    let largest = control[0];
    if (largest == 0u) {
        control[2] = 1u;
        return;
    }
    if (params.iteration == LAST_ITERATION) {
        fail(STATUS_INVALID_SURFACE);
        control[2] = 1u;
        return;
    }
    let witness = control[1];
    let residual = float_from_ordered(largest);
    let side_blend = bitcast<f32>(residuals[witness * RESIDUAL_WORDS + 1u]);
    let at = witness * PREPARED_WORDS;
    let weights = vec3<f32>(prepared[at + 6u], prepared[at + 7u], prepared[at + 8u]);
    let center_blend = host_sub(1.0, side_blend);
    let spread = host_add(host_mul(center_blend, center_blend), host_mul(side_blend, side_blend));
    let denominator = host_add(
        host_add(
            host_mul(host_mul(weights.x, weights.x), spread), host_mul(host_mul(weights.y, weights.y), spread)
        ),
        host_mul(host_mul(weights.z, weights.z), spread),
    );
    if (residual > 0.0 && denominator > 1e-8) {
        for (var k = 0u; k < 3u; k = k + 1u) {
            let share = host_mul(residual, weights[k]);
            profile[k] = host_add(profile[k], host_div(host_mul(share, center_blend), denominator));
            profile[3u + k] = host_add(profile[3u + k], host_div(host_mul(share, side_blend), denominator));
        }
    }
    control[0] = 0u;
    control[1] = 0xffffffffu;
}
"#;

/// One invocation: reject a fit that travelled further than the carrier's
/// scale allows.
pub(crate) const FINISH: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> profile: array<f32>;
@group(0) @binding(2) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(1)
fn main() {
    for (var k = 0u; k < 6u; k = k + 1u) {
        let value = profile[k];
        if (!is_finite(value) || value > host_mul(MAX_FIT_CORRECTION, z_scale())) {
            fail(STATUS_INVALID_SURFACE);
        }
    }
}
"#;

/// One invocation per rear carrier row: for one side, close the rear
/// plate's width onto the front's edge, fading out above the lap's height.
pub(crate) const LAP: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> front: array<f32>;
@group(0) @binding(2) var<storage, read_write> positions: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

const LAP_CLEARANCE: f32 = 0.003;
const LAP_FULL_HEIGHT: f32 = 1.17;
const LAP_FADE_HEIGHT: f32 = 0.10;
const LAP_START_U: f32 = 0.50;
const ROWS: u32 = V_SAMPLES + SKIRT_SAMPLES - 1u;

fn front_at(i: u32) -> vec3<f32> {
    return vec3<f32>(front[i * 3u], front[i * 3u + 1u], front[i * 3u + 2u]);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let row = id.x;
    if (row >= ROWS) {
        return;
    }
    let width = params.width;
    let side = select(-1.0, 1.0, params.side != 0u);
    let column = select(0u, width - 1u, params.side != 0u);
    var edge: array<vec3<f32>, ROWS>;
    for (var r = 0u; r < ROWS; r = r + 1u) {
        let point = local(front_at(r * width + column));
        var j = r;
        loop {
            if (j == 0u || !(edge[j - 1u].y > point.y)) {
                break;
            }
            edge[j] = edge[j - 1u];
            j = j - 1u;
        }
        edge[j] = point;
    }
    let original = positions_at(row * width + column);
    let original_local = local(original);
    let height_blend = host_sub(
        1.0, host_smoothstep(host_div(host_sub(reference_height(original), LAP_FULL_HEIGHT), LAP_FADE_HEIGHT))
    );
    if (height_blend == 0.0) {
        return;
    }
    let height = original_local.y;
    var front_point = edge[ROWS - 1u];
    if (height < edge[0].y) {
        front_point = edge[0];
    }
    for (var r = 0u; r + 1u < ROWS; r = r + 1u) {
        if (edge[r].y <= height && height <= edge[r + 1u].y) {
            let t = host_div(host_sub(height, edge[r].y), max(host_sub(edge[r + 1u].y, edge[r].y), 1e-6));
            front_point = host_add3(host_scale3(edge[r], host_sub(1.0, t)), host_scale3(edge[r + 1u], t));
            break;
        }
    }
    let target_x = host_add(front_point.x, host_mul(side, host_add(wall_thickness(), LAP_CLEARANCE)));
    let offset = world(vec3<f32>(host_sub(target_x, original_local.x), 0.0, 0.0));
    for (var index = 0u; index < width; index = index + 1u) {
        let u = host_add(-1.0, host_div(host_mul(2.0, f32(index)), f32(width - 1u)));
        let lateral_blend = host_smoothstep(
            host_div(host_sub(host_mul(u, side), LAP_START_U), host_sub(1.0, LAP_START_U))
        );
        let at = row * width + index;
        positions_set(
            at, host_add3(positions_at(at), host_scale3(offset, host_mul(height_blend, lateral_blend)))
        );
    }
}
"#;
