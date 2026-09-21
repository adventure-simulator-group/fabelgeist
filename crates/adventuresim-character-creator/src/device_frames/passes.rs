//! The frame passes in WGSL: the body's extremes, the orientation from the
//! rig, the owned skin's bounds in the frame, and the region's finish.

use adventuresim_armor_model::ArmorGpu;
use adventuresim_armor_model::gpu::{device_error, wgsl};
use anyhow::Result;

pub(super) const ORDERED_POSITIVE_INFINITY: u32 = 0xff80_0000;
pub(super) const ORDERED_NEGATIVE_INFINITY: u32 = 0x007f_ffff;

/// The four frame passes, in order: extremes, orient, bounds, finish.
pub(super) fn kernels(gpu: &ArmorGpu) -> Result<[std::sync::Arc<fabelgeist_compute::Kernel>; 4]> {
    let compile = |entry: &str| {
        gpu.cache()
            .get(gpu.context(), &source(entry))
            .map_err(device_error)
    };
    Ok([
        compile(EXTREMES)?,
        compile(ORIENT)?,
        compile(BOUNDS)?,
        compile(FINISH)?,
    ])
}

fn source(entry: &str) -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> joint_indices: array<u32>;
@group(0) @binding(2) var<storage, read> joint_weights: array<f32>;
@group(0) @binding(3) var<storage, read> joints: array<f32>;
@group(0) @binding(4) var<storage, read> owned: array<u32>;
@group(0) @binding(5) var<storage, read_write> reductions: array<atomic<u32>>;
@group(0) @binding(6) var<storage, read_write> frame: array<f32>;
@group(0) @binding(7) var<storage, read_write> status: array<atomic<u32>>;
struct Params {{
    count: u32,
    rule: u32,
    joint0: u32,
    joint1: u32,
    joint2: u32,
    joint3: u32,
    joint4: u32,
    joint5: u32,
    side: f32,
    pad0: f32,
}};
@group(0) @binding(8) var<uniform> params: Params;
{ordered}
{positions}

const RULE_LIMB: u32 = 0u;
const RULE_SHOULDER: u32 = 1u;
const RULE_FOOT: u32 = 2u;
const RULE_HAND: u32 = 3u;
const RULE_HEAD: u32 = 4u;
const RULE_ELBOW: u32 = 5u;
const RULE_THUMB: u32 = 6u;
const RULE_KNEE: u32 = 7u;

{frame_constants}
const MINIMUM_THUMB_RADIUS_M: f32 = 0.008;
const ELBOW_JOINT_BAND_M: f32 = 0.045;
const COP_HEIGHT_TO_WIDTH: f32 = 0.85;

fn joint(index: u32) -> vec3<f32> {{
    return vec3<f32>(joints[index * 8u], joints[index * 8u + 1u], joints[index * 8u + 2u]);
}}

fn fail(bit: u32) {{
    atomicOr(&status[0], bit);
}}

fn host_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {{
    return (a.x * b.x + a.y * b.y) + a.z * b.z;
}}

fn normalized(a: vec3<f32>) -> vec3<f32> {{
    let l = sqrt(host_dot(a, a));
    if (!(l > 1e-6)) {{
        fail(COINCIDENT);
        return vec3<f32>(0.0, 1.0, 0.0);
    }}
    return a / l;
}}

fn owned_weight(vertex: u32) -> f32 {{
    var weight = 0.0;
    for (var k = 0u; k < 8u; k = k + 1u) {{
        if (owned[joint_indices[vertex * 8u + k]] != 0u) {{
            weight = weight + joint_weights[vertex * 8u + k];
        }}
    }}
    return weight;
}}

fn origin() -> vec3<f32> {{ return vec3<f32>(frame[0], frame[1], frame[2]); }}
fn axis(i: u32) -> vec3<f32> {{
    return vec3<f32>(frame[3u + i * 3u], frame[4u + i * 3u], frame[5u + i * 3u]);
}}
fn half_extent(i: u32) -> f32 {{ return frame[12u + i]; }}
fn set_vector(at: u32, v: vec3<f32>) {{
    frame[at] = v.x;
    frame[at + 1u] = v.y;
    frame[at + 2u] = v.z;
}}
fn local(p: vec3<f32>) -> vec3<f32> {{
    let d = p - origin();
    return vec3<f32>(host_dot(d, axis(0u)), host_dot(d, axis(1u)), host_dot(d, axis(2u)));
}}
fn point(l: vec3<f32>) -> vec3<f32> {{
    return origin() + (axis(0u) * l.x + axis(1u) * l.y) + axis(2u) * l.z;
}}

