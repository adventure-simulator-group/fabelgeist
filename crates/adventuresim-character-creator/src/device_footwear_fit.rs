//! Footwear carriers moved onto measured sections: onto the anatomical
//! envelope, out over the garments beneath a boot, and the boot's ankle
//! faired outward.

use anyhow::Result;
use fabelgeist_armor::ArmorGpu;
use fabelgeist_armor::gpu::wgsl;
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, PassParameters, ShaderSource};

use crate::device_foot_sections::common;

/// How a fit moves carriers onto the sections.
#[derive(Clone, Copy)]
pub(crate) enum SectionFit<'a> {
    /// Onto the anatomical envelope, from inside it.
    Envelope,
    /// Out over garment layers, fading in above the lowest garment hem;
    /// `hems` holds each garment's axial bounds.
    Layer { hems: &'a Buffer },
}

/// Record the fit of the first `count` carriers onto `sections`, measured
/// between the first pair of `bounds` with a footwear gauge's `clearance`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn record_section_fit(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    frame: &Buffer,
    bounds: &Buffer,
    sections: &Buffer,
    clearance: f32,
    fit: SectionFit,
    carriers: &Buffer,
    count: u32,
) -> Result<()> {
    // The kernel's `ENVELOPE` mode, or the layer fit.
    let (mode, hems) = match fit {
        SectionFit::Envelope => (0u32, bounds.clone()),
        SectionFit::Layer { hems } => (1, hems.clone()),
    };
    let mut parameters = PassParameters::new();
    parameters.insert("count".into(), (count).into());
    parameters.insert("mode".into(), (mode).into());
    parameters.insert("pad1".into(), (0u32).into());
    parameters.insert("pad2".into(), (0u32).into());
    parameters.insert("clearance".into(), (clearance).into());
    parameters.insert("pad3".into(), (0.0f32).into());
    parameters.insert("pad4".into(), (0.0f32).into());
    parameters.insert("pad5".into(), (0.0f32).into());
    parameters.insert("frames".into(), (frame.clone()).into());
    parameters.insert("bounds".into(), (bounds.clone()).into());
    parameters.insert("sections".into(), (sections.clone()).into());
    parameters.insert("hems".into(), (hems).into());
    parameters.insert("carriers".into(), (carriers.clone()).into());
    // Only the fairing keeps radii; the binding still needs a buffer.
    parameters.insert(
        "radii".into(),
        (gpu.scratch((4u64).into(), ("unused fairing radii").into())?).into(),
    );
    let kernel = gpu
        .cache()
        .get(gpu.context(), &fit_source(FIT))
        .map_err(fabelgeist_armor::GenerateError::from)?;
    batch
        .dispatch_items(&kernel, &parameters, (count).into())
        .map_err(fabelgeist_armor::GenerateError::from)?;
    Ok(())
}

/// Record the relaxation of a boot's ankle valleys outward, along its
/// authored meridians, about the top section's centre.
#[allow(clippy::too_many_arguments)]
pub(crate) fn record_ankle_fairing(
    gpu: &ArmorGpu,
    batch: &mut KernelBatch,
    frame: &Buffer,
    sections: &Buffer,
    clearance: f32,
    hems: &Buffer,
    carriers: &Buffer,
    count: u32,
) -> Result<()> {
    let mut parameters = PassParameters::new();
    parameters.insert("count".into(), (count).into());
    parameters.insert("mode".into(), (0u32).into());
    parameters.insert("pad1".into(), (0u32).into());
    parameters.insert("pad2".into(), (0u32).into());
    parameters.insert("clearance".into(), (clearance).into());
    parameters.insert("pad3".into(), (0.0f32).into());
    parameters.insert("pad4".into(), (0.0f32).into());
    parameters.insert("pad5".into(), (0.0f32).into());
    parameters.insert("frames".into(), (frame.clone()).into());
    parameters.insert("bounds".into(), (hems.clone()).into());
    parameters.insert("sections".into(), (sections.clone()).into());
    parameters.insert("hems".into(), (hems.clone()).into());
    parameters.insert("carriers".into(), (carriers.clone()).into());
    parameters.insert(
        "radii".into(),
        (gpu.scratch((count as u64 * 8).into(), ("ankle fairing radii").into())?).into(),
    );
    let kernel = gpu
        .cache()
        .get(gpu.context(), &fit_source(FAIR))
        .map_err(fabelgeist_armor::GenerateError::from)?;
    batch
        .dispatch(&kernel, &parameters, ([1, 1, 1]).into())
        .map_err(fabelgeist_armor::GenerateError::from)?;
    Ok(())
}

