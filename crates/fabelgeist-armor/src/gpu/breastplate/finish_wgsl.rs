//! The fitted breastplate finished in WGSL: the plates' extrusion, the
//! front's refinement and flutes, and the solid shell's placement.

/// Words of a refined vertex's place on the coarse carrier: the two coarse
/// vertices it lies between and how far along.
pub(crate) const CARRIER_WORDS: u32 = 3;

/// Derivatives of the seated carrier at a detail-independent chart spacing.
/// Uneven flute columns must not make metal-gauge directions depend on tiny
/// triangles or on which incident diagonal contributes the most area.
pub(crate) const PLATE_NORMALS: &str = r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> columns: array<f32>;
@group(0) @binding(2) var<storage, read_write> normals: array<f32>;
@group(0) @binding(3) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(4) var<uniform> params: Params;

// Different neckline heights do not define a lateral derivative. Interpolate
// each column rail at the queried physical height before differencing it.
// At a trim edge, continue the nearest carrier interval; the trim does not
// change the underlying sheet's tangent field.
fn rail_at_height(column: u32, height: f32, skirt: bool) -> vec3<f32> {
    var a = positions_at(column);
    var b = positions_at(params.width + column);
    if (skirt) { b = positions_at(V_SAMPLES * params.width + column); }
    let intervals = select(V_SAMPLES - 2u, SKIRT_SAMPLES - 1u, skirt);
    for (var row = 0u; row < intervals; row += 1u) {
        let first = select(row, select(V_SAMPLES + row - 1u, 0u, row == 0u), skirt);
        let second = select(row + 1u, V_SAMPLES + row, skirt);
        a = positions_at(first * params.width + column);
        b = positions_at(second * params.width + column);
        if (select(height <= b.y, height >= b.y, skirt)) { break; }
    }
    let blend = host_div(host_sub(height, a.y), host_sub(b.y, a.y));
    return host_add3(host_scale3(a, host_sub(1.0, blend)), host_scale3(b, blend));
}
fn at_column(row: u32, u: f32, height: f32) -> vec3<f32> {
    var low = 0u;
    var high = params.width - 1u;
    while (low + 1u < high) {
        let middle = (low + high) / 2u;
        if (columns[middle] <= u) { low = middle; } else { high = middle; }
    }
    let blend = clamp(host_div(host_sub(u, columns[low]), host_sub(columns[high], columns[low])), 0.0, 1.0);
    return host_add3(host_scale3(rail_at_height(low, height, row >= V_SAMPLES), host_sub(1.0, blend)),
        host_scale3(rail_at_height(high, height, row >= V_SAMPLES), blend));
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let row = i / params.width;
    let column = i % params.width;
    let u = columns[column];
    let span = host_div(2.0, f32(U_SAMPLES - 1u));
    let height = positions_at(i).y;
    let lateral = host_sub3(at_column(row, min(host_add(u, span), 1.0), height),
        at_column(row, max(host_sub(u, span), -1.0), height));
    var below = select(row - 1u, 0u, row == 0u);
    var above = min(row + 1u, V_SAMPLES - 2u);
    var vertical: vec3<f32>;
    if (row >= V_SAMPLES) {
        below = select(row - 1u, 0u, row == V_SAMPLES);
        above = min(row + 1u, V_SAMPLES + SKIRT_SAMPLES - 2u);
        vertical = host_sub3(positions_at(above * params.width + column),
            positions_at(below * params.width + column));
    } else {
        let step = host_div(host_sub(positions_at((V_SAMPLES - 2u) * params.width + column).y,
            positions_at(column).y), f32(V_SAMPLES - 2u));
        vertical = host_sub3(rail_at_height(column, host_add(height, step), false),
            rail_at_height(column, host_sub(height, step), false));
    }
    var normal = host_cross(lateral, vertical);
    if ((row >= V_SAMPLES) != (params.rear != 0u)) { normal = -normal; }
    let normalized = unit(normal);
    if (normalized.w == 0.0) {
        fail(STATUS_DEGENERATE);
    }
    normals_set(i, normalized.xyz);
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

// The flutes' relief at a main-grid vertex: `PlateFluting::relief` in the signed chart
// coordinate.
fn flute_relief_at(u: f32, point: vec3<f32>) -> f32 {
    let t = clamp(host_div(host_sub(reference_height(point), FRONT_HEIGHTS[0]),
        host_sub(FRONT_NECK_Y[0], FRONT_HEIGHTS[0])), 0.0, 1.0);
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
    let row = i / width;
    var point = coarse_at(i);
    let direction = coarse_normals_at(i);
    if (fluted() && row < V_SAMPLES) {
        let p = local(point);
        let relief_direction = world(unit(vec3<f32>(host_sub(p.x, lateral_origin()), 0.0,
            host_sub(p.z, coronal_origin()))).xyz);
        point = host_add3(point, host_scale3(relief_direction, flute_relief_at(columns[i % width], point)));
    }
    positions_set(i, point);
    extrusion_set(i, direction);
    carrier[i * CARRIER_WORDS] = bitcast<f32>(i);
    carrier[i * CARRIER_WORDS + 1u] = bitcast<f32>(i);
    carrier[i * CARRIER_WORDS + 2u] = 0.0;
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