{entry}
"#,
        ordered = wgsl::ORDERED_FLOAT,
        positions = wgsl::read_points("positions"),
        frame_constants = super::frame_constants(),
    )
}

/// The body's top, and each side's floor, for the head and feet.
const EXTREMES: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count || (params.rule != RULE_HEAD && params.rule != RULE_FOOT)) {
        return;
    }
    let p = positions_at(i);
    atomicMax(&reductions[0], ordered_from_float(p.y));
    if (p.x > 0.0) {
        atomicMin(&reductions[1], ordered_from_float(p.y));
    }
    if (p.x < 0.0) {
        atomicMin(&reductions[2], ordered_from_float(p.y));
    }
}
"#;

/// One invocation: the frame's axes, provisional origin and landmark span.
const ORIENT: &str = r#"
@compute @workgroup_size(1)
fn main() {
    if (params.rule == RULE_ELBOW) {
        let o = joint(params.joint0);
        let upper = normalized(joint(params.joint1) - o);
        let lower = normalized(joint(params.joint2) - o);
        let axial = normalized(upper - lower);
        let posterior = normalized(-upper - lower);
        var outward = normalized(cross(axial, posterior));
        if (outward.x * params.side < 0.0) {
            outward = -outward;
        }
        set_vector(0u, o);
        set_vector(3u, outward);
        set_vector(6u, axial);
        set_vector(9u, posterior);
        frame[15] = 0.0;
        return;
    }
    if (params.rule == RULE_THUMB) {
        let root = joint(params.joint0);
        let tip = joint(params.joint1);
        // Normalised without a coincidence check.
        let span = root - tip;
        let axial = span * (1.0 / sqrt(host_dot(span, span)));
        let q = array<f32, 4>(
            joints[params.joint2 * 8u + 3u],
            joints[params.joint2 * 8u + 4u],
            joints[params.joint2 * 8u + 5u],
            joints[params.joint2 * 8u + 6u],
        );
        // The modeled thumbnail marks local -Y on the left, +Y on the right.
        let dorsal_axis = vec3<f32>(
            2.0 * (q[0] * q[1] - q[2] * q[3]),
            1.0 - 2.0 * (q[0] * q[0] + q[2] * q[2]),
            2.0 * (q[1] * q[2] + q[0] * q[3]),
        ) * -params.side;
        let projected = dorsal_axis - axial * host_dot(dorsal_axis, axial);
        let dorsal = projected * (1.0 / sqrt(host_dot(projected, projected)));
        set_vector(0u, (root + tip) * 0.5);
        set_vector(3u, cross(axial, dorsal));
        set_vector(6u, axial);
        set_vector(9u, dorsal);
        frame[12] = MINIMUM_THUMB_RADIUS_M;
        frame[13] = host_dot(span, axial) * 0.5;
        frame[14] = MINIMUM_THUMB_RADIUS_M;
        frame[15] = 0.0;
        return;
    }
    var proximal = joint(params.joint0);
    var distal = joint(params.joint1);
    if (params.rule == RULE_HEAD) {
        let head = joint(params.joint0);
        let top = float_from_ordered(atomicLoad(&reductions[0]));
        proximal = vec3<f32>(head.x, top, head.z);
        distal = vec3<f32>(head.x, joint(params.joint1).y, head.z);
    }
    if (params.rule == RULE_FOOT) {
        let ankle = joint(params.joint0);
        var floor = float_from_ordered(atomicLoad(&reductions[1]));
        if (params.side < 0.0) {
            floor = float_from_ordered(atomicLoad(&reductions[2]));
        }
        proximal = vec3<f32>(ankle.x, ankle.y + 0.025, ankle.z);
        distal = vec3<f32>(ankle.x, floor, ankle.z);
    }
    let axial = normalized(proximal - distal);
    var front: vec3<f32>;
    if (params.rule == RULE_SHOULDER) {
        let up = vec3<f32>(0.0, 1.0, 0.0);
        front = normalized(up - axial * host_dot(up, axial));
    } else if (params.rule == RULE_FOOT) {
        let foot = joint(params.joint1) - joint(params.joint0);
        front = normalized(vec3<f32>(foot.x, 0.0, foot.z));
    } else if (params.rule == RULE_HAND) {
        let away = joint(params.joint2) - joint(params.joint3);
        let across = normalized(away - axial * host_dot(away, axial));
        front = cross(across, axial);
    } else {
        let eyes = (joint(params.joint2) + joint(params.joint3)) * 0.5;
        let facing = eyes - joint(params.joint4);
        front = normalized(vec3<f32>(facing.x, 0.0, facing.z));
    }
    let across = normalized(cross(axial, front));
    var anterior = cross(across, axial);
    if (params.rule == RULE_HAND && params.side < 0.0) {
        anterior = -anterior;
    }
    set_vector(0u, (proximal + distal) * 0.5);
    set_vector(3u, across);
    set_vector(6u, axial);
    set_vector(9u, anterior);
    let d = proximal - distal;
    frame[15] = sqrt(host_dot(d, d));
}
"#;

