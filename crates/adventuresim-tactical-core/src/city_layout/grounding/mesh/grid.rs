//! Disjoint support cells preserve vertical steps instead of interpolating them.
use super::*;

#[derive(Clone, Copy)]
struct SupportCell([Vec3; 4]);

pub(super) fn compile(plan: &CompoundSupportPlan) -> PropertySupportMesh {
    let mut mesh = super::empty(plan);
    let half = plan.property.plot.dimensions_metres * 0.5;
    let local = |point| {
        plan.property
            .plot
            .orientation
            .world_to_local(point - plan.property.plot.centre_metres)
    };
    let begin = local(plan.passage.start_metres).y;
    let sign = (local(plan.passage.end_metres).y - begin).signum();
    let mut xs = vec![-half.x, half.x, plan.split_frontage_metres];
    let mut zs = vec![-half.y, half.y];
    zs.extend(plan.court_profile.points.iter().map(|p| p.distance_metres));
    zs.extend(
        plan.passage_profile
            .points
            .iter()
            .map(|p| begin + sign * p.distance_metres),
    );
    for stair in &plan.court_stairs {
        let (x, z) = stair.local_cuts();
        xs.extend(x);
        zs.extend(z);
    }
    if let Some(entry) = &plan.street_entry {
        for corner in entry.support_region.corners() {
            let point = local(corner);
            xs.push(point.x);
            zs.push(point.y);
        }
    }
    bounded_cuts(&mut xs, half.x, plan.limits.contact_tolerance_metres);
    bounded_cuts(&mut zs, half.y, plan.limits.contact_tolerance_metres);
    let columns = xs.len() - 1;
    let mut cells = Vec::new();
    for z in zs.windows(2) {
        for x in xs.windows(2) {
            let centre = Vec2::new((x[0] + x[1]) * 0.5, (z[0] + z[1]) * 0.5);
            let point = plan.property.plot.centre_metres
                + plan.property.plot.orientation.local_to_world(centre);
            if plan
                .street_entry
                .as_ref()
                .is_some_and(|entry| entry.support_region.contains(point))
            {
                cells.push(None);
                continue;
            }
            let side = plan.property.boundary.gate.hinge.opposite().sign();
            let heights = if (centre.x - plan.split_frontage_metres) * side > 0.0 {
                [z[0], z[1]].map(|z| plan.passage_profile.height_at((z - begin) * sign))
            } else {
                [plan.main_height(centre); 2]
            };
            let points = [
                world(plan, x[0], z[0], heights[0]),
                world(plan, x[1], z[0], heights[0]),
                world(plan, x[1], z[1], heights[1]),
                world(plan, x[0], z[1], heights[1]),
            ];
            mesh.quad(points, false);
            cells.push(Some(SupportCell(points)));
        }
    }
    append_retaining_faces(&mut mesh, &cells, columns);
    append_street_approach(plan, &mut mesh, begin, sign, half.y);
    mesh
}

fn world(plan: &CompoundSupportPlan, x: f32, z: f32, height: f32) -> Vec3 {
    let p = plan.property.plot.centre_metres
        + plan
            .property
            .plot
            .orientation
            .local_to_world(Vec2::new(x, z));
    Vec3::new(p.x, height, p.y)
}

fn bounded_cuts(cuts: &mut Vec<f32>, half: f32, tolerance: f32) {
    cuts.retain(|v| *v >= -half && *v <= half);
    cuts.sort_by(f32::total_cmp);
    // Independently transformed landings can differ by float rounding. Their
    // declared contact tolerance prevents microscopic, ill-conditioned cells.
    cuts.dedup_by(|a, b| (*a - *b).abs() <= tolerance);
}

fn append_retaining_faces(
    mesh: &mut PropertySupportMesh,
    cells: &[Option<SupportCell>],
    columns: usize,
) {
    for (i, cell) in cells.iter().enumerate() {
        let Some(SupportCell(a)) = cell else { continue };
        let mut edges = Vec::new();
        if i % columns + 1 < columns
            && let Some(SupportCell(b)) = cells[i + 1]
        {
            edges.push([a[1], a[2], b[3], b[0]]);
        }
        if i + columns < cells.len()
            && let Some(SupportCell(b)) = cells[i + columns]
        {
            edges.push([a[3], a[2], b[1], b[0]]);
        }
        for edge in edges {
            if (edge[0].y - edge[3].y).abs() > mesh.contact_tolerance_metres
                || (edge[1].y - edge[2].y).abs() > mesh.contact_tolerance_metres
            {
                mesh.quad(edge, true);
            }
        }
    }
}

fn append_street_approach(
    plan: &CompoundSupportPlan,
    mesh: &mut PropertySupportMesh,
    begin: f32,
    sign: f32,
    half_depth: f32,
) {
    let boundary = begin.clamp(-half_depth, half_depth);
    let mut rows = vec![begin, boundary];
    let range = begin.min(boundary)..=begin.max(boundary);
    rows.extend(
        plan.passage_profile
            .points
            .iter()
            .map(|p| begin + sign * p.distance_metres)
            .filter(|z| range.contains(z)),
    );
    rows.sort_by(f32::total_cmp);
    rows.dedup_by(|a, b| (*a - *b).abs() <= plan.limits.contact_tolerance_metres);
    let centre = plan
        .property
        .plot
        .orientation
        .world_to_local(plan.passage.start_metres - plan.property.plot.centre_metres)
        .x;
    let half_width = plan.passage.half_width_metres;
    for band in rows.windows(2) {
        let heights =
            [band[0], band[1]].map(|z| plan.passage_profile.height_at((z - begin) * sign));
        mesh.quad(
            [
                world(plan, centre - half_width, band[0], heights[0]),
                world(plan, centre + half_width, band[0], heights[0]),
                world(plan, centre + half_width, band[1], heights[1]),
                world(plan, centre - half_width, band[1], heights[1]),
            ],
            false,
        );
    }
}
