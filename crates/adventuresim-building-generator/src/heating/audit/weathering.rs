//! Check actual folded-sheet coverage independently of the construction records.
use super::*;

pub(super) fn audit(
    plan: &BuildingPlan,
    heating: &DomesticHeatingPlan,
    face: &RoofFace,
    shaft: SpatialBounds<Architectural>,
) -> Result<bool, crate::GenerationError> {
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
        return Ok(false);
    }
    let min = Vec2::new(shaft.min().metres().x, shaft.min().metres().z);
    let max = Vec2::new(shaft.max().metres().x, shaft.max().metres().z);
    let highest = [min, Vec2::new(min.x, max.y), max, Vec2::new(max.x, min.y)]
        .map(|p| super::super::placement::roof_height(face, p))
        .into_iter()
        .fold(f32::NEG_INFINITY, f32::max);
    let tops = upstands
        .iter()
        .map(|s| Ok(s.cuboid_bounds()?.max().metres().y))
        .collect::<Result<Vec<_>, crate::GenerationError>>()?;
    let top = tops[0];
    if top < highest + 0.15
        || top > shaft.max().metres().y - 0.3
        || tops
            .iter()
            .any(|y| (y - top).abs() > GEOMETRY_TOLERANCE_METRES)
    {
        return Ok(false);
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
            section(ends, outward, [-0.002, 0.004], [low - 0.01, top])?,
        )? || !covers(
            &counter,
            section(ends, outward, [-0.025, 0.012], [top, top + 0.003])?,
        )? || !covers(
            &counter,
            section(ends, outward, [0.008, 0.012], [top - 0.06, top])?,
        )? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn section(
    ends: [Vec2; 2],
    outward: Vec2,
    depth: [f32; 2],
    height: [f32; 2],
) -> Result<SpatialBounds<Architectural>, crate::GenerationError> {
    let p = ends[0] + outward * depth[0];
    let q = ends[1] + outward * depth[1];
    let min = p.min(q);
    let max = p.max(q);
    Ok(SpatialBounds::<Architectural>::from_metres(
        Vec3::new(min.x, height[0], min.y),
        Vec3::new(max.x, height[1], max.y),
    )?)
}

fn covers(
    solids: &[&ResolvedSolid],
    required: SpatialBounds<Architectural>,
) -> Result<bool, crate::GenerationError> {
    crate::geometry_index::try_any(solids.iter(), |s| {
        let actual = s.cuboid_bounds()?;
        Ok::<bool, crate::GenerationError>(
            (required.min().metres() - actual.min().metres()).min_element() >= -0.0001
                && (actual.max().metres() - required.max().metres()).min_element() >= -0.0001,
        )
    })
}

pub(super) fn continuous_pan(
    plan: &BuildingPlan,
    heating: &DomesticHeatingPlan,
    face: &RoofFace,
    shaft: SpatialBounds<Architectural>,
) -> Result<bool, crate::GenerationError> {
    let sheets = heating
        .roof
        .flashing
        .iter()
        .filter_map(|id| plan.resolved_geometry.solids.iter().find(|s| s.id == *id))
        .collect::<Vec<_>>();
    if sheets.len() != 4 {
        return Ok(false);
    }
    let normal = |s: &ResolvedSolid| {
        bevy::math::Quat::from_euler(
            bevy::math::EulerRot::YXZ,
            s.yaw_radians.radians(),
            s.crossfall_radians.radians(),
            s.longfall_radians.radians(),
        ) * Vec3::Y
    };
    let reference_normal = normal(sheets[0]);
    let downhill = Vec3::new(face.plane.normal.x, 0.0, face.plane.normal.z).normalize();
    if reference_normal.dot(downhill) <= 0.0 {
        return Ok(false);
    }
    for index in 0..4 {
        let sheet = sheets[index];
        let next = sheets[(index + 1) % 4];
        if normal(sheet).dot(reference_normal) < 0.99999
            || (sheet.centre.metres() - sheets[0].centre.metres())
                .dot(reference_normal)
                .abs()
                > 0.001
        {
            return Ok(false);
        }
        let Some(contact) = super::super::contact::measured(sheet, next)? else {
            return Ok(false);
        };
        // A contact inside the masonry does not close an outboard pan corner.
        if rect(contact).difference(&rect(shaft)).unsigned_area() < 0.001 {
            return Ok(false);
        }
    }
    Ok(true)
}
