//! The unfluted carriers in WGSL: the main grid and the skirt below it.
//!
//! Every authored column is evaluated directly at the final carrier resolution,
//! with the skirt hanging from the chart's first row.

/// Floats of a regular chart sample: carrier point, chart
/// angle and the authored section's depth.
pub(crate) const COARSE_WORDS: u32 = 5;

/// One invocation per sample of the regular chart's main grid: the coarse
/// carrier, before the grid resamples it onto the plate's columns.
pub(crate) const COARSE_ENTRY: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read_write> coarse: array<f32>;
@group(0) @binding(2) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(3) var<storage, read> columns: array<f32>;
@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.width * V_SAMPLES) {
        return;
    }
    let rear = params.rear != 0u;
    let row = i / params.width;
    let column = columns[i % params.width];
    let t = host_div(f32(row), f32(V_SAMPLES - 1u));
    let bottom = bottom_height(rear);
    let y = sample_y(rear, column, t);
    var u = column;
    if (!rear && fluted()) {
        let phase = clamp(host_div(host_sub(y, bottom), host_sub(neckline_y(rear, 1.0), bottom)), 0.0, 1.0);
        u = fan(column, phase);
    }
    let theta = chart_theta(rear, u, y);
    let sample = carrier_point(rear, theta, y);
    if (shape_failed) {
        fail(STATUS_DEGENERATE);
    }
    let at = i * COARSE_WORDS;
    var words = array<f32, 5>(
        sample.point.x, sample.point.y, sample.point.z,
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
    let seam_theta = coarse_word(k, 3u);
    let edge_theta = coarse_word(params.width - 1u, 3u);
    var drop: f32 = 0.058;
    let lateral_share = skirt_lateral_share(rear);
    let sagittal_share = skirt_sagittal_share(rear);
    var a0 = FRONT_RADIUS_X[0];
    var b0 = FRONT_RADIUS_Z[0];
    let bias = skirt_center_bias(rear);
    if (rear) {
        drop = 0.056;
        a0 = BACK_RADIUS_X[0];
        b0 = host_mul(BACK_RADIUS_Z[0], back_depth());
    }
    let radial_flare = skirt_flare();
    let lateral_flare = host_mul(radial_flare, lateral_share);
    let sagittal_flare = host_mul(radial_flare, sagittal_share);
    let flare_ratio = host_div(radial_flare, 0.030);
    let angle = skirt_angle(seam_theta, edge_theta, t);
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
        let waist = host_sub(coarse_word(k, 4u), host_mul(b0, cosines[1]));
        z = host_add(host_add(host_mul(reach, cosines[0]), center_bias), waist);
    }
    let mapped = mapped_point(vec3<f32>(x, y, z));
    return world(nominal_layer(rear, mapped));
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
        point = coarse_vector(i, 0u);
    } else {
        let t = host_div(f32(row - V_SAMPLES + 1u), f32(SKIRT_SAMPLES - 1u));
        point = skirt_point(rear, column, u, t);
    }
    positions_set(i, point);
}
"#;
