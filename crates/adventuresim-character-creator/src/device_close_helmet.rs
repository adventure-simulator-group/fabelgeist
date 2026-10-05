//! The close helmet fitted on the device.
//!
//! The helmet is sized by broad sections of the head and neck skin: eight
//! height bands of it, each bounded by one reduction, then turned into the
//! helmet's profile by one invocation. Which skin belongs to the head and
//! neck is a matter of skin weights alone, so the host lists it once.

use fabelgeist_gpu::prelude::BufferUpload;
use std::sync::Arc;

use anyhow::Result;
use fabelgeist_armor::gpu::{
    CLOSE_HELMET_PROFILE_WORDS, FIT_PROFILE_WORD, device_error, record_close_helmet, wgsl,
};
use fabelgeist_armor::{ArmorGpu, CloseHelmetDesign};
use fabelgeist_compute::{Kernel, KernelBatch, host_float};
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use crate::armor_frames::FitRegion;
use crate::device_frames::{DeviceFrame, DeviceWearer};
use crate::device_piece::DeviceRecording;

/// Height bands the profile measures, each as low and high bounds on three
/// axes, then a sample count and a pad.
const BANDS: usize = 8;
const BAND_WORDS: usize = 8;
const ORDERED_POSITIVE_INFINITY: u32 = 0xff80_0000;
const ORDERED_NEGATIVE_INFINITY: u32 = 0x007f_ffff;

impl DeviceWearer<'_> {
    /// Record a close helmet fitted to this wearer, short of thickening.
    pub fn record_fitted_close_helmet(
        &self,
        batch: &mut KernelBatch,
        design: &CloseHelmetDesign,
    ) -> Result<DeviceRecording> {
        let frame = self.record_head_frame(batch)?;
        let fit = self.record_close_helmet_profile(batch, design, &frame)?;
        let part = record_close_helmet(self.gpu, batch, design, &fit, &frame.frame)?;
        Ok(DeviceRecording {
            part,
            frames: vec![(frame, FitRegion::Head)],
            checks: Vec::new(),
        })
    }

    /// The skin that sizes a helmet reaching down the neck: the head's and
    /// the neck's, each vertex once, in order.
    pub(crate) fn head_and_neck_support(&self, lower: FitRegion) -> Result<Vec<u32>> {
        let mut support = self.host.support_indices(FitRegion::Head)?;
        support.extend(self.host.support_indices(lower)?);
        support.sort_unstable();
        support.dedup();
        Ok(support.into_iter().map(|i| i as u32).collect())
    }

    /// Record the helmet's profile, measured on the head and neck skin, into
    /// a new fit buffer: the helmet's own frame, then its profile.
    /// Unsupported sections fail the head frame.
    fn record_close_helmet_profile(
        &self,
        batch: &mut KernelBatch,
        design: &CloseHelmetDesign,
        frame: &DeviceFrame,
    ) -> Result<Buffer> {
        let gpu = self.gpu;
        let support = self.head_and_neck_support(FitRegion::Neck)?;
        let wall = design.fit.wall_thickness.metres();
        let fit = gpu.scratch(
            ((FIT_PROFILE_WORD + CLOSE_HELMET_PROFILE_WORDS) * 4) as u64,
            "close helmet fit",
        )?;
        let mut bands = Vec::with_capacity(BANDS * BAND_WORDS);
        for _ in 0..BANDS {
            bands.extend([ORDERED_POSITIVE_INFINITY; 3]);
            bands.extend([ORDERED_NEGATIVE_INFINITY; 3]);
            bands.extend([0, 0]);
        }
        let mut parameters = PassParameters::new();
        parameters.insert("count", support.len() as u32);
        for pad in ["pad0", "pad1", "pad2"] {
            parameters.insert(pad, 0u32);
        }
        parameters.insert("positions", self.body.positions.clone());
        parameters.insert(
            "support",
            gpu.upload(BufferUpload::from_elements(&support))?,
        );
        parameters.insert("frame", frame.frame.clone());
        parameters.insert(
            "design",
            gpu.upload(BufferUpload::from_elements(&[
                design.neck_length.metres(),
                design.back_edge_lift.metres(),
                design.fit.clearance.metres() + wall,
                design.face_clearance.metres() + wall,
                wall,
                design.temple_clearance.metres(),
                0.0,
            ]))?,
        );
        parameters.insert("bands", gpu.upload(BufferUpload::from_elements(&bands))?);
        parameters.insert("fit", fit.clone());
        parameters.insert("status", frame.status.clone());
        let [measure, profile] = kernels(gpu)?;
        batch
            .dispatch_items(&measure, &parameters, (support.len() as u32).into())
            .map_err(device_error)?;
        batch
            .dispatch(&profile, &parameters, [1, 1, 1].into())
            .map_err(device_error)?;
        Ok(fit)
    }
}

