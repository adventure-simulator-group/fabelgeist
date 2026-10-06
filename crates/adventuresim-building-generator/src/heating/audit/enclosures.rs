//! Actual material must enclose the stove and cooking hood and retain the fire wall.
use super::*;
const SECTION_OFFSET_METRES: f32 = 0.003;

pub(super) fn audit(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    issues: &mut Vec<AuditIssue>,
) -> Result<(), crate::GenerationError> {
    let axis = h.kitchen_axis.vector();
    let centre = h.centre_metres.metres();
    let floor_height = h.floor_height_metres.metres();
    let passage = |kind| {
        h.passages
            .iter()
            .find(|p| p.kind == kind)
            .and_then(|p| plan.resolved_geometry.voids.iter().find(|v| v.id == p.void))
    };
    let ports = [
        HeatingPassageKind::StoveFirebox,
        HeatingPassageKind::StoveSmokeReturn,
    ]
    .into_iter()
    .filter_map(passage)
    .map(|v| v.bounds)
    .collect::<Vec<_>>();
    let normal_axis = if axis.x.abs() > 0.5 { 0 } else { 2 };
    let outward = if normal_axis == 0 { axis.x } else { axis.y };
    for kind in [
        HeatingPassageKind::StoveChamber,
        HeatingPassageKind::HearthMouth,
    ] {
        let Some(chamber) = passage(kind) else {
            continue;
        };
        for axis in 0..3 {
            for sign in [-1.0, 1.0] {
                if kind == HeatingPassageKind::HearthMouth && axis == normal_axis && sign == outward
                {
                    continue;
                }
                let at = if sign < 0.0 {
                    chamber.bounds.min().metres()[axis] - SECTION_OFFSET_METRES
                } else {
                    chamber.bounds.max().metres()[axis] + SECTION_OFFSET_METRES
                };
                let mut expected = geo::MultiPolygon::new(vec![project(chamber.bounds, axis)]);
                let throat = passage(HeatingPassageKind::HoodThroat).map(|v| v.bounds);
                for port in ports.iter().copied().chain(throat) {
                    if crosses(port, axis, at) {
                        expected = expected.difference(&project(port, axis));
                    }
                }
                let material = section(plan, h, axis, at)?;
                if expected.difference(&material).unsigned_area() > 0.00001 {
                    fail(
                        issues,
                        "open_domestic_appliance",
                        "actual material does not enclose the stove or hood outside its intended ports",
                    );
                }
            }
        }
    }
    let tangent = Vec2::new(-axis.y, axis.x);
    let half = tangent.abs() * 0.48 + axis.abs() * 0.15;
    let bounds = SpatialBounds::<Architectural>::from_metres(
        Vec3::new(centre.x - half.x, floor_height, centre.y - half.y),
        Vec3::new(
            centre.x + half.x,
            floor_height + super::super::placement::FIRE_WALL_PATCH_HEIGHT_METRES,
            centre.y + half.y,
        ),
    )?;
    let at = if normal_axis == 0 { centre.x } else { centre.y };
    let mut expected = geo::MultiPolygon::new(vec![project(bounds, normal_axis)]);
    for port in ports {
        if crosses(port, normal_axis, at) {
            expected = expected.difference(&project(port, normal_axis));
        }
    }
    let _: () = if expected
        .difference(&section(plan, h, normal_axis, at)?)
        .unsigned_area()
        > 0.00001
    {
        fail(
            issues,
            "incomplete_domestic_fire_wall",
            "the replaced partition patch lacks masonry outside the two fire ports",
        );
    };
    Ok(())
}
fn crosses(bounds: SpatialBounds<Architectural>, axis: usize, at: f32) -> bool {
    bounds.min().metres()[axis] < at && bounds.max().metres()[axis] > at
}
fn project(bounds: SpatialBounds<Architectural>, axis: usize) -> geo::Polygon<f32> {
    let axes = match axis {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    };
    geo::Rect::new(
        geo::coord! {x:bounds.min().metres()[axes[0]],y:bounds.min().metres()[axes[1]]},
        geo::coord! {x:bounds.max().metres()[axes[0]],y:bounds.max().metres()[axes[1]]},
    )
    .to_polygon()
}
fn section(
    plan: &BuildingPlan,
    h: &DomesticHeatingPlan,
    axis: usize,
    at: f32,
) -> Result<geo::MultiPolygon<f32>, crate::GenerationError> {
    let mut coverage = geo::MultiPolygon::new(vec![]);
    for part in h
        .parts
        .iter()
        .filter(|p| p.kind != HeatingPartKind::RoofFlashing)
    {
        if let Some(solid) = plan
            .resolved_geometry
            .solids
            .iter()
            .find(|s| s.id == part.solid)
        {
            let b = solid.cuboid_bounds()?;
            if crosses(b, axis, at) {
                coverage = coverage.union(&project(b, axis));
            }
        }
    }
    Ok(coverage)
}
