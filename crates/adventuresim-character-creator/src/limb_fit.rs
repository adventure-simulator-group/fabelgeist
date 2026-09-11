//! Anatomical fitting for independent limb-armor carriers.
//!
//! Thumb placement uses actual rig anchors. Footwear cross-sections fit the foot
//! and the lower leg as one smooth envelope, without copying skin topology.

use adventuresim_armor_model::{
    LimbArmorDesign, PartFrame, PartMesh, PlateGauge, generate_gauntlet_thumb, generate_limb_armor,
};
use anyhow::{Context, Result};

use crate::armor_clearance::{LAME_SPACING_GAUGES, PlateFit};
use crate::armor_frames::{FitRegion, Side, Wearer};

const PROFILE_SAMPLES: usize = 48;
const PROFILE_WINDOW_M: f32 = 0.008;
const MINIMUM_RADIAL_EXTENT_M: f32 = 0.008;
const PROFILE_CLEARANCE_MARGIN_M: f32 = 0.002;
#[path = "boot_layer_fit.rs"]
mod boot_layer_fit;
#[path = "sabaton_fit.rs"]
mod sabaton_fit;
// Linear identity blends need a little extra room at the torso-facing armpit
// edge. Distal trimming creates that space without widening the arm cylinder.
const REREBRACE_AXILLARY_MORPH_TRIM_M: f32 = 0.030;

pub fn fitted_limb(
    design: &LimbArmorDesign,
    wearer: &Wearer<'_>,
    region: FitRegion,
) -> Result<PartMesh> {
    let frame = wearer.frame(region)?;
    let mesh = generate_limb_armor(design, &frame)?;
    match design {
        LimbArmorDesign::MittenGauntlet(d) => {
            let FitRegion::Hand(side) = region else {
                anyhow::bail!("mitten requires hand landmarks")
            };
            let mut mesh = crate::armor_clearance::fit(
                &mesh,
                wearer,
                region,
                d.gauge.clearance,
                d.gauge.thickness,
                PlateFit::Mitten {
                    cuff_length: d.cuff_length.unit(),
                    cuff_clearance: d.cuff_clearance,
                },
            )?;
            mesh.append(generate_gauntlet_thumb(d, &thumb_frame(wearer, side)?)?);
            Ok(mesh)
        }
        LimbArmorDesign::LeatherBoot(d) => {
            let FitRegion::Foot(side) = region else {
                anyhow::bail!("boot requires foot landmarks")
            };
            let mesh = fit_foot_envelope(mesh, d.gauge, None, wearer, side, &frame)?;
            boot_layer_fit::fit(mesh, d.gauge, wearer, side, &frame)
        }
        LimbArmorDesign::Sabaton(d) => {
            let FitRegion::Foot(side) = region else {
                anyhow::bail!("sabaton requires foot landmarks")
            };
            sabaton_fit::fit(mesh, d, wearer, side, &frame)
        }
        LimbArmorDesign::Greave(d) => crate::armor_clearance::fit(
            &mesh,
            wearer,
            region,
            d.gauge.clearance,
            d.gauge.thickness,
            PlateFit::Greave(d),
        ),
        LimbArmorDesign::Cuisse(d) => {
            let mesh = trim_proximal(&mesh, &frame, region)?;
            crate::armor_clearance::fit(
                &mesh,
                wearer,
                region,
                d.gauge.clearance,
                d.gauge.thickness,
                PlateFit::Cuisse(d),
            )
        }
        LimbArmorDesign::Rerebrace(d) => {
            let mesh = trim_proximal(&mesh, &frame, region)?;
            crate::armor_clearance::fit(
                &mesh,
                wearer,
                region,
                d.gauge.clearance,
                d.gauge.thickness,
                PlateFit::Rerebrace(d),
            )
        }
        _ => Ok(mesh),
    }
}

