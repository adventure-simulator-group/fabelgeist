//! The fitted breastplate finished in WGSL: the plates' extrusion, the
//! front's refinement and flutes, and the solid shell's placement.

/// Words of a refined vertex's place on the coarse carrier: the two coarse
/// vertices it lies between and how far along.
pub(crate) const CARRIER_WORDS: u32 = 3;
/// Rows of the front's upper rim whose extrusion continues the row below.
pub(crate) const UPPER_RIM_ROWS: u32 = 3;

/// One invocation per plate vertex: its area-weighted normal, summed in
/// face order with every step rounded in a fixed order, so the sum is
/// deterministic; the flutes are raised along these.
pub(crate) const PLATE_NORMALS: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> faces: array<u32>;
@group(0) @binding(2) var<storage, read> offsets: array<u32>;
@group(0) @binding(3) var<storage, read> incident: array<u32>;
@group(0) @binding(4) var<storage, read_write> normals: array<f32>;
@group(0) @binding(5) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(6) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    var sum = vec3<f32>(0.0);
    for (var k = offsets[i]; k < offsets[i + 1u]; k = k + 1u) {
        let face = incident[k];
        let a = positions_at(faces[face * 3u]);
        let normal = host_cross(
            host_sub3(positions_at(faces[face * 3u + 1u]), a),
            host_sub3(positions_at(faces[face * 3u + 2u]), a)
        );
        let area = host_dot(normal, normal);
        if (area <= 1e-16 || !is_finite(area)) {
            fail(STATUS_DEGENERATE);
        }
        sum = host_add3(sum, normal);
    }
    let normalized = unit(sum);
    if (normalized.w == 0.0) {
        fail(STATUS_DEGENERATE);
    }
    normals_set(i, normalized.xyz);
}
"#;

/// One invocation per vertex of the front's upper rim rows: continue the
/// extrusion field from the row below the rim.
pub(crate) const RIM: &str = r#"
@group(0) @binding(0) var<storage, read_write> normals: array<f32>;
@group(0) @binding(1) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(2) var<uniform> params: Params;


@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let width = params.width;
    if (i >= UPPER_RIM_ROWS * width) {
        return;
    }
    let start = V_SAMPLES - 1u - UPPER_RIM_ROWS;
    let row = start + 1u + i / width;
    let column = i % width;
    let blend = host_div(f32(row - start), f32(UPPER_RIM_ROWS));
    let index = row * width + column;
    let blended = unit(
        host_add3(
            host_scale3(normals_at(index), host_sub(1.0, blend)),
            host_scale3(normals_at(start * width + column), blend)
        )
    );
    if (blended.w == 0.0) {
        fail(STATUS_DEGENERATE);
    }
    normals_set(index, blended.xyz);
}
"#;

/// One invocation per vertex of the refined front: its place on the fitted
/// coarse carrier, with flute relief along the carrier's extrusion.
pub(crate) const REFINE: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> coarse: array<f32>;
@group(0) @binding(2) var<storage, read> coarse_normals: array<f32>;
@group(0) @binding(3) var<storage, read> columns: array<f32>;
@group(0) @binding(4) var<storage, read_write> positions: array<f32>;
@group(0) @binding(5) var<storage, read_write> extrusion: array<f32>;
@group(0) @binding(6) var<storage, read_write> carrier: array<f32>;
@group(0) @binding(7) var<uniform> params: Params;

fn lateral(i: u32) -> f32 {
    return host_dot(coarse_at(i), frame_lateral());
}

// `PlateFluting::fan_coordinate` over the whole chart, in the signed chart coordinate.
fn fan(u: f32, t: f32) -> f32 {
    let span = flute_spread();
    let spread = host_add(
        flute_lower_spread(), host_mul(host_sub(1.0, flute_lower_spread()), host_smoothstep(t))
    );
    let absolute = abs(u);
    var mapped: f32;
    if (absolute <= span) {
        mapped = host_mul(absolute, spread);
    } else {
        mapped = host_add(
            host_mul(span, spread),
            host_div(
                host_mul(host_sub(absolute, span), host_sub(1.0, host_mul(span, spread))), host_sub(1.0, span)
            )
        );
    }
    return sign(u) * mapped;
}

// Where a flute column samples the coarse carrier: flute columns laid out in lateral
// planes of the fitted carrier, fading back to the fan coordinate outside the flutes.
fn flute_coordinate(u: f32, row: u32) -> f32 {
    let t = host_div(f32(row), f32(V_SAMPLES - 1u));
    let baseline = host_mul(host_add(fan(u, t), 1.0), 0.5);
    let activity = host_mul(
        host_smoothstep(host_div(t, flute_start())),
        host_smoothstep(host_div(host_sub(1.0, t), host_sub(1.0, flute_end())))
    );
    if (activity == 0.0 || u == 0.0) {
        return baseline;
    }
    let reference_row = V_SAMPLES / 2u * U_SAMPLES;
    let left = lateral(reference_row);
    let right = lateral(reference_row + U_SAMPLES - 1u);
    let current = row * U_SAMPLES;
    let current_left = lateral(current);
    let current_right = lateral(current + U_SAMPLES - 1u);
    let center = lateral(current + U_SAMPLES / 2u);
    let half_width = min(
        min(host_mul(host_sub(right, left), 0.5), host_sub(center, current_left)),
        host_sub(current_right, center)
    );
    let span = flute_spread();
    let spread = host_add(
        flute_lower_spread(), host_mul(host_sub(1.0, flute_lower_spread()), host_smoothstep(t))
    );
    let target_lateral = host_add(center, host_mul(host_mul(clamp(u, -span, span), half_width), spread));
    var along = baseline;
    for (var column = 0u; column + 1u < U_SAMPLES; column = column + 1u) {
        let a = lateral(current + column);
        let b = lateral(current + column + 1u);
        if (a <= target_lateral && target_lateral <= b && host_sub(b, a) > 1e-8) {
            along = host_div(
                host_add(f32(column), host_div(host_sub(target_lateral, a), host_sub(b, a))),
                f32(U_SAMPLES - 1u)
            );
            break;
        }
    }
    if (abs(u) > span) {
        let endpoint = select(1.0, 0.0, u < 0.0);
        along = host_add(
            along, host_div(host_mul(host_sub(endpoint, along), host_sub(abs(u), span)), host_sub(1.0, span))
        );
    }
    return host_add(host_mul(baseline, host_sub(1.0, activity)), host_mul(along, activity));
}

