//! Seat an independently suspended armpit disc in front of the torso and arm.
use crate::{
    armor_frames::{Side, Wearer},
    armor_layer::ArmorLayerSurface,
};
use adventuresim_armor_model::{PartFrame, PartMesh, SpaulderDesign, generate_besagew};
use anyhow::{Context, Result};
use bevy::math::{Vec2, Vec3};

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
    let mut mesh = crate::spaulder_fit::fit(design, wearer, side, layers)?;
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
    let disc = generate_besagew(d, design.gauge, wearer.detail)?;
    // The outer ring defines the exported polygon, including its runtime LOD.
    let mut outline = disc
        .positions
        .iter()
        .filter_map(|p| {
            let xy = Vec2::new(p[0], p[1]);
            ((xy.length() - d.radius.metres()).abs() < f32::EPSILON).then_some(xy)
        })
        .collect::<Vec<_>>();
    outline.sort_by(|a, b| a.y.atan2(a.x).total_cmp(&b.y.atan2(b.x)));
    outline.dedup();
    let mut depth = f32::NEG_INFINITY;
    for (points, faces, relief) in std::iter::once((
        wearer.positions,
        wearer.faces,
        design.gauge.clearance.metres(),
    ))
    .chain(std::iter::once((
        mesh.positions.as_slice(),
        mesh.indices.as_chunks::<3>().0,
        d.plate_clearance.metres(),
    )))
    .chain(layers.iter().map(|l| {
        (
            l.positions,
            l.faces,
            l.relief.metres() + d.plate_clearance.metres(),
        )
    })) {
        let triangles = faces.iter().map(|face| {
            face.map(|i| {
                let delta = std::array::from_fn(|axis| points[i as usize][axis] - center[axis]);
                Vec3::new(dot(delta, transverse), delta[1], dot(delta, normal))
            })
        });
        if let Some(support) = crate::besagew_support::depth(triangles, &outline) {
            depth = depth.max(support + relief);
        }
    }
    anyhow::ensure!(depth.is_finite(), "besagew has no anatomical support");
    for i in 0..3 {
        center[i] += normal[i] * (depth + design.gauge.thickness.metres());
    }
    let frame = PartFrame {
        detail: wearer.detail,
        origin: center,
        axes: [transverse, [0.0, 1.0, 0.0], normal],
        half_extents: [d.radius.metres(); 3],
    };
    mesh.append(disc.transformed(&frame));
    Ok(mesh)
}
