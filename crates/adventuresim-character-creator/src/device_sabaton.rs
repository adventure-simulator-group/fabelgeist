//! The sabaton's fit on the device.
//!
//! Lengthwise foot sections -- the lateral centre, half width and dorsal
//! height of the skin near each of 24 stations from the ankle cut to the
//! toes -- are exact atomic reductions over the foot's skin. Every carrier
//! then takes the rounded section of its station, the toe cap's carriers
//! drawn forward over the toes.
//!
//! The toe cap's tip is fitted where its section collapses: whether it
//! collapses, and how far a nearly collapsed section still reaches, keep
//! every rounding difference of their inputs. So, in the bit-exact foot
//! frame, every step is rounded in a fixed order with exact device arithmetic
//! (see `fabelgeist_compute::host_float`); only the transcendental functions,
//! whose results the tip does not amplify, are the device's.

use anyhow::Result;
use fabelgeist_armor::gpu::{device_error, wgsl};
use fabelgeist_armor::{DevicePart, FootArmorDesign, GenerateError};
use fabelgeist_compute::{KernelBatch, host_float};
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_gpu::prelude::PassParameters;

use crate::armor_frames::FitRegion;
use crate::device_frames::{DeviceFrame, DeviceWearer};
use crate::device_piece::DeviceCheck;

/// Stations along the foot profile.
const STATIONS: u32 = 24;
/// Ordered-float words per station: nearest distance, low x, high x, top.
const STATION_WORDS: u32 = 4;
/// The status bit raised when the ankle cutaway consumes the foot.
const TRIM_EXCEEDS_FOOT: u32 = 1;
/// The share of the foot's half length an ankle cutaway may consume.
const TRIMMABLE_FOOT_SHARE: f32 = 0.70;

const ORDERED_POSITIVE_INFINITY: u32 = 0xff80_0000;
const ORDERED_NEGATIVE_INFINITY: u32 = 0x007f_ffff;

impl DeviceWearer<'_> {
    /// Record the sabaton fit of `part`'s carriers in the foot `frame`; the
    /// returned check fails with [`GenerateError::SabatonTrimExceedsFoot`]
    /// when the ankle cutaway consumes the foot.
    pub(crate) fn record_sabaton_fit(
        &self,
        batch: &mut KernelBatch,
        design: &FootArmorDesign,
        region: FitRegion,
        frame: &DeviceFrame,
        part: &DevicePart,
    ) -> Result<DeviceCheck> {
        let gpu = self.gpu;
        let support = self.record_any_region_support(batch, &[region])?;
        let mut profile = vec![ORDERED_NEGATIVE_INFINITY];
        for _ in 0..STATIONS {
            profile.extend([
                ORDERED_POSITIVE_INFINITY,
                ORDERED_POSITIVE_INFINITY,
                ORDERED_NEGATIVE_INFINITY,
                ORDERED_NEGATIVE_INFINITY,
            ]);
        }
        let mut shell_of = vec![0u32; part.carrier_count() as usize];
        for shell in 0..part.shell_count() {
            for carrier in part.shell_carriers(shell) {
                shell_of[carrier as usize] = shell as u32;
            }
        }
        let status = gpu.scratch(4, "sabaton status")?;
        let cutaway = design.ankle_cutaway.metres();
        let mut parameters = PassParameters::new();
        parameters.insert("count", self.body.vertex_count);
        parameters.insert("carriers_count", part.carrier_count());
        parameters.insert("toe_shell", u32::from(design.lame_count));
        parameters.insert("pad0", 0u32);
        parameters.insert(host_float::ZERO_FIELD, 0u32);
        for pad in ["pad1", "pad2", "pad3"] {
            parameters.insert(pad, 0.0f32);
        }
        let gauge = design.gauge.thickness.metres();
        for (name, value) in [
            ("cutaway", cutaway),
            ("trim_limit", cutaway / TRIMMABLE_FOOT_SHARE),
            ("toe_extension", design.toe_extension.metres()),
            ("toe_width", design.toe_width.unit()),
            ("toe_roundness", design.toe_roundness.unit()),
            ("instep_height", design.instep_height.unit()),
            ("clearance", design.gauge.clearance.metres()),
            ("gauge", gauge),
        ] {
            parameters.insert(name, value);
        }
        parameters.insert("frames", frame.frame.clone());
        parameters.insert("positions", self.body.positions.clone());
        parameters.insert("support", support);
        parameters.insert(
            "profile",
            gpu.upload(BufferUpload::from_elements(&profile))?,
        );
        parameters.insert(
            "shell_of",
            gpu.upload(BufferUpload::from_elements(&shell_of))?,
        );
        parameters.insert("carriers", part.carriers().clone());
        parameters.insert("status", status.clone());
        for (entry, items) in [
            (FRONT, self.body.vertex_count),
            (NEAREST, self.body.vertex_count),
            (SLICE, self.body.vertex_count),
            (FIT, part.carrier_count()),
        ] {
            let kernel = gpu
                .cache()
                .get(gpu.context(), &source(entry).into())
                .map_err(device_error)?;
            batch
                .dispatch_items(&kernel, &parameters, items)
                .map_err(device_error)?;
        }
        let frame = frame.clone();
        Ok(DeviceCheck::Device(Box::new(move |gpu| {
            let status = status.clone();
            let frame = frame.clone();
            Box::pin(async move {
                if gpu.read_async::<u32>(&status).await?[0] & TRIM_EXCEEDS_FOOT == 0 {
                    return Ok(());
                }
                let length = frame.read_async(gpu).await?.half_extents[2];
                Err(GenerateError::SabatonTrimExceedsFoot {
                    cutaway_m: cutaway,
                    available_span_m: length * TRIMMABLE_FOOT_SHARE,
                }
                .into())
            })
        })))
    }
}

