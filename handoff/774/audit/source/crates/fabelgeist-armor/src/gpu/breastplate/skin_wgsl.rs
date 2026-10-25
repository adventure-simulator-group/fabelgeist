//! Skin and morph correspondence in WGSL. Each mid vertex takes the
//! closest torso triangle to it; skin and atlas coordinates come from that
//! triangle, and a morph sample moves the shell by the body's own
//! displacement there.

/// Words of a closest-triangle sample: body face, three weights.
pub(crate) const SAMPLE_WORDS: u32 = 4;
/// Words of a morph sample: two samples and the blend between them.
pub(crate) const MORPH_WORDS: u32 = 9;
/// Words of a solid vertex's skin: atlas coordinate, eight joints, eight
/// weights.
pub(crate) const SKIN_WORDS: u32 = 18;

/// One invocation per query point: the closest point on the eligible torso
/// faces in the wearer's frame; ties go to the first face.
pub(crate) const SAMPLE: &str = r#"
@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> front: array<f32>;
@group(0) @binding(2) var<storage, read> back: array<f32>;
@group(0) @binding(3) var<storage, read> coarse: array<f32>;
@group(0) @binding(4) var<storage, read> eligible: array<u32>;
@group(0) @binding(5) var<storage, read> body_faces: array<u32>;
@group(0) @binding(6) var<storage, read> body_local: array<f32>;
@group(0) @binding(7) var<storage, read_write> samples: array<u32>;
@group(0) @binding(8) var<uniform> params: Params;

// Ericson's closest point on a triangle, as barycentric weights, rounded in a fixed
// order with exact device arithmetic: a query on a seam picks its side by these roundings.
fn closest_weights(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
    let ab = host_sub3(b, a);
    let ac = host_sub3(c, a);
    let ap = host_sub3(p, a);
    let d1 = host_dot(ab, ap);
    let d2 = host_dot(ac, ap);
    if (d1 <= 0.0 && d2 <= 0.0) {
        return vec3<f32>(1.0, 0.0, 0.0);
    }
    let bp = host_sub3(p, b);
    let d3 = host_dot(ab, bp);
    let d4 = host_dot(ac, bp);
    if (d3 >= 0.0 && d4 <= d3) {
        return vec3<f32>(0.0, 1.0, 0.0);
    }
    let vc = host_sub(host_mul(d1, d4), host_mul(d3, d2));
    if (vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0) {
        let v = host_div(d1, host_sub(d1, d3));
        return vec3<f32>(host_sub(1.0, v), v, 0.0);
    }
    let cp = host_sub3(p, c);
    let d5 = host_dot(ab, cp);
    let d6 = host_dot(ac, cp);
    if (d6 >= 0.0 && d5 <= d6) {
        return vec3<f32>(0.0, 0.0, 1.0);
    }
    let vb = host_sub(host_mul(d5, d2), host_mul(d1, d6));
    if (vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0) {
        let w = host_div(d2, host_sub(d2, d6));
        return vec3<f32>(host_sub(1.0, w), 0.0, w);
    }
    let va = host_sub(host_mul(d3, d6), host_mul(d5, d4));
    if (va <= 0.0 && host_sub(d4, d3) >= 0.0 && host_sub(d5, d6) >= 0.0) {
        let w = host_div(host_sub(d4, d3), host_add(host_sub(d4, d3), host_sub(d5, d6)));
        return vec3<f32>(0.0, host_sub(1.0, w), w);
    }
    let denominator = host_div(1.0, host_add(host_add(va, vb), vc));
    let v = host_mul(vb, denominator);
    let w = host_mul(vc, denominator);
    return vec3<f32>(host_sub(host_sub(1.0, v), w), v, w);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let q = id.x;
    if (q >= params.count) {
        return;
    }
    var point: vec3<f32>;
    if (q < params.front_count) {
        point = front_at(q);
    } else if (q < params.front_count + params.extra) {
        point = back_at(q - params.front_count);
    } else {
        point = coarse_at(q - params.front_count - params.extra);
    }
    let target_point = local(point);
    var best = MAX_FINITE;
    var best_face = 0u;
    var best_weights = vec3<f32>(0.0);
    var found = false;
    for (var e = 0u; e < params.torso_count; e = e + 1u) {
        let face = eligible[e];
        let a = body_local_at(body_faces[face * 3u]);
        let b = body_local_at(body_faces[face * 3u + 1u]);
        let c = body_local_at(body_faces[face * 3u + 2u]);
        let weights = closest_weights(target_point, a, b, c);
        let closest = host_add3(
            host_add3(host_add3(vec3<f32>(0.0), host_scale3(a, weights.x)), host_scale3(b, weights.y)),
            host_scale3(c, weights.z)
        );
        let delta = host_sub3(closest, target_point);
        let distance = host_dot(delta, delta);
        if (!found || distance < best) {
            found = true;
            best = distance;
            best_face = face;
            best_weights = weights;
        }
    }
    let at = q * SAMPLE_WORDS;
    samples[at] = best_face;
    samples[at + 1u] = bitcast<u32>(best_weights.x);
    samples[at + 2u] = bitcast<u32>(best_weights.y);
    samples[at + 3u] = bitcast<u32>(best_weights.z);
}
"#;

/// One invocation per mid vertex: the two samples its morph displacement
/// blends. A refined front vertex follows the coarse carrier it lies on.
pub(crate) const MORPH_SAMPLES: &str = r#"
@group(0) @binding(0) var<storage, read> samples: array<u32>;
@group(0) @binding(1) var<storage, read> carrier: array<f32>;
@group(0) @binding(2) var<storage, read_write> morph_samples: array<u32>;
@group(0) @binding(3) var<uniform> params: Params;

