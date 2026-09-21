//! Limb armor fitted on the device.

use adventuresim_armor_model::gpu::{device_error, wgsl};
use adventuresim_armor_model::{DevicePart, LimbArmorDesign, record_limb_armor};
use anyhow::{Result, bail};
use fabelgeist_compute::KernelBatch;
use fabelgeist_gpu::prelude::PassParameters;

use crate::armor_frames::{FitRegion, Side};
use crate::device_clearance::ClearanceFit;
use crate::device_clearance::PlateFit;
use crate::device_frames::{DeviceFrame, DeviceWearer};
use crate::device_piece::DeviceRecording;

/// Linear identity blends need a little extra room at the torso-facing armpit
/// edge. Distal trimming creates that space without widening the arm cylinder.
const REREBRACE_AXILLARY_MORPH_TRIM_M: f32 = 0.030;

impl DeviceWearer<'_> {
    /// Record a limb design's charts and fit, short of thickening.
    pub fn record_fitted_limb(
        &self,
        batch: &mut KernelBatch,
        design: &LimbArmorDesign,
        region: FitRegion,
    ) -> Result<DeviceRecording> {
        let gpu = self.gpu;
        let frame = self.record_frame(batch, region)?;
        let part = record_limb_armor(gpu, batch, design, &frame.frame)?;
        let clearance =
            |part: &DevicePart, gauge: adventuresim_armor_model::PlateGauge, style| ClearanceFit {
                region,
                clearance: gauge.clearance.metres(),
                thickness: gauge.thickness.metres(),
                style,
                count: part.carrier_count(),
                cuff: 0,
            };
        match design {
            LimbArmorDesign::Greave(d) => {
                let fit = clearance(&part, d.gauge, PlateFit::Greave(d));
                self.record_clearance_fit(batch, &frame, &fit, part.carriers(), part.status())?;
            }
            LimbArmorDesign::Cuisse(d) => {
                self.record_proximal_trim(batch, &frame, &part, region)?;
                let fit = clearance(&part, d.gauge, PlateFit::Cuisse(d));
                self.record_clearance_fit(batch, &frame, &fit, part.carriers(), part.status())?;
            }
            LimbArmorDesign::Rerebrace(d) => {
                self.record_proximal_trim(batch, &frame, &part, region)?;
                let fit = clearance(&part, d.gauge, PlateFit::Rerebrace(d));
                self.record_clearance_fit(batch, &frame, &fit, part.carriers(), part.status())?;
            }
            LimbArmorDesign::Poleyn(_)
            | LimbArmorDesign::Couter(_)
            | LimbArmorDesign::Spaulder(_) => {}
            _ => bail!("{design:?} is not fitted on the device yet"),
        }
        Ok(DeviceRecording {
            part,
            frames: vec![(frame, region)],
            checks: Vec::new(),
        })
    }

    /// Record the proximal trim of an upper-limb plate: its top edge cut
    /// back on the side toward the body, per shell.
    fn record_proximal_trim(
        &self,
        batch: &mut KernelBatch,
        frame: &DeviceFrame,
        part: &DevicePart,
        region: FitRegion,
    ) -> Result<()> {
        let (side, depth, reserve) = match region {
            FitRegion::Thigh(side) => (side, 0.65, 0.0),
            FitRegion::UpperArm(side) => (side, 1.5, REREBRACE_AXILLARY_MORPH_TRIM_M),
            _ => bail!("proximal trim requires an upper limb"),
        };
        let gpu = self.gpu;
        let shells = part.shell_count();
        let mut shell_of = vec![0u32; part.carrier_count() as usize];
        for shell in 0..shells {
            for carrier in part.shell_carriers(shell) {
                shell_of[carrier as usize] = shell as u32;
            }
        }
        let mut bounds = Vec::with_capacity(shells * 2);
        for _ in 0..shells {
            bounds.extend([0xff80_0000u32, 0x007f_ffff]);
        }
        let mut parameters = PassParameters::new();
        parameters.insert("count", part.carrier_count());
        parameters.insert("pad0", 0u32);
        parameters.insert("pad1", 0u32);
        parameters.insert("pad2", 0u32);
        parameters.insert(
            "outward",
            if matches!(side, Side::Left) {
                1.0f32
            } else {
                -1.0
            },
        );
        parameters.insert("depth", depth);
        parameters.insert("reserve", reserve);
        parameters.insert("pad3", 0.0f32);
        parameters.insert("frames", frame.frame.clone());
        parameters.insert("shell_of", gpu.upload(&shell_of)?);
        parameters.insert("bounds", gpu.upload(&bounds)?);
        parameters.insert("carriers", part.carriers().clone());
        for entry in [TRIM_BOUNDS, TRIM] {
            let kernel = gpu
                .cache()
                .get(gpu.context(), &trim_source(entry))
                .map_err(device_error)?;
            batch
                .dispatch_items(&kernel, &parameters, part.carrier_count())
                .map_err(device_error)?;
        }
        Ok(())
    }
}

