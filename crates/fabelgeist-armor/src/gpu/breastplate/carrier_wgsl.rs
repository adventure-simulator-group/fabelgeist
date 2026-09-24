//! The unfluted carriers in WGSL: the main grid and the skirt below it.
//!
//! The regular chart's samples are evaluated once each -- the long exact
//! shape functions run once per sample -- and the grid resamples them, with
//! the skirt hanging from the chart's first row.

/// Floats of a regular chart sample: carrier point, world normal, chart
/// angle and the authored section's depth.
pub(crate) const COARSE_WORDS: u32 = 8;

/// One invocation per sample of the regular chart's main grid: the coarse
/// carrier, before the grid resamples it onto the plate's columns.
pub(crate) const COARSE_ENTRY: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read_write> coarse: array<f32>;
@group(0) @binding(2) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= U_SAMPLES * V_SAMPLES) {
        return;
    }
    let rear = params.rear != 0u;
    let row = i / U_SAMPLES;
    let u = host_add(-1.0, host_div(host_mul(2.0, f32(i % U_SAMPLES)), f32(U_SAMPLES - 1u)));
    let t = host_div(f32(row), f32(V_SAMPLES - 1u));
    let bottom = bottom_height(rear);
    let y = host_add(bottom, host_mul(t, host_sub(top_y(rear, u), bottom)));
    let theta = chart_theta(rear, u, y);
    let sample = carrier_point(rear, theta, y);
    if (shape_failed) {
        fail(STATUS_DEGENERATE);
    }
    let at = i * COARSE_WORDS;
    var words = array<f32, 8>(
        sample.point.x, sample.point.y, sample.point.z,
        sample.normal.x, sample.normal.y, sample.normal.z,
        theta, sample.raw_depth,
    );
    for (var k = 0u; k < COARSE_WORDS; k = k + 1u) {
        coarse[at + k] = words[k];
    }
}
"#;

/// One invocation per carrier vertex of an unfluted plate: the main grid's
/// rows resampled from the regular chart, then the skirt below the seam,
/// which hangs from the chart's first row.
pub(crate) const GRID_ENTRY: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> columns: array<f32>;
@group(0) @binding(2) var<storage, read> coarse: array<f32>;
@group(0) @binding(3) var<storage, read_write> positions: array<f32>;
@group(0) @binding(4) var<uniform> params: Params;

fn coarse_word(sample: u32, word: u32) -> f32 {
    return coarse[sample * COARSE_WORDS + word];
}

fn coarse_vector(sample: u32, word: u32) -> vec3<f32> {
    return vec3<f32>(
        coarse_word(sample, word), coarse_word(sample, word + 1u), coarse_word(sample, word + 2u)
    );
}

// The skirt's point at `t` below column `k` of the seam.
fn skirt_point(rear: bool, k: u32, u: f32, t: f32) -> vec3<f32> {
    let bottom = bottom_height(rear);
    let seam_theta = coarse_word(k, 6u);
    let edge_theta = coarse_word(U_SAMPLES - 1u, 6u);
    var drop: f32 = 0.058;
    var lateral_share: f32 = 1.0;
    var sagittal_share: f32 = 0.75;
    var a0 = FRONT_RADIUS_X[0];
    var b0 = FRONT_RADIUS_Z[0];
    var bias: f32 = 0.006;
    if (rear) {
        drop = 0.056;
        lateral_share = 5.0 / 6.0;
        sagittal_share = 1.0;
        a0 = BACK_RADIUS_X[0];
        b0 = host_mul(BACK_RADIUS_Z[0], back_depth());
        bias = -0.004;
    }
    let radial_flare = skirt_flare();
    let lateral_flare = host_mul(radial_flare, lateral_share);
    let sagittal_flare = host_mul(radial_flare, sagittal_share);
    let flare_ratio = host_div(radial_flare, 0.030);
    let angular_inset = host_mul(host_mul(host_mul(2.0, t), side_return()), RADIANS_PER_DEGREE);
    let angle = host_mul(
        seam_theta,
        host_sub(1.0, host_div(angular_inset, max(abs(edge_theta), host_add(angular_inset, 1e-6))))
    );
    // Sines and cosines of the skirt's angle and of the seam's.
    var sines: array<f32, 2>;
    var cosines: array<f32, 2>;
    for (var j = 0u; j < opaque(2u); j = j + 1u) {
        let at = select(angle, seam_theta, j == 1u);
        sines[j] = host_sin(at);
        cosines[j] = host_cos(at);
    }
    let x = host_mul(host_add(a0, host_mul(lateral_flare, t)), sines[0]);
    var point_drop = 0.0;
    if (!rear) {
        point_drop = host_mul(waist_point(), waist_point_weight_of(sines[1]));
    }
    let y = host_sub(host_sub(bottom, host_mul(host_mul(drop, skirt_length()), t)), point_drop);
    let center_bias = host_mul(host_mul(host_mul(bias, flare_ratio), t), host_sub(1.0, host_mul(u, u)));
    let reach = host_add(b0, host_mul(sagittal_flare, t));
    var z: f32;
    if (rear) {
        z = host_add(host_mul(-reach, cosines[0]), center_bias);
    } else {
        let waist = host_sub(coarse_word(k, 7u), host_mul(b0, cosines[1]));
        z = host_add(host_add(host_mul(reach, cosines[0]), center_bias), waist);
    }
    let mapped = mapped_point(vec3<f32>(x, y, z));
    let normal = coarse_vector(k, 3u);
    return world(host_add3(mapped, host_scale3(local(normal), plate_clearance(rear))));
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let width = params.width;
    if (i >= width * (V_SAMPLES + SKIRT_SAMPLES - 1u)) {
        return;
    }
    let rear = params.rear != 0u;
    let row = i / width;
    let column = i % width;
    let u = columns[column];
    var point: vec3<f32>;
    if (row < V_SAMPLES) {
        let sample = clamp(
            host_mul(host_mul(host_add(u, 1.0), 0.5), f32(U_SAMPLES - 1u)), 0.0, f32(U_SAMPLES - 1u)
        );
        let lower = min(u32(floor(sample)), U_SAMPLES - 2u);
        let blend = host_sub(sample, f32(lower));
        let first = row * U_SAMPLES + lower;
        point = host_add3(
            host_scale3(coarse_vector(first, 0u), host_sub(1.0, blend)),
            host_scale3(coarse_vector(first + 1u, 0u), blend),
        );
    } else {
        let t = host_div(f32(row - V_SAMPLES + 1u), f32(SKIRT_SAMPLES - 1u));
        point = skirt_point(rear, column, u, t);
    }
    positions_set(i, point);
}
"#;
