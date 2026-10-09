//! Local radial torso support, independent of global shape controls.

/// Per-carrier-point lateral/depth section center and validity.
pub(crate) const CENTER_WORDS: u32 = 4;

/// Main rows run waist to neck; appended skirt rows run waist to hem.
/// Neighborhoods use physical hem-to-neck order, never storage adjacency.
pub(crate) const NEIGHBORS: &str = r#"
fn neighbor_row(row: u32, delta: i32, rows: u32) -> u32 {
    var ordered = V_SAMPLES + SKIRT_SAMPLES - 2u - row;
    if (row < V_SAMPLES) { ordered = SKIRT_SAMPLES - 1u + row; }
    let next = u32(clamp(i32(ordered) + delta, 0, i32(rows) - 1));
    if (next < SKIRT_SAMPLES - 1u) { return V_SAMPLES + SKIRT_SAMPLES - 2u - next; }
    return next - (SKIRT_SAMPLES - 1u);
}
"#;

pub(crate) const FIT_COMMON: &str = r#"
fn reference_height(point: vec3<f32>) -> f32 {
    return host_add(
        REFERENCE_RIG_NECK_HEIGHT,
        host_div(host_sub(local(point).y, neck_local().y), max(host_mul(y_scale(), plate_length()), 1e-6)),
    );
}

fn radial(point: vec3<f32>, center: vec3<f32>) -> vec4<f32> {
    let p = local(point);
    let delta = vec3<f32>(host_sub(p.x, lateral_origin()), 0.0, host_sub(p.z, coronal_origin()));
    let radius = host_length(delta);
    if (!(radius > 1e-6)) {
        return vec4<f32>(0.0);
    }
    let direction = host_scale3(delta, host_div(1.0, radius));
    let side_blend = host_smoothstep(host_div(host_sub(abs(direction.x), 0.45), 0.45));
    return vec4<f32>(direction.x, direction.z, radius, side_blend);
}
"#;

pub(crate) const MEASURE: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> positions: array<f32>;
@group(0) @binding(2) var<storage, read> centers: array<f32>;
@group(0) @binding(3) var<storage, read> torso_faces: array<u32>;
@group(0) @binding(4) var<storage, read> body_local: array<f32>;
@group(0) @binding(5) var<storage, read_write> support_radii: array<f32>;
@group(0) @binding(6) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(7) var<storage, read> columns: array<f32>;
@group(0) @binding(8) var<uniform> params: Params;
fn body_radius(origin: vec3<f32>, direction: vec3<f32>) -> f32 {
    // The fixed chart origin need not lie inside every torso section. Its
    // outermost forward hit is the envelope; a nearer entry hit is not.
    var extent = 0.0;
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
            extent = max(extent, distance);
        }
    }
    return select(MAX_FINITE, extent, extent > 0.0);
}


@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x + params.extra;
    if (id.x >= params.count) { return; }
    let center = vec3<f32>(centers[i * CENTER_WORDS], centers[i * CENTER_WORDS + 1u], centers[i * CENTER_WORDS + 2u]);
    let point = positions_at(i);
    let state = radial(point, center);
    // An unsupported ray retains the authored section as its required radius.
    // It participates in the same envelope as occupied rays: leaving it
    // unchanged after contracting its neighbours would introduce a sharp fold.
    support_radii[i] = state.z;
    if (center.z == 0.0 || state.z == 0.0) { return; }
    let rear = params.rear != 0u;
    if ((rear && state.y >= 0.0) || (!rear && state.y <= 0.0)) { return; }
    let extent = body_radius(vec3<f32>(lateral_origin(), local(point).y, coronal_origin()), vec3<f32>(state.x, 0.0, state.y));
    if (!is_finite(extent)) { return; }
    var relief = max(host_mul(host_sub(back_depth(), 1.0), extent), 0.0);
    if (!rear) {
        let phase = clamp(host_div(host_sub(reference_height(point), FRONT_HEIGHTS[0]),
            host_sub(FRONT_NECK_Y[0], FRONT_HEIGHTS[0])), 0.0, 1.0);
        relief = max(host_mul(host_mul(front_profile_depth(phase, state.x, state.y), z_scale()), state.y), 0.0);
    }
    let row = i / params.width;
    if (row >= V_SAMPLES) {
        let t = host_div(f32(row - V_SAMPLES + 1u), f32(SKIRT_SAMPLES - 1u));
        let u = columns[i % params.width];
        relief = host_add(relief, skirt_radial_relief(rear, u, t, state.xy));
    }
    let support = host_add(host_add(extent, max(plate_clearance(rear), FIT_SURFACE_MARGIN)), relief);
    if (point.y <= centers[i * CENTER_WORDS + 3u] &&
        host_sub(support, state.z) > host_mul(MAX_FIT_CORRECTION, z_scale())) { fail(STATUS_INVALID_SURFACE); }
    support_radii[i] = support;
}
"#;