fn kernels(gpu: &ArmorGpu) -> Result<[Arc<Kernel>; 2]> {
    let compile = |entry: &str| {
        gpu.cache()
            .get(gpu.context(), &source(entry))
            .map_err(device_error)
    };
    Ok([compile(MEASURE)?, compile(PROFILE)?])
}

/// The head-frame height span of each measured band.
const BAND_SPAN: &str = r#"
// The height band `band` spans, in the head frame.
fn band_span(band: u32) -> vec2<f32> {
    let above = MAX_FINITE;
    let chin = -half_height();
    let front_hem = host_sub(host_mul(-half_height(), NECK_HEM_HEAD_RATIO), design[NECK_LENGTH]);
    switch band {
        case 0u: {
            return vec2<f32>(host_mul(half_height(), BROW_HEIGHT), above);
        }
        case 1u: {
            return vec2<f32>(chin, above);
        }
        case 2u: {
            return vec2<f32>(host_add(chin, JAW_WIDTH_LOWER_M), host_add(chin, JAW_WIDTH_UPPER_M));
        }
        case 3u: {
            return vec2<f32>(chin, host_add(chin, JAW_FACE_HEIGHT_M));
        }
        case 4u: {
            let jaw_height = host_mul(chin, CHIN_HEAD_RATIO);
            let height = host_add(
                jaw_height,
                host_mul(host_sub(front_hem, jaw_height), SUBMENTAL_NECK_FRACTION),
            );
            return vec2<f32>(
                host_sub(height, SECTION_HALF_BAND_M),
                host_add(height, SECTION_HALF_BAND_M),
            );
        }
        case 5u: {
            let side_hem = host_add(front_hem, design[BACK_EDGE_LIFT]);
            return vec2<f32>(
                host_sub(side_hem, SECTION_HALF_BAND_M),
                host_add(side_hem, SECTION_HALF_BAND_M),
            );
        }
        case 6u: {
            let nape = host_add(chin, design[BACK_EDGE_LIFT]);
            return vec2<f32>(
                host_sub(nape, SECTION_HALF_BAND_M),
                host_add(nape, SECTION_HALF_BAND_M),
            );
        }
        default: {
            return vec2<f32>(
                host_sub(front_hem, SECTION_HALF_BAND_M),
                host_add(front_hem, SECTION_HALF_BAND_M),
            );
        }
    }
}

"#;

fn source(entry: &str) -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> positions: array<f32>;
@group(0) @binding(1) var<storage, read> support: array<u32>;
@group(0) @binding(2) var<storage, read> frame: array<f32>;
@group(0) @binding(3) var<storage, read> design: array<f32>;
@group(0) @binding(4) var<storage, read_write> bands: array<atomic<u32>>;
@group(0) @binding(5) var<storage, read_write> fit: array<f32>;
@group(0) @binding(6) var<storage, read_write> status: array<atomic<u32>>;
{math}
{counted}
@group(0) @binding(7) var<uniform> params: Params;
{ordered}
{zero_hook}{host_float}
{positions}

const NECK_LENGTH: u32 = 0u;
const BACK_EDGE_LIFT: u32 = 1u;
const GAP: u32 = 2u;
const FACE_GAP: u32 = 3u;
const WALL: u32 = 4u;
const TEMPLE_CLEARANCE: u32 = 5u;
const ZERO: u32 = 6u;

const NO_ENVELOPE: u32 = 2u;
const INVALID: u32 = 4u;

const SKULL: u32 = 0u;
const HEAD_WIDTH: u32 = 1u;
const JAW: u32 = 2u;
const FACE: u32 = 3u;
const SUBMENTAL: u32 = 4u;
const NECK: u32 = 5u;
const NAPE: u32 = 6u;
const THROAT: u32 = 7u;
const BANDS: u32 = {bands}u;
const BAND_WORDS: u32 = {band_words}u;
const MINIMUM_SECTION_SAMPLES: u32 = 4u;

const SECTION_HALF_BAND_M: f32 = 0.006;
const JAW_WIDTH_LOWER_M: f32 = 0.000;
const JAW_WIDTH_UPPER_M: f32 = 0.020;
const JAW_FACE_HEIGHT_M: f32 = 0.018;
const SUBMENTAL_NECK_FRACTION: f32 = 0.35;
const NECK_HEM_HEAD_RATIO: f32 = 1.20;
const CHIN_HEAD_RATIO: f32 = 1.03;
const BROW_HEIGHT: f32 = 0.09;
const PROFILE: u32 = {profile}u;

fn fail(bit: u32) {{
    atomicOr(&status[0], bit);
}}

