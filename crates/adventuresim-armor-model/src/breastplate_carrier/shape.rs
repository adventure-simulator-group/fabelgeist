//! Evaluate authored front and rear carrier shapes.

use super::*;

const WAIST_POINT_BLEND_HEIGHT: f32 = 0.45;
const RIDGE_FADE_START: f32 = 0.65;
const BACK_UPPER_SECTION_START: f32 = 1.20;
const BACK_UPPER_SECTION_BLEND: f32 = 0.08;
const BACK_UPPER_SECTION_POWER: f32 = 0.55;

pub(super) fn mapped_height(
    reference_y: f32,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> f32 {
    local(wearer.anchors.neck_base, wearer.frame)[1]
        + (reference_y - REFERENCE_RIG_NECK_HEIGHT) * wearer.y_scale * design.plate_length.unit()
}

pub(super) fn lateral_scale(reference_y: f32, wearer: Wearer<'_>) -> f32 {
    let shoulder_blend = smoothstep((reference_y - 1.355) / 0.100);
    wearer.x_scale * (1.0 - shoulder_blend) + wearer.shoulder_x_scale * shoulder_blend
}

pub(super) fn mapped_point(
    reference: [f32; 3],
    _rear: bool,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> [f32; 3] {
    let vertical_t = ((reference[1] - FRONT_HEIGHTS[0])
        / (REFERENCE_RIG_NECK_HEIGHT - FRONT_HEIGHTS[0]))
        .clamp(0.0, 1.0);
    let waist_scale = 1.0 + (design.waist_width.unit() - 1.0) * (1.0 - smoothstep(vertical_t));
    [
        wearer.lateral_origin + reference[0] * lateral_scale(reference[1], wearer) * waist_scale,
        mapped_height(reference[1], wearer, design),
        wearer.coronal_origin + (reference[2] - REFERENCE_SECTION_CENTER_DEPTH) * wearer.z_scale,
    ]
}

pub(super) fn front_top_y(u: f32, design: &BreastplateDesign) -> f32 {
    let depth = design.neck_depth.unit();
    let neck =
        FRONT_NECK_Y.map(|y| REFERENCE_RIG_NECK_HEIGHT - (REFERENCE_RIG_NECK_HEIGHT - y) * depth);
    let attach = REFERENCE_RIG_NECK_HEIGHT
        - (REFERENCE_RIG_NECK_HEIGHT - 1.390) * design.arm_opening_depth.unit();
    let a = u.abs();
    if a <= 0.5 {
        cubic(a, &NECK_U, &neck, true)
    } else {
        let s = 1.0 - ((a - 0.5) * std::f32::consts::PI).cos();
        neck[2] * (1.0 - s) + attach * s
    }
}

pub(super) fn back_top_y(u: f32, design: &BreastplateDesign) -> f32 {
    let depth = design.neck_depth.unit();
    let neck =
        BACK_NECK_Y.map(|y| REFERENCE_RIG_NECK_HEIGHT - (REFERENCE_RIG_NECK_HEIGHT - y) * depth);
    let attach = REFERENCE_RIG_NECK_HEIGHT
        - (REFERENCE_RIG_NECK_HEIGHT - 1.375) * design.arm_opening_depth.unit();
    let a = u.abs();
    if a <= 0.5 {
        cubic(a, &NECK_U, &neck, true)
    } else {
        let s = 1.0 - ((a - 0.5) * std::f32::consts::PI).cos();
        neck[2] * (1.0 - s) + attach * s
    }
}

pub(super) fn top_neck_scale(
    reference_y: f32,
    bottom: f32,
    top: f32,
    design: &BreastplateDesign,
) -> f32 {
    let blend = smoothstep((reference_y - bottom) / (top - bottom).max(1e-6));
    1.0 + (design.neck_width.unit() - 1.0) * blend
}

pub(super) fn waist_point_weight(theta: f32, design: &BreastplateDesign) -> f32 {
    (1.0 - theta.sin().abs() / design.profile.waist_point_width.unit()).max(0.0)
}

pub(super) fn front_raw(theta: f32, y: f32, design: &BreastplateDesign) -> [f32; 3] {
    let a = cubic(y, &FRONT_HEIGHTS, &FRONT_RADIUS_X, false);
    let b = cubic(y, &FRONT_HEIGHTS, &FRONT_RADIUS_Z, false);
    let phase = ((y - FRONT_HEIGHTS[0]) / (FRONT_NECK_Y[0] - FRONT_HEIGHTS[0])).clamp(0.0, 1.0);
    let peak = design.profile.projection_height.unit();
    let bell = if phase <= peak {
        smoothstep(phase / peak)
    } else {
        smoothstep((1.0 - phase) / (1.0 - peak))
    };
    const UPPER_CHEST_PHASE: f32 = 0.65;
    let upper_bell = smoothstep(phase / UPPER_CHEST_PHASE)
        * (1.0 - smoothstep((phase - UPPER_CHEST_PHASE) / (1.0 - UPPER_CHEST_PHASE)));
    let crown = design.profile.projection.metres() * bell
        - design.profile.upper_chest_recession.metres() * upper_bell
        + design.profile.waist_projection.metres() * (1.0 - smoothstep(phase));
    let ridge = design.profile.medial_ridge.metres()
        * (1.0 - smoothstep((phase - RIDGE_FADE_START) / (1.0 - RIDGE_FADE_START)));
    let fullness_power = 2.0 / design.profile.fullness.unit();
    let point_drop = design.profile.waist_point.metres()
        * (1.0 - smoothstep(phase / WAIST_POINT_BLEND_HEIGHT))
        * waist_point_weight(theta, design);
    let upper = smoothstep((y - 1.445) / 0.035);
    let recession = 0.050 * upper;
    let lift = 0.040 * upper;
    let sin = theta.sin();
    let cos = theta.cos();
    [
        a * sin,
        y - point_drop,
        b * cos + crown * cos.max(0.0).powf(fullness_power) + ridge * (1.0 - sin.abs()) - recession
            + lift * sin.powi(2),
    ]
}

pub(super) fn back_raw(
    theta: f32,
    y: f32,
    design: &BreastplateDesign,
) -> Result<[f32; 3], GenerateError> {
    let cos = theta.cos();
    if cos <= 0.0 {
        return Err(GenerateError::Degenerate);
    }
    let a = cubic(y, &BACK_HEIGHTS, &BACK_RADIUS_X, false);
    let depth_scale = design.back_depth.unit();
    let b = cubic(y, &BACK_HEIGHTS, &BACK_RADIUS_Z, false) * depth_scale;
    let upper = smoothstep((y - BACK_UPPER_SECTION_START) / BACK_UPPER_SECTION_BLEND);
    let power = 1.0 + (BACK_UPPER_SECTION_POWER - 1.0) * upper;
    Ok([a * theta.sin(), y, -b * cos.powf(power)])
}

pub(super) fn carrier_point(
    rear: bool,
    theta: f32,
    y: f32,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> Result<([f32; 3], [f32; 3]), GenerateError> {
    let raw = if rear {
        back_raw(theta, y, design)?
    } else {
        front_raw(theta, y, design)
    };
    let du = 0.0005;
    let dy = 0.0002;
    let raw_theta = if rear {
        back_raw(theta + du, y, design)?
    } else {
        front_raw(theta + du, y, design)
    };
    let raw_y = if rear {
        back_raw(theta, y + dy, design)?
    } else {
        front_raw(theta, y + dy, design)
    };
    let center = mapped_point(raw, rear, wearer, design);
    let tangent_theta = sub(mapped_point(raw_theta, rear, wearer, design), center);
    let tangent_y = sub(mapped_point(raw_y, rear, wearer, design), center);
    let mut normal = normalized(cross(tangent_theta, tangent_y))?;
    if rear {
        normal = scale(normal, -1.0);
    }
    let clearance = if rear {
        design.back_clearance.metres()
    } else {
        design.front_clearance.metres()
    };
    Ok((
        world(add(center, scale(normal, clearance)), wearer.frame),
        world(normal, wearer.frame),
    ))
}

fn front_limit_theta(u: f32, y: f32, design: &BreastplateDesign) -> f32 {
    let trim_y = 1.250 + (y - 1.250) * design.arm_opening_depth.unit();
    let limit =
        cubic(trim_y, &FRONT_TRIM_HEIGHTS, &FRONT_LIMIT_DEGREES, false) * design.side_return.unit();
    (limit * u * top_neck_scale(y, FRONT_HEIGHTS[0], REFERENCE_CARRIER_TOP_HEIGHT, design))
        .to_radians()
}

fn back_limit_theta(u: f32, y: f32, design: &BreastplateDesign) -> f32 {
    let trim_y = 1.240 + (y - 1.240) * design.arm_opening_depth.unit();
    let limit =
        cubic(trim_y, &BACK_TRIM_HEIGHTS, &BACK_LIMIT_DEGREES, false) * design.side_return.unit();
    const MAX_REAR_RETURN_DEGREES: f32 = 89.0;
    let edge = (limit * top_neck_scale(y, BACK_HEIGHTS[0], REFERENCE_CARRIER_TOP_HEIGHT, design))
        .min(MAX_REAR_RETURN_DEGREES);
    (edge * u).to_radians()
}

/// The upper armscye is an independent quarter-ellipse boundary. Blending its
/// lateral coordinate into the main chart avoids a hooked rim when the section
/// trim angle changes rapidly near the shoulder.
fn arm_cutout_theta(rear: bool, u: f32, y: f32, design: &BreastplateDesign) -> f32 {
    let angle = if rear {
        back_limit_theta
    } else {
        front_limit_theta
    };
    let original = angle(u, y, design);
    if u.abs() <= 0.5 {
        return original;
    }
    let top = if rear { back_top_y } else { front_top_y };
    let heights = if rear {
        &BACK_HEIGHTS[..]
    } else {
        &FRONT_HEIGHTS[..]
    };
    let radii = if rear {
        &BACK_RADIUS_X[..]
    } else {
        &FRONT_RADIUS_X[..]
    };
    let neck_y = top(0.5, design);
    let arm_y = top(1.0, design);
    let neck_x = cubic(neck_y, heights, radii, false) * angle(0.5, neck_y, design).sin();
    let arm_x = cubic(arm_y, heights, radii, false) * angle(1.0, arm_y, design).sin();
    let along = ((u.abs() - 0.5) * std::f32::consts::PI).sin();
    let radius = cubic(y, heights, radii, false);
    let boundary_y = top(u, design);
    let inner_x = radius * angle(0.5, y, design).sin();
    let boundary_inner_x =
        cubic(boundary_y, heights, radii, false) * angle(0.5, boundary_y, design).sin();
    // Extend the boundary through the interior while matching the unchanged
    // central chart at |u|=.5. The correction vanishes on the upper boundary.
    let x = neck_x + (arm_x - neck_x) * along + (inner_x - boundary_inner_x) * (1.0 - along);
    let boundary = (x / radius).clamp(0.0, 0.995).asin() * u.signum();
    let phase = ((y - heights[0]) / (top(u, design) - heights[0])).clamp(0.0, 1.0);
    let blend = smoothstep((phase - 0.75) / 0.25);
    original * (1.0 - blend) + boundary * blend
}

pub(super) fn front_theta(u: f32, y: f32, design: &BreastplateDesign) -> f32 {
    arm_cutout_theta(false, u, y, design)
}

pub(super) fn back_theta(u: f32, y: f32, design: &BreastplateDesign) -> f32 {
    arm_cutout_theta(true, u, y, design)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upper_recession_preserves_the_waist_and_neck_profile() {
        let mut design = BreastplateDesign::tapul();
        let before = design.clone();
        design.profile.upper_chest_recession.0 = 0;
        for height in [FRONT_HEIGHTS[0], FRONT_NECK_Y[0]] {
            assert_eq!(
                front_raw(0.0, height, &before),
                front_raw(0.0, height, &design)
            );
        }
        let upper = FRONT_HEIGHTS[0] + (FRONT_NECK_Y[0] - FRONT_HEIGHTS[0]) * 0.65;
        assert!(front_raw(0.0, upper, &before)[2] < front_raw(0.0, upper, &design)[2] - 0.03);
    }

    #[test]
    fn armscye_patch_meets_the_central_chart() {
        for mut design in [BreastplateDesign::default(), BreastplateDesign::peascod()] {
            for (neck, depth) in [(1000, 1100), (1100, 1200)] {
                design.neck_width = crate::Permille(neck);
                design.arm_opening_depth = crate::Permille(depth);
                for rear in [false, true] {
                    let top = if rear { back_top_y } else { front_top_y };
                    let bottom = if rear {
                        BACK_HEIGHTS[0]
                    } else {
                        FRONT_HEIGHTS[0]
                    };
                    for row in 0..=100 {
                        let y = bottom + (top(0.5, &design) - bottom) * row as f32 / 100.0;
                        let inner = arm_cutout_theta(rear, 0.5, y, &design);
                        let outer = arm_cutout_theta(rear, 0.500001, y, &design);
                        assert!(
                            (outer - inner).abs() < 1e-4,
                            "patch discontinuity: rear={rear}, y={y}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn cubic_interpolates_control_values() {
        for (x, y) in FRONT_HEIGHTS.into_iter().zip(FRONT_RADIUS_X) {
            assert!((cubic(x, &FRONT_HEIGHTS, &FRONT_RADIUS_X, false) - y).abs() < 1e-6);
        }
    }

    #[test]
    fn neckline_has_bilateral_symmetry_and_a_round_center() {
        let design = BreastplateDesign::default();
        assert_eq!(front_top_y(-0.37, &design), front_top_y(0.37, &design));
        assert!((front_top_y(0.001, &design) - front_top_y(0.0, &design)).abs() < 1e-5);
        assert!(front_top_y(0.5, &design) > front_top_y(0.0, &design));
    }
    #[test]
    fn rear_return_stays_in_its_chart_and_peascod_height_edits_are_continuous() {
        let mut design = BreastplateDesign::peascod();
        design.neck_width = crate::Permille(1100);
        design.side_return = crate::Permille(1080);
        for row in 0..=100 {
            let y = BACK_HEIGHTS[0] + (BACK_NECK_Y[0] - BACK_HEIGHTS[0]) * row as f32 / 100.0;
            assert!(back_raw(back_theta(1.0, y, &design), y, &design).is_ok());
        }
        let mut adjusted = design.clone();
        adjusted.profile.projection_height.0 += 1;
        for row in 0..=100 {
            let y = FRONT_HEIGHTS[0] + (FRONT_NECK_Y[0] - FRONT_HEIGHTS[0]) * row as f32 / 100.0;
            assert!(
                (front_raw(0.0, y, &design)[2] - front_raw(0.0, y, &adjusted)[2]).abs() < 0.001
            );
        }
        assert_eq!(
            front_raw(0.0, FRONT_HEIGHTS[0], &design),
            front_raw(0.0, FRONT_HEIGHTS[0], &adjusted)
        );
    }
}