fn copy_sample(query: u32, out: u32) {
    for (var k = 0u; k < SAMPLE_WORDS; k = k + 1u) {
        morph_samples[out + k] = samples[query * SAMPLE_WORDS + k];
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let mid = id.x;
    if (mid >= params.count) {
        return;
    }
    let out = mid * MORPH_WORDS;
    if (mid < params.front_count) {
        let coarse = params.front_count + params.extra;
        copy_sample(coarse + bitcast<u32>(carrier[mid * CARRIER_WORDS]), out);
        copy_sample(coarse + bitcast<u32>(carrier[mid * CARRIER_WORDS + 1u]), out + SAMPLE_WORDS);
        morph_samples[out + 8u] = bitcast<u32>(carrier[mid * CARRIER_WORDS + 2u]);
    } else {
        copy_sample(mid, out);
        copy_sample(mid, out + SAMPLE_WORDS);
        morph_samples[out + 8u] = 0u;
    }
}
"#;

/// One invocation per solid vertex: its atlas coordinate and four strongest
/// joints from its mid vertex's sample.
pub(crate) const SKIN: &str = r#"
@group(0) @binding(0) var<storage, read> sources: array<u32>;
@group(0) @binding(1) var<storage, read> samples: array<u32>;
@group(0) @binding(2) var<storage, read> body_faces: array<u32>;
@group(0) @binding(3) var<storage, read> atlas_faces: array<u32>;
@group(0) @binding(4) var<storage, read> atlas: array<f32>;
@group(0) @binding(5) var<storage, read> body_joint_indices: array<u32>;
@group(0) @binding(6) var<storage, read> body_joint_weights: array<f32>;
@group(0) @binding(7) var<storage, read_write> skin: array<u32>;
@group(0) @binding(8) var<uniform> params: Params;

const KEPT_JOINTS: u32 = 4u;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let at = (sources[i] & 0x7fffffffu) * SAMPLE_WORDS;
    let face = samples[at];
    var uv = vec2<f32>(0.0);
    var joints: array<u32, 24>;
    var weights: array<f32, 24>;
    var m = 0u;
    for (var c = 0u; c < 3u; c = c + 1u) {
        let share = bitcast<f32>(samples[at + 1u + c]);
        let coordinate = atlas_faces[face * 3u + c];
        uv.x = uv.x + atlas[coordinate * 2u] * share;
        uv.y = uv.y + atlas[coordinate * 2u + 1u] * share;
        let vertex = body_faces[face * 3u + c];
        for (var k = 0u; k < 8u; k = k + 1u) {
            let weight = body_joint_weights[vertex * 8u + k];
            if (!(weight > 1e-7)) {
                continue;
            }
            let joint = body_joint_indices[vertex * 8u + k];
            let contribution = share * weight;
            var found = false;
            for (var j = 0u; j < m; j = j + 1u) {
                if (joints[j] == joint) {
                    weights[j] = weights[j] + contribution;
                    found = true;
                }
            }
            if (!found) {
                joints[m] = joint;
                weights[m] = contribution;
                m = m + 1u;
            }
        }
    }
    let kept = min(m, KEPT_JOINTS);
    for (var slot = 0u; slot < kept; slot = slot + 1u) {
        var best = slot;
        for (var j = slot + 1u; j < m; j = j + 1u) {
            if (weights[j] > weights[best] || (weights[j] == weights[best] && joints[j] < joints[best])) {
                best = j;
            }
        }
        let joint = joints[best];
        joints[best] = joints[slot];
        joints[slot] = joint;
        let weight = weights[best];
        weights[best] = weights[slot];
        weights[slot] = weight;
    }
    var total = 0.0;
    for (var slot = 0u; slot < kept; slot = slot + 1u) {
        total = total + weights[slot];
    }
    total = max(total, 1e-8);
    let out = i * SKIN_WORDS;
    skin[out] = bitcast<u32>(uv.x);
    skin[out + 1u] = bitcast<u32>(uv.y);
    for (var slot = 0u; slot < 8u; slot = slot + 1u) {
        var joint = 0u;
        var weight = 0.0;
        if (slot < kept) {
            joint = joints[slot];
            weight = weights[slot] / total;
        }
        skin[out + 2u + slot] = joint;
        skin[out + 10u + slot] = bitcast<u32>(weight);
    }
}
"#;

/// One invocation per solid vertex: the morph sample's displacement of the
/// body, carried onto the fitted shell.
pub(crate) const MORPH: &str = r#"
@group(0) @binding(0) var<storage, read> sources: array<u32>;
@group(0) @binding(1) var<storage, read> morph_samples: array<u32>;
@group(0) @binding(2) var<storage, read> body_faces: array<u32>;
@group(0) @binding(3) var<storage, read> original: array<f32>;
@group(0) @binding(4) var<storage, read> target_body: array<f32>;
@group(0) @binding(5) var<storage, read> base: array<f32>;
@group(0) @binding(6) var<storage, read_write> positions: array<f32>;
@group(0) @binding(7) var<uniform> params: Params;

fn displacement(at: u32) -> vec3<f32> {
    let face = morph_samples[at];
    var sum = vec3<f32>(0.0);
    for (var c = 0u; c < 3u; c = c + 1u) {
        let vertex = body_faces[face * 3u + c];
        let weight = bitcast<f32>(morph_samples[at + 1u + c]);
        sum = sum + (target_body_at(vertex) - original_at(vertex)) * weight;
    }
    return sum;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let at = (sources[i] & 0x7fffffffu) * MORPH_WORDS;
    let blend = bitcast<f32>(morph_samples[at + 8u]);
    let delta = displacement(at) * (1.0 - blend) + displacement(at + SAMPLE_WORDS) * blend;
    positions_set(i, base_at(i) + delta);
}
"#;
