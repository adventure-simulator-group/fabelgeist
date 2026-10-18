//! Cut faces close the gap between unchanged source and lower owned surfaces.
use super::*;
use crate::city_layout::grounding::boundaries;

pub(super) fn compile(
    plan: &PropertySupportSurface,
    source: &GeographicSurface,
) -> Result<Vec<[Vec3; 3]>, SupportDiagnostic> {
    let supports: Vec<_> = plan
        .mesh
        .support_triangles
        .iter()
        .map(|indices| {
            GroundTriangle::new(indices.map(|i| plan.mesh.positions[i as usize]))
                .ok_or_else(|| geometry_rejection(plan, SupportGeometryIssue::Topology))
        })
        .collect::<Result<_, _>>()?;
    let mut faces = Vec::new();
    for segment in boundaries::exterior(&plan.clipping_outlines) {
        let source_sections = sections(
            segment,
            source
                .intersecting_boundary_segment(
                    segment,
                    plan.limits.contact_tolerance_metres.metres(),
                )
                .map_err(|issue| geometry_rejection(plan, issue))?,
            plan.limits.contact_tolerance_metres.metres(),
        );
        let support_sections = sections(
            segment,
            supports.iter(),
            plan.limits.contact_tolerance_metres.metres(),
        );
        let mut stations = vec![0.0, 1.0];
        for (begin, end, _) in source_sections.iter().chain(&support_sections) {
            stations.extend([*begin, *end]);
        }
        stations.sort_by(f64::total_cmp);
        stations.dedup();
        for pair in stations.windows(2) {
            if pair[1] <= pair[0] {
                continue;
            }
            let midpoint = (pair[0] + pair[1]) * 0.5;
            let Some((_, _, natural)) = source_sections
                .iter()
                .find(|(begin, end, _)| *begin <= midpoint && midpoint <= *end)
            else {
                let uncovered = (segment[1] - segment[0]).length() * (pair[1] - pair[0]);
                if uncovered > f64::from(plan.limits.contact_tolerance_metres.metres()) {
                    return Err(plan.rejection(
                        SupportConstraint::BoundaryCoverage,
                        SupportBoundary::GeographicSurface,
                        segment[0].lerp(segment[1], midpoint).as_vec2(),
                        uncovered as f32,
                        plan.limits.contact_tolerance_metres.metres(),
                    ));
                }
                continue;
            };
            let Some((_, _, support)) = support_sections
                .iter()
                .filter(|(begin, end, _)| *begin <= midpoint && midpoint <= *end)
                .max_by(|a, b| {
                    a.2.height_f64(segment[0].lerp(segment[1], midpoint))
                        .total_cmp(&b.2.height_f64(segment[0].lerp(segment[1], midpoint)))
                })
            else {
                let uncovered = (segment[1] - segment[0]).length() * (pair[1] - pair[0]);
                if uncovered > f64::from(plan.limits.contact_tolerance_metres.metres()) {
                    return Err(plan.rejection(
                        SupportConstraint::BoundaryCoverage,
                        SupportBoundary::PropertyReservation,
                        segment[0].lerp(segment[1], midpoint).as_vec2(),
                        uncovered as f32,
                        plan.limits.contact_tolerance_metres.metres(),
                    ));
                }
                continue;
            };
            append_faces(segment, pair, natural, support, plan.limits, &mut faces);
        }
    }
    Ok(faces)
}

fn append_faces(
    segment: [bevy::math::DVec2; 2],
    stations: &[f64],
    natural: &GroundTriangle,
    support: &GroundTriangle,
    limits: SupportLimits,
    faces: &mut Vec<[Vec3; 3]>,
) {
    let mut points = [stations[0], stations[1]].map(|t| segment[0].lerp(segment[1], t));
    let difference = |point| natural.height_f64(point) - support.height_f64(point);
    let heights = points.map(difference);
    if heights.into_iter().fold(f64::NEG_INFINITY, f64::max)
        <= f64::from(limits.contact_tolerance_metres.metres())
    {
        return;
    }
    if heights[0] < 0.0 {
        points[0] = points[0].lerp(points[1], heights[0] / (heights[0] - heights[1]));
    }
    if heights[1] < 0.0 {
        points[1] = points[0].lerp(
            points[1],
            difference(points[0]) / (difference(points[0]) - heights[1]),
        );
    }
    let lower = points.map(|p| Vec3::new(p.x as f32, support.height_f64(p) as f32, p.y as f32));
    let upper = points.map(|p| Vec3::new(p.x as f32, natural.height_f64(p) as f32, p.y as f32));
    // The face points into the excavated owned region. Its opposite
    // side remains unchanged natural ground; no new walkable top is added.
    for triangle in [
        [lower[0], upper[1], upper[0]],
        [lower[0], lower[1], upper[1]],
    ] {
        if (triangle[1] - triangle[0])
            .cross(triangle[2] - triangle[0])
            .length_squared()
            > 0.0
        {
            faces.push(triangle);
        }
    }
}

fn sections<'a>(
    segment: [bevy::math::DVec2; 2],
    triangles: impl Iterator<Item = &'a GroundTriangle>,
    tolerance_metres: f32,
) -> Vec<(f64, f64, &'a GroundTriangle)> {
    triangles
        .filter_map(|triangle| {
            boundaries::segment_interval(
                segment,
                &triangle.points().map(|p| p.xz().as_dvec2()),
                f64::from(tolerance_metres),
            )
            .map(|(begin, end)| (begin, end, triangle))
        })
        .collect()
}
