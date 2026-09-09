//! Long formed plates, joint cops and overlapping shoulder lames.

use std::f32::consts::{PI, TAU};

use super::mesh::{ALONG, AROUND, lerp, patch, smooth};
use super::{CuisseDesign, GreaveDesign, JointCupDesign, RerebraceDesign, SpaulderDesign};
use crate::{
    GenerateError,
    parametric::{PartFrame, PartMesh},
};

pub(super) fn greave(d: &GreaveDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let [width, length, depth] = fit.half_extents;
    let clearance = d.gauge.clearance.metres() + d.gauge.thickness.metres();
    // A narrow ankle, high calf belly, then a reduced knee throat.
    patch(AROUND, ALONG, true, d.gauge.thickness.metres(), |u, v| {
        let theta = TAU * u;
        let shape = if v < 0.66 {
            lerp(d.ankle_taper.unit(), 1.0, smooth(v / 0.66))
        } else {
            lerp(1.0, 0.88, smooth((v - 0.66) / 0.34))
        };
        let ridge = theta.cos().max(0.0).powi(8) * d.shin_ridge.metres();
        [
            (width * shape + clearance) * theta.sin(),
            (2.0 * v - 1.0) * length * d.length.unit(),
            (depth * shape + clearance) * theta.cos() + ridge,
        ]
    })
}

pub(super) fn cuisse(d: &CuisseDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let [width, length, depth] = fit.half_extents;
    let clearance = d.gauge.clearance.metres() + d.gauge.thickness.metres();
    patch(AROUND, ALONG, false, d.gauge.thickness.metres(), |u, v| {
        let theta = (2.0 * u - 1.0) * PI * d.wrap.unit();
        let taper = lerp(d.knee_taper.unit(), 1.0, smooth(v));
        // Lower front edge rises slightly at the sides to clear a flexing knee.
        let y = (2.0 * v - 1.0) * length * d.length.unit()
            + length * 0.09 * theta.sin().abs() * (1.0 - v).powi(3);
        [
            (width * taper + clearance) * theta.sin(),
            y,
            (depth * taper + clearance) * theta.cos(),
        ]
    })
}

pub(super) fn rerebrace(d: &RerebraceDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let [width, length, depth] = fit.half_extents;
    let clearance = d.gauge.clearance.metres() + d.gauge.thickness.metres();
    patch(AROUND, ALONG, false, d.gauge.thickness.metres(), |u, v| {
        let theta = (2.0 * u - 1.0) * PI * d.wrap.unit();
        let taper = lerp(d.distal_taper.unit(), 1.0, smooth(v));
        [
            (width * taper + clearance) * theta.sin(),
            (2.0 * v - 1.0) * length * d.length.unit() - length * 0.15,
            (depth * taper + clearance) * theta.cos(),
        ]
    })
}

pub(super) fn joint_cup(d: &JointCupDesign, fit: &PartFrame) -> Result<PartMesh, GenerateError> {
    let [width, length, depth] = fit.half_extents;
    const JOINT_EDGE_MARGIN_M: f32 = 0.004;
    let clearance = d.gauge.clearance.metres() + d.gauge.thickness.metres() + JOINT_EDGE_MARGIN_M;
    // One continuous formed sheet flows from the projecting joint dish into
    // the side fan. Axial crown and lateral fan spread are independent controls.
    patch(AROUND, ALONG, false, d.gauge.thickness.metres(), |u, v| {
        let basic_angle = lerp(-PI * 0.52, PI * (0.52 + d.wing.unit() * 0.35), u);
        let axial = 2.0 * v - 1.0;
        let fan = smooth(((basic_angle / PI - 0.20) / 0.38).clamp(0.0, 1.0));
        let theta = basic_angle - fan * PI * 0.13 * axial.powi(2);
        let crown = 1.0 + d.dome.unit() * 0.8 * (1.0 - axial * axial) * (1.0 - fan);
        [
            (width + clearance) * theta.sin() * (1.0 - 0.12 * axial.powi(2) * (1.0 - fan)),
            axial * (length + clearance) * d.length.unit() * lerp(0.72, 0.95 + d.wing.unit(), fan),
            (depth + clearance) * theta.cos() * crown,
        ]
    })
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
        mesh.append(patch(AROUND, 6, false, gauge, |u, v| {
            let axial = lerp(bottom, top, v);
            let theta = (2.0 * u - 1.0) * PI * 0.56;
            let crown = lerp(0.88, d.crown.unit(), smooth(axial));
            let step = gauge * 1.8 * lame as f32;
            [
                (width * crown + clearance + step) * theta.sin(),
                lerp(-1.65, -0.10, axial) * length * d.length.unit(),
                (depth * crown + clearance + step) * theta.cos(),
            ]
        })?);
    }
    // The crown returns toward the neck but ends at an open medial boundary.
    // A fan converging on the upper-arm axis would bury its tip in the clavicle.
    mesh.append(patch(AROUND, 10, false, gauge, |u, v| {
        let theta = (2.0 * u - 1.0) * PI * 0.5;
        let latitude = v * PI * 0.25;
        let step = gauge * 1.8 * lames as f32;
        [
            (width * d.crown.unit() + clearance + step) * theta.sin() * latitude.cos(),
            -length * 0.16 + length * 0.80 * latitude.sin(),
            (depth * d.crown.unit() + clearance + step) * theta.cos() * latitude.cos(),
        ]
    })?);
    Ok(mesh)
}
