//! Footwear sections measured on the device, and read by the footwear fits.
//!
//! Forty-eight sections span a part's axial extent in the foot frame. Each
//! gathers the points nearest its height, bounds them, and scales a
//! superellipse through the bounds until it encloses them. Every reduction is
//! an exact atomic minimum or maximum of ordered floats, so the points may
//! arrive in any order: skin vertices for the anatomical envelope, or the
//! garment slices a boot shaft must clear.

use anyhow::Result;
use fabelgeist_armor::gpu::wgsl;
use fabelgeist_compute::{KernelBatch, host_float};
use fabelgeist_gpu::prelude::BufferUpload;
use fabelgeist_gpu::prelude::PassParameterName;
use fabelgeist_gpu::prelude::{Buffer, PassParameters, ShaderSource};

use crate::armor_frames::FitRegion;
use crate::device_frames::DeviceWearer;

/// Sections along the footwear.
pub(crate) const PROFILE_SAMPLES: u32 = 48;
/// Ordered-float words per section: nearest distance, low x and z, high x
/// and z, and the enclosing scale.
const SECTION_WORDS: usize = 6;

const ORDERED_POSITIVE_INFINITY: u32 = 0xff80_0000;
const ORDERED_NEGATIVE_INFINITY: u32 = 0x007f_ffff;

/// A float as its ordered unsigned encoding, as `wgsl::ORDERED_FLOAT` has it.
fn ordered(value: f32) -> u32 {
    let bits = value.to_bits();
    if bits & 0x8000_0000 != 0 {
        !bits
    } else {
        bits | 0x8000_0000
    }
}

