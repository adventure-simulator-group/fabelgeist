//! Chausses fitted on the device around a cage of the leg's skin sections.
//!
//! A boot's shaft must clear either leg garment worn beneath it, so the boot
//! fit needs both chausses on the wearer. Every tube carrier follows a
//! Hermite curve through the hip, knee and ankle; at its station the cage
//! measures the enclosing skin section across the curve and seats the
//! carrier around it. One invocation fits one carrier against every skin
//! vertex of the leg.

use adventuresim_armor_model::gpu::{device_error, wgsl};
use adventuresim_armor_model::{
    DevicePart, GarmentArmorDesign, GarmentArmorKind, gpu::record_garment_tube,
};
use anyhow::{Context, Result};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::{Buffer, PassParameters};

use crate::armor_frames::{FitRegion, Side};
use crate::device_frames::{DeviceFrame, DeviceWearer};

/// Room between a fitted garment's section and its wall.
const GARMENT_FIT_MARGIN_M: f32 = 0.006;

impl DeviceWearer<'_> {
    /// Record `kind` of chausses on the `side` leg, fitted and thickened, in
    /// the whole-leg `frame` with its skin `support`.
    pub(crate) fn record_fitted_chausses(
        &self,
        batch: &mut KernelBatch,
        kind: GarmentArmorKind,
        side: Side,
        frame: &DeviceFrame,
        support: &Buffer,
    ) -> Result<DevicePart> {
        let gpu = self.gpu;
        let design = GarmentArmorDesign::new(kind);
        let mut part = record_garment_tube(gpu, batch, &design, &frame.frame)?;
        let prefix = side.prefix();
        let anchor = |name: &str| -> Result<u32> {
            let name = format!("{prefix}_{name}");
            self.host
                .joint_names
                .iter()
                .position(|n| *n == name)
                .map(|i| i as u32)
                .with_context(|| format!("missing garment landmark {name}"))
        };
        let mut parameters = PassParameters::new();
        parameters.insert("count", self.body.vertex_count);
        parameters.insert("carriers_count", part.carrier_count());
        parameters.insert("hip", anchor("upleg")?);
        parameters.insert("knee", anchor("lowleg")?);
        parameters.insert("ankle", anchor("foot")?);
        parameters.insert(
            "quilted",
            u32::from(kind == GarmentArmorKind::PaddedChausses),
        );
        parameters.insert("pad0", 0u32);
        parameters.insert("pad1", 0u32);
        parameters.insert(
            "gap",
            design.clearance.metres() + design.wall_thickness.metres() + GARMENT_FIT_MARGIN_M,
        );
        parameters.insert("pad2", 0.0f32);
        parameters.insert("pad3", 0.0f32);
        parameters.insert("pad4", 0.0f32);
        parameters.insert("frames", frame.frame.clone());
        parameters.insert("positions", self.body.positions.clone());
        parameters.insert("joints", self.body.joints.clone());
        parameters.insert("support", support.clone());
        parameters.insert("carriers", part.carriers().clone());
        let kernel = gpu
            .cache()
            .get(gpu.context(), &cage_source())
            .map_err(device_error)?;
        batch
            .dispatch_items(&kernel, &parameters, part.carrier_count())
            .map_err(device_error)?;
        part.record_shells(gpu, batch)?;
        Ok(part)
    }

    /// The whole-leg frame and skin both chausses are fitted in.
    pub(crate) fn record_leg_cage(
        &self,
        batch: &mut KernelBatch,
        side: Side,
    ) -> Result<(DeviceFrame, Buffer)> {
        let region = FitRegion::WholeLeg(side);
        Ok((
            self.record_frame(batch, region)?,
            self.record_any_region_support(batch, &[region])?,
        ))
    }
}

fn cage_source() -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> frames: array<f32>;
@group(0) @binding(1) var<storage, read> positions: array<f32>;
@group(0) @binding(2) var<storage, read> joints: array<f32>;
@group(0) @binding(3) var<storage, read> support: array<u32>;
@group(0) @binding(4) var<storage, read_write> carriers: array<f32>;
struct Params {{
    count: u32,
    carriers_count: u32,
    hip: u32,
    knee: u32,
    ankle: u32,
    quilted: u32,
    pad0: u32,
    pad1: u32,
    gap: f32,
    pad2: f32,
    pad3: f32,
    pad4: f32,
}};
@group(0) @binding(5) var<uniform> params: Params;
{math}
{frame}
{positions}
{carriers}
{cage}
"#,
        math = wgsl::MATH,
        frame = wgsl::FRAME,
        positions = wgsl::read_points("positions"),
        carriers = wgsl::points("carriers"),
        cage = CAGE,
    )
}

