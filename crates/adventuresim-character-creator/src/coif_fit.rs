//! Coif cloth follows smooth chest/back sections while leaving shoulders open.

use adventuresim_armor_model::{
    CoifDesign, CoifDrapeProfile, CoifFlapDrape, CoifNeckDrape, PartFrame, PartMesh,
    generate_coif_with_drape,
};
use anyhow::{Context, Result, ensure};

use crate::armor_frames::{FitRegion, Wearer};

const SECTION_HALF_HEIGHT_M: f32 = 0.020;
const TRANSVERSE_BAND_HALF_WIDTH_M: f32 = 0.030;
const DRAPE_EASE_M: f32 = 0.004;
const MINIMUM_SECTION_POINTS: usize = 3;
const SIDE_NECK_BASE_RISE: f32 = 0.38;
const BACK_NECK_BASE_RISE: f32 = 0.24;
const NECK_WIDTH_SECTION_HALF_HEIGHT_M: f32 = 0.006;

pub fn fit(design: &CoifDesign, wearer: &Wearer<'_>) -> Result<PartMesh> {
    let frame = wearer.frame(FitRegion::Head)?;
    let mut support = wearer.support_indices(FitRegion::Torso)?;
    support.extend(wearer.support_indices(FitRegion::Neck)?);
    support.sort_unstable();
    support.dedup();
    let samples = support
        .iter()
        .map(|i| local(&frame, wearer.positions[*i]))
        .collect::<Vec<_>>();
    let gap = design.fit.clearance.metres() + design.fit.wall_thickness.metres() + DRAPE_EASE_M;
    let neck = neck_boundary(design, wearer, &frame, &samples, gap)?;
    let mut profile = CoifDrapeProfile::with_neck(design, neck);
    let half_width = profile.flap_half_width(design);
    fit_flap(
        &mut profile.front,
        &samples,
        half_width,
        gap,
        profile.neck.center_depth,
        Facing::Front,
    )?;
    fit_flap(
        &mut profile.back,
        &samples,
        half_width,
        gap,
        profile.neck.center_depth,
        Facing::Back,
    )?;
    Ok(generate_coif_with_drape(design, &frame, &profile)?)
}

fn neck_boundary(
    design: &CoifDesign,
    wearer: &Wearer<'_>,
    frame: &PartFrame,
    samples: &[[f32; 3]],
    gap: f32,
) -> Result<CoifNeckDrape> {
    let joint = |name: &str| -> Result<[f32; 3]> {
        let i = wearer
            .joint_names
            .iter()
            .position(|n| n == name)
            .with_context(|| format!("missing coif neck landmark {name}"))?;
        Ok(local(frame, std::array::from_fn(|a| wearer.joints[i][a])))
    };
    let base = joint("c_neck")?;
    let chin = joint("c_jaw_null")?;
    let span = joint("c_head")?[1] - base[1];
    ensure!(span > 0.0, "invalid coif neck landmarks");
    let front_height = chin[1] + (base[1] - chin[1]) * design.neck_coverage.unit();
    let side_height = front_height + span * SIDE_NECK_BASE_RISE;
    let back_height = front_height + span * BACK_NECK_BASE_RISE;
    let mut width = 0.0_f32;
    let mut count = 0;
    for point in samples
        .iter()
        .filter(|p| (p[1] - side_height).abs() < NECK_WIDTH_SECTION_HALF_HEIGHT_M)
    {
        width = width.max(point[0].abs());
        count += 1;
    }
    ensure!(
        count >= MINIMUM_SECTION_POINTS,
        "insufficient neck/shoulder junction support"
    );
    Ok(CoifNeckDrape {
        front_height,
        side_height,
        back_height,
        half_width: width + gap,
        center_depth: base[2],
        front_depth: section_depth(samples, front_height, 0.0, base[2], Facing::Front)? + gap,
        back_depth: section_depth(samples, back_height, 0.0, base[2], Facing::Back)? - gap,
    })
}

fn local(frame: &PartFrame, p: [f32; 3]) -> [f32; 3] {
    frame
        .axes
        .map(|axis| (0..3).map(|i| axis[i] * (p[i] - frame.origin[i])).sum())
}

#[derive(Clone, Copy)]
enum Facing {
    Front,
    Back,
}

impl Facing {
    fn sign(self) -> f32 {
        match self {
            Self::Front => 1.0,
            Self::Back => -1.0,
        }
    }
}

fn fit_flap(
    flap: &mut CoifFlapDrape,
    samples: &[[f32; 3]],
    width: f32,
    gap: f32,
    center_depth: f32,
    facing: Facing,
) -> Result<()> {
    for section in &mut flap.sections {
        section.center_depth = section_depth(samples, section.height, 0.0, center_depth, facing)?
            + facing.sign() * gap;
        section.edge_depth = section_depth(samples, section.height, width, center_depth, facing)?
            + facing.sign() * gap;
    }
    Ok(())
}

fn section_depth(
    samples: &[[f32; 3]],
    height: f32,
    across: f32,
    center_depth: f32,
    facing: Facing,
) -> Result<f32> {
    let mut depth = f32::NEG_INFINITY;
    let mut count = 0;
    for p in samples.iter().filter(|p| {
        (p[1] - height).abs() <= SECTION_HALF_HEIGHT_M
            && (p[0].abs() - across).abs() <= TRANSVERSE_BAND_HALF_WIDTH_M
            && (p[2] - center_depth) * facing.sign() > 0.0
    }) {
        depth = depth.max(p[2] * facing.sign());
        count += 1;
    }
    ensure!(
        count >= MINIMUM_SECTION_POINTS,
        "insufficient coif drape support at height {height}, width {across}"
    );
    Ok(depth * facing.sign())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transverse_sections_distinguish_spine_and_shoulder_depth() {
        let samples = [
            [0.0, 0.0, -0.10],
            [0.01, 0.0, -0.11],
            [-0.01, 0.0, -0.10],
            [0.06, 0.0, -0.13],
            [-0.06, 0.0, -0.12],
            [0.07, 0.0, -0.13],
        ];
        assert_eq!(
            section_depth(&samples, 0.0, 0.0, 0.0, Facing::Back).unwrap(),
            -0.11
        );
        assert_eq!(
            section_depth(&samples, 0.0, 0.06, 0.0, Facing::Back).unwrap(),
            -0.13
        );
    }

    #[test]
    fn missing_flap_support_is_an_error() {
        assert!(section_depth(&[], 0.0, 0.05, 0.0, Facing::Front).is_err());
        let front_only = [[0.04, 0.0, 0.05], [0.05, 0.0, 0.06], [0.06, 0.0, 0.05]];
        assert!(section_depth(&front_only, 0.0, 0.05, 0.0, Facing::Back).is_err());
    }

    #[test]
    fn body_center_selects_anterior_chest_behind_the_head_center() {
        let samples = [[0.04, 0.0, -0.01], [0.05, 0.0, -0.005], [0.06, 0.0, -0.012]];
        assert_eq!(
            section_depth(&samples, 0.0, 0.05, -0.04, Facing::Front).unwrap(),
            -0.005
        );
    }
}