impl DeviceWearer<'_> {
    /// Record which body vertices support any of `regions`: nonzero where a
    /// region's joints own at least a minimum share of the vertex's skin
    /// weight.
    pub(crate) fn record_any_region_support(
        &self,
        batch: &mut KernelBatch,
        regions: &[FitRegion],
    ) -> Result<Buffer> {
        anyhow::ensure!(regions.len() <= 32, "too many support regions");
        let mut masks = vec![0u32; self.host.joint_names.len()];
        for (bit, region) in regions.iter().enumerate() {
            let owned = self.host.owned_joints(&region.owners());
            for (mask, owns) in masks.iter_mut().zip(owned) {
                *mask |= u32::from(owns) << bit;
            }
        }
        let gpu = self.gpu;
        let support = gpu.scratch(
            (self.body.vertex_count as u64 * 4).into(),
            ("footwear support").into(),
        )?;
        let mut parameters = PassParameters::new();
        parameters.insert("count".into(), (self.body.vertex_count).into());
        parameters.insert("regions".into(), (regions.len() as u32).into());
        parameters.insert("pad1".into(), (0u32).into());
        parameters.insert("pad2".into(), (0u32).into());
        parameters.insert(
            PassParameterName::from(host_float::ZERO_FIELD),
            (0u32).into(),
        );
        for pad in ["pad3", "pad4", "pad5"] {
            parameters.insert(pad.into(), (0.0f32).into());
        }
        parameters.insert(
            "joint_indices".into(),
            (self.body.joint_indices.clone()).into(),
        );
        parameters.insert(
            "joint_weights".into(),
            (self.body.joint_weights.clone()).into(),
        );
        parameters.insert(
            "masks".into(),
            (gpu.upload(BufferUpload::from_elements(&masks))?).into(),
        );
        parameters.insert("support".into(), (support.clone()).into());
        let kernel = gpu
            .cache()
            .get(gpu.context(), &support_source())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        batch
            .dispatch_items(&kernel, &parameters, (self.body.vertex_count).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        Ok(support)
    }

    /// Record the body's vertices in `frame`, each flagged by `support`.
    pub(crate) fn record_body_points(
        &self,
        batch: &mut KernelBatch,
        frame: &Buffer,
        support: &Buffer,
    ) -> Result<Buffer> {
        let gpu = self.gpu;
        let points = gpu.scratch(
            (self.body.vertex_count as u64 * 16).into(),
            ("footwear body points").into(),
        )?;
        let mut parameters = counted(self.body.vertex_count);
        parameters.insert("frames".into(), (frame.clone()).into());
        parameters.insert("positions".into(), (self.body.positions.clone()).into());
        parameters.insert("support".into(), (support.clone()).into());
        parameters.insert("points".into(), (points.clone()).into());
        let kernel = gpu
            .cache()
            .get(gpu.context(), &points_source())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        batch
            .dispatch_items(&kernel, &parameters, (self.body.vertex_count).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
        Ok(points)
    }
}

fn counted(count: u32) -> PassParameters {
    let mut parameters = PassParameters::new();
    parameters.insert("count".into(), (count).into());
    for pad in ["pad0", "pad1", "pad2"] {
        parameters.insert(pad.into(), (0u32).into());
    }
    parameters
}

/// A fresh pair of ordered-float axial bounds, lowest then highest.
pub(crate) fn axial_bounds(gpu: &fabelgeist_armor::ArmorGpu, pairs: usize) -> Result<Buffer> {
    let words = [ORDERED_POSITIVE_INFINITY, ORDERED_NEGATIVE_INFINITY].repeat(pairs);
    Ok(gpu.upload(BufferUpload::from_elements(&words))?)
}

/// Record the axial extent of `count` points in `frame` into the bounds
/// pair at `pair` of `bounds`.
pub(crate) fn record_axial_bounds(
    gpu: &fabelgeist_armor::ArmorGpu,
    batch: &mut KernelBatch,
    frame: &Buffer,
    positions: &Buffer,
    count: u32,
    bounds: &Buffer,
    pair: u32,
) -> Result<()> {
    let mut parameters = PassParameters::new();
    parameters.insert("count".into(), (count).into());
    parameters.insert("pair".into(), (pair).into());
    parameters.insert("pad1".into(), (0u32).into());
    parameters.insert("pad2".into(), (0u32).into());
    parameters.insert("frames".into(), (frame.clone()).into());
    parameters.insert("positions".into(), (positions.clone()).into());
    parameters.insert("bounds".into(), (bounds.clone()).into());
    let kernel = gpu
        .cache()
        .get(gpu.context(), &bounds_source())
        .map_err(fabelgeist_armor::GenerateError::from)?;
    batch
        .dispatch_items(&kernel, &parameters, (count).into())
        .map_err(fabelgeist_armor::GenerateError::from)?;
    Ok(())
}

/// Record the sections of `points` (local points, flagged in `w`) between
/// the first pair of `bounds`.
pub(crate) fn record_sections(
    gpu: &fabelgeist_armor::ArmorGpu,
    batch: &mut KernelBatch,
    frame: &Buffer,
    bounds: &Buffer,
    points: &Buffer,
    count: u32,
) -> Result<Buffer> {
    let mut words = Vec::with_capacity(PROFILE_SAMPLES as usize * SECTION_WORDS);
    for _ in 0..PROFILE_SAMPLES {
        words.extend([
            ORDERED_POSITIVE_INFINITY,
            ORDERED_POSITIVE_INFINITY,
            ORDERED_POSITIVE_INFINITY,
            ORDERED_NEGATIVE_INFINITY,
            ORDERED_NEGATIVE_INFINITY,
            ordered(1.0),
        ]);
    }
    let sections = gpu.upload(BufferUpload::from_elements(&words))?;
    let mut parameters = counted(count);
    parameters.insert("frames".into(), (frame.clone()).into());
    parameters.insert("bounds".into(), (bounds.clone()).into());
    parameters.insert("points".into(), (points.clone()).into());
    parameters.insert("sections".into(), (sections.clone()).into());
    for entry in [NEAREST, SLICE, ENCLOSE] {
        let kernel = gpu
            .cache()
            .get(gpu.context(), &sections_source(entry))
            .map_err(fabelgeist_armor::GenerateError::from)?;
        batch
            .dispatch_items(&kernel, &parameters, (count).into())
            .map_err(fabelgeist_armor::GenerateError::from)?;
    }
    Ok(sections)
}

/// Constants and helpers every footwear kernel shares.
pub(crate) fn common() -> String {
    format!(
        r#"
{math}
{frame}
{ordered}

const SAMPLES: u32 = {samples}u;
const WORDS: u32 = {words}u;
const PROFILE_WINDOW_M: f32 = 0.008;
const MINIMUM_RADIAL_EXTENT_M: f32 = 0.008;
const PROFILE_CLEARANCE_MARGIN_M: f32 = 0.002;

fn host_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {{
    return (a.x * b.x + a.y * b.y) + a.z * b.z;
}}

fn local(f: Frame, p: vec3<f32>) -> vec3<f32> {{
    let d = p - f.origin;
    return vec3<f32>(host_dot(d, f.x), host_dot(d, f.y), host_dot(d, f.z));
}}

fn sample_height(index: u32, low: f32, high: f32) -> f32 {{
    return low + (high - low) * f32(index) / f32(SAMPLES - 1u);
}}

// The superellipse exponent: rounded-rectangular at the sole, elliptic up
// the shaft.
fn foot_exponent(height: f32, foot_height: f32) -> f32 {{
    return 2.0 + 2.0 * clamp(1.0 - (height + foot_height) / (foot_height * 1.4), 0.0, 1.0);
}}

fn superellipse(delta: vec2<f32>, radius: vec2<f32>, exponent: f32) -> f32 {{
    return pow(
        pow(abs(delta.x / radius.x), exponent) + pow(abs(delta.y / radius.y), exponent),
        1.0 / exponent,
    );
}}
"#,
        math = wgsl::MATH,
        frame = wgsl::FRAME,
        ordered = wgsl::ORDERED_FLOAT,
        samples = PROFILE_SAMPLES,
        words = SECTION_WORDS,
    )
}

/// Support weights summed in joint-slot order with exact device arithmetic
/// (`fabelgeist_compute::host_float`): a vertex at the threshold falls on
/// the same side whatever the device's fused operations.
fn support_source() -> ShaderSource {
    ShaderSource::from(format!(
        "{}{}{}{SUPPORT}",
        host_float::PARAMS_ZERO_HOOK,
        host_float::wgsl(),
        crate::armor_frames::skin_support_wgsl(),
    ))
}

const SUPPORT: &str = r#"
@group(0) @binding(0) var<storage, read> joint_indices: array<u32>;
@group(0) @binding(1) var<storage, read> joint_weights: array<f32>;
@group(0) @binding(2) var<storage, read> masks: array<u32>;
@group(0) @binding(3) var<storage, read_write> support: array<u32>;
struct Params {
    count: u32,
    regions: u32,
    pad1: u32,
    pad2: u32,
    zero: u32,
    pad3: f32,
    pad4: f32,
    pad5: f32,
};
@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    var supported = 0u;
    for (var region = 0u; region < params.regions; region = region + 1u) {
        var weight = 0.0;
        for (var k = 0u; k < 8u; k = k + 1u) {
            if ((masks[joint_indices[i * 8u + k]] & (1u << region)) != 0u) {
                weight = host_add(weight, joint_weights[i * 8u + k]);
            }
        }
        if (weight >= SKIN_SUPPORT_THRESHOLD) {
            supported = 1u;
        }
    }
    support[i] = supported;
}
"#;