const CAGE: &str = r#"
const SECTION_HALF_WIDTH_M: f32 = 0.018;
const MINIMUM_SECTION_SAMPLES: u32 = 12u;
const MINIMUM_SECTION_RADIUS_M: f32 = 0.005;
const QUILT_RELIEF_M: f32 = 0.0015;
const EPSILON: f32 = 1.1920929e-7;

fn host_dot(a: vec3<f32>, b: vec3<f32>) -> f32 {
    return (a.x * b.x + a.y * b.y) + a.z * b.z;
}

fn normalized(a: vec3<f32>) -> vec3<f32> {
    return a * (1.0 / max(sqrt(host_dot(a, a)), EPSILON));
}

fn host_cross(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x);
}

fn joint(index: u32) -> vec3<f32> {
    return vec3<f32>(joints[index * 8u], joints[index * 8u + 1u], joints[index * 8u + 2u]);
}

struct Station {
    center: vec3<f32>,
    tangent: vec3<f32>,
};

// A Hermite curve through hip, knee and ankle, continued straight past the
// ankle; the tangent points back toward the hip.
fn limb_station(a: vec3<f32>, b: vec3<f32>, segment: u32, t: f32) -> Station {
    if (t > 1.0) {
        return Station(b + (b - a) * (t - 1.0), normalized(a - b));
    }
    let bend_tangent = (joint(params.ankle) - joint(params.hip)) * 0.5;
    var m0 = bend_tangent;
    var m1 = b - a;
    if (segment == 0u) {
        m0 = b - a;
        m1 = bend_tangent;
    }
    let t3 = pow3(t);
    let position = (2.0 * t3 - 3.0 * t * t + 1.0) * a
        + (t3 - 2.0 * t * t + t) * m0
        + (-2.0 * t3 + 3.0 * t * t) * b
        + (t3 - t * t) * m1;
    let derivative = (6.0 * t * t - 6.0 * t) * a
        + (3.0 * t * t - 4.0 * t + 1.0) * m0
        + (-6.0 * t * t + 6.0 * t) * b
        + (3.0 * t * t - 2.0 * t) * m1;
    return Station(position, normalized(derivative * -1.0));
}

struct Section {
    lo: vec3<f32>,
    hi: vec3<f32>,
};

fn in_axes(axes: array<vec3<f32>, 3>, center: vec3<f32>, vertex: u32) -> vec3<f32> {
    let d = positions_at(vertex) - center;
    return vec3<f32>(host_dot(axes[0], d), host_dot(axes[1], d), host_dot(axes[2], d));
}

