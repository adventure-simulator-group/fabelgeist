//! The bracer's rings in WGSL: one invocation per ring, welding the skin's
//! edge crossings in face order, sorting them by angle and bracketing every
//! column.

use super::bracer_wgsl::DESIGN;

/// One invocation per ring: its crossings, their angles, and its column samples.
pub(crate) fn contour() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> surface: array<u32>;
@group(0) @binding(2) var<storage, read> axial: array<f32>;
@group(0) @binding(3) var<storage, read> design: array<f32>;
@group(0) @binding(4) var<storage, read> axis: array<f32>;
@group(0) @binding(5) var<storage, read_write> points: array<f32>;
@group(0) @binding(6) var<storage, read_write> samples: array<u32>;
@group(0) @binding(7) var<storage, read_write> status: array<atomic<u32>>;
@group(0) @binding(8) var<uniform> params: Params;
{fluting}
{design}
{body}
"#,
        fluting = super::chart::FLUTING,
        design = DESIGN,
        body = CONTOUR_BODY,
    )
}

const CONTOUR_BODY: &str = r#"
var<private> ring_base: u32;

fn point(i: u32) -> vec3<f32> {
    return vec3<f32>(positions[i * 3u], positions[i * 3u + 1u], positions[i * 3u + 2u]);
}

fn body_of(vertex: u32) -> u32 {
    return surface[HEADER + vertex * 2u];
}

fn crossing_at(i: u32) -> u32 {
    return ring_base + i * POINT_WORDS;
}

fn crossing_position(i: u32) -> vec3<f32> {
    let at = crossing_at(i);
    return vec3<f32>(points[at + 5u], points[at + 6u], points[at + 7u]);
}

fn swap_crossings(i: u32, j: u32) {
    let a = crossing_at(i);
    let b = crossing_at(j);
    for (var k = 0u; k < POINT_WORDS; k = k + 1u) {
        let held = points[a + k];
        points[a + k] = points[b + k];
        points[b + k] = held;
    }
}

// Blend two crossings' samples into sample `slot`, merging repeated vertices
// and dropping negligible weights.
fn write_blend(slot: u32, a: u32, b: u32, factor: f32) {
    var keys: array<u32, 4>;
    var weights: array<f32, 4>;
    var n = 0u;
    for (var side = 0u; side < 2u; side = side + 1u) {
        var at = crossing_at(a);
        var scale = 1.0 - factor;
        if (side == 1u) {
            at = crossing_at(b);
            scale = factor;
        }
        let count = bitcast<u32>(points[at]);
        for (var e = 0u; e < count; e = e + 1u) {
            let key = bitcast<u32>(points[at + 1u + e]);
            let weight = points[at + 3u + e] * scale;
            if (!(weight > 1e-7)) {
                continue;
            }
            var found = false;
            for (var i = 0u; i < n; i = i + 1u) {
                if (keys[i] == key) {
                    weights[i] = weights[i] + weight;
                    found = true;
                }
            }
            if (!found) {
                keys[n] = key;
                weights[n] = weight;
                n = n + 1u;
            }
        }
    }
    for (var i = 1u; i < n; i = i + 1u) {
        var j = i;
        loop {
            if (j == 0u || keys[j - 1u] < keys[j]) {
                break;
            }
            let key = keys[j];
            keys[j] = keys[j - 1u];
            keys[j - 1u] = key;
            let weight = weights[j];
            weights[j] = weights[j - 1u];
            weights[j - 1u] = weight;
            j = j - 1u;
        }
    }
    var total = 0.0;
    for (var i = 0u; i < n; i = i + 1u) {
        total = total + weights[i];
    }
    let out = slot * SAMPLE_WORDS;
    samples[out] = n;
    for (var i = 0u; i < 4u; i = i + 1u) {
        var key = 0u;
        var weight = 0.0;
        if (i < n) {
            key = keys[i];
            weight = weights[i] / total;
        }
        samples[out + 1u + i] = key;
        samples[out + 5u + i] = bitcast<u32>(weight);
    }
}

