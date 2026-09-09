//! Mitten, articulated foot defenses and smooth leather footwear.

use std::f32::consts::{PI, TAU};

use super::mesh::{AROUND, boot_shell, half_dome, lerp, patch, smooth};
use super::{BootDesign, FootArmorDesign, GauntletDesign};
use crate::{
    GenerateError,
    parametric::{PartFrame, PartMesh},
};

pub(super) fn gauntlet(d: &GauntletDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let [width, length, depth] = fit.half_extents;
    let gauge = d.gauge.thickness.metres();
    let clearance = d.gauge.clearance.metres() + gauge;
    let mut mesh = patch(AROUND, 8, true, gauge, |u, v| {
        let theta = TAU * u;
        let radius = lerp(0.73, 0.73 * d.cuff_flare.unit(), smooth(v));
        [
            (width * radius + clearance) * theta.sin(),
            length * (1.0 + v * d.cuff_length.unit() * 2.0),
            (depth * lerp(1.0, 1.35, v) + clearance) * theta.cos(),
        ]
    })?;
    let count = usize::from(d.finger_lames);
    for lame in 0..count {
        let low = lame as f32 / count as f32;
        let high = ((lame + 1) as f32 / count as f32 + 0.045).min(1.0);
        mesh.append(patch(AROUND, 5, false, gauge, |u, v| {
            let axial = lerp(low, high, v);
            let theta = (2.0 * u - 1.0) * PI * 0.5;
            let taper = lerp(0.84, 1.0, smooth(axial));
            let step = gauge * 1.5 * lame as f32;
            [
                (width * taper + clearance + step) * theta.sin(),
                length * lerp(-0.81, 1.04, axial),
                (depth + clearance + step) * theta.cos(),
            ]
        })?);
    }
    let mut fingertips = half_dome(
        width * 0.84 + clearance,
        depth + clearance,
        0.0,
        length * 0.78,
        length * 1.10,
        gauge,
    )?;
    for point in &mut fingertips.positions {
        *point = [point[0], -point[2], point[1]];
    }
    mesh.append(fingertips);
    Ok(mesh)
}

/// The thumb has its own anatomical axis; it cannot be placed from hand width.
pub(super) fn gauntlet_thumb(
    d: &GauntletDesign,
    fit: &PartFrame,
) -> Result<PartMesh, GenerateError> {
    let [width, length, depth] = fit.half_extents;
    let gauge = d.gauge.thickness.metres();
    let clearance = d.gauge.clearance.metres() + gauge;
    let mut mesh = patch(24, 10, false, gauge, |u, v| {
        let theta = (2.0 * u - 1.0) * PI * 0.5;
        [
            (width + clearance) * theta.sin(),
            length * lerp(-0.68, 0.55, v),
            (depth + clearance) * theta.cos(),
        ]
    })?;
    let mut tip = half_dome(
        width + clearance,
        depth + clearance,
        0.0,
        length * 0.65,
        length + clearance,
        gauge,
    )?;
    for point in &mut tip.positions {
        *point = [point[0], -point[2], point[1]];
    }
    mesh.append(tip);
    Ok(mesh)
}

fn foot_width(t: f32, toe_width: f32) -> f32 {
    if t < 0.55 {
        lerp(0.63, 1.0, smooth(t / 0.55))
    } else {
        lerp(1.0, toe_width, smooth((t - 0.55) / 0.45))
    }
}

pub(super) fn sabaton(d: &FootArmorDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let [width, height, length] = fit.half_extents;
    let gauge = d.gauge.thickness.metres();
    let clearance = d.gauge.clearance.metres() + gauge;
    let mut mesh = PartMesh::new();
    let count = usize::from(d.lame_count);
    // Transverse instep lames descend from the ankle toward a rounded toe cap.
    for lame in 0..count {
        let low = lame as f32 / count as f32;
        let high = ((lame + 1) as f32 / count as f32 + 0.055).min(1.0);
        mesh.append(patch(AROUND, 4, false, gauge, |u, v| {
            let t = lerp(low, high, v);
            let theta = (0.5 - u) * PI;
            let step = gauge * 1.5 * (count - lame) as f32;
            let arch = height * lerp(1.75, 1.1, smooth(t));
            [
                (width * foot_width(t, d.toe_width.unit()) + clearance + step) * theta.sin(),
                -height + gauge + (arch + step) * theta.cos(),
                length * lerp(-0.05, 0.65, t),
            ]
        })?);
    }
    mesh.append(half_dome(
        width * d.toe_width.unit() + clearance,
        height * 1.1,
        -height + gauge,
        length * 0.60,
        length + d.toe_extension.metres(),
        gauge,
    )?);
    Ok(mesh)
}

pub(super) fn boot(d: &BootDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let [width, height, length] = fit.half_extents;
    let gauge = d.gauge.thickness.metres();
    let clearance = d.gauge.clearance.metres() + gauge;
    let ankle_z = -length * 0.45;
    let ankle_width = width * 0.69 + clearance;
    let ankle_depth = length * 0.35 + clearance;
    let upper_height = height * 1.65 + clearance + gauge;
    let total_height = upper_height + d.shaft_height.metres();
    // One continuous carrier from rounded sole perimeter through the vamp to
    // the shaft opening. Broad toe/heel quadrants enclose a foot's corners;
    // an ellipse drawn only through its axial extremes clips those corners.
    boot_shell(AROUND, 40, gauge, |u, v| {
        let theta = TAU * u;
        let toe_factor = lerp(0.72, d.toe_width.unit(), (theta.cos() + 1.0) * 0.5);
        let rounded = |value: f32| value.signum() * value.abs().powf(0.55);
        let outer_x = (width * toe_factor + clearance) * rounded(theta.sin());
        let outer_z = (length + clearance) * rounded(theta.cos());
        let inner_x = ankle_width * theta.sin();
        let inner_z = ankle_z + ankle_depth * theta.cos();
        let rise = total_height * v;
        let transition = smooth((rise / upper_height).min(1.0));
        let shaft = ((rise - upper_height) / d.shaft_height.metres()).max(0.0);
        let flare = lerp(1.0, d.shaft_flare.unit(), smooth(shaft));
        [
            lerp(outer_x, inner_x * flare, transition),
            -height - gauge + rise,
            lerp(outer_z, ankle_z + (inner_z - ankle_z) * flare, transition),
        ]
    })
}