/// Bounds of the owned skin in the provisional frame.
const BOUNDS: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let weight = owned_weight(i);
    if (weight < SKIN_SUPPORT_THRESHOLD) {
        return;
    }
    let l = local(positions_at(i));
    if (params.rule == RULE_ELBOW) {
        if (abs(l.y) > ELBOW_JOINT_BAND_M) {
            return;
        }
    } else if (params.rule == RULE_THUMB) {
        // Thumb-zero weights include the palm heel beyond the plate's root.
        if (l.y > half_extent(1u)) {
            return;
        }
    } else if (abs(l.y) > frame[15] * 0.55) {
        return;
    }
    for (var a = 0u; a < 3u; a = a + 1u) {
        atomicMin(&reductions[3u + a], ordered_from_float(l[a]));
        atomicMax(&reductions[6u + a], ordered_from_float(l[a]));
    }
}
"#;

/// One invocation: centre and size the frame, then the region's own rules.
const FINISH: &str = r#"
fn low(a: u32) -> f32 { return float_from_ordered(atomicLoad(&reductions[3u + a])); }
fn high(a: u32) -> f32 { return float_from_ordered(atomicLoad(&reductions[6u + a])); }

@compute @workgroup_size(1)
fn main() {
    let finite = abs(low(0u)) <= 3.4e38 && abs(low(2u)) <= 3.4e38
        && abs(high(0u)) <= 3.4e38 && abs(high(2u)) <= 3.4e38;
    if (!finite || (params.rule != RULE_ELBOW && params.rule != RULE_THUMB
        && !(abs(low(1u)) <= 3.4e38 && abs(high(1u)) <= 3.4e38))) {
        fail(NO_ENVELOPE);
        return;
    }
    if (params.rule == RULE_ELBOW) {
        frame[12] = max(0.0, max(abs(low(0u)), abs(high(0u))));
        frame[14] = max(0.0, max(abs(low(2u)), abs(high(2u))));
        if (!(frame[12] > 0.0 && frame[14] > 0.0)) {
            fail(NO_ENVELOPE);
        }
        frame[13] = frame[12] * COP_HEIGHT_TO_WIDTH;
    } else if (params.rule == RULE_THUMB) {
        set_vector(0u, point(vec3<f32>((low(0u) + high(0u)) * 0.5, 0.0, (low(2u) + high(2u)) * 0.5)));
        frame[12] = max((high(0u) - low(0u)) * 0.5, MINIMUM_THUMB_RADIUS_M);
        frame[14] = max((high(2u) - low(2u)) * 0.5, MINIMUM_THUMB_RADIUS_M);
    } else {
        let length = frame[15];
        set_vector(0u, point(vec3<f32>((low(0u) + high(0u)) * 0.5, 0.0, (low(2u) + high(2u)) * 0.5)));
        frame[12] = max((high(0u) - low(0u)) * 0.5, MINIMUM_REGION_RADIUS_M);
        frame[13] = max(length * 0.5, MINIMUM_REGION_RADIUS_M);
        frame[14] = max((high(2u) - low(2u)) * 0.5, MINIMUM_REGION_RADIUS_M);
        if (params.rule == RULE_SHOULDER) {
            set_vector(0u, joint(params.joint5));
            frame[13] = frame[13] * 0.48;
        }
        if (params.rule == RULE_KNEE) {
            set_vector(0u, joint(params.joint5));
            frame[13] = frame[12] * 0.85;
            // A cop's side fan must point away from the body on both limbs.
            if (frame[3] * params.side < 0.0) {
                set_vector(3u, -axis(0u));
            }
        }
    }
    // The checks of `PartFrame::validate`: orthonormal axes, positive extents,
    // finite words.
    var valid = true;
    for (var a = 0u; a < 3u; a = a + 1u) {
        for (var b = 0u; b < 3u; b = b + 1u) {
            let expected = select(0.0, 1.0, a == b);
            if (!(abs(host_dot(axis(a), axis(b)) - expected) < 0.001)) {
                valid = false;
            }
        }
        if (!(half_extent(a) > 0.0)) {
            valid = false;
        }
    }
    for (var w = 0u; w < 15u; w = w + 1u) {
        if (!(abs(frame[w]) <= 3.4e38)) {
            valid = false;
        }
    }
    if (!valid) {
        fail(INVALID);
    }
}
"#;