@compute @workgroup_size(1)
fn main(@builtin(workgroup_id) group: vec3<u32>) {
    let ring = group.x;
    ring_base = ring * params.count * POINT_WORDS;
    load_flute();
    let factor = f32(ring) / f32(ALONG);
    let target_axial = clamp(design[5] + (design[6] - design[5]) * factor, 1e-4, 1.0 - 1e-4);
    let weld = axis[3];
    let faces = surface[1];
    var count = 0u;
    for (var f = 0u; f < faces; f = f + 1u) {
        for (var e = 0u; e < 3u; e = e + 1u) {
            let a = surface[params.faces_at + f * 3u + e];
            let b = surface[params.faces_at + f * 3u + (e + 1u) % 3u];
            let start = axial[body_of(a)];
            let span = axial[body_of(b)] - start;
            if (abs(span) <= 1e-7) {
                continue;
            }
            let t = (target_axial - start) / span;
            if (!(t >= -1e-6 && t <= 1.0 + 1e-6)) {
                continue;
            }
            let clamped = clamp(t, 0.0, 1.0);
            var keys = vec2<u32>(a, b);
            var shares = vec2<f32>(1.0 - clamped, clamped);
            if (b < a) {
                keys = keys.yx;
                shares = shares.yx;
            }
            var kept_keys: array<u32, 2>;
            var kept: array<f32, 2>;
            var n = 0u;
            for (var i = 0u; i < 2u; i = i + 1u) {
                if (shares[i] > 1e-7) {
                    kept_keys[n] = keys[i];
                    kept[n] = shares[i];
                    n = n + 1u;
                }
            }
            var total = 0.0;
            for (var i = 0u; i < n; i = i + 1u) {
                total = total + kept[i];
            }
            var position = vec3<f32>(0.0);
            for (var i = 0u; i < n; i = i + 1u) {
                kept[i] = kept[i] / total;
                position = position + point(body_of(kept_keys[i])) * kept[i];
            }
            var welded = false;
            for (var j = 0u; j < count; j = j + 1u) {
                let d = position - crossing_position(j);
                if (sqrt(host_dot(d, d)) <= weld) {
                    welded = true;
                    break;
                }
            }
            if (welded) {
                continue;
            }
            if (count >= params.count) {
                fail(STATUS_INVALID_SURFACE);
                return;
            }
            let at = crossing_at(count);
            points[at] = bitcast<f32>(n);
            for (var i = 0u; i < 2u; i = i + 1u) {
                var key = 0u;
                var weight = 0.0;
                if (i < n) {
                    key = kept_keys[i];
                    weight = kept[i];
                }
                points[at + 1u + i] = bitcast<f32>(key);
                points[at + 3u + i] = weight;
            }
            points[at + 5u] = position.x;
            points[at + 6u] = position.y;
            points[at + 7u] = position.z;
            count = count + 1u;
        }
    }
    if (count < 3u) {
        fail(EMPTY_CONTOUR);
        return;
    }
    var sum = vec3<f32>(0.0);
    for (var j = 0u; j < count; j = j + 1u) {
        sum = sum + crossing_position(j);
    }
    let center = sum * (1.0 / f32(count));
    let u = vec3<f32>(axis[4], axis[5], axis[6]);
    let v = vec3<f32>(axis[8], axis[9], axis[10]);
    for (var j = 0u; j < count; j = j + 1u) {
        let radial = crossing_position(j) - center;
        points[crossing_at(j) + 8u] = atan2(host_dot(radial, v), host_dot(radial, u));
    }
    // Stable insertion sort by angle: equal angles keep their weld order.
    for (var i = 1u; i < count; i = i + 1u) {
        var j = i;
        loop {
            if (j == 0u || !(points[crossing_at(j - 1u) + 8u] > points[crossing_at(j) + 8u])) {
                break;
            }
            swap_crossings(j - 1u, j);
            j = j - 1u;
        }
    }
    let last = count - 1u;
    for (var c = 0u; c < params.around; c = c + 1u) {
        var column = design[DESIGN_COLUMNS + c];
        if (fluted()) {
            column = flute_fan(column, 1.0 - factor);
        }
        let target_angle = -PI + TAU * column;
        var a = last;
        var b = 0u;
        var start_angle = points[crossing_at(last) + 8u];
        var end_angle = points[crossing_at(0u) + 8u] + TAU;
        var bracketed = false;
        for (var e = 0u; e < last; e = e + 1u) {
            let low = points[crossing_at(e) + 8u];
            let high = points[crossing_at(e + 1u) + 8u];
            if (target_angle >= low && target_angle <= high) {
                a = e;
                b = e + 1u;
                start_angle = low;
                end_angle = high;
                bracketed = true;
                break;
            }
        }
        if (!bracketed && target_angle < points[crossing_at(0u) + 8u]) {
            start_angle = points[crossing_at(last) + 8u] - TAU;
            end_angle = points[crossing_at(0u) + 8u];
        }
        write_blend(ring * params.around + c, a, b, (target_angle - start_angle) / (end_angle - start_angle));
    }
}
"#;
