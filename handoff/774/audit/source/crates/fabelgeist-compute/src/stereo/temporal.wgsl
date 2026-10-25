// Temporal reprojection and filtering.
//
// Each pixel's point is taken back into the previous frame through the rig's
// motion, looked up in the history there, and the history's point brought
// forward again. Where the two agree -- in inverse range, which is what
// disparity noise is even in -- they are averaged, the history weighted by
// how many frames it has agreed for; where they do not, something moved or
// came out from behind something, and the current frame starts again. A
// pixel the matcher rejected this frame is filled from the history, with a
// confidence that decays each frame until the hole is let go.
//
// The motion is whatever the caller knows -- the identity for a camera on a
// tripod, a tracker's answer otherwise -- as rigid transforms in the grid frame.

struct Params {
    width: u32,
    height: u32,
    grid_kind: u32,
    geometry: u32,
    grid: vec4<f32>,
    previous_from_current: mat4x4<f32>,
    current_from_previous: mat4x4<f32>,
    baseline: f32,
    inverse_tolerance: f32,
    decay: f32,
    max_age: f32,
    reset: u32,
}

// Below this, a point was filled in rather than matched.
const GUESSED: f32 = 0.05;

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> points: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> history: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> history_age: array<f32>;
@group(0) @binding(4) var<storage, read_write> next_history: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> next_age: array<f32>;
@group(0) @binding(6) var<storage, read_write> filtered_distance: array<f32>;

fn locate(point: vec3<f32>, size: vec2<f32>) -> vec3<f32> {
    if (params.geometry == 1u) {
        return ods_pixel(params.grid, size, 0.5 * params.baseline, point, true);
    }
    return grid_pixel(params.grid_kind, params.grid, size, normalize(point));
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.width || id.y >= params.height) { return; }
    let size = vec2<f32>(f32(params.width), f32(params.height));
    let p = id.y * params.width + id.x;
    let current = points[p];
    // A point the matcher only filled in is a guess, and where
    // there is history the history is the better guess.
    let current_valid = current.w >= GUESSED;
    let current_guessed = current.w > 0.0 && !current_valid;

    // The point to go looking for in the past: this frame's, or, where this
    // frame has none, the one the history held here.
    var anchor = current.xyz;
    var have_anchor = current_valid;
    if (!current_valid && params.reset == 0u && history[p].w > 0.0) {
        anchor = (params.current_from_previous * vec4<f32>(history[p].xyz, 1.0)).xyz;
        have_anchor = true;
    }

    var held = vec4<f32>(0.0);
    var held_age = 0.0;
    var predicted = vec3<f32>(0.0);
    var held_valid = false;
    if (params.reset == 0u && have_anchor) {
        let before = (params.previous_from_current * vec4<f32>(anchor, 1.0)).xyz;
        let q = locate(before, size);
        if (q.z > 0.5) {
            let qi = u32(q.y) * params.width + u32(q.x);
            held = history[qi];
            held_age = history_age[qi];
            if (held.w > 0.0) {
                held_valid = true;
                predicted = (params.current_from_previous * vec4<f32>(held.xyz, 1.0)).xyz;
            }
        }
    }

    var out = vec4<f32>(0.0);
    var age = 0.0;
    if (current_valid && held_valid) {
        let inverse_current = 1.0 / max(length(current.xyz), 1e-4);
        let inverse_held = 1.0 / max(length(predicted), 1e-4);
        if (abs(inverse_current - inverse_held) <= params.inverse_tolerance) {
            let weight_held = min(held_age, params.max_age) * held.w;
            let weight_current = current.w;
            let inverse = (weight_held * inverse_held + weight_current * inverse_current)
                / max(weight_held + weight_current, 1e-6);
            out = vec4<f32>(normalize(current.xyz) / inverse, max(current.w, held.w * params.decay));
            age = min(held_age + 1.0, params.max_age);
        } else {
            out = current;
            age = 1.0;
        }
    } else if (current_valid) {
        out = current;
        age = 1.0;
    } else if (held_valid && held.w * params.decay >= 0.05) {
        out = vec4<f32>(predicted, held.w * params.decay);
        age = held_age;
    } else if (current_guessed) {
        out = current;
        age = 0.0;
    }
    next_history[p] = out;
    next_age[p] = age;
    filtered_distance[p] = select(0.0, length(out.xyz), out.w > 0.0);
}