fn trim_source(entry: &str) -> String {
    format!(
        r#"
@group(0) @binding(0) var<storage, read> frames: array<f32>;
@group(0) @binding(1) var<storage, read> shell_of: array<u32>;
@group(0) @binding(2) var<storage, read_write> bounds: array<atomic<u32>>;
@group(0) @binding(3) var<storage, read_write> carriers: array<f32>;
struct Params {{
    count: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
    outward: f32,
    depth: f32,
    reserve: f32,
    pad3: f32,
}};
@group(0) @binding(4) var<uniform> params: Params;
{math}
{frame}
{ordered}
{carriers}

fn host_local(f: Frame, p: vec3<f32>) -> vec3<f32> {{
    let d = p - f.origin;
    return vec3<f32>(
        (d.x * f.x.x + d.y * f.x.y) + d.z * f.x.z,
        (d.x * f.y.x + d.y * f.y.y) + d.z * f.y.z,
        (d.x * f.z.x + d.y * f.z.y) + d.z * f.z.z,
    );
}}

{entry}
"#,
        math = wgsl::MATH,
        frame = wgsl::FRAME,
        ordered = wgsl::ORDERED_FLOAT,
        carriers = wgsl::points("carriers"),
    )
}

/// Each shell's axial extent in the frame.
const TRIM_BOUNDS: &str = r#"
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let y = host_local(frame_at(0u), carriers_at(i)).y;
    let shell = shell_of[i];
    atomicMin(&bounds[shell * 2u], ordered_from_float(y));
    atomicMax(&bounds[shell * 2u + 1u], ordered_from_float(y));
}
"#;

/// Cut the upper boundary and interpolate toward it: a high-power axial
/// displacement could reverse rows and fold the sheet back.
const TRIM: &str = r#"
const MINIMUM_REMAINING_SPAN: f32 = 0.30;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) {
        return;
    }
    let fit = frame_at(0u);
    let shell = shell_of[i];
    let low = float_from_ordered(atomicLoad(&bounds[shell * 2u]));
    let high = float_from_ordered(atomicLoad(&bounds[shell * 2u + 1u]));
    let maximum_trim = (high - low) * (1.0 - MINIMUM_REMAINING_SPAN);
    var p = host_local(fit, carriers_at(i));
    let radial = fit.x.x * p.x + fit.z.x * p.z;
    let inward = clamp(-params.outward * radial / max(sqrt(p.x * p.x + p.z * p.z), 1e-6), 0.0, 1.0);
    let proximal = clamp((p.y - low) / (high - low), 0.0, 1.0);
    let trim = min(
        fit.half_extents.y * params.depth * inward * inward + params.reserve * inward,
        maximum_trim,
    );
    p.y = p.y - trim * proximal;
    carriers_set(i, frame_point(fit, p));
}
"#;
