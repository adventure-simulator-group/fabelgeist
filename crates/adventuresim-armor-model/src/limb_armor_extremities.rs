//! Mitten, articulated foot defenses and smooth leather footwear.

use std::f32::consts::{PI, TAU};

use super::mesh::{AROUND, boot_shell, half_dome, lerp, smooth};
use super::{BootDesign, FootArmorDesign, GauntletDesign};
use crate::{
    GenerateError,
    parametric::{PartFrame, PartMesh},
};

pub(super) fn gauntlet(d: &GauntletDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let [width, length, depth] = fit.half_extents;
    let gauge = d.gauge.thickness.metres();
    let clearance = d.gauge.clearance.metres() + gauge;
    let mut mesh = crate::plate_patch::fluted_radial_patch(
        12,
        true,
        gauge,
        d.fluting.as_ref(),
        [0.0, 1.0],
        |_, _| gauge * 2.0,
        |u, v| {
            let theta = TAU * (u - 0.5);
            let radius = lerp(0.73, 0.73 * d.cuff_flare.unit(), smooth(v));
            [
                (width * radius + clearance + d.cuff_clearance.metres()) * theta.sin(),
                length * (0.94 + v * d.cuff_length.unit() * 2.0),
                (depth * lerp(1.0, 1.35, v) + clearance + d.cuff_clearance.metres()) * theta.cos(),
            ]
        },
    )?;
    let count = usize::from(d.finger_lames);
    for lame in 0..count {
        let low = lame as f32 / count as f32;
        let high = ((lame + 1) as f32 / count as f32 + 0.045).min(1.0);
        let point = |u: f32, v: f32| {
            let axial = lerp(low, high, v);
            let theta = (2.0 * u - 1.0) * PI * 0.5;
            let taper = if axial < 0.55 {
                lerp(0.84, d.knuckle_width.unit(), smooth(axial / 0.55))
            } else {
                lerp(d.knuckle_width.unit(), 0.73, smooth((axial - 0.55) / 0.45))
            };
            [
                (width * taper + clearance) * theta.sin(),
                length * lerp(-0.81, 1.04, axial),
                (depth + clearance) * theta.cos()
                    + d.knuckle_ridge.metres()
                        * (1.0 - ((axial - 0.55) / 0.18).abs()).max(0.0).powi(2)
                        * theta.cos().max(0.0),
            ]
        };
        mesh.append(if lame == 0 {
            crate::plate_patch::capped_fluted_patch(
                8,
                gauge,
                d.fluting.as_ref(),
                [1.0 - low, 1.0 - high],
                point,
                length * 0.29,
                |_, axial| gauge * 2.0 * (axial - (1.0 - high)) / (high - low),
            )?
        } else {
            crate::plate_patch::fluted_radial_patch(
                8,
                false,
                gauge,
                d.fluting.as_ref(),
                [1.0 - low, 1.0 - high],
                |_, axial| gauge * 2.0 * (axial - (1.0 - high)) / (high - low),
                point,
            )?
        });
    }
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
    crate::plate_patch::capped_fluted_patch(
        10,
        gauge,
        None,
        [0.0, 1.0],
        |u, v| {
            // Leave the palm-facing boundary beneath the mitten side plates.
            let theta = (2.0 * u - 1.0) * PI * 0.30;
            // The distal thumb plate ends before the mitten's metacarpal coverage.
            [
                (width + clearance) * theta.sin(),
                length * lerp(-0.68, 0.30, v),
                (depth + clearance) * theta.cos(),
            ]
        },
        length * 0.48 + clearance,
        |_, _| 0.0,
    )
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
    let available_span_m = length * 0.70;
    // Compare the required foot length so an exactly consumed span does not
    // slip through when multiplying the rounded length back by 0.70.
    if length <= d.ankle_cutaway.metres() / 0.70 {
        return Err(GenerateError::SabatonTrimExceedsFoot {
            cutaway_m: d.ankle_cutaway.metres(),
            available_span_m,
        });
    }
    let gauge = d.gauge.thickness.metres();
    let clearance = d.gauge.clearance.metres() + gauge;
    let mut mesh = PartMesh::new();
    let count = usize::from(d.lame_count);
    // Transverse instep lames descend from the ankle toward a rounded toe cap.
    for lame in 0..count {
        let low = lame as f32 / count as f32;
        let high = ((lame + 1) as f32 / count as f32 + 0.080).min(1.0);
        mesh.append(crate::plate_patch::fluted_foot_patch(
            8,
            -height + gauge,
            gauge,
            d.fluting.as_ref(),
            [low, high],
            |_, axial| gauge * 2.0 * (axial - low) / (high - low),
            |u, v| {
                let t = lerp(low, high, v);
                let theta = (0.5 - u) * PI;

                let arch = height * d.instep_height.unit() * lerp(1.75, 1.1, smooth(t));
                [
                    (width * foot_width(t, d.toe_width.unit()) + clearance) * theta.sin(),
                    -height + gauge + arch * theta.cos(),
                    lerp(-length * 0.05 + d.ankle_cutaway.metres(), length * 0.65, t),
                ]
            },
        )?);
    }
    mesh.append(half_dome(
        width * d.toe_width.unit() + clearance,
        height * 1.1 * d.instep_height.unit(),
        -height + gauge,
        length * 0.60,
        length + d.toe_extension.metres(),
        gauge,
        d.toe_roundness.unit(),
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
