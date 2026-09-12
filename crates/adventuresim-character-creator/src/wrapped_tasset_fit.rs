//! Independently measured thigh sections for long suspended tasset courses.
use crate::{
    armor_frames::{FitRegion, Side, Wearer},
    tasset_carrier::TassetCarrier,
};
use adventuresim_armor_model::{
    GarmentArmorDesign, PartMesh, TassetSide, TassetSpan, WrappedTassetDesign,
    generate_wrapped_tasset,
};
use anyhow::{Context, Result, ensure};

#[path = "tasset_layer_seat.rs"]
mod layer_seat;
use layer_seat::TassetLayerSeat;

const FIT_RESERVE_M: f32 = 0.004;

pub(crate) fn fit(
    design: &GarmentArmorDesign,
    shape: &WrappedTassetDesign,
    wearer: &Wearer<'_>,
    top: f32,
    layers: &[crate::armor_layer::ArmorLayerSurface<'_>],
) -> Result<PartMesh> {
    let mut mesh = PartMesh::new();
    let mut waist_indices = wearer.support_indices(FitRegion::Hips)?;
    waist_indices.extend(wearer.support_indices(FitRegion::Torso)?);
    let waist = waist_indices
        .into_iter()
        .map(|i| wearer.positions[i])
        .collect::<Vec<_>>();
    for (side, prefix) in [(Side::Left, "l"), (Side::Right, "r")] {
        let joint = |name: &str| -> Result<[f32; 8]> {
            let i = wearer
                .joint_names
                .iter()
                .position(|n| n == name)
                .with_context(|| format!("missing tasset landmark {name}"))?;
            Ok(wearer.joints[i])
        };
        let hip = joint(&format!("{prefix}_upleg"))?;
        let knee = joint(&format!("{prefix}_lowleg"))?;
        let bottom = hip[1] + (knee[1] - hip[1]) * shape.knee_reach.unit();
        let bottom = top + (bottom - top) * design.length.unit();
        ensure!(
            top > bottom,
            "tasset suspension must lie above its knee hem"
        );
        let indices = wearer.support_indices(FitRegion::Thigh(side))?;
        let points = indices
            .iter()
            .map(|i| wearer.positions[*i])
            .collect::<Vec<_>>();
        ensure!(!points.is_empty(), "tasset has no thigh support");
        let mut owned = vec![false; wearer.positions.len()];
        for region in [
            FitRegion::Thigh(side),
            FitRegion::LowerLeg(side),
            FitRegion::Hips,
            FitRegion::Torso,
        ] {
            for index in wearer.support_indices(region)? {
                owned[index] = true;
            }
        }
        let triangles: Vec<_> = wearer
            .faces
            .iter()
            .filter(|face| face.iter().any(|i| owned[*i as usize]))
            .map(|face| face.map(|i| wearer.positions[i as usize]))
            .collect();
        let carrier = TassetCarrier::new(&points, &waist, &triangles, bottom, top)?;
        let span = TassetSpan::new(bottom, top)?;
        let seat = TassetLayerSeat::new(layers, wearer, design, shape, bottom, top)?;
        let gap = design.clearance.metres() + design.wall_thickness.metres() + FIT_RESERVE_M;
        let tasset_side = match side {
            Side::Left => TassetSide::Left,
            Side::Right => TassetSide::Right,
        };
        let side_mesh = generate_wrapped_tasset(design, tasset_side, span, |angle, height| {
            let axial = span.axial(height);
            let flare = 1.0 + design.flare.unit() * (1.0 - axial);
            let direction = [angle.sin(), angle.cos()];
            let (center, radius) = carrier.sample(height, direction);
            let radius = radius * flare.max(1.0) + gap;
            let x = center[0] + radius * direction[0];
            let point = [x, height, center[1] + radius * direction[1]];
            let seated = seat.as_ref().map_or(point, |seat| seat.point(point));
            [seated[0], seated[2]]
        })?;
        ensure!(
            side_mesh
                .positions
                .iter()
                .all(|p| carrier.contains_height(p[1])),
            "shaped tasset exceeds its measured anatomical support span"
        );
        mesh.append(side_mesh);
    }
    Ok(mesh)
}
