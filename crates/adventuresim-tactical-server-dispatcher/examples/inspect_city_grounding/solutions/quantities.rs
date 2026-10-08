//! Rough construction quantities; complete triangle intersections bound cut/fill.
use super::*;
use crate::footprint::{clip, height_in_triangle};
use adventuresim_building_generator::spatial_geometry::PositiveLength;

const PERIMETER_QUANTITY_PROBE_METRES: f32 = 1.0;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum QuantityError {
    Support {
        diagnostic: adventuresim_tactical_core::city_layout::grounding::SupportDiagnostic,
    },
    NonFiniteLocation {
        point: Vec2,
    },
    NonFiniteElevation {
        point: Vec2,
    },
}

pub(super) fn measure(
    plan: &CompoundSupportPlan,
    geographic: &[[Vec3; 3]],
    terrain_height: &impl Fn(Vec2) -> Option<f32>,
    assumed_wall_thickness: PositiveLength,
) -> Result<Value, QuantityError> {
    // The perimeter/volume integration retains its native metre arithmetic;
    // the shared positive leaf owns admission before this numerical kernel.
    let assumed_wall_thickness_metres = assumed_wall_thickness.metres();
    let mesh = plan
        .mesh()
        .map_err(|diagnostic| QuantityError::Support { diagnostic })?;
    let mut covered = 0.0;
    let mut fill = 0.0;
    let mut cut = 0.0;
    let mut maximum = (0.0_f32, Vec2::ZERO);
    let mut rejection = None;
    for indices in mesh.support_triangles() {
        let support = indices.map(|i| mesh.positions()[i as usize]);
        let clipper = [support[0].xz(), support[2].xz(), support[1].xz()];
        for source in geographic {
            let polygon = clip(source.map(|v| v.xz()).to_vec(), &clipper);
            let polygon = clip(polygon, &plan.reservation().corners());
            if polygon.len() < 3 {
                continue;
            }
            let differences: Vec<_> = polygon
                .iter()
                .map(|p| height_in_triangle(support, *p) - height_in_triangle(*source, *p))
                .collect();
            for (point, difference) in polygon.iter().zip(&differences) {
                if difference.abs() > maximum.0 {
                    maximum = (difference.abs(), *point);
                }
                if let Err(error) = plan.validate_displacement_at(
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::from_metres(
                        *point,
                    )
                    .ok_or(QuantityError::NonFiniteLocation { point: *point })?,
                    SupportElevation::from_metres(height_in_triangle(*source, *point))
                        .ok_or(QuantityError::NonFiniteElevation { point: *point })?,
                ) {
                    rejection.get_or_insert(error);
                }
            }
            covered += area(&polygon);
            fill += signed_volume(&polygon, &differences, 1.0);
            cut += signed_volume(&polygon, &differences, -1.0);
        }
    }
    let internal_faces = mesh
        .retaining_triangles()
        .iter()
        .map(|indices| {
            let [a, b, c] = indices.map(|i| mesh.positions()[i as usize]);
            f64::from((b - a).cross(c - a).length()) * 0.5
        })
        .sum::<f64>();
    let perimeter_faces = perimeter_faces(plan, terrain_height)?;
    Ok(json!({
        "covered_reservation_area_m2":covered,
        "required_reservation_area_m2":area(&plan.reservation().corners()),
        "fill_volume_m3":fill,"cut_volume_m3":cut,
        "maximum_absolute_displacement_m":maximum.0,"maximum_displacement_location_m":maximum.1,
        "grading_rejection":rejection,
        "internal_retaining_and_riser_face_area_m2":internal_faces,
        "perimeter_face_area_estimate_m2":perimeter_faces,
        "perimeter_quantity_probe_spacing_m":PERIMETER_QUANTITY_PROBE_METRES,
        "assumed_retaining_thickness_m":assumed_wall_thickness_metres,
        "masonry_volume_estimate_m3":perimeter_faces.map(|p|(internal_faces+p)*f64::from(assumed_wall_thickness_metres)),
        "quantity_scope":"Rough comparative quantities, not structural wall sizing or historical costs. Cut/fill integrates source/support triangle intersections; perimeter face area uses one-metre probes. Footings, cellar excavation, neighbouring levels and drainage are not represented. An absent perimeter estimate is not zero work.",
    }))
}

fn perimeter_faces(
    plan: &CompoundSupportPlan,
    height: &impl Fn(Vec2) -> Option<f32>,
) -> Result<Option<f64>, QuantityError> {
    let corners = plan.reservation().corners();
    // The perimeter/volume integration retains its native metre arithmetic;
    // the shared positive leaf owns admission before this numerical kernel.
    let assumed_wall_thickness_metres = assumed_wall_thickness.metres();
    let mesh = plan
        .mesh()
        .map_err(|diagnostic| QuantityError::Support { diagnostic })?;
    let mut total = 0.0;
    for i in 0..corners.len() {
        let a = corners[i];
        let b = corners[(i + 1) % corners.len()];
        let stations = (a.distance(b) / PERIMETER_QUANTITY_PROBE_METRES).ceil() as usize;
        let interval = f64::from(a.distance(b)) / stations as f64;
        let mut previous = None;
        for station in 0..=stations {
            let p = a.lerp(b, station as f32 / stations as f32);
            let Some(geographic) = height(p) else {
                return Ok(None);
            };
            let difference = mesh
                .elevations_at(
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::from_metres(p)
                        .ok_or(QuantityError::NonFiniteLocation { point: p })?,
                )
                .iter()
                .map(|h| (h.metres() - geographic).abs())
                .max_by(f32::total_cmp);
            let Some(difference) = difference else {
                return Ok(None);
            };
            if let Some(value) = previous {
                total += interval * f64::from(value + difference) * 0.5;
            }
            previous = Some(difference);
        }
    }
    Ok(Some(total))
}

fn area(polygon: &[Vec2]) -> f64 {
    if polygon.len() < 3 {
        return 0.0;
    }
    let origin = polygon[0].as_dvec2();
    polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
        .map(|(a, b)| ((*a).as_dvec2() - origin).perp_dot((*b).as_dvec2() - origin))
        .sum::<f64>()
        .abs()
        * 0.5
}

fn signed_volume(polygon: &[Vec2], differences: &[f32], sign: f32) -> f64 {
    let mut clipped = Vec::new();
    for i in 0..polygon.len() {
        let j = (i + 1) % polygon.len();
        let a = differences[i] * sign;
        let b = differences[j] * sign;
        if a >= 0.0 {
            clipped.push((polygon[i], a));
        }
        if (a >= 0.0) != (b >= 0.0) {
            clipped.push((polygon[i].lerp(polygon[j], a / (a - b)), 0.0));
        }
    }
    if clipped.len() < 3 {
        return 0.0;
    }
    (1..clipped.len() - 1)
        .map(|i| {
            let [a, b, c] = [clipped[0], clipped[i], clipped[i + 1]];
            area(&[a.0, b.0, c.0]) * f64::from(a.1 + b.1 + c.1) / 3.0
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use adventuresim_building_generator::spatial_geometry::PositiveLength;
    #[test]
    fn linear_cut_fill_is_split_at_the_zero_line_instead_of_cancelling_out() {
        let square = [Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y];
        assert!((signed_volume(&square, &[-1.0, 1.0, 1.0, -1.0], 1.0) - 0.25).abs() < 1e-9);
        assert!((signed_volume(&square, &[-1.0, 1.0, 1.0, -1.0], -1.0) - 0.25).abs() < 1e-9);
    }
}
