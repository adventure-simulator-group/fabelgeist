//! Seat an independently suspended armpit disc in front of the torso and arm.
use crate::{
    armor_frames::{Side, Wearer},
    armor_layer::ArmorLayerSurface,
};
use adventuresim_armor_model::{PartFrame, PartMesh, SpaulderDesign, generate_besagew};
use anyhow::{Context, Result};

pub(crate) fn fit(
    design: &SpaulderDesign,
    wearer: &Wearer<'_>,
    side: Side,
    layers: &[ArmorLayerSurface<'_>],
) -> Result<PartMesh> {
    let d = design
        .besagew
        .as_ref()
        .context("besagew construction required")?;
    let mut mesh = crate::spaulder_fit::fit(design, wearer, side)?;
    let (name, sign) = match side {
        Side::Left => ("l_uparm", 1.0),
        Side::Right => ("r_uparm", -1.0),
    };
    let joint = wearer
        .joint_names
        .iter()
        .position(|n| n == name)
        .context("missing shoulder joint")?;
    let anchor = wearer.joints[joint];
    let angle = sign * f32::from(d.outward_tilt.0) / 1000.0;
    let normal = [angle.sin(), 0.0, angle.cos()];
    let transverse = [angle.cos(), 0.0, -angle.sin()];
    let mut center = [
        anchor[0] - sign * d.medial_offset.metres(),
        anchor[1] - d.shoulder_drop.metres(),
        anchor[2],
    ];
    let dot = |a: [f32; 3], b: [f32; 3]| (0..3).map(|i| a[i] * b[i]).sum::<f32>();
    let reach = d.radius.metres() * 1.5;
    let mut depth = f32::NEG_INFINITY;
    for (points, relief) in std::iter::once((wearer.positions, design.gauge.clearance.metres()))
        .chain(std::iter::once((
            mesh.positions.as_slice(),
            d.plate_clearance.metres(),
        )))
        .chain(
            layers
                .iter()
                .map(|l| (l.positions, l.relief.metres() + d.plate_clearance.metres())),
        )
    {
        for p in points {
            let delta = std::array::from_fn(|i| p[i] - center[i]);
            if dot(delta, transverse).abs() < reach && delta[1].abs() < reach {
                depth = depth.max(dot(delta, normal) + relief);
            }
        }
    }
    anyhow::ensure!(depth.is_finite(), "besagew has no anatomical support");
    for i in 0..3 {
        center[i] += normal[i] * (depth + design.gauge.thickness.metres());
    }
    let frame = PartFrame {
        origin: center,
        axes: [transverse, [0.0, 1.0, 0.0], normal],
        half_extents: [d.radius.metres(); 3],
    };
    mesh.append(generate_besagew(d, design.gauge)?.transformed(&frame));
    Ok(mesh)
}
