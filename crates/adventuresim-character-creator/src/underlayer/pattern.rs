//! Anatomical garment volumes; Boolean cuts operate before normal standoff.
use super::*;
use crate::{
    armor_frames::{FitRegion, Side},
    surface_cut::{ConvexRegion, Plane, dot},
};
use anyhow::Context;

const TUBE_SIDES: usize = 8;
const GARMENT_HEM_DROP_M: f32 = 0.035;
const CUFF_CLEARANCE_M: f32 = 0.015;
const ARM_ROOT_OVERLAP_M: f32 = 0.035;
const SHOULDER_SEAM_OVERLAP_M: f32 = 0.055;
const COLLAR_ABOVE_NECK_M: f32 = 0.075;
const AXILLA_BELOW_SHOULDER_M: f32 = 0.105;
const HOSE_MEDIAL_GAP_M: f32 = 0.003;

#[path = "mail_regions.rs"]
mod mail_regions;

pub fn regions(
    design: &UnderlayerDesign,
    placement: &str,
    body: &Wearer<'_>,
) -> Result<(Vec<ConvexRegion>, Vec<ConvexRegion>)> {
    let joint = |name: &str| -> Result<[f32; 3]> {
        let i = body
            .joint_names
            .iter()
            .position(|n| n == name)
            .with_context(|| format!("missing underlayer landmark {name}"))?;
        Ok([body.joints[i][0], body.joints[i][1], body.joints[i][2]])
    };
    let include;
    let mut subtract = design
        .cuts
        .iter()
        .map(|cut| box_region(cut.minimum.0, cut.maximum.0))
        .collect::<Vec<_>>();
    match design.kind {
        UnderlayerKind::ArmingDoublet => include = doublet(design, body, &joint)?,
        UnderlayerKind::PaddedHose => {
            let (panels, cuts) = hose(design, placement, body, &joint)?;
            include = panels;
            subtract.extend(cuts);
        }
        UnderlayerKind::MailVoiders => include = voiders(design, &joint)?,
        UnderlayerKind::MailBrayette => include = mail_regions::brayette(design, body, &joint)?,
        UnderlayerKind::MailKneeVoider => {
            include = mail_regions::knee(design, placement, body, &joint)?
        }
        UnderlayerKind::MailStandard => include = mail_regions::standard(design, body, &joint)?,
    }
    Ok((include, subtract))
}

fn doublet(
    design: &UnderlayerDesign,
    body: &Wearer<'_>,
    joint: &impl Fn(&str) -> Result<[f32; 3]>,
) -> Result<Vec<ConvexRegion>> {
    let mut include = Vec::new();
    let hip = joint("c_spine0")?;
    let neck = joint("c_neck")?;
    let left = joint("l_uparm")?;
    let right = joint("r_uparm")?;
    let depth = body.frame(FitRegion::Torso)?.half_extents[2] * 2.0;
    let bottom = neck[1] - (neck[1] - hip[1]) * design.length.unit() - GARMENT_HEM_DROP_M;
    include.push(box_region(
        [right[0] - SHOULDER_SEAM_OVERLAP_M, bottom, hip[2] - depth],
        [
            left[0] + SHOULDER_SEAM_OVERLAP_M,
            neck[1] + COLLAR_ABOVE_NECK_M,
            hip[2] + depth,
        ],
    ));
    for (prefix, side) in [("l", Side::Left), ("r", Side::Right)] {
        let shoulder = joint(&format!("{prefix}_uparm"))?;
        let elbow = joint(&format!("{prefix}_lowarm"))?;
        let wrist = joint(&format!("{prefix}_wrist"))?;
        let frame = body.frame(FitRegion::WholeArm(side))?;
        let radius = frame.half_extents[0].max(frame.half_extents[2]) * 1.5;
        include.push(tube(
            shoulder,
            elbow,
            radius,
            ARM_ROOT_OVERLAP_M,
            ARM_ROOT_OVERLAP_M,
        ));
        let cuff =
            std::array::from_fn(|i| elbow[i] + (wrist[i] - elbow[i]) * design.sleeve_length.unit());
        include.push(tube(
            elbow,
            cuff,
            radius,
            ARM_ROOT_OVERLAP_M,
            -CUFF_CLEARANCE_M,
        ));
    }
    Ok(include)
}

