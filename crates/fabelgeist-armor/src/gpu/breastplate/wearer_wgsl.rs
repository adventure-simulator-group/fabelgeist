//! The wearer measured in WGSL: its frame, anchors and scales, and the
//! torso section centres the fit measures against.

/// One invocation: the wearer's frame, anchors and semantic regression.
pub(crate) const WEARER: &str = r#"
@group(0) @binding(0) var<storage, read_write> plate: array<f32>;
@group(0) @binding(1) var<storage, read> rig: array<f32>;
@group(0) @binding(2) var<storage, read> surface: array<u32>;
@group(0) @binding(3) var<storage, read> semantic: array<f32>;
@group(0) @binding(4) var<storage, read> positions: array<f32>;
@group(0) @binding(5) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(6) var<uniform> params: Params;

fn rig_point(slot: u32) -> vec3<f32> {
    return vec3<f32>(rig[slot * 3u], rig[slot * 3u + 1u], rig[slot * 3u + 2u]);
}

@compute @workgroup_size(1)
fn main() {
    let hint = rig_point(0u);
    let front = unit(vec3<f32>(hint.x, 0.0, hint.z));
    let lateral = unit(host_cross(vec3<f32>(0.0, 1.0, 0.0), front.xyz));
    if (front.w == 0.0 || lateral.w == 0.0) {
        fail(STATUS_DEGENERATE);
        return;
    }
    for (var k = 0u; k < 3u; k = k + 1u) {
        plate[WEARER + k] = lateral[k];
        plate[WEARER + 3u + k] = front[k];
    }
    let neck = local(rig_point(1u));
    let shoulder_half_width = host_mul(
        host_add(
            abs(host_sub(local(rig_point(2u)).x, neck.x)), abs(host_sub(local(rig_point(3u)).x, neck.x))
        ),
        0.5,
    );
    let n = surface[0];
    let count = f32(n);
    var level_sum = 0.0;
    var height_sum = 0.0;
    for (var s = 0u; s < n; s = s + 1u) {
        let body = surface[HEADER + s * 2u];
        level_sum = host_add(level_sum, semantic[body * 2u + 1u]);
        height_sum = host_add(height_sum, local(positions_at(body)).y);
    }
    let mean_level = host_div(level_sum, count);
    let mean_height = host_div(height_sum, count);
    var variance = 0.0;
    var covariance = 0.0;
    var center_front = -MAX_FINITE;
    for (var s = 0u; s < n; s = s + 1u) {
        let body = surface[HEADER + s * 2u];
        let level = host_sub(semantic[body * 2u + 1u], mean_level);
        variance = host_add(variance, host_mul(level, level));
        let point = local(positions_at(body));
        covariance = host_add(covariance, host_mul(level, host_sub(point.y, mean_height)));
        let lateral_coordinate = semantic[body * 2u];
        let vertical = semantic[body * 2u + 1u];
        if (abs(lateral_coordinate) < 0.12 && vertical >= 0.25 && vertical <= 0.75) {
            center_front = max(center_front, point.z);
        }
    }
    if (!is_finite(center_front)) {
        fail(STATUS_INVALID_SURFACE);
    }
    let slope = host_div(covariance, max(variance, 1e-8));
    let scale = host_div(abs(slope), REFERENCE_SEMANTIC_HEIGHT);
    for (var k = 0u; k < 3u; k = k + 1u) {
        plate[WEARER + 6u + k] = neck[k];
    }
    plate[WEARER + 15u] = host_sub(neck.y, host_mul(0.36, scale));
    plate[WEARER + 16u] = host_sub(neck.y, host_mul(0.10, scale));
    plate[WEARER + 17u] = slope;
    plate[WEARER + 18u] = shoulder_half_width;
}
"#;

/// One invocation per torso face corner: the torso's extent within the
/// fitted height band, as ordered floats.
pub(crate) const TORSO_BOUNDS: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> torso_faces: array<u32>;
@group(0) @binding(2) var<storage, read> body_local: array<f32>;
@group(0) @binding(3) var<storage, read_write> bounds: array<atomic<u32>>;
@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let point = body_local_at(torso_faces[i]);
    if (point.y >= plate[WEARER + 15u] && point.y <= plate[WEARER + 16u]) {
        for (var k = 0u; k < 3u; k = k + 1u) {
            atomicMin(&bounds[k], ordered_from_float(point[k]));
            atomicMax(&bounds[3u + k], ordered_from_float(point[k]));
        }
    }
}
"#;

