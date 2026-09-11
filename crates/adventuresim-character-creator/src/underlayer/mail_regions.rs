//! Separate hip defense, rear-knee strips and standing neck mail.
use super::*;

const BRAYETTE_WAIST_ABOVE_PELVIS_M: f32 = 0.10;
const BRAYETTE_LEG_FRACTION: f32 = 0.30;
const STANDARD_COLLAR_HEIGHT_M: f32 = 0.065;
const STANDARD_BIB_DEPTH_M: f32 = 0.13;
const KNEE_STRIP_GAP_M: f32 = 0.024;
const KNEE_STRIP_LENGTH_M: f32 = 0.12;

pub(super) fn brayette(
    design: &UnderlayerDesign,
    body: &Wearer<'_>,
    joint: &impl Fn(&str) -> Result<[f32; 3]>,
) -> Result<Vec<ConvexRegion>> {
    let pelvis = joint("c_spine0")?;
    let hip = joint("l_upleg")?;
    let knee = joint("l_lowleg")?;
    let top = pelvis[1] + BRAYETTE_WAIST_ABOVE_PELVIS_M;
    let hem = hip[1] - (hip[1] - knee[1]) * BRAYETTE_LEG_FRACTION;
    let bottom = top - (top - hem) * design.length.unit();
    let extent = body.frame(FitRegion::Hips)?.half_extents;
    // A continuous pelvic surface supplies the crotch bridge and both leg
    // openings, rather than two disconnected thigh sleeves.
    Ok(vec![box_region(
        [
            pelvis[0] - extent[0] * 2.,
            bottom,
            pelvis[2] - extent[2] * 2.,
        ],
        [pelvis[0] + extent[0] * 2., top, pelvis[2] + extent[2] * 2.],
    )])
}

pub(super) fn knee(
    design: &UnderlayerDesign,
    placement: &str,
    body: &Wearer<'_>,
    joint: &impl Fn(&str) -> Result<[f32; 3]>,
) -> Result<Vec<ConvexRegion>> {
    let side = Side::from_placement(placement)?;
    let prefix = if matches!(side, Side::Left) { "l" } else { "r" };
    let knee = joint(&format!("{prefix}_lowleg"))?;
    let frame = body.frame(FitRegion::Knee(side))?;
    let half_height = KNEE_STRIP_LENGTH_M * design.length.unit() * 0.5;
    let half_width = design.patch_width.metres() * 0.5;
    let rear_depth = frame.half_extents[2] * 2.;
    // The A 6147 cutting pattern places narrow longitudinal mail strips
    // alongside one another across the knee flexion zone.
    Ok([
        [-half_width, -KNEE_STRIP_GAP_M * 0.5],
        [KNEE_STRIP_GAP_M * 0.5, half_width],
    ]
    .map(|[low, high]| {
        box_region(
            [knee[0] + low, knee[1] - half_height, knee[2] - rear_depth],
            [knee[0] + high, knee[1] + half_height, knee[2]],
        )
    })
    .to_vec())
}

pub(super) fn standard(
    design: &UnderlayerDesign,
    body: &Wearer<'_>,
    joint: &impl Fn(&str) -> Result<[f32; 3]>,
) -> Result<Vec<ConvexRegion>> {
    let neck = joint("c_neck")?;
    let frame = body.frame(FitRegion::Neck)?;
    let half_width = frame.half_extents[0] + design.patch_width.metres() * 0.5;
    let depth = frame.half_extents[2] * 2.;
    let mut region = box_region(
        [
            neck[0] - half_width,
            neck[1] - STANDARD_BIB_DEPTH_M * design.length.unit(),
            neck[2] - depth,
        ],
        [
            neck[0] + half_width,
            neck[1] + STANDARD_COLLAR_HEIGHT_M * design.length.unit(),
            neck[2] + depth,
        ],
    );
    let drop = STANDARD_BIB_DEPTH_M * design.length.unit();
    for sign in [-1., 1.] {
        region.push(Plane {
            normal: [sign * drop / half_width, -1., 0.],
            offset: drop - neck[1] + sign * drop / half_width * neck[0],
        });
    }
    Ok(vec![region])
}