fn trim_proximal(mesh: &PartMesh, frame: &PartFrame, region: FitRegion) -> Result<PartMesh> {
    let (side, depth) = match region {
        FitRegion::Thigh(side) => (side, 0.65),
        FitRegion::UpperArm(side) => (side, 1.5),
        _ => anyhow::bail!("proximal trim requires an upper limb"),
    };
    let outward = if matches!(side, Side::Left) {
        1.0
    } else {
        -1.0
    };
    Ok(mesh.refit_surfaces(|points| {
        const MINIMUM_REMAINING_SPAN: f32 = 0.30;
        let low = points
            .iter()
            .map(|p| local(frame, *p)[1])
            .fold(f32::INFINITY, f32::min);
        let high = points
            .iter()
            .map(|p| local(frame, *p)[1])
            .fold(f32::NEG_INFINITY, f32::max);
        let maximum_trim = (high - low) * (1.0 - MINIMUM_REMAINING_SPAN);
        for point in points {
            let mut p = local(frame, *point);
            let radial = frame.axes[0][0] * p[0] + frame.axes[2][0] * p[2];
            let inward = (-outward * radial / p[0].hypot(p[2]).max(1e-6)).clamp(0.0, 1.0);
            let proximal = ((p[1] - low) / (high - low)).clamp(0.0, 1.0);
            let reserve = if matches!(region, FitRegion::UpperArm(_)) {
                REREBRACE_AXILLARY_MORPH_TRIM_M * inward
            } else {
                0.0
            };
            // Cut the upper boundary and interpolate toward it. A high-power
            // axial displacement can reverse rows and fold the sheet back.
            let trim = (frame.half_extents[1] * depth * inward.powi(2) + reserve).min(maximum_trim);
            p[1] -= trim * proximal;
            *point = frame.point(p);
        }
    })?)
}

fn prefix(side: Side) -> &'static str {
    match side {
        Side::Left => "l",
        Side::Right => "r",
    }
}

fn joint(wearer: &Wearer<'_>, name: &str) -> Result<[f32; 3]> {
    let index = wearer
        .joint_names
        .iter()
        .position(|n| n == name)
        .with_context(|| format!("missing limb landmark {name}"))?;
    Ok(std::array::from_fn(|axis| wearer.joints[index][axis]))
}

#[path = "thumb_fit.rs"]
mod thumb_fit;
use thumb_fit::thumb_frame;

#[derive(Clone, Copy)]
struct FootSection {
    center: [f32; 2],
    radius: [f32; 2],
}

