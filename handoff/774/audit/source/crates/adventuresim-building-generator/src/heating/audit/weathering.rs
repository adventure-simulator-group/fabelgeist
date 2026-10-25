//! Check actual folded-sheet coverage independently of the construction records.
use super::super::weather_sections::{SheetSection, WeatherSide};
use super::*;
use crate::GenerationResult as Result;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::SignedLength;

// These acceptance margins are smaller than the authored folds. Keeping them
// independent lets the audit detect a shortened lap or insufficient engagement.
const MIN_UPSTAND_HEIGHT_METRES: f32 = 0.15;
const STACK_TOP_RESERVE_METRES: f32 = 0.3;
const CORNER_LAP_METRES: f32 = 0.002;
const UPSTAND_INNER_DEPTH_METRES: f32 = -0.002;
const UPSTAND_OUTER_DEPTH_METRES: f32 = 0.004;
const UPSTAND_FOOT_LAP_METRES: f32 = 0.01;
const REGLET_INNER_DEPTH_METRES: f32 = -0.025;
const SKIRT_OUTER_DEPTH_METRES: f32 = 0.012;
const MIN_COUNTERFLASHING_THICKNESS_METRES: f32 = 0.003;
const SKIRT_INNER_DEPTH_METRES: f32 = 0.008;
const MIN_SKIRT_DROP_METRES: f32 = 0.06;
const COVERAGE_TOLERANCE_METRES: f32 = 0.0001;
const MIN_PAN_NORMAL_ALIGNMENT: f32 = 0.99999;
const PAN_PLANE_TOLERANCE_METRES: f32 = 0.001;
const MIN_OUTBOARD_CONTACT_SQUARE_METRES: f32 = 0.001;

pub(super) fn audit(
    plan: &BuildingPlan,
    heating: &DomesticHeatingPlan,
    face: &RoofFace,
    shaft: SpatialBounds<Architectural>,
) -> Result<bool> {
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
        .collect::<Result<Vec<_>>>()?;
    let top = tops[0];
    if top < highest + MIN_UPSTAND_HEIGHT_METRES
        || top > shaft.max().metres().y - STACK_TOP_RESERVE_METRES
        || tops
            .iter()
            .any(|y| (y - top).abs() > GEOMETRY_TOLERANCE_METRES)
    {
        return Ok(false);
    }
    for side in WeatherSide::around(shaft)? {
        let [start, end] = side.ends;
        let low = super::super::placement::roof_height(face, start)?
            .metres()
            .min(super::super::placement::roof_height(face, end)?.metres());
        // Require positive masonry engagement, apron contact and corner lap.
        let tangent = (end.metres() - start.metres()).normalize();
        let side = WeatherSide {
            ends: [
                ArchitecturalPlanPoint::from_metres(start.metres() - tangent * CORNER_LAP_METRES)?,
                ArchitecturalPlanPoint::from_metres(end.metres() + tangent * CORNER_LAP_METRES)?,
            ],
            outward: side.outward,
        };
        if !side.has_weather_coverage(
            &upstands,
            &counter,
            crate::spatial_geometry::Elevation::from_metres(low)?,
            crate::spatial_geometry::Elevation::from_metres(top)?,
        )? {
            return Ok(false);
        }
    }
    Ok(true)
}

impl WeatherSide {
    fn has_weather_coverage(
        self,
        upstands: &[&ResolvedSolid],
        counter: &[&ResolvedSolid],
        low: crate::spatial_geometry::Elevation<Architectural>,
        top: crate::spatial_geometry::Elevation<Architectural>,
    ) -> Result<bool> {
        let side = self;
        let low = low.metres();
        let top_metres = top.metres();
        Ok(covers(
            upstands,
            SheetSection::new(
                SignedLength::from_metres(UPSTAND_INNER_DEPTH_METRES)?,
                SignedLength::from_metres(UPSTAND_OUTER_DEPTH_METRES)?,
                crate::spatial_geometry::Elevation::from_metres(low - UPSTAND_FOOT_LAP_METRES)?,
                top,
            )?
            .bounds(side)?,
        )? && covers(
            counter,
            SheetSection::new(
                SignedLength::from_metres(REGLET_INNER_DEPTH_METRES)?,
                SignedLength::from_metres(SKIRT_OUTER_DEPTH_METRES)?,
                top,
                crate::spatial_geometry::Elevation::from_metres(
                    top_metres + MIN_COUNTERFLASHING_THICKNESS_METRES,
                )?,
            )?
            .bounds(side)?,
        )? && covers(
            counter,
            SheetSection::new(
                SignedLength::from_metres(SKIRT_INNER_DEPTH_METRES)?,
                SignedLength::from_metres(SKIRT_OUTER_DEPTH_METRES)?,
                crate::spatial_geometry::Elevation::from_metres(
                    top_metres - MIN_SKIRT_DROP_METRES,
                )?,
                top,
            )?
            .bounds(side)?,
        )?)
    }
}

fn covers(solids: &[&ResolvedSolid], required: SpatialBounds<Architectural>) -> Result<bool> {
    crate::geometry_index::try_any(solids.iter(), |s| {
        let actual = s.cuboid_bounds()?;
        Ok::<bool, crate::GenerationError>(
            (required.min().metres() - actual.min().metres()).min_element()
                >= -COVERAGE_TOLERANCE_METRES
                && (actual.max().metres() - required.max().metres()).min_element()
                    >= -COVERAGE_TOLERANCE_METRES,
        )
    })
}

pub(super) fn continuous_pan(
    plan: &BuildingPlan,
    heating: &DomesticHeatingPlan,
    face: &RoofFace,
    shaft: SpatialBounds<Architectural>,
) -> Result<bool> {
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
        if normal(sheet).dot(reference_normal) < MIN_PAN_NORMAL_ALIGNMENT
            || (sheet.centre.metres() - sheets[0].centre.metres())
                .dot(reference_normal)
                .abs()
                > PAN_PLANE_TOLERANCE_METRES
        {
            return Ok(false);
        }
        let Some(contact) = super::super::contact::measured(sheet, next)? else {
            return Ok(false);
        };
        // A contact inside the masonry does not close an outboard pan corner.
        if rect(contact).difference(&rect(shaft)).unsigned_area()
            < MIN_OUTBOARD_CONTACT_SQUARE_METRES
        {
            return Ok(false);
        }
    }
    Ok(true)
}
