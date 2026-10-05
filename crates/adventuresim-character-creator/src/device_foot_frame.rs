//! The foot frame on the device.
//!
//! The sabaton's toe tip is fitted where its section collapses to a point:
//! whether it collapses, and how far a nearly collapsed section still
//! reaches, keep every rounding difference of the frame. So the frame is
//! rounded in a fixed order of operations with exact device arithmetic (see
//! `fabelgeist_compute::host_float`), making it deterministic and independent
//! of the device's fused operations. A reduction finds the floor under the
//! foot, one invocation orients the frame from the ankle down to the floor
//! and toward the ball, a reduction bounds the foot's own skin in it, and one
//! invocation centres and sizes it.

use fabelgeist_gpu::prelude::BufferUpload;
use std::sync::Arc;

use anyhow::{Context, Result};
use fabelgeist_armor::gpu::{device_error, wgsl};
use fabelgeist_compute::{Kernel, KernelBatch, host_float};
use fabelgeist_gpu::prelude::PassParameters;

use crate::armor_frames::{FitRegion, Side};
use crate::device_frames::{DeviceFrame, DeviceWearer, FRAME_WORDS};

const ORDERED_POSITIVE_INFINITY: u32 = 0xff80_0000;
const ORDERED_NEGATIVE_INFINITY: u32 = 0x007f_ffff;

impl DeviceWearer<'_> {
    /// Record the `side` foot's frame.
    pub fn record_foot_frame(&self, batch: &mut KernelBatch, side: Side) -> Result<DeviceFrame> {
        let gpu = self.gpu;
        let host = self.host;
        let prefix = side.prefix();
        let joint = |name: &str| -> Result<u32> {
            let name = format!("{prefix}_{name}");
            host.joint_names
                .iter()
                .position(|n| *n == name)
                .map(|i| i as u32)
                .with_context(|| format!("missing armor landmark {name}"))
        };
        let owners = FitRegion::Foot(side).owners();
        let frame = DeviceFrame {
            frame: gpu.scratch(FRAME_WORDS * 4, "foot frame")?,
            status: gpu.scratch(4, "foot frame status")?,
        };
        let mut parameters = PassParameters::new();
        parameters.insert("count", self.body.vertex_count);
        parameters.insert("ankle", joint("foot")?);
        parameters.insert("ball", joint("ball")?);
        parameters.insert("left", u32::from(matches!(side, Side::Left)));
        parameters.insert(host_float::ZERO_FIELD, 0u32);
        for pad in ["pad0", "pad1", "pad2"] {
            parameters.insert(pad, 0.0f32);
        }
        parameters.insert("positions", self.body.positions.clone());
        parameters.insert("joint_indices", self.body.joint_indices.clone());
        parameters.insert("joint_weights", self.body.joint_weights.clone());
        parameters.insert("joints", self.body.joints.clone());
        parameters.insert(
            "owned",
            gpu.upload(BufferUpload::from_elements(&host.owned_joints(&owners)))?,
        );
        // The floor, then the foot's lower and upper bounds.
        parameters.insert(
            "reductions",
            gpu.upload(BufferUpload::from_elements(&[
                ORDERED_POSITIVE_INFINITY,
                ORDERED_POSITIVE_INFINITY,
                ORDERED_POSITIVE_INFINITY,
                ORDERED_POSITIVE_INFINITY,
                ORDERED_NEGATIVE_INFINITY,
                ORDERED_NEGATIVE_INFINITY,
                ORDERED_NEGATIVE_INFINITY,
            ]))?,
        );
        parameters.insert("frame", frame.frame.clone());
        parameters.insert("status", frame.status.clone());
        let vertices = self.body.vertex_count;
        let [floor, orient, bounds, finish] = kernels(gpu)?;
        batch
            .dispatch_items(&floor, &parameters, (vertices).into())
            .map_err(device_error)?;
        batch
            .dispatch(&orient, &parameters, [1, 1, 1].into())
            .map_err(device_error)?;
        batch
            .dispatch_items(&bounds, &parameters, (vertices).into())
            .map_err(device_error)?;
        batch
            .dispatch(&finish, &parameters, [1, 1, 1].into())
            .map_err(device_error)?;
        Ok(frame)
    }
}