/// Dilation followed by averaging is smooth and never lowers any point's
/// required support: every neighborhood contributing to it includes that point.
pub(crate) const ENVELOPE: &str = r#"
@group(0) @binding(0) var<storage, read> support_radii: array<f32>;
@group(0) @binding(1) var<storage, read_write> envelope: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) { return; }
    let row = i / params.width; let column = i % params.width;
    let rows = params.count / params.width;
    var support = 0.0;
    for (var dr = -1; dr <= 1; dr += 1) {
        for (var dc = -1; dc <= 1; dc += 1) {
            let r = neighbor_row(row, dr, rows);
            let c = u32(clamp(i32(column) + dc, 0, i32(params.width) - 1));
            support = max(support, support_radii[r * params.width + c]);
        }
    }
    envelope[i] = support;
}
"#;

pub(crate) const APPLY: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read_write> positions: array<f32>;
@group(0) @binding(2) var<storage, read> centers: array<f32>;
@group(0) @binding(3) var<storage, read> envelope: array<f32>;
@group(0) @binding(4) var<storage, read> support_radii: array<f32>;
@group(0) @binding(5) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(6) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count || support_radii[i] == 0.0) { return; }
    let center = vec3<f32>(centers[i * CENTER_WORDS], centers[i * CENTER_WORDS + 1u], centers[i * CENTER_WORDS + 2u]);
    let point = positions_at(i);
    let state = radial(point, center);
    let row = i / params.width; let column = i % params.width;
    let rows = params.count / params.width;
    var sum = 0.0;
    for (var dr = -1; dr <= 1; dr += 1) {
        for (var dc = -1; dc <= 1; dc += 1) {
            let r = neighbor_row(row, dr, rows);
            let c = u32(clamp(i32(column) + dc, 0, i32(params.width) - 1));
            sum = host_add(sum, envelope[r * params.width + c]);
        }
    }
    var radius = host_div(sum, 9.0);
    if (params.radial_fit == 1u) { radius = max(radius, state.z); }
    let correction = host_sub(radius, state.z);
    if (point.y <= centers[i * CENTER_WORDS + 3u] &&
        correction > host_mul(MAX_FIT_CORRECTION, z_scale())) { fail(STATUS_INVALID_SURFACE); }
    positions_set(i, world(host_add3(local(point), host_scale3(vec3<f32>(state.x, 0.0, state.y), correction))));
}
"#;

/// The neckline cuts the fitted sheet; it is not another independent section.
/// Evaluate its boundary on the same physical rails used for the carrier.
pub(crate) const TRIM_CARRIER: &str = r#"
@group(0) @binding(0) var<storage, read_write> positions: array<f32>;
@group(0) @binding(1) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let column = id.x;
    if (column >= params.width) { return; }
    let index = (V_SAMPLES - 1u) * params.width + column;
    let height = positions_at(index).y;
    var a = positions_at(column);
    var b = positions_at(params.width + column);
    for (var row = 0u; row < V_SAMPLES - 2u; row += 1u) {
        a = positions_at(row * params.width + column);
        b = positions_at((row + 1u) * params.width + column);
        if (height <= b.y) { break; }
    }
    let t = host_div(host_sub(height, a.y), host_sub(b.y, a.y));
    let point = host_add3(host_scale3(a, host_sub(1.0, t)), host_scale3(b, t));
    positions_set(index, vec3<f32>(point.x, height, point.z));
}
"#;

/// Check the newly evaluated trim against its own actual body ray.
pub(crate) const CHECK_SUPPORT: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> positions: array<f32>;
@group(0) @binding(2) var<storage, read> support_radii: array<f32>;
@group(0) @binding(3) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(4) var<uniform> params: Params;
@group(0) @binding(5) var<storage, read> arm_distances: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.count) { return; }
    let i = id.x + params.extra;
    // This neckline sample is discarded by the independent arm trim.
    if (arm_distances[i] > 0.0) { return; }
    if (support_radii[i] > radial(positions_at(i), vec3<f32>(0.0)).z + 0.00001) {
        fail(STATUS_INVALID_SURFACE);
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
        let front_column = select(0u, params.front_count - 1u, params.side != 0u);
        let point = local(front_at(r * params.front_count + front_column));
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
    if (height_blend == 0.0 && side_lap_reserve() == 0.0) {
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
    // Front and rear courses can have different chevron heights and lift
    // phases at one side closure. Reserve their maximum relative lift even
    // where the solid plate's authored side lap fades toward the armscye.
    let reserve = world(vec3<f32>(host_mul(side, side_lap_reserve()), 0.0, 0.0));
    for (var index = 0u; index < width; index = index + 1u) {
        let u = host_add(-1.0, host_div(host_mul(2.0, f32(index)), f32(width - 1u)));
        let lateral_blend = host_smoothstep(
            host_div(host_sub(host_mul(u, side), LAP_START_U), host_sub(1.0, LAP_START_U))
        );
        let at = row * width + index;
        positions_set(
            at, host_add3(positions_at(at), host_scale3(
                host_add3(host_scale3(offset, height_blend), reserve), lateral_blend))
        );
    }
}
"#;
