//! Fitted leather closures and their independent metal hardware.
use adventuresim_armor_model::{
    ArmorComponentMaterial, ArmorComponentRole, Millimeters, Milliradians, PartFrame, PartMesh,
    Permille,
};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

use crate::armor_frames::{FitRegion, Wearer};
const ANKLE_SUPPORT_SWEEP_M: f32 = 0.04;
const ELBOW_SUPPORT_SWEEP_M: f32 = 0.04;
/// Largest supported strap sweep, rounded down to a whole milliradian.
pub const FULL_TURN_MILLIRADIANS: u16 = 6283;
/// Minimum sweep with room to form a retention closure.
pub const MIN_STRAP_ARC_MILLIRADIANS: u16 = 500;
mod assembly;
pub mod catalog;
mod mesh;
mod section;
mod support;
pub mod suspension;
#[cfg(test)]
mod tests;

/// Dimensions are physical, independent of the size of the wearer.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrapDesign {
    pub width: Millimeters,
    pub thickness: Millimeters,
    pub count: u8,
    pub height: Permille,
    pub spacing: Permille,
    pub start_angle: Milliradians,
    pub end_angle: Milliradians,
    pub buckle_position: Permille,
    pub leather_color: [u8; 3],
    pub lining_clearance: Millimeters,
    pub underarm_drop: Millimeters,
}

impl StrapDesign {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            (8..=35).contains(&self.width.0),
            "strap width must be 8–35 mm"
        );
        ensure!(
            (1..=4).contains(&self.thickness.0),
            "strap thickness must be 1–4 mm"
        );
        ensure!(
            self.thickness.0 * 4 <= self.width.0,
            "leather gauge must not exceed one quarter of strap width"
        );
        ensure!((1..=3).contains(&self.count), "closure requires 1–3 straps");
        let spread = f32::from(self.count - 1) * self.spacing.unit() * 0.5;
        ensure!(
            self.height.unit() - spread >= 0.1 && self.height.unit() + spread <= 0.9,
            "straps must remain within the central 80% of the plate span"
        );
        ensure!(
            self.start_angle.0 <= FULL_TURN_MILLIRADIANS
                && self.start_angle.0 < self.end_angle.0
                && (MIN_STRAP_ARC_MILLIRADIANS..=FULL_TURN_MILLIRADIANS)
                    .contains(&(self.end_angle.0 - self.start_angle.0)),
            "strap arc must start in the first revolution and span 0.5 radians to one turn"
        );
        ensure!(
            (150..=850).contains(&self.buckle_position.0),
            "buckle must lie inside the strap arc"
        );
        ensure!(
            self.lining_clearance.0 <= 15 && self.underarm_drop.0 <= 60,
            "strap lining allowance is at most 15 mm and underarm drop at most 60 mm"
        );
        Ok(())
    }

    /// Place a measured polar angle on this unwrapped arc. Referencing its
    /// midpoint also keeps geometry just outside either endpoint on that end.
    fn arc_fraction(&self, angle: f32) -> f32 {
        let start = self.start_angle.radians();
        let span = self.end_angle.radians() - start;
        let middle = start + span * 0.5;
        let offset = (angle - middle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        ((middle + offset - start) / span).clamp(0.0, 1.0)
    }
}

impl Default for StrapDesign {
    fn default() -> Self {
        Self {
            width: Millimeters(18),
            thickness: Millimeters(2),
            count: 1,
            height: Permille(400),
            spacing: Permille(350),
            start_angle: Milliradians(1414),
            end_angle: Milliradians(4869),
            buckle_position: Permille(155),
            leather_color: [92, 34, 19],
            lining_clearance: Millimeters(6),
            underarm_drop: Millimeters(0),
        }
    }
}

/// Rear closure bands terminate on the anterior plate's lateral surfaces.
/// Convex cross-sections represent leather held in tension around the limb.
pub fn rear_closures(
    plate: &PartMesh,
    wearer: &Wearer<'_>,
    frame: &PartFrame,
    region: FitRegion,
    support: Option<&PartMesh>,
    design: &StrapDesign,
) -> Result<PartMesh> {
    design.validate()?;
    let body = section::local_points(wearer.positions, frame);
    let owned = wearer
        .support_indices(region)?
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let faces = wearer
        .faces
        .iter()
        .copied()
        .filter(|face| face.iter().any(|i| owned.contains(&(*i as usize))))
        .collect::<Vec<_>>();
    let metal = section::local_points(&plate.positions, frame);
    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;
    for vertex in support::attachment_faces(plate)?.iter().flatten() {
        let point = metal[*vertex as usize];
        lo = lo.min(point[1]);
        hi = hi.max(point[1]);
    }
    let support_mesh = support::local_mesh(plate, support, frame, region, design)?;
    let metal_faces = support_mesh.indices.as_chunks::<3>().0;
    let metal = support_mesh.positions;
    let mut leather = PartMesh::new();
    let mut buckles = PartMesh::new();
    for index in 0..design.count {
        let fraction = design.height.unit()
            + design.spacing.unit() * (f32::from(index) - f32::from(design.count - 1) * 0.5);
        let height = lo + (hi - lo) * fraction;
        let mut section = section::ClosureSection::new(
            &body,
            &faces,
            &metal,
            metal_faces,
            height - design.underarm_drop.metres() * 0.5,
            design.width.metres() * 1.4 + design.underarm_drop.metres(),
            design.lining_clearance.metres(),
        )?;
        if design.underarm_drop.0 > 0 {
            section.follow_underarm(
                section::SupportSurfaces {
                    body: &body,
                    body_faces: &faces,
                    plate: &metal,
                    plate_faces: metal_faces,
                },
                height,
                design,
            )?;
        }
        let support_sweep = match region {
            FitRegion::Foot(_) => ANKLE_SUPPORT_SWEEP_M,
            FitRegion::Elbow(_) => ELBOW_SUPPORT_SWEEP_M,
            _ => 0.0,
        };
        if support_sweep > 0.0
            && let Some(support) = support
        {
            section.include_support(
                &section::local_points(&support.positions, frame),
                support.indices.as_chunks::<3>().0,
                height,
                design.width.metres() * 1.4 + support_sweep,
            );
        }
        let (strap, hardware) = mesh::closure(&section, height, design)?;
        leather.append(strap);
        buckles.append(hardware);
    }
    let mut result = finish(leather, buckles, design.leather_color);
    if design.underarm_drop.0 > 0 {
        for p in &mut result.positions {
            let fraction = design.arc_fraction(p[0].atan2(p[2]));
            p[1] -= design.underarm_drop.metres() * (fraction * std::f32::consts::PI).sin();
        }
    }
    Ok(result.transformed(frame))
}

fn finish(leather: PartMesh, buckles: PartMesh, color: [u8; 3]) -> PartMesh {
    let mut result = leather.with_component(ArmorComponentRole::LeatherStraps, None);
    let [r, g, b] = color.map(|v| f32::from(v) / 255.0);
    result.components[0].material = Some(ArmorComponentMaterial {
        base_color: [r, g, b, 1.0],
        metallic: 0.0,
        roughness: 0.72,
    });
    result.append(buckles.with_component(ArmorComponentRole::Buckles, None));
    result.components[1].material = Some(ArmorComponentMaterial {
        base_color: [0.55, 0.53, 0.49, 1.0],
        metallic: 1.0,
        roughness: 0.32,
    });
    result
}