fn source(entry: &str) -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> frames: array<f32>;
@group(0) @binding(1) var<storage, read> positions: array<f32>;
@group(0) @binding(2) var<storage, read> support: array<u32>;
@group(0) @binding(3) var<storage, read_write> profile: array<atomic<u32>>;
@group(0) @binding(4) var<storage, read> shell_of: array<u32>;
@group(0) @binding(5) var<storage, read_write> carriers: array<f32>;
@group(0) @binding(6) var<storage, read_write> status: array<atomic<u32>>;
struct Params {{
    count: u32,
    carriers_count: u32,
    toe_shell: u32,
    pad0: u32,
    zero: u32,
    pad1: f32,
    pad2: f32,
    pad3: f32,
    cutaway: f32,
    trim_limit: f32,
    toe_extension: f32,
    toe_width: f32,
    toe_roundness: f32,
    instep_height: f32,
    clearance: f32,
    gauge: f32,
}};
@group(0) @binding(7) var<uniform> params: Params;
{math}
{frame}
{ordered}
{zero_hook}{host_float}
{positions}
{carriers}

const STATIONS: u32 = {stations}u;
const WORDS: u32 = {words}u;
const SLICE_WINDOW_M: f32 = 0.008;
const TOE_SETBACK_M: f32 = 0.004;
const FIT_MARGIN_M: f32 = 0.002;
// Below this an authored section has collapsed to the toe's nose.
const COLLAPSED_SECTION_M: f32 = 1e-6;
const TRIM_EXCEEDS_FOOT: u32 = {trim}u;

// `p` in the frame: each axis dotted with the offset from the origin.
fn local(f: Frame, p: vec3<f32>) -> vec3<f32> {{
    let d = host_sub3(p, f.origin);
    return vec3<f32>(host_dot(d, f.x), host_dot(d, f.y), host_dot(d, f.z));
}}

// A local point placed along the frame's axes, as `PartFrame::point` places it.
fn placed(f: Frame, l: vec3<f32>) -> vec3<f32> {{
    var p: vec3<f32>;
    for (var c = 0u; c < 3u; c = c + 1u) {{
        p[c] = host_add(
            f.origin[c],
            host_add(host_add(host_mul(f.x[c], l.x), host_mul(f.y[c], l.y)), host_mul(f.z[c], l.z)),
        );
    }}
    return p;
}}

fn word(at: u32) -> f32 {{
    return float_from_ordered(atomicLoad(&profile[at]));
}}

fn foot_front() -> f32 {{
    return word(0u);
}}

fn profile_start() -> f32 {{
    return host_add(host_mul(-frame_at(0u).half_extents.z, 0.05), params.cutaway);
}}

fn profile_end() -> f32 {{
    return host_sub(foot_front(), TOE_SETBACK_M);
}}