// The flutes' relief at a main-grid vertex: `PlateFluting::relief` in the signed chart
// coordinate.
fn flute_relief_at(u: f32, row: u32) -> f32 {
    let t = host_div(f32(row), f32(V_SAMPLES - 1u));
    let fade = host_mul(
        host_smoothstep(host_div(host_sub(t, flute_start()), flute_fade())),
        host_smoothstep(host_div(host_sub(flute_end(), t), flute_fade())),
    );
    let pitch = host_div(host_mul(2.0, flute_spread()), flute_count());
    let half_width = host_mul(host_mul(pitch, flute_width()), 0.5);
    let slot = floor(host_div(host_add(u, flute_spread()), pitch));
    if (slot < 0.0 || slot >= flute_count()) {
        return 0.0;
    }
    let center = host_add(-flute_spread(), host_mul(host_add(slot, 0.5), pitch));
    let distance = host_div(abs(host_sub(u, center)), half_width);
    if (distance >= 1.0) {
        return 0.0;
    }
    let relief = host_mul(host_add(1.0, host_cos(host_mul(PI, distance))), 0.5);
    return host_mul(host_mul(flute_depth(), relief), fade);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let width = params.width;
    if (i >= width * (V_SAMPLES + SKIRT_SAMPLES - 1u)) {
        return;
    }
    if (!fluted()) {
        positions_set(i, coarse_at(i));
        extrusion_set(i, coarse_normals_at(i));
        carrier[i * CARRIER_WORDS] = bitcast<f32>(i);
        carrier[i * CARRIER_WORDS + 1u] = bitcast<f32>(i);
        carrier[i * CARRIER_WORDS + 2u] = 0.0;
        return;
    }
    let row = i / width;
    let u = columns[i % width];
    var start: u32;
    var along: f32;
    if (row < V_SAMPLES) {
        start = row * U_SAMPLES;
        along = flute_coordinate(u, row);
    } else {
        start = U_SAMPLES * V_SAMPLES + (row - V_SAMPLES) * U_SAMPLES;
        along = host_mul(host_add(fan(u, 0.0), 1.0), 0.5);
    }
    let sample = host_mul(clamp(along, 0.0, 1.0), f32(U_SAMPLES - 1u));
    let lower = min(u32(floor(sample)), U_SAMPLES - 2u);
    let t = host_sub(sample, f32(lower));
    let a = start + lower;
    var point = host_add3(host_scale3(coarse_at(a), host_sub(1.0, t)), host_scale3(coarse_at(a + 1u), t));
    let direction = host_add3(
        host_scale3(coarse_normals_at(a), host_sub(1.0, t)), host_scale3(coarse_normals_at(a + 1u), t)
    );
    if (row < V_SAMPLES) {
        let relief = flute_relief_at(u, row);
        if (relief != 0.0) {
            point = host_add3(point, host_scale3(direction, relief));
        }
    }
    positions_set(i, point);
    extrusion_set(i, direction);
    carrier[i * CARRIER_WORDS] = bitcast<f32>(a);
    carrier[i * CARRIER_WORDS + 1u] = bitcast<f32>(a + 1u);
    carrier[i * CARRIER_WORDS + 2u] = t;
}
"#;

/// One invocation per solid vertex: its mid vertex, or a gauge outward.
pub(crate) const SOLID: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> sources: array<u32>;
@group(0) @binding(2) var<storage, read> front: array<f32>;
@group(0) @binding(3) var<storage, read> front_extrusion: array<f32>;
@group(0) @binding(4) var<storage, read> back: array<f32>;
@group(0) @binding(5) var<storage, read> back_extrusion: array<f32>;
@group(0) @binding(6) var<storage, read_write> positions: array<f32>;
@group(0) @binding(7) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let source = sources[i];
    let mid = source & 0x7fffffffu;
    var point: vec3<f32>;
    var direction: vec3<f32>;
    if (mid < params.front_count) {
        point = front_at(mid);
        direction = front_extrusion_at(mid);
    } else {
        point = back_at(mid - params.front_count);
        direction = back_extrusion_at(mid - params.front_count);
    }
    if ((source & 0x80000000u) != 0u) {
        point = point + direction * wall_thickness();
    }
    positions_set(i, point);
}
"#;