fn fit_foot_envelope(
    mesh: PartMesh,
    gauge: PlateGauge,
    lames: Option<u8>,
    wearer: &Wearer<'_>,
    side: Side,
    frame: &PartFrame,
) -> Result<PartMesh> {
    let mut support = wearer.support_indices(FitRegion::Foot(side))?;
    support.extend(wearer.support_indices(FitRegion::LowerLeg(side))?);
    let points = support
        .into_iter()
        .map(|i| local(frame, wearer.positions[i]))
        .collect::<Vec<_>>();
    let low = mesh
        .positions
        .iter()
        .map(|p| local(frame, *p)[1])
        .fold(f32::INFINITY, f32::min);
    let high = mesh
        .positions
        .iter()
        .map(|p| local(frame, *p)[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let sections = foot_sections(&points, low, high, gauge, frame.half_extents[1]);
    let mut carrier_index = 0;
    Ok(mesh.refit_surfaces(|carrier| {
        let layer = lames.map_or(0.0, |count| {
            usize::from(count).saturating_sub(carrier_index) as f32
                * gauge.thickness.metres()
                * LAME_SPACING_GAUGES
        });
        carrier_index += 1;
        for point in carrier {
            let mut p = local(frame, *point);
            // The sole's central fan vertex is an interior point, not part of
            // the enclosing perimeter. Keep it inside the fitted bottom ring.
            if p[1] < low + PROFILE_CLEARANCE_MARGIN_M && p[0].hypot(p[2]) < MINIMUM_RADIAL_EXTENT_M
            {
                continue;
            }
            let axial = ((p[1] - low) / (high - low) * (PROFILE_SAMPLES - 1) as f32)
                .clamp(0.0, (PROFILE_SAMPLES - 1) as f32);
            let index = (axial.floor() as usize).min(PROFILE_SAMPLES - 2);
            let fraction = axial - index as f32;
            let a = sections[index];
            let b = sections[index + 1];
            let center: [f32; 2] =
                std::array::from_fn(|i| a.center[i] + (b.center[i] - a.center[i]) * fraction);
            let radius: [f32; 2] = std::array::from_fn(|i| {
                a.radius[i] + (b.radius[i] - a.radius[i]) * fraction + layer
            });
            let delta = [p[0] - center[0], p[2] - center[1]];
            let exponent = foot_exponent(p[1], frame.half_extents[1]);
            let normalized = ((delta[0] / radius[0]).abs().powf(exponent)
                + (delta[1] / radius[1]).abs().powf(exponent))
            .powf(exponent.recip());
            if normalized < 1.0 && normalized > 0.001 {
                p[0] = center[0] + delta[0] / normalized;
                p[2] = center[1] + delta[1] / normalized;
                *point = frame.point(p);
            }
        }
    })?)
}

fn foot_sections(
    points: &[[f32; 3]],
    low: f32,
    high: f32,
    gauge: PlateGauge,
    foot_height: f32,
) -> Vec<FootSection> {
    let clearance =
        gauge.clearance.metres() + gauge.thickness.metres() + PROFILE_CLEARANCE_MARGIN_M;
    let mut sections = Vec::with_capacity(PROFILE_SAMPLES);
    for index in 0..PROFILE_SAMPLES {
        let height = low + (high - low) * index as f32 / (PROFILE_SAMPLES - 1) as f32;
        let nearest = points
            .iter()
            .map(|p| (p[1] - height).abs())
            .fold(f32::INFINITY, f32::min);
        let slice = points
            .iter()
            .filter(|p| (p[1] - height).abs() <= nearest + PROFILE_WINDOW_M)
            .collect::<Vec<_>>();
        let min: [f32; 2] =
            [0, 2].map(|axis| slice.iter().map(|p| p[axis]).fold(f32::INFINITY, f32::min));
        let max: [f32; 2] = [0, 2].map(|axis| {
            slice
                .iter()
                .map(|p| p[axis])
                .fold(f32::NEG_INFINITY, f32::max)
        });
        let center = std::array::from_fn(|axis| (min[axis] + max[axis]) * 0.5);
        let radius: [f32; 2] = std::array::from_fn(|axis| {
            ((max[axis] - min[axis]) * 0.5).max(MINIMUM_RADIAL_EXTENT_M)
        });
        let exponent = foot_exponent(height, foot_height);
        let enclosing = slice
            .iter()
            .map(|p| {
                (((p[0] - center[0]) / radius[0]).abs().powf(exponent)
                    + ((p[2] - center[1]) / radius[1]).abs().powf(exponent))
                .powf(exponent.recip())
            })
            .fold(1.0_f32, f32::max);
        sections.push(FootSection {
            center,
            radius: radius.map(|r| r * enclosing + clearance),
        });
    }
    // Smooth the measured contour without propagating the heel's widest slice
    // up the shaft, which would create a projecting shelf above the heel.
    let original = sections.clone();
    for (index, section) in sections.iter_mut().enumerate() {
        let a = original[index.saturating_sub(1)];
        let b = original[index];
        let c = original[(index + 1).min(PROFILE_SAMPLES - 1)];
        for axis in 0..2 {
            section.center[axis] = (a.center[axis] + 2.0 * b.center[axis] + c.center[axis]) * 0.25;
            section.radius[axis] = (a.radius[axis] + 2.0 * b.radius[axis] + c.radius[axis]) * 0.25;
        }
    }
    sections
}

fn foot_exponent(height: f32, foot_height: f32) -> f32 {
    2.0 + 2.0 * (1.0 - (height + foot_height) / (foot_height * 1.4)).clamp(0.0, 1.0)
}

fn local(frame: &PartFrame, p: [f32; 3]) -> [f32; 3] {
    frame.axes.map(|a| dot(subtract(p, frame.origin), a))
}
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}
fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    a.map(|v| v * s)
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn normalize(a: [f32; 3]) -> [f32; 3] {
    scale(a, dot(a, a).sqrt().recip())
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