fn station_z(station: u32) -> f32 {{
    let start = profile_start();
    return host_add(
        start,
        host_div(host_mul(host_sub(profile_end(), start), f32(station)), f32(STATIONS - 1u)),
    );
}}

fn station_at(station: u32) -> u32 {{
    return 1u + station * WORDS;
}}

{entry}
"#,
        math = wgsl::MATH,
        frame = wgsl::FRAME,
        ordered = wgsl::ORDERED_FLOAT,
        zero_hook = host_float::PARAMS_ZERO_HOOK,
        host_float = host_float::wgsl(),
        positions = wgsl::read_points("positions"),
        carriers = wgsl::points("carriers"),
        stations = STATIONS,
        words = STATION_WORDS,
        trim = TRIM_EXCEEDS_FOOT,
    )
}

/// The foot's front, and whether its length leaves room for the cutaway.
const FRONT: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let fit = frame_at(0u);
    if (i == 0u && fit.half_extents.z <= params.trim_limit) {
        atomicOr(&status[0], TRIM_EXCEEDS_FOOT);
    }
    if (support[i] != 0u) {
        atomicMax(&profile[0], ordered_from_float(local(fit, positions_at(i)).z));
    }
}
"#;

/// Each station's distance to its nearest skin.
const NEAREST: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count || support[i] == 0u) {
        return;
    }
    let z = local(frame_at(0u), positions_at(i)).z;
    for (var station = 0u; station < STATIONS; station = station + 1u) {
        let distance = abs(host_sub(z, station_z(station)));
        atomicMin(&profile[station_at(station)], ordered_from_float(distance));
    }
}
"#;

/// The lateral bounds and dorsal top of the skin within the slice window.
const SLICE: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count || support[i] == 0u) {
        return;
    }
    let p = local(frame_at(0u), positions_at(i));
    for (var station = 0u; station < STATIONS; station = station + 1u) {
        let at = station_at(station);
        let distance = abs(host_sub(p.z, station_z(station)));
        if (distance <= host_add(word(at), SLICE_WINDOW_M)) {
            atomicMin(&profile[at + 1u], ordered_from_float(p.x));
            atomicMax(&profile[at + 2u], ordered_from_float(p.x));
            atomicMax(&profile[at + 3u], ordered_from_float(p.y));
        }
    }
}
"#;

/// One invocation per carrier: seat it on the rounded foot section.
const FIT: &str = r#"
// Lateral centre, half width and dorsal height of a station.
fn section(station: u32) -> vec3<f32> {
    let at = station_at(station);
    let lo = word(at + 1u);
    let hi = word(at + 2u);
    return vec3<f32>(
        host_mul(host_add(lo, hi), 0.5),
        host_mul(host_sub(hi, lo), 0.5),
        word(at + 3u),
    );
}

fn cube(x: f32) -> f32 {
    return host_mul(host_mul(x, x), x);
}

// A cubic B-spline through the stations: continuous slope and curvature
// instead of raw skin slices.
fn profile_at(z: f32) -> vec3<f32> {
    let last = f32(STATIONS - 1u);
    let start = profile_start();
    let station = clamp(
        host_mul(host_div(host_sub(z, start), host_sub(profile_end(), start)), last),
        0.0,
        last,
    );
    let index = i32(floor(station));
    let t = host_sub(station, f32(index));
    let weights = array<f32, 4>(
        host_div(cube(host_sub(1.0, t)), 6.0),
        host_div(host_add(host_sub(host_mul(3.0, cube(t)), host_mul(host_mul(6.0, t), t)), 4.0), 6.0),
        host_div(
            host_add(
                host_add(host_add(host_mul(-3.0, cube(t)), host_mul(host_mul(3.0, t), t)), host_mul(3.0, t)),
                1.0,
            ),
            6.0,
        ),
        host_div(cube(t), 6.0),
    );
    var sum = vec3<f32>(0.0);
    for (var offset = 0; offset < 4; offset = offset + 1) {
        let at = u32(clamp(index + offset - 1, 0, i32(STATIONS) - 1));
        let value = section(at);
        for (var c = 0u; c < 3u; c = c + 1u) {
            sum[c] = host_add(sum[c], host_mul(weights[offset], value[c]));
        }
    }
    return sum;
}