fn axis(i: u32) -> vec3<f32> {{
    return vec3<f32>(frame[3u + i * 3u], frame[4u + i * 3u], frame[5u + i * 3u]);
}}

fn half_height() -> f32 {{
    return frame[13];
}}

{band_span}
{entry}
"#,
        band_span = BAND_SPAN,
        math = wgsl::MATH,
        counted = wgsl::COUNTED,
        ordered = wgsl::ORDERED_FLOAT,
        zero_hook = host_float::zero_hook("bitcast<u32>(design[ZERO])"),
        host_float = host_float::wgsl(),
        positions = wgsl::read_points("positions"),
        bands = BANDS,
        band_words = BAND_WORDS,
        profile = FIT_PROFILE_WORD,
    )
}

/// Each supported sample, in the head frame, widens every band it lies in.
const MEASURE: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let d = host_sub3(positions_at(support[i]), vec3<f32>(frame[0], frame[1], frame[2]));
    let p = vec3<f32>(host_dot(axis(0u), d), host_dot(axis(1u), d), host_dot(axis(2u), d));
    for (var band = 0u; band < BANDS; band = band + 1u) {
        let span = band_span(band);
        if (!(p.y >= span.x && p.y <= span.y)) {
            continue;
        }
        if (!(abs(p.x) <= 3.4e38 && abs(p.z) <= 3.4e38)) {
            fail(INVALID);
            return;
        }
        let at = band * BAND_WORDS;
        atomicAdd(&bands[at + 6u], 1u);
        for (var a = 0u; a < 3u; a = a + 1u) {
            atomicMin(&bands[at + a], ordered_from_float(p[a]));
            atomicMax(&bands[at + 3u + a], ordered_from_float(p[a]));
        }
    }
}
"#;

/// One invocation: the helmet's sections from the bands, then its own frame
/// and the profile into the fit buffer.
const PROFILE: &str = r#"
fn low(band: u32, a: u32) -> f32 {
    return float_from_ordered(atomicLoad(&bands[band * BAND_WORDS + a]));
}

fn high(band: u32, a: u32) -> f32 {
    return float_from_ordered(atomicLoad(&bands[band * BAND_WORDS + 3u + a]));
}

fn half_width(band: u32) -> f32 {
    return max(abs(low(band, 0u)), abs(high(band, 0u)));
}

@compute @workgroup_size(1)
fn main() {
    for (var band = 0u; band < BANDS; band = band + 1u) {
        if (atomicLoad(&bands[band * BAND_WORDS + 6u]) < MINIMUM_SECTION_SAMPLES) {
            fail(NO_ENVELOPE);
            return;
        }
    }
    let gap = design[GAP];
    let face_gap = design[FACE_GAP];
    var profile: array<f32, 11>;
    profile[0] = host_add(half_width(SKULL), gap);
    profile[1] = max(
        host_add(half_width(SKULL), gap),
        host_add(host_add(half_width(HEAD_WIDTH), design[WALL]), design[TEMPLE_CLEARANCE]),
    );
    profile[2] = host_add(high(SKULL, 2u), gap);
    profile[3] = host_sub(low(SKULL, 2u), gap);
    profile[4] = host_add(half_width(JAW), face_gap);
    profile[5] = host_add(high(FACE, 2u), face_gap);
    profile[6] = host_add(high(SUBMENTAL, 2u), face_gap);
    profile[7] = host_add(half_width(NECK), face_gap);
    profile[8] = host_add(high(THROAT, 2u), face_gap);
    profile[9] = host_sub(low(NECK, 2u), face_gap);
    profile[10] = host_sub(low(NAPE, 2u), face_gap);
    // A valid profile is finite, with the skull's front ahead of its back, the jaw ahead of the
    // throat ahead of the nape, and positive half widths.
    var valid = profile[2] > profile[3] && profile[8] > profile[9] && profile[5] > profile[8];
    for (var k = 0u; k < 11u; k = k + 1u) {
        valid = valid && abs(profile[k]) <= 3.4e38;
    }
    for (var k = 0u; k < 8u; k = k + 1u) {
        let width = k == 0u || k == 1u || k == 4u || k == 7u;
        valid = valid && (!width || profile[k] > 0.0);
    }
    if (!valid) {
        fail(INVALID);
        return;
    }
    // The helmet's own frame: identity axes at the origin, sized as the head.
    for (var w = 0u; w < PROFILE; w = w + 1u) {
        fit[w] = 0.0;
    }
    fit[3] = 1.0;
    fit[7] = 1.0;
    fit[11] = 1.0;
    fit[12] = frame[12];
    fit[13] = frame[13];
    fit[14] = frame[14];
    for (var k = 0u; k < 11u; k = k + 1u) {
        fit[PROFILE + k] = profile[k];
    }
}
"#;