fn fit_source(entry: &str) -> ShaderSource {
    ShaderSource::from(format!(
        r#"
@group(0) @binding(0) var<storage, read> frames: array<f32>;
@group(0) @binding(1) var<storage, read> bounds: array<u32>;
@group(0) @binding(2) var<storage, read> sections: array<u32>;
@group(0) @binding(3) var<storage, read> hems: array<u32>;
@group(0) @binding(4) var<storage, read_write> carriers: array<f32>;
@group(0) @binding(5) var<storage, read_write> radii: array<f32>;
struct Params {{
    count: u32,
    mode: u32,
    pad1: u32,
    pad2: u32,
    clearance: f32,
    pad3: f32,
    pad4: f32,
    pad5: f32,
}};
@group(0) @binding(6) var<uniform> params: Params;
{common}
{carriers}

const ENVELOPE: u32 = 0u;
const BOOT_HEM_TRANSITION_M: f32 = 0.04;
const ANKLE_FAIRING_BELOW_HEM_M: f32 = 0.05;
const ANKLE_FAIRING_ABOVE_HEM_M: f32 = 0.08;
const ANKLE_FAIRING_PASSES: u32 = 64u;
const RING_PLANE_TOLERANCE_M: f32 = 0.000001;
const EPSILON: f32 = 1.1920929e-7;

fn word(section: u32, slot: u32) -> f32 {{
    return float_from_ordered(sections[section * WORDS + slot]);
}}

// A measured section: centre, then enclosing radii with the gauge's room.
fn measured(section: u32) -> vec4<f32> {{
    let lo = vec2<f32>(word(section, 1u), word(section, 2u));
    let hi = vec2<f32>(word(section, 3u), word(section, 4u));
    let radius = max((hi - lo) * 0.5, vec2<f32>(MINIMUM_RADIAL_EXTENT_M));
    return vec4<f32>((lo + hi) * 0.5, radius * word(section, 5u) + params.clearance);
}}

// A section smoothed with its neighbours, clamped at the ends.
fn smoothed(section: u32) -> vec4<f32> {{
    let a = measured(max(section, 1u) - 1u);
    let b = measured(section);
    let c = measured(min(section + 1u, SAMPLES - 1u));
    return (a + 2.0 * b + c) * 0.25;
}}

fn lowest_hem() -> f32 {{
    return min(float_from_ordered(hems[0]), float_from_ordered(hems[2]));
}}

{entry}
"#,
        common = common(),
        carriers = wgsl::points("carriers"),
    ))
}

