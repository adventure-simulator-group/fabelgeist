//! Check actual folded-sheet coverage independently of the construction records.
use super::super::weather_sections::{SheetSection, WeatherSide};
use super::*;
use crate::plan_geometry::ArchitecturalPlanPoint;

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
        .into_iter()
        .try_fold(f32::NEG_INFINITY, |high, p| {
            Ok::<_, crate::GenerationError>(
                high.max(
                    super::super::placement::roof_height(
                        face,
                        ArchitecturalPlanPoint::from_metres(p)?,
                    )?
                    .metres(),
                ),
            )
        })?;
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
    for side in WeatherSide::around(shaft)? {
        let [start, end] = side.ends.map(|point| point.metres());
        let low = super::super::placement::roof_height(
            face,
            ArchitecturalPlanPoint::from_metres(start)?,
        )?
        .metres()
        .min(
            super::super::placement::roof_height(face, ArchitecturalPlanPoint::from_metres(end)?)?
                .metres(),
        );
        // Require positive masonry engagement, apron contact and corner lap.
        let tangent = (end - start).normalize();
        let side = WeatherSide {
            ends: [
                ArchitecturalPlanPoint::from_metres(start - tangent * 0.002)?,
                ArchitecturalPlanPoint::from_metres(end + tangent * 0.002)?,
            ],
            outward: side.outward,
        };
        if !covers(
            &upstands,
            SheetSection::from_metres(-0.002, 0.004, low - 0.01, top)?.bounds(side)?,
        )? || !covers(
            &counter,
            SheetSection::from_metres(-0.025, 0.012, top, top + 0.003)?.bounds(side)?,
        )? || !covers(
            &counter,
            SheetSection::from_metres(0.008, 0.012, top - 0.06, top)?.bounds(side)?,
        )? {
            return Ok(false);
        }
    }
    Ok(true)
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
