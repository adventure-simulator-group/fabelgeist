//! Check actual folded-sheet coverage independently of the construction records.
use super::*;

pub(super) fn audit(
    plan: &BuildingPlan,
    heating: &DomesticHeatingPlan,
    face: &RoofFace,
    shaft: ResolvedBounds,
) -> bool {
    let sheets = |kind| {
        heating
            .parts
            .iter()
            .filter(|p| p.kind == kind)
            .filter_map(|p| {
                (p.material == BuildingLodMaterial::LeadAlloy)
                    .then(|| {
                        plan.resolved_geometry
                            .solids
                            .iter()
                            .find(|s| s.id == p.solid)
                    })
                    .flatten()
            })
            .collect::<Vec<_>>()
    };
    let upstands = sheets(HeatingPartKind::RoofUpstand);
    let counter = sheets(HeatingPartKind::RoofCounterFlashing);
    if upstands.len() != 4 || counter.len() != 8 {
        return false;
    }
    let min = Vec2::new(shaft.min.x, shaft.min.z);
    let max = Vec2::new(shaft.max.x, shaft.max.z);
    let highest = [min, Vec2::new(min.x, max.y), max, Vec2::new(max.x, min.y)]
        .map(|p| super::super::placement::roof_height(face, p))
        .into_iter()
        .fold(f32::NEG_INFINITY, f32::max);
    let tops = upstands
        .iter()
        .map(|s| s.cuboid_bounds().max.y)
        .collect::<Vec<_>>();
    let top = tops[0];
    if top < highest + 0.15
        || top > shaft.max.y - 0.3
        || tops
            .iter()
            .any(|y| (y - top).abs() > GEOMETRY_TOLERANCE_METRES)
    {
        return false;
    }
    for (start, end, outward) in [
        (min, Vec2::new(min.x, max.y), -Vec2::X),
        (Vec2::new(max.x, min.y), max, Vec2::X),
        (min, Vec2::new(max.x, min.y), -Vec2::Y),
        (Vec2::new(min.x, max.y), max, Vec2::Y),
    ] {
        let low = super::super::placement::roof_height(face, start)
            .min(super::super::placement::roof_height(face, end));
        // Require positive masonry engagement, apron contact and corner lap.
        let tangent = (end - start).normalize();
        let ends = [start - tangent * 0.002, end + tangent * 0.002];
        if !covers(
            &upstands,
            section(ends, outward, [-0.002, 0.004], [low - 0.01, top]),
        ) || !covers(
            &counter,
            section(ends, outward, [-0.025, 0.012], [top, top + 0.003]),
        ) || !covers(
            &counter,
            section(ends, outward, [0.008, 0.012], [top - 0.06, top]),
        ) {
            return false;
        }
    }
    true
}

fn section(ends: [Vec2; 2], outward: Vec2, depth: [f32; 2], height: [f32; 2]) -> ResolvedBounds {
    let p = ends[0] + outward * depth[0];
    let q = ends[1] + outward * depth[1];
    let min = p.min(q);
    let max = p.max(q);
    ResolvedBounds {
        min: Vec3::new(min.x, height[0], min.y),
        max: Vec3::new(max.x, height[1], max.y),
    }
}

fn covers(solids: &[&ResolvedSolid], required: ResolvedBounds) -> bool {
    solids.iter().any(|s| {
        let actual = s.cuboid_bounds();
        (required.min - actual.min).min_element() >= -0.0001
            && (actual.max - required.max).min_element() >= -0.0001
    })
}

pub(super) fn continuous_pan(
    plan: &BuildingPlan,
    heating: &DomesticHeatingPlan,
    face: &RoofFace,
    shaft: ResolvedBounds,
) -> bool {
    let sheets = heating
        .roof
        .flashing
        .iter()
        .filter_map(|id| plan.resolved_geometry.solids.iter().find(|s| s.id == *id))
        .collect::<Vec<_>>();
    if sheets.len() != 4 {
        return false;
    }
    let normal = |s: &ResolvedSolid| {
        bevy::math::Quat::from_euler(
            bevy::math::EulerRot::YXZ,
            s.yaw_radians,
            s.crossfall_radians,
            s.longfall_radians,
        ) * Vec3::Y
    };
    let reference_normal = normal(sheets[0]);
    let downhill = Vec3::new(face.plane.normal.x, 0.0, face.plane.normal.z).normalize();
    if reference_normal.dot(downhill) <= 0.0 {
        return false;
    }
    for index in 0..4 {
        let sheet = sheets[index];
        let next = sheets[(index + 1) % 4];
        if normal(sheet).dot(reference_normal) < 0.99999
            || (sheet.centre - sheets[0].centre)
                .dot(reference_normal)
                .abs()
                > 0.001
        {
            return false;
        }
        let Some(contact) = super::super::contact::measured(sheet, next) else {
            return false;
        };
        // A contact inside the masonry does not close an outboard pan corner.
        if rect(contact).difference(&rect(shaft)).unsigned_area() < 0.001 {
            return false;
        }
    }
    true
}