fn kernels(gpu: &fabelgeist_armor::ArmorGpu) -> Result<[Arc<Kernel>; 4]> {
    let compile = |entry: &str| {
        gpu.cache()
            .get(gpu.context(), &source(entry))
            .map_err(device_error)
    };
    Ok([
        compile(FLOOR)?,
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
    ankle: u32,
    ball: u32,
    left: u32,
    zero: u32,
    pad0: f32,
    pad1: f32,
    pad2: f32,
}};
@group(0) @binding(8) var<uniform> params: Params;
{ordered}
{zero_hook}{host_float}
{positions}

{frame_constants}// The frame starts a little above the ankle joint.
const ANKLE_RISE_M: f32 = 0.025;
// Owned skin further along the axis than this fraction of the landmark span
// belongs to the leg, not the foot's envelope.
const ENVELOPE_SPAN_FRACTION: f32 = 0.55;

fn fail(bit: u32) {{
    atomicOr(&status[0], bit);
}}

fn joint(index: u32) -> vec3<f32> {{
    return vec3<f32>(joints[index * 8u], joints[index * 8u + 1u], joints[index * 8u + 2u]);
}}

fn axis(i: u32) -> vec3<f32> {{
    return vec3<f32>(frame[3u + i * 3u], frame[4u + i * 3u], frame[5u + i * 3u]);
}}

fn origin() -> vec3<f32> {{
    return vec3<f32>(frame[0], frame[1], frame[2]);
}}

fn set_vector(at: u32, v: vec3<f32>) {{
    frame[at] = v.x;
    frame[at + 1u] = v.y;
    frame[at + 2u] = v.z;
}}

// The unit vector along `a`; a near-zero vector fails as coincident landmarks.
fn normalized(a: vec3<f32>) -> vec3<f32> {{
    let l = host_sqrt(host_dot(a, a));
    if (!(l > 1e-6)) {{
        fail(COINCIDENT);
        return vec3<f32>(0.0, 1.0, 0.0);
    }}
    return vec3<f32>(host_div(a.x, l), host_div(a.y, l), host_div(a.z, l));
}}

fn midpoint(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {{
    return vec3<f32>(
        host_mul(host_add(a.x, b.x), 0.5),
        host_mul(host_add(a.y, b.y), 0.5),
        host_mul(host_add(a.z, b.z), 0.5),
    );
}}

{entry}
"#,
        ordered = wgsl::ORDERED_FLOAT,
        zero_hook = host_float::PARAMS_ZERO_HOOK,
        host_float = host_float::wgsl(),
        positions = wgsl::read_points("positions"),
        frame_constants = crate::device_frames::frame_constants(),
    )
}

/// The floor under each side of the body.
const FLOOR: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let p = positions_at(i);
    if (p.y == p.y && ((params.left != 0u && p.x > 0.0) || (params.left == 0u && p.x < 0.0))) {
        atomicMin(&reductions[0], ordered_from_float(p.y));
    }
}
"#;

/// One invocation: the frame's axes, provisional origin and landmark span,
/// from just above the ankle to the floor, facing along the foot.
const ORIENT: &str = r#"
@compute @workgroup_size(1)
fn main() {
    let ankle = joint(params.ankle);
    let floor = float_from_ordered(atomicLoad(&reductions[0]));
    let proximal = vec3<f32>(ankle.x, host_add(ankle.y, ANKLE_RISE_M), ankle.z);
    let distal = vec3<f32>(ankle.x, floor, ankle.z);
    let axial = normalized(host_sub3(proximal, distal));
    let foot = host_sub3(joint(params.ball), ankle);
    let front = normalized(vec3<f32>(foot.x, 0.0, foot.z));
    let across = normalized(host_cross(axial, front));
    let anterior = host_cross(across, axial);
    set_vector(0u, midpoint(proximal, distal));
    set_vector(3u, across);
    set_vector(6u, axial);
    set_vector(9u, anterior);
    let span = host_sub3(proximal, distal);
    frame[15] = host_sqrt(host_dot(span, span));
}
"#;

/// Bounds of the foot's own skin in the provisional frame.
const BOUNDS: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    var weight = 0.0;
    for (var k = 0u; k < 8u; k = k + 1u) {
        if (owned[joint_indices[i * 8u + k]] != 0u) {
            weight = host_add(weight, joint_weights[i * 8u + k]);
        }
    }
    if (weight < SKIN_SUPPORT_THRESHOLD) {
        return;
    }
    let d = host_sub3(positions_at(i), origin());
    let local = vec3<f32>(host_dot(d, axis(0u)), host_dot(d, axis(1u)), host_dot(d, axis(2u)));
    if (abs(local.y) > host_mul(frame[15], ENVELOPE_SPAN_FRACTION)) {
        return;
    }
    for (var a = 0u; a < 3u; a = a + 1u) {
        atomicMin(&reductions[1u + a], ordered_from_float(local[a]));
        atomicMax(&reductions[4u + a], ordered_from_float(local[a]));
    }
}
"#;

/// One invocation: centre the frame on the skin across and front to back,
/// and size it.
const FINISH: &str = r#"
fn low(a: u32) -> f32 {
    return float_from_ordered(atomicLoad(&reductions[1u + a]));
}

fn high(a: u32) -> f32 {
    return float_from_ordered(atomicLoad(&reductions[4u + a]));
}

@compute @workgroup_size(1)
fn main() {
    for (var a = 0u; a < 3u; a = a + 1u) {
        if (!(abs(low(a)) <= 3.4e38) || !(abs(high(a)) <= 3.4e38)) {
            fail(NO_ENVELOPE);
            return;
        }
    }
    let center = vec3<f32>(
        host_mul(host_add(low(0u), high(0u)), 0.5),
        0.0,
        host_mul(host_add(low(2u), high(2u)), 0.5),
    );
    // The centre placed along the frame's axes, as `PartFrame::point` places it.
    var placed: vec3<f32>;
    let start = origin();
    for (var c = 0u; c < 3u; c = c + 1u) {
        let along = host_add(
            host_add(host_mul(axis(0u)[c], center.x), host_mul(axis(1u)[c], center.y)),
            host_mul(axis(2u)[c], center.z),
        );
        placed[c] = host_add(start[c], along);
    }
    set_vector(0u, placed);
    frame[12] = max(host_mul(host_sub(high(0u), low(0u)), 0.5), MINIMUM_REGION_RADIUS_M);
    frame[13] = max(host_mul(frame[15], 0.5), MINIMUM_REGION_RADIUS_M);
    frame[14] = max(host_mul(host_sub(high(2u), low(2u)), 0.5), MINIMUM_REGION_RADIUS_M);
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
        if (!(frame[12u + a] > 0.0)) {
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