// The skin section across the curve: the support within the band, or the
// twelve nearest the plane when the band holds fewer; then the ellipse
// through its bounds, scaled to enclose the band.
fn enclosing_section(center: vec3<f32>, axes: array<vec3<f32>, 3>) -> Section {
    var band = 0u;
    var supported = 0u;
    var lo = vec3<f32>(3.4e38);
    var hi = vec3<f32>(-3.4e38);
    for (var v = 0u; v < params.count; v = v + 1u) {
        if (support[v] == 0u) {
            continue;
        }
        supported = supported + 1u;
        let s = in_axes(axes, center, v);
        if (abs(s.y) < SECTION_HALF_WIDTH_M) {
            band = band + 1u;
            lo = min(lo, s);
            hi = max(hi, s);
        }
    }
    if (band < MINIMUM_SECTION_SAMPLES) {
        // The nearest samples to the plane, ties to the lowest vertex.
        lo = vec3<f32>(3.4e38);
        hi = vec3<f32>(-3.4e38);
        let wanted = min(MINIMUM_SECTION_SAMPLES, supported);
        var last_distance = -1.0;
        var last_vertex = 0u;
        for (var taken = 0u; taken < wanted; taken = taken + 1u) {
            var best_distance = 3.4e38;
            var best_vertex = 0xffffffffu;
            for (var v = 0u; v < params.count; v = v + 1u) {
                if (support[v] == 0u) {
                    continue;
                }
                let d = abs(in_axes(axes, center, v).y);
                let after = d > last_distance || (d == last_distance && v > last_vertex);
                let better = d < best_distance || (d == best_distance && v < best_vertex);
                if (after && better) {
                    best_distance = d;
                    best_vertex = v;
                }
            }
            let s = in_axes(axes, center, best_vertex);
            lo = min(lo, s);
            hi = max(hi, s);
            last_distance = best_distance;
            last_vertex = best_vertex;
        }
    }
    let radial_center = vec2<f32>((lo.x + hi.x) * 0.5, (lo.z + hi.z) * 0.5);
    let radius = vec2<f32>(
        max((hi.x - lo.x) * 0.5, MINIMUM_SECTION_RADIUS_M),
        max((hi.z - lo.z) * 0.5, MINIMUM_SECTION_RADIUS_M),
    );
    var scale = 1.0;
    for (var v = 0u; v < params.count; v = v + 1u) {
        if (support[v] == 0u) {
            continue;
        }
        let s = in_axes(axes, center, v);
        if (abs(s.y) < SECTION_HALF_WIDTH_M) {
            scale = max(scale, sqrt(
                pow2((s.x - radial_center.x) / radius.x) + pow2((s.z - radial_center.y) / radius.y),
            ));
        }
    }
    lo.x = radial_center.x - radius.x * scale;
    hi.x = radial_center.x + radius.x * scale;
    lo.z = radial_center.y - radius.y * scale;
    hi.z = radial_center.y + radius.y * scale;
    return Section(lo, hi);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.carriers_count) {
        return;
    }
    let fit = frame_at(0u);
    let hip = joint(params.hip);
    let knee = joint(params.knee);
    let ankle = joint(params.ankle);
    let d = carriers_at(i) - fit.origin;
    let in_frame = vec3<f32>(host_dot(fit.x, d), host_dot(fit.y, d), host_dot(fit.z, d));
    let raw_t = max((fit.half_extents.y - in_frame.y) / (2.0 * fit.half_extents.y), 0.0);
    let angle = atan2(in_frame.x / fit.half_extents.x, in_frame.z / fit.half_extents.z);
    let radial_direction = fit.x * sin(angle) + fit.z * cos(angle);
    let trunkward = vec3<f32>(0.0, hip.y, hip.z) - hip;
    let toward_trunk = normalized(trunkward - fit.y * host_dot(trunkward, fit.y));
    let medial = max(host_dot(radial_direction, toward_trunk), 0.0);
    // The proximal opening climbs over the outer hip and drops beneath the
    // groin: a planar ring centred on the bone would cut the trunk.
    let setback = 0.015 + 0.20 * pow2(medial);
    let t = raw_t * 0.985 + setback * pow3(max(1.0 - raw_t, 0.0));
    let thigh = sqrt(host_dot(knee - hip, knee - hip));
    let shank = sqrt(host_dot(ankle - knee, ankle - knee));
    let bend = thigh / (thigh + shank);
    var station: Station;
    if (t < bend) {
        station = limb_station(hip, knee, 0u, t / bend);
    } else {
        station = limb_station(knee, ankle, 1u, (t - bend) / (1.0 - bend));
    }
    let tangent = station.tangent;
    let front = normalized(fit.z - tangent * host_dot(fit.z, tangent));
    let across = host_cross(tangent, front);
    let section = enclosing_section(station.center, array<vec3<f32>, 3>(across, tangent, front));
    let radius = vec2<f32>((section.hi.x - section.lo.x) * 0.5, (section.hi.z - section.lo.z) * 0.5);
    var quilting = 0.0;
    if (params.quilted != 0u) {
        quilting = QUILT_RELIEF_M * pow2(cos(angle * 6.0));
    }
    let offset_x = (section.hi.x + section.lo.x) * 0.5 + (radius.x + params.gap + quilting) * sin(angle);
    let offset_z = (section.hi.z + section.lo.z) * 0.5 + (radius.y + params.gap + quilting) * cos(angle);
    carriers_set(i, station.center + (across * offset_x + front * offset_z));
}
"#;