/// One invocation per carrier: the envelope fit or the layer fit.
const FIT: &str = r#"
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let fit = frame_at(0u);
    let low = float_from_ordered(bounds[0]);
    let high = float_from_ordered(bounds[1]);
    var p = local(fit, carriers_at(i));
    var blend = 0.0;
    if (params.mode == ENVELOPE) {
        // The sole's central fan vertex is interior to the enclosing
        // perimeter; it stays inside the fitted bottom ring.
        if (p.y < low + PROFILE_CLEARANCE_MARGIN_M
            && sqrt(p.x * p.x + p.z * p.z) < MINIMUM_RADIAL_EXTENT_M) {
            return;
        }
    } else {
        blend = clamp((p.y - lowest_hem() + BOOT_HEM_TRANSITION_M) / BOOT_HEM_TRANSITION_M, 0.0, 1.0);
        if (blend == 0.0) {
            return;
        }
    }
    let last = f32(SAMPLES - 1u);
    let axial = clamp((p.y - low) / (high - low) * last, 0.0, last);
    let index = min(u32(floor(axial)), SAMPLES - 2u);
    let fraction = axial - f32(index);
    let a = smoothed(index);
    let b = smoothed(index + 1u);
    let center = a.xy + (b.xy - a.xy) * fraction;
    let radius = a.zw + (b.zw - a.zw) * fraction;
    let delta = vec2<f32>(p.x - center.x, p.z - center.y);
    let normalized = superellipse(delta, radius, foot_exponent(p.y, fit.half_extents.y));
    if (params.mode == ENVELOPE) {
        if (normalized < 1.0 && normalized > 0.001) {
            p.x = center.x + delta.x / normalized;
            p.z = center.y + delta.y / normalized;
            carriers_set(i, frame_point(fit, p));
        }
    } else if (normalized > EPSILON && normalized < 1.0) {
        let expansion = (1.0 / normalized - 1.0) * blend * blend * (3.0 - 2.0 * blend);
        p.x = p.x + delta.x * expansion;
        p.z = p.z + delta.y * expansion;
        carriers_set(i, frame_point(fit, p));
    }
}
"#;

/// One workgroup: relax the radii of rings near the hem toward the mean of
/// the rings above and below, never inward, in lockstep passes.
const FAIR: &str = r#"
const LANES: u32 = 256u;

var<workgroup> ring_size: u32;

fn ring_height(i: u32) -> f32 {
    return local(frame_at(0u), carriers_at(i)).y;
}

@compute @workgroup_size(256)
fn main(@builtin(local_invocation_index) lane: u32) {
    let fit = frame_at(0u);
    let n = params.count;
    let center = smoothed(SAMPLES - 1u).xy;
    let hem = lowest_hem();
    if (lane == 0u) {
        let first = ring_height(0u);
        var size = 0u;
        loop {
            if (size >= n || !(abs(ring_height(size) - first) < RING_PLANE_TOLERANCE_M)) {
                break;
            }
            size = size + 1u;
        }
        ring_size = size;
    }
    let size = workgroupUniformLoad(&ring_size);
    let rings = n / size;
    for (var i = lane; i < n; i = i + LANES) {
        let p = local(fit, carriers_at(i));
        radii[i] = length(vec2<f32>(p.x - center.x, p.z - center.y));
    }
    storageBarrier();
    for (var pass_index = 0u; pass_index < ANKLE_FAIRING_PASSES; pass_index = pass_index + 1u) {
        let read_at = (pass_index % 2u) * n;
        let write_at = n - read_at;
        for (var i = lane; i < n; i = i + LANES) {
            var radius = radii[read_at + i];
            let row = i / size;
            if (row >= 1u && row + 1u < rings) {
                let y = ring_height(i);
                if (y > hem - ANKLE_FAIRING_BELOW_HEM_M && y < hem + ANKLE_FAIRING_ABOVE_HEM_M) {
                    radius = max(radius, (radii[read_at + i - size] + radii[read_at + i + size]) * 0.5);
                }
            }
            radii[write_at + i] = radius;
        }
        storageBarrier();
    }
    // An even number of passes ends in the first half.
    for (var i = lane; i < n; i = i + LANES) {
        var p = local(fit, carriers_at(i));
        let radius = length(vec2<f32>(p.x - center.x, p.z - center.y));
        if (radius > EPSILON && radii[i] > radius) {
            let scale = radii[i] / radius;
            p.x = center.x + (p.x - center.x) * scale;
            p.z = center.y + (p.z - center.y) * scale;
            carriers_set(i, frame_point(fit, p));
        }
    }
}
"#;