/// One invocation: the wearer's scales from its torso and shoulders.
pub(crate) const SCALES: &str = r#"
@group(0) @binding(0) var<storage, read_write> plate: array<f32>;
@group(0) @binding(1) var<storage, read> bounds: array<u32>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(1)
fn main() {
    let low = vec3<f32>(
        float_from_ordered(bounds[0]), float_from_ordered(bounds[1]), float_from_ordered(bounds[2])
    );
    let high = vec3<f32>(
        float_from_ordered(bounds[3]), float_from_ordered(bounds[4]), float_from_ordered(bounds[5])
    );
    plate[WEARER + 9u] = clamp(
        host_div(max(abs(high.x), abs(low.x)), REFERENCE_TORSO_HALF_WIDTH), 0.65, 1.55
    );
    plate[WEARER + 10u] = clamp(host_div(plate[WEARER + 18u], REFERENCE_SHOULDER_HALF_WIDTH), 0.65, 1.55);
    plate[WEARER + 11u] = clamp(host_div(abs(plate[WEARER + 17u]), REFERENCE_SEMANTIC_HEIGHT), 0.70, 1.45);
    plate[WEARER + 12u] = clamp(
        host_div(host_mul(host_sub(high.z, low.z), 0.5), REFERENCE_SECTION_RADIUS), 0.65, 1.80
    );
    plate[WEARER + 13u] = plate[WEARER + 6u];
    plate[WEARER + 14u] = host_mul(host_add(low.z, high.z), 0.5);
}
"#;

/// One invocation per body vertex: its position in the wearer's frame.
pub(crate) const BODY_LOCAL: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> positions: array<f32>;
@group(0) @binding(2) var<storage, read_write> body_local: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    body_local_set(i, local(positions_at(i)));
}
"#;

/// One invocation per carrier vertex: the centre of the torso section at
/// its height, the middle of the bounds of the torso's edge crossings.
pub(crate) const CENTERS: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> positions: array<f32>;
@group(0) @binding(2) var<storage, read> torso_faces: array<u32>;
@group(0) @binding(3) var<storage, read> body_local: array<f32>;
@group(0) @binding(4) var<storage, read_write> centers: array<f32>;
@group(0) @binding(5) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let y = local(positions_at(i)).y;
    let infinity = MAX_FINITE;
    var low = vec2<f32>(infinity);
    var high = vec2<f32>(-infinity);
    var crossings = 0u;
    let faces = params.torso_count;
    for (var f = 0u; f < faces; f = f + 1u) {
        var triangle: array<vec3<f32>, 3>;
        for (var c = 0u; c < 3u; c = c + 1u) {
            triangle[c] = body_local_at(torso_faces[f * 3u + c]);
        }
        for (var e = 0u; e < 3u; e = e + 1u) {
            let a = triangle[e];
            let b = triangle[(e + 1u) % 3u];
            let da = host_sub(a.y, y);
            let db = host_sub(b.y, y);
            if ((da < 0.0 && db < 0.0) || (da > 0.0 && db > 0.0) || abs(host_sub(da, db)) <= 1e-9) {
                continue;
            }
            let t = host_div(da, host_sub(da, db));
            if (!(t >= -1e-5 && t <= 1.0 + 1e-5)) {
                continue;
            }
            let crossing = vec2<f32>(
                host_add(a.x, host_mul(host_sub(b.x, a.x), t)), host_add(a.z, host_mul(host_sub(b.z, a.z), t))
            );
            low = min(low, crossing);
            high = max(high, crossing);
            crossings = crossings + 1u;
        }
    }
    let at = i * CENTER_WORDS;
    centers[at] = host_mul(host_add(low.x, high.x), 0.5);
    centers[at + 1u] = host_mul(host_add(low.y, high.y), 0.5);
    centers[at + 2u] = select(0.0, 1.0, crossings >= 4u);
}
"#;
