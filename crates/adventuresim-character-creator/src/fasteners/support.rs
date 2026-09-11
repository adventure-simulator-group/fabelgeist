//! Plate attachment anchors and neighboring support for taut leather closures.
use super::{StrapDesign, section};
use crate::armor_frames::FitRegion;
use adventuresim_armor_model::{PartFrame, PartMesh};
use anyhow::{Result, ensure};

const ADJACENT_PLATE_ALLOWANCE_M: f32 = 0.002;
const ELBOW_ATTACHMENT_ALLOWANCE_M: f32 = 0.004;
const SHOULDER_ANCHOR_HALF_ANGLE: f32 = 0.15;

pub(super) fn local_mesh(
    plate: &PartMesh,
    support: Option<&PartMesh>,
    frame: &PartFrame,
    region: FitRegion,
    design: &StrapDesign,
) -> Result<PartMesh> {
    let mut metal = section::local_points(&plate.positions, frame);
    let mut metal_faces = plate.indices.as_chunks::<3>().0.to_vec();
    if design.underarm_drop.0 > 0 {
        validate_anchors(&metal, &metal_faces, design)?;
    }
    if let Some(support) = support {
        let offset = metal.len() as u32;
        let allowance = if matches!(region, FitRegion::Elbow(_)) {
            ELBOW_ATTACHMENT_ALLOWANCE_M
        } else {
            ADJACENT_PLATE_ALLOWANCE_M
        };
        metal.extend(
            section::local_points(&support.positions, frame)
                .into_iter()
                .map(|mut p| {
                    let radius = p[0].hypot(p[2]);
                    if radius > 0.0 {
                        let factor = (radius + allowance) / radius;
                        p[0] *= factor;
                        p[2] *= factor;
                    }
                    p
                }),
        );
        metal_faces.extend(
            support
                .indices
                .as_chunks::<3>()
                .0
                .iter()
                .map(|face| face.map(|i| i + offset)),
        );
    }
    let mut mesh = PartMesh::new();
    mesh.positions = metal;
    mesh.indices = metal_faces.into_iter().flatten().collect();
    Ok(mesh)
}

fn validate_anchors(points: &[[f32; 3]], faces: &[[u32; 3]], design: &StrapDesign) -> Result<()> {
    let low = points.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
    let high = points
        .iter()
        .map(|p| p[1])
        .fold(f32::NEG_INFINITY, f32::max);
    for band in 0..design.count {
        let fraction = design.height.unit()
            + design.spacing.unit() * (f32::from(band) - f32::from(design.count - 1) * 0.5);
        let height = low + (high - low) * fraction;
        for end in [design.start_angle.radians(), design.end_angle.radians()] {
            let supported = faces.iter().any(|face| {
                let p = face.map(|i| points[i as usize]);
                let near_end = p.iter().all(|p| {
                    let angle = p[0].atan2(p[2]) - end;
                    angle.sin().atan2(angle.cos()).abs() <= SHOULDER_ANCHOR_HALF_ANGLE
                });
                let ymin = p.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
                let ymax = p.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max);
                near_end
                    && ymin <= height + design.width.metres() * 0.7
                    && ymax >= height - design.width.metres() * 0.7
            });
            ensure!(
                supported,
                "shoulder strap endpoint misses its plate attachment"
            );
        }
    }
    Ok(())
}
