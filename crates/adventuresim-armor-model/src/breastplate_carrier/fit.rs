//! Fit carrier sections to the wearer clearance enclosure.

use super::*;

pub(super) fn body_lateral_range(point: [f32; 3], wearer: Wearer<'_>) -> Option<(f32, f32)> {
    let target = local(point, wearer.frame);
    let mut minimum = f32::INFINITY;
    let mut maximum = f32::NEG_INFINITY;
    for face in wearer.torso_faces {
        let [a, b, c] = face.map(|index| {
            local(
                wearer.clearance.enclosure_vertices[index as usize].position,
                wearer.frame,
            )
        });
        let denominator = (b[1] - c[1]) * (a[2] - c[2]) + (c[2] - b[2]) * (a[1] - c[1]);
        if denominator.abs() <= 1e-10 {
            continue;
        }
        let u =
            ((b[1] - c[1]) * (target[2] - c[2]) + (c[2] - b[2]) * (target[1] - c[1])) / denominator;
        let v =
            ((c[1] - a[1]) * (target[2] - c[2]) + (a[2] - c[2]) * (target[1] - c[1])) / denominator;
        let w = 1.0 - u - v;
        if u >= -1e-5 && v >= -1e-5 && w >= -1e-5 {
            let lateral = u * a[0] + v * b[0] + w * c[0];
            minimum = minimum.min(lateral);
            maximum = maximum.max(lateral);
        }
    }
    minimum.is_finite().then_some((minimum, maximum))
}

