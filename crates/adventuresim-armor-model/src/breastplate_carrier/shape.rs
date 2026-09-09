//! Evaluate authored front and rear carrier shapes.

use super::*;

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
        let s = smoothstep((a - 0.5) * 2.0);
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
        let s = smoothstep((a - 0.5) * 2.0);
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

pub(super) fn front_raw(theta: f32, y: f32, design: &BreastplateDesign) -> [f32; 3] {
    let a = cubic(y, &FRONT_HEIGHTS, &FRONT_RADIUS_X, false);
    let b = cubic(y, &FRONT_HEIGHTS, &FRONT_RADIUS_Z, false);
    let phase = ((y - 1.06) / 0.39).clamp(0.0, 1.0);
    let crown = design.front_crown.metres() * (std::f32::consts::PI * phase).sin().powi(2);
    let upper = smoothstep((y - 1.445) / 0.035);
    let recession = 0.050 * upper;
    let lift = 0.040 * upper;
    let sin = theta.sin();
    let cos = theta.cos();
    [
        a * sin,
        y,
        b * cos + crown * cos.powi(4) - recession + lift * sin.powi(2),
    ]
}

pub(super) fn back_raw(theta: f32, y: f32) -> Result<[f32; 3], GenerateError> {
    let cos = theta.cos();
    if cos <= 0.0 {
        return Err(GenerateError::Degenerate);
    }
    let a = cubic(y, &BACK_HEIGHTS, &BACK_RADIUS_X, false);
    let b = cubic(y, &BACK_HEIGHTS, &BACK_RADIUS_Z, false);
    Ok([a * theta.sin(), y, -b * cos.powf(0.55)])
}

pub(super) fn carrier_point(
    rear: bool,
    theta: f32,
    y: f32,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> Result<([f32; 3], [f32; 3]), GenerateError> {
    let raw = if rear {
        back_raw(theta, y)?
    } else {
        front_raw(theta, y, design)
    };
    let du = 0.0005;
    let dy = 0.0002;
    let raw_theta = if rear {
        back_raw(theta + du, y)?
    } else {
        front_raw(theta + du, y, design)
    };
    let raw_y = if rear {
        back_raw(theta, y + dy)?
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

pub(super) fn front_theta(u: f32, y: f32, design: &BreastplateDesign) -> f32 {
    let trim_y = 1.250 + (y - 1.250) * design.arm_opening_depth.unit();
    let limit =
        cubic(trim_y, &FRONT_TRIM_HEIGHTS, &FRONT_LIMIT_DEGREES, false) * design.side_return.unit();
    (limit * u * top_neck_scale(y, FRONT_HEIGHTS[0], REFERENCE_CARRIER_TOP_HEIGHT, design))
        .to_radians()
}

pub(super) fn back_theta(u: f32, y: f32, design: &BreastplateDesign) -> f32 {
    let trim_y = 1.240 + (y - 1.240) * design.arm_opening_depth.unit();
    let limit =
        cubic(trim_y, &BACK_TRIM_HEIGHTS, &BACK_LIMIT_DEGREES, false) * design.side_return.unit();
    (limit * u * top_neck_scale(y, BACK_HEIGHTS[0], REFERENCE_CARRIER_TOP_HEIGHT, design))
        .to_radians()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
