//! Long formed plates, joint cops and overlapping shoulder lames.

use std::f32::consts::{PI, TAU};

use super::mesh::{ALONG, fluted_patch, lerp, smooth};
use super::{CuisseDesign, GreaveDesign, JointCupDesign, RerebraceDesign, SpaulderDesign};
use crate::{
    GenerateError,
    parametric::{PartFrame, PartMesh},
};

pub(super) fn greave(d: &GreaveDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let [width, length, depth] = fit.half_extents;
    let clearance = d.gauge.clearance.metres() + d.gauge.thickness.metres();
    // A narrow ankle, high calf belly, then a reduced knee throat.
    fluted_patch(
        ALONG,
        true,
        d.gauge.thickness.metres(),
        d.fluting.as_ref(),
        [0.0, 1.0],
        |u, _| (TAU * (u - 0.5)).cos().max(0.0).powi(8) * d.shin_ridge.metres(),
        |u, v| {
            let theta = TAU * (u - 0.5);
            let shape = if v < d.calf_height.unit() {
                lerp(d.ankle_taper.unit(), 1.0, smooth(v / d.calf_height.unit()))
            } else {
                lerp(
                    1.0,
                    d.knee_taper.unit(),
                    smooth((v - d.calf_height.unit()) / (1.0 - d.calf_height.unit())),
                )
            };
            [
                (width * shape + clearance) * theta.sin(),
                (2.0 * v - 1.0) * length * d.length.unit()
                    - d.ankle_extension.metres() * (1.0 - v) * (1.0 - theta.cos().max(0.0).powi(4)),
                (depth * shape + clearance) * theta.cos(),
            ]
        },
    )
}

pub(super) fn cuisse(d: &CuisseDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let [width, length, depth] = fit.half_extents;
    let clearance = d.gauge.clearance.metres() + d.gauge.thickness.metres();
    fluted_patch(
        ALONG,
        false,
        d.gauge.thickness.metres(),
        d.fluting.as_ref(),
        [0.0, 1.0],
        |u, v| {
            d.center_ridge.metres()
                * ((2.0 * u - 1.0) * PI * d.wrap.unit())
                    .cos()
                    .max(0.0)
                    .powi(8)
                * (PI * v).sin().powi(2)
        },
        |u, v| {
            let theta = (2.0 * u - 1.0) * PI * d.wrap.unit();
            let taper = lerp(d.knee_taper.unit(), 1.0, smooth(v));
            // Lower front edge rises slightly at the sides to clear a flexing knee.
            let y = (2.0 * v - 1.0) * length * d.length.unit()
                + length * 0.09 * theta.sin().abs() * (1.0 - v).powi(3)
                + d.upper_edge_slope.metres() * theta.sin() * v.powi(3);
            [
                (width * taper + clearance) * theta.sin(),
                y,
                (depth * taper + clearance) * theta.cos(),
            ]
        },
    )
}

pub(super) fn rerebrace(d: &RerebraceDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let [width, length, depth] = fit.half_extents;
    let clearance = d.gauge.clearance.metres() + d.gauge.thickness.metres();
    crate::plate_patch::fluted_radial_patch(
        ALONG,
        false,
        d.gauge.thickness.metres(),
        d.fluting.as_ref(),
        [0.0, 1.0],
        |u, v| {
            d.center_ridge.metres()
                * ((2.0 * u - 1.0) * PI * d.wrap.unit())
                    .cos()
                    .max(0.0)
                    .powi(8)
                * (PI * v).sin().powi(2)
        },
        |u, v| {
            let theta = (2.0 * u - 1.0) * PI * d.wrap.unit();
            let taper = lerp(d.distal_taper.unit(), 1.0, smooth(v));
            [
                (width * taper + clearance) * theta.sin(),
                (2.0 * v - 1.0) * length * d.length.unit() - length * 0.15,
                (depth * d.section_depth.unit() * taper + clearance) * theta.cos(),
            ]
        },
    )
}