// `signum(a) * |a|^0.60`: a low, rounded rectangular section.
fn rounded(a: f32) -> f32 {
    return sign(a) * pow(abs(a), 0.60);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.carriers_count) {
        return;
    }
    let fit = frame_at(0u);
    let width = fit.half_extents.x;
    let height = fit.half_extents.y;
    let length = fit.half_extents.z;
    let gauge = params.gauge;
    let gap = host_add(host_add(params.clearance, gauge), FIT_MARGIN_M);
    let sole = host_add(-height, gauge);
    let front = foot_front();
    let end = max(
        host_add(length, params.toe_extension),
        host_add(host_add(front, gap), params.toe_extension),
    );
    let ankle_start = profile_start();
    var p: vec3<f32>;
    if (i + 1u == params.carriers_count) {
        // The toe cap's tip, placed from its authored local position rather
        // than read from the carrier: its nearly collapsed section amplifies
        // any rounding of the carrier.
        let tip = vec3<f32>(0.0, sole, host_add(length, params.toe_extension));
        p = local(fit, placed(fit, tip));
    } else {
        p = local(fit, carriers_at(i));
    }
    let original_z = p.z;
    var authored_width: f32;
    var authored_height: f32;
    if (shell_of[i] == params.toe_shell) {
        let nose_start = host_mul(length, 0.60);
        let t = clamp(
            host_div(
                host_sub(original_z, nose_start),
                host_add(host_mul(length, 0.40), params.toe_extension),
            ),
            0.0,
            1.0,
        );
        let cosine = host_sqrt(host_sub(1.0, host_mul(t, t)));
        p.z = host_add(nose_start, host_mul(host_sub(end, nose_start), t));
        authored_width = host_mul(
            host_add(host_add(host_mul(width, params.toe_width), params.clearance), gauge),
            pow(cosine, params.toe_roundness),
        );
        authored_height = host_mul(host_mul(host_mul(height, 1.1), params.instep_height), cosine);
    } else {
        let t = clamp(
            host_div(host_sub(original_z, ankle_start), host_sub(host_mul(length, 0.65), ankle_start)),
            0.0,
            1.0,
        );
        var taper: f32;
        if (t < 0.55) {
            taper = host_add(0.63, host_mul(0.37, host_smoothstep(host_div(t, 0.55))));
        } else {
            taper = host_add(
                1.0,
                host_mul(host_sub(params.toe_width, 1.0), host_smoothstep(host_div(host_sub(t, 0.55), 0.45))),
            );
        }
        authored_width = host_add(host_add(host_mul(width, taper), params.clearance), gauge);
        authored_height = host_mul(
            host_mul(height, params.instep_height),
            host_sub(1.75, host_mul(0.65, host_smoothstep(t))),
        );
    }
    if (authored_width < COLLAPSED_SECTION_M || authored_height < COLLAPSED_SECTION_M) {
        carriers_set(i, placed(fit, vec3<f32>(0.0, sole, end)));
        return;
    }
    let angle = atan2(
        host_div(p.x, authored_width),
        host_div(host_sub(p.y, sole), authored_height),
    );
    let slice = profile_at(p.z);
    let nose = clamp(
        host_div(host_sub(p.z, host_add(front, gap)), max(host_sub(host_sub(end, front), gap), 0.001)),
        0.0,
        1.0,
    );
    let return_scale = host_sqrt(host_sub(1.0, host_mul(nose, nose)));
    let toe_blend = clamp(host_div(host_sub(host_div(p.z, length), 0.15), 0.5), 0.0, 1.0);
    let breadth = host_add(
        host_add(slice.y, gap),
        host_mul(host_mul(host_mul(width, 0.8), host_sub(params.toe_width, 0.8)), toe_blend),
    );
    p.x = host_add(
        host_mul(slice.x, return_scale),
        host_mul(host_mul(breadth, pow(return_scale, params.toe_roundness)), rounded(sin(angle))),
    );
    let rise = host_add(
        host_add(host_sub(slice.z, sole), gap),
        host_mul(host_mul(height, 0.3), host_sub(params.instep_height, 0.85)),
    );
    p.y = host_add(sole, host_mul(host_mul(rise, return_scale), rounded(cos(angle))));
    carriers_set(i, placed(fit, p));
}
"#;