pub(super) fn reference_height(
    point: [f32; 3],
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> f32 {
    let neck_y = local(wearer.anchors.neck_base, wearer.frame)[1];
    REFERENCE_RIG_NECK_HEIGHT
        + (local(point, wearer.frame)[1] - neck_y)
            / (wearer.y_scale * design.plate_length.unit()).max(1e-6)
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct FitProfile {
    center: [f32; 3],
    side: [f32; 3],
}

pub(super) fn height_weights(reference_y: f32, bottom: f32) -> [f32; 3] {
    let t = ((reference_y - bottom) / (REFERENCE_CARRIER_TOP_HEIGHT - bottom)).clamp(0.0, 1.0);
    [(1.0 - t).powi(2), 2.0 * t * (1.0 - t), t.powi(2)]
}

pub(super) fn weighted_value(weights: [f32; 3], controls: [f32; 3]) -> f32 {
    weights
        .into_iter()
        .zip(controls)
        .map(|(weight, control)| weight * control)
        .sum()
}

pub(super) fn torso_section_center(local_y: f32, wearer: Wearer<'_>) -> Option<[f32; 2]> {
    let mut minimum = [f32::INFINITY; 2];
    let mut maximum = [f32::NEG_INFINITY; 2];
    let mut crossings = 0usize;
    for face in wearer.torso_faces {
        let triangle = face.map(|index| {
            local(
                wearer.clearance.enclosure_vertices[index as usize].position,
                wearer.frame,
            )
        });
        for (a, b) in [
            (triangle[0], triangle[1]),
            (triangle[1], triangle[2]),
            (triangle[2], triangle[0]),
        ] {
            let da = a[1] - local_y;
            let db = b[1] - local_y;
            if (da < 0.0 && db < 0.0) || (da > 0.0 && db > 0.0) || (da - db).abs() <= 1e-9 {
                continue;
            }
            let t = da / (da - db);
            if !(-1e-5..=1.0 + 1e-5).contains(&t) {
                continue;
            }
            let crossing = [a[0] + (b[0] - a[0]) * t, a[2] + (b[2] - a[2]) * t];
            for axis in 0..2 {
                minimum[axis] = minimum[axis].min(crossing[axis]);
                maximum[axis] = maximum[axis].max(crossing[axis]);
            }
            crossings += 1;
        }
    }
    (crossings >= 4).then(|| {
        [
            (minimum[0] + maximum[0]) * 0.5,
            (minimum[1] + maximum[1]) * 0.5,
        ]
    })
}

pub(super) fn radial_direction(
    point: [f32; 3],
    wearer: Wearer<'_>,
) -> Option<([f32; 3], f32, f32)> {
    let local_point = local(point, wearer.frame);
    let center = torso_section_center(local_point[1], wearer)?;
    let delta = [local_point[0] - center[0], 0.0, local_point[2] - center[1]];
    let radius = length(delta);
    (radius > 1e-6).then(|| {
        let direction = scale(delta, radius.recip());
        let side_blend = smoothstep((direction[0].abs() - 0.45) / 0.45);
        (direction, radius, side_blend)
    })
}

pub(super) fn body_radial_extent(
    point: [f32; 3],
    direction: [f32; 3],
    wearer: Wearer<'_>,
) -> Option<f32> {
    let local_point = local(point, wearer.frame);
    let center = torso_section_center(local_point[1], wearer)?;
    let origin = [center[0], local_point[1], center[1]];
    let mut minimum = f32::INFINITY;
    for face in wearer.torso_faces {
        let [a, b, c] = face.map(|index| {
            local(
                wearer.clearance.enclosure_vertices[index as usize].position,
                wearer.frame,
            )
        });
        let edge_ab = sub(b, a);
        let edge_ac = sub(c, a);
        let h = cross(direction, edge_ac);
        let determinant = dot(edge_ab, h);
        if determinant.abs() <= 1e-9 {
            continue;
        }
        let inverse = determinant.recip();
        let from_a = sub(origin, a);
        let u = inverse * dot(from_a, h);
        if !(-1e-5..=1.0 + 1e-5).contains(&u) {
            continue;
        }
        let q = cross(from_a, edge_ab);
        let v = inverse * dot(direction, q);
        if v < -1e-5 || u + v > 1.0 + 1e-5 {
            continue;
        }
        let distance = inverse * dot(edge_ac, q);
        if distance >= 1e-6 {
            minimum = minimum.min(distance);
        }
    }
    minimum.is_finite().then_some(minimum)
}

pub(super) fn update_radial_profile(
    fit: &mut FitProfile,
    height_weights: [f32; 3],
    side_blend: f32,
    residual: f32,
) {
    let center_blend = 1.0 - side_blend;
    let denominator = height_weights
        .into_iter()
        .map(|weight| weight * weight * (center_blend * center_blend + side_blend * side_blend))
        .sum::<f32>();
    if residual > 0.0 && denominator > 1e-8 {
        for (index, weight) in height_weights.into_iter().enumerate() {
            fit.center[index] += residual * weight * center_blend / denominator;
            fit.side[index] += residual * weight * side_blend / denominator;
        }
    }
}

pub(super) fn apply_fit(
    position: [f32; 3],
    original: [f32; 3],
    rear: bool,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
    fit: FitProfile,
) -> [f32; 3] {
    let reference_y = reference_height(original, wearer, design);
    let bottom = if rear {
        BACK_HEIGHTS[0]
    } else {
        FRONT_HEIGHTS[0]
    };
    let weights = height_weights(reference_y, bottom);
    let Some((direction, _, side_blend)) = radial_direction(original, wearer) else {
        return position;
    };
    let offset = weighted_value(weights, fit.center) * (1.0 - side_blend)
        + weighted_value(weights, fit.side) * side_blend;
    world(
        add(local(position, wearer.frame), scale(direction, offset)),
        wearer.frame,
    )
}

pub(super) fn section_clearance_fit(
    mesh: &mut MidMesh,
    rear: bool,
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> Result<FitProfile, GenerateError> {
    let original = mesh.positions.clone();
    // Seating and section correction must preserve the requested inner room,
    // including where a lateral return approaches a mail-covered armpit.
    let clearance = if rear {
        design.back_clearance.metres()
    } else {
        design.front_clearance.metres()
    }
    .max(FIT_SURFACE_MARGIN);
    let bottom = if rear {
        BACK_HEIGHTS[0]
    } else {
        FRONT_HEIGHTS[0]
    };
    let mut fit = FitProfile::default();
    for iteration in 0..=24 {
        for (position, original) in mesh.positions.iter_mut().zip(&original) {
            *position = apply_fit(*original, *original, rear, wearer, design, fit);
        }
        let mut constraint = None::<(f32, [f32; 3], f32, [f32; 3], f32)>;
        for (position, original) in mesh.positions.iter().zip(&original) {
            let reference_y = reference_height(*original, wearer, design);
            let weights = height_weights(reference_y, bottom);
            let Some((direction, radius, side_blend)) = radial_direction(*position, wearer) else {
                continue;
            };
            if (rear && direction[2] >= 0.0) || (!rear && direction[2] <= 0.0) {
                continue;
            }
            let Some(body_radius) = body_radial_extent(*position, direction, wearer) else {
                continue;
            };
            let residual = body_radius + clearance - radius;
            if residual > 1e-5 && constraint.is_none_or(|current| residual > current.0) {
                constraint = Some((
                    residual,
                    weights,
                    side_blend,
                    local(*position, wearer.frame),
                    body_radius,
                ));
            }
        }
        let Some((residual, weights, side_blend, point, body_radius)) = constraint else {
            break;
        };
        if iteration == 24 {
            if report_fit() {
                eprintln!(
                    "breastplate nonconverged section_fit rear={rear} residual={residual} witness={point:?} body_radius={body_radius}"
                );
            }
            return Err(GenerateError::InvalidSurface);
        }
        update_radial_profile(&mut fit, weights, side_blend, residual);
        if report_fit() {
            eprintln!(
                "breastplate fit_iteration rear={rear} fit={fit:?} witness={point:?} body_radius={body_radius}"
            );
        }
    }
    // The carrier radii scale with torso depth; its admissible fitting travel
    // must use the same scale for full-strength identity export targets.
    if fit
        .center
        .into_iter()
        .chain(fit.side)
        .any(|value| !value.is_finite() || value > MAX_FIT_CORRECTION * wearer.z_scale)
    {
        if report_fit() {
            eprintln!("breastplate rejected section_fit rear={rear} fit={fit:?}");
        }
        return Err(GenerateError::InvalidSurface);
    }
    for (position, original) in mesh.positions.iter_mut().zip(&original) {
        *position = apply_fit(*original, *original, rear, wearer, design, fit);
    }
    Ok(fit)
}

pub(super) fn build_pair(
    wearer: Wearer<'_>,
    design: &BreastplateDesign,
) -> Result<(MidMesh, MidMesh), GenerateError> {
    if report_fit() {
        for reference_y in [1.040_f32, 1.120, 1.200, 1.280, 1.340, 1.400] {
            let probe = world(
                [
                    wearer.lateral_origin,
                    mapped_height(reference_y, wearer, design),
                    wearer.coronal_origin,
                ],
                wearer.frame,
            );
            eprintln!(
                "breastplate lateral_station y={reference_y} range={:?}",
                body_lateral_range(probe, wearer)
            );
        }
    }
    let mut smooth_design = design.clone();
    smooth_design.fluting = None;
    let mut front = build_mid(false, wearer, &smooth_design)?;
    let mut back = build_mid(true, wearer, design)?;
    let front_fit = section_clearance_fit(&mut front, false, wearer, design)?;
    let back_fit = section_clearance_fit(&mut back, true, wearer, design)?;
    if report_fit() {
        eprintln!("breastplate section_fit front={front_fit:?} back={back_fit:?}");
    }
    align_back_lap_width(&mut back, &front, wearer, design);
    // Restore the enclosure with a smooth profile after seating the lap.
    section_clearance_fit(&mut back, true, wearer, design)?;
    upper_extrusion(&mut front)?;
    front = refine_front(front, wearer, design)?;
    apply_fluting(&mut front, design)?;
    Ok((front, back))
}