pub(super) fn joint_cup(d: &JointCupDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let [width, length, depth] = fit.half_extents;
    const JOINT_EDGE_MARGIN_M: f32 = 0.004;
    let clearance = d.gauge.clearance.metres() + d.gauge.thickness.metres() + JOINT_EDGE_MARGIN_M;
    // One continuous formed sheet flows from the projecting joint dish into
    // the side fan. Axial crown and lateral fan spread are independent controls.
    crate::plate_patch::fluted_radial_patch(
        ALONG,
        false,
        d.gauge.thickness.metres(),
        d.fluting.as_ref(),
        [0.0, 1.0],
        |_, _| 0.0,
        |u, v| {
            let basic_angle = lerp(-PI * 0.52, PI * (0.52 + d.wing.unit() * 0.35), u);
            let axial = 2.0 * v - 1.0;
            let fan = smooth(((basic_angle / PI - 0.20) / 0.38).clamp(0.0, 1.0));
            let end_fraction = ((u - 0.72) / 0.28).clamp(0.0, 1.0);
            let rounded_tip = lerp(
                1.0,
                (1.0 - 0.96 * end_fraction.powi(2)).sqrt(),
                d.wing_roundness.unit(),
            );
            let theta = basic_angle;
            let distal_scale = lerp(1.0, d.distal_wing_scale.unit(), smooth((-axial).max(0.0)));
            let crown = 1.0 + d.dome.unit() * 0.8 * (1.0 - axial * axial) * (1.0 - fan);
            let section_width = (width + clearance) * (1.0 - 0.12 * axial * axial * (1.0 - fan));
            let section_depth = (depth + clearance) * crown;
            let radius = ((theta.sin() / section_width).powi(2)
                + (theta.cos() / section_depth).powi(2))
            .sqrt()
            .recip();
            // The fan grows beyond the enclosing cup. Its notch removes only
            // that extra plate reach, never the anatomical cup underneath it.
            let rim_flare = d.proximal_flare.metres() * axial.max(0.0).powi(2);
            let wing_reach = width
                * d.wing.unit()
                * fan
                * (1.0 - d.wing_notch.unit() * (1.0 - axial * axial).powi(2));
            [
                (radius + wing_reach + rim_flare) * theta.sin(),
                axial
                    * (length + clearance)
                    * d.length.unit()
                    * lerp(
                        0.72,
                        (0.95 + d.wing.unit()) * d.wing_height.unit() * distal_scale,
                        fan,
                    )
                    * rounded_tip,
                (radius + wing_reach + rim_flare) * theta.cos()
                    + d.center_ridge.metres()
                        * theta.cos().max(0.0).powi(8)
                        * (1.0 - axial * axial),
            ]
        },
    )
}

pub(super) fn spaulder(d: &SpaulderDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let [width, length, depth] = fit.half_extents;
    let gauge = d.gauge.thickness.metres();
    let clearance = d.gauge.clearance.metres() + gauge;
    let mut mesh = PartMesh::new();
    let lames = usize::from(d.lame_count);
    // The fitted z axis points over the deltoid, so the angular return reaches
    // both the anterior and posterior shoulder instead of facing only forward.
    for lame in 0..lames {
        let bottom = lame as f32 / lames as f32;
        let top = ((lame + 1) as f32 / lames as f32 + 0.055).min(1.0);
        mesh.append(crate::plate_patch::fluted_radial_patch(
            8,
            false,
            gauge,
            d.fluting.as_ref(),
            [bottom, top],
            |_, _| 0.0,
            |u, v| {
                let axial = lerp(bottom, top, v);
                let theta = lerp(
                    -PI * d.wrap.unit() * d.rear_extension.unit(),
                    PI * d.wrap.unit(),
                    u,
                );
                let crown = lerp(0.88, d.crown.unit(), smooth(axial));
                let section_depth = lerp(0.78, d.crown.unit(), smooth(axial));
                let step = gauge * 2.0 * (1.0 - v);
                let radius = ((theta.sin() / (width * crown + clearance)).powi(2)
                    + (theta.cos() / (depth * section_depth + clearance)).powi(2))
                .sqrt()
                .recip()
                    + step
                    + d.lower_flare.metres() * (1.0 - axial) * theta.sin().powi(2);
                [
                    radius * theta.sin(),
                    lerp(-1.65, -0.10, axial) * length * d.length.unit(),
                    radius * theta.cos(),
                ]
            },
        )?);
    }
    // A formed crown closes toward a shared apex above the shoulder.
    // Its neckward reach moves that apex without opening the plate.
    mesh.append(crate::plate_patch::fluted_crown_patch(
        12,
        [0.0, -length * 0.16, 0.0],
        gauge,
        d.fluting.as_ref(),
        // The upper return needs space above the clavicle and shoulder roll.
        |_, axial| 0.004 * ((axial - 0.5) * 2.0).powi(3),
        |u, v| {
            let theta = lerp(
                -PI * d.wrap.unit().min(0.60) * d.rear_extension.unit(),
                PI * d.wrap.unit().min(0.60),
                u,
            );
            let latitude = v * PI * 0.5;
            let step = gauge * 2.0;
            let radius = ((theta.sin() / (width * d.crown.unit() + clearance)).powi(2)
                + (theta.cos() / (depth * d.crown.unit() + clearance)).powi(2))
            .sqrt()
            .recip()
                + step;
            [
                radius * theta.sin() * latitude.cos(),
                -length * 0.16 + length * 1.40 * latitude.sin(),
                radius * theta.cos() * latitude.cos()
                    + depth * (1.10 - d.crown_reach.unit()) * (1.0 - latitude.cos()),
            ]
        },
    )?);
    Ok(mesh)
}