fn points_source() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> frames: array<f32>;
@group(0) @binding(1) var<storage, read> positions: array<f32>;
@group(0) @binding(2) var<storage, read> support: array<u32>;
@group(0) @binding(3) var<storage, read_write> points: array<vec4<f32>>;
{counted}
@group(0) @binding(4) var<uniform> params: Params;
{common}
{positions}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let i = id.x;
    if (i >= params.count) {{
        return;
    }}
    let p = local(frame_at(0u), positions_at(i));
    points[i] = vec4<f32>(p, select(0.0, 1.0, support[i] != 0u));
}}
"#,
        counted = wgsl::COUNTED,
        common = common(),
        positions = wgsl::read_points("positions"),
    ))
}

fn bounds_source() -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> frames: array<f32>;
@group(0) @binding(1) var<storage, read> positions: array<f32>;
@group(0) @binding(2) var<storage, read_write> bounds: array<atomic<u32>>;
struct Params {{
    count: u32,
    pair: u32,
    pad1: u32,
    pad2: u32,
}};
@group(0) @binding(3) var<uniform> params: Params;
{common}
{positions}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let i = id.x;
    if (i >= params.count) {{
        return;
    }}
    let y = local(frame_at(0u), positions_at(i)).y;
    atomicMin(&bounds[params.pair * 2u], ordered_from_float(y));
    atomicMax(&bounds[params.pair * 2u + 1u], ordered_from_float(y));
}}
"#,
        common = common(),
        positions = wgsl::read_points("positions"),
    ))
}

fn sections_source(entry: &str) -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> frames: array<f32>;
@group(0) @binding(1) var<storage, read> bounds: array<u32>;
@group(0) @binding(2) var<storage, read> points: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> sections: array<atomic<u32>>;
{counted}
@group(0) @binding(4) var<uniform> params: Params;
{common}

fn low() -> f32 {{ return float_from_ordered(bounds[0]); }}
fn high() -> f32 {{ return float_from_ordered(bounds[1]); }}
fn word(section: u32, slot: u32) -> f32 {{
    return float_from_ordered(atomicLoad(&sections[section * WORDS + slot]));
}}
fn in_slice(section: u32, y: f32) -> bool {{
    return abs(y - sample_height(section, low(), high())) <= word(section, 0u) + PROFILE_WINDOW_M;
}}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
    let i = id.x;
    if (i >= params.count) {{
        return;
    }}
    let p = points[i];
    if (p.w == 0.0) {{
        return;
    }}
    for (var section = 0u; section < SAMPLES; section = section + 1u) {{
        let at = section * WORDS;
        {entry}
    }}
}}
"#,
        counted = wgsl::COUNTED,
        common = common(),
    ))
}

/// The distance from each section's height to its nearest point.
const NEAREST: &str = r#"
atomicMin(&sections[at], ordered_from_float(abs(p.y - sample_height(section, low(), high()))));
"#;

/// The bounds of the points within a window of that distance.
const SLICE: &str = r#"
if (in_slice(section, p.y)) {
    atomicMin(&sections[at + 1u], ordered_from_float(p.x));
    atomicMin(&sections[at + 2u], ordered_from_float(p.z));
    atomicMax(&sections[at + 3u], ordered_from_float(p.x));
    atomicMax(&sections[at + 4u], ordered_from_float(p.z));
}
"#;

/// The scale at which the bounds' superellipse encloses them.
const ENCLOSE: &str = r#"
if (in_slice(section, p.y)) {
    let lo = vec2<f32>(word(section, 1u), word(section, 2u));
    let hi = vec2<f32>(word(section, 3u), word(section, 4u));
    let center = (lo + hi) * 0.5;
    let radius = max((hi - lo) * 0.5, vec2<f32>(MINIMUM_RADIAL_EXTENT_M));
    let exponent = foot_exponent(sample_height(section, low(), high()), frame_at(0u).half_extents.y);
    let scale = superellipse(vec2<f32>(p.x, p.z) - center, radius, exponent);
    atomicMax(&sections[at + 5u], ordered_from_float(scale));
}
"#;