fn hose(
    design: &UnderlayerDesign,
    placement: &str,
    body: &Wearer<'_>,
    joint: &impl Fn(&str) -> Result<[f32; 3]>,
) -> Result<(Vec<ConvexRegion>, Vec<ConvexRegion>)> {
    let mut include = Vec::new();
    let mut subtract = Vec::new();
    let side = Side::from_placement(placement)?;
    let prefix = if matches!(side, Side::Left) { "l" } else { "r" };
    let hip = joint(&format!("{prefix}_upleg"))?;
    let knee = joint(&format!("{prefix}_lowleg"))?;
    let ankle = joint(&format!("{prefix}_foot"))?;
    let ankle = std::array::from_fn(|i| knee[i] + (ankle[i] - knee[i]) * design.length.unit());
    let frame = body.frame(FitRegion::WholeLeg(side))?;
    let radius = frame.half_extents[0].max(frame.half_extents[2]) * 1.5;
    include.push(tube(
        hip,
        knee,
        radius,
        GARMENT_HEM_DROP_M,
        ARM_ROOT_OVERLAP_M,
    ));
    include.push(tube(
        knee,
        ankle,
        radius,
        ARM_ROOT_OVERLAP_M,
        -CUFF_CLEARANCE_M,
    ));
    let sign = if matches!(side, Side::Left) {
        1.0
    } else {
        -1.0
    };
    subtract.push(vec![Plane {
        normal: [sign, 0.0, 0.0],
        offset: HOSE_MEDIAL_GAP_M,
    }]);
    Ok((include, subtract))
}

fn voiders(
    design: &UnderlayerDesign,
    joint: &impl Fn(&str) -> Result<[f32; 3]>,
) -> Result<Vec<ConvexRegion>> {
    let mut include = Vec::new();
    let half = design.patch_width.metres() * 0.5;
    for (prefix, outward) in [("l", 1.0), ("r", -1.0)] {
        let shoulder = joint(&format!("{prefix}_uparm"))?;
        let elbow = joint(&format!("{prefix}_lowarm"))?;
        let axilla = [
            shoulder[0] + outward * half,
            shoulder[1] - AXILLA_BELOW_SHOULDER_M,
            shoulder[2],
        ];
        let axilla_half = half * 2.0;
        include.push(box_region(
            [
                axilla[0] - axilla_half,
                axilla[1] - axilla_half,
                axilla[2] - axilla_half * 2.0,
            ],
            [
                axilla[0] + axilla_half,
                axilla[1] + axilla_half,
                axilla[2] + axilla_half * 2.0,
            ],
        ));
        include.push(box_region(
            [elbow[0] - half, elbow[1] - half, elbow[2]],
            [elbow[0] + half, elbow[1] + half, elbow[2] + half * 2.0],
        ));
    }
    Ok(include)
}

fn box_region(minimum: [f32; 3], maximum: [f32; 3]) -> ConvexRegion {
    (0..3)
        .flat_map(|axis| {
            let mut normal = [0.0; 3];
            normal[axis] = 1.0;
            [
                Plane {
                    normal,
                    offset: maximum[axis],
                },
                Plane {
                    normal: normal.map(|v| -v),
                    offset: -minimum[axis],
                },
            ]
        })
        .collect()
}

fn tube(start: [f32; 3], end: [f32; 3], radius: f32, before: f32, after: f32) -> ConvexRegion {
    let axis = unit(std::array::from_fn(|i| end[i] - start[i]));
    let across = unit([axis[1], -axis[0], 0.0]);
    let other = [
        axis[1] * across[2] - axis[2] * across[1],
        axis[2] * across[0] - axis[0] * across[2],
        axis[0] * across[1] - axis[1] * across[0],
    ];
    let mut planes = vec![
        Plane {
            normal: axis.map(|v| -v),
            offset: -dot(axis, start) + before,
        },
        Plane {
            normal: axis,
            offset: dot(axis, end) + after,
        },
    ];
    for i in 0..TUBE_SIDES {
        let angle = std::f32::consts::TAU * i as f32 / TUBE_SIDES as f32;
        let n = std::array::from_fn(|j| across[j] * angle.cos() + other[j] * angle.sin());
        planes.push(Plane {
            normal: n,
            offset: dot(n, start) + radius,
        });
    }
    planes
}

fn unit(v: [f32; 3]) -> [f32; 3] {
    let length = dot(v, v).sqrt().max(f32::EPSILON);
    v.map(|x| x / length)
}
