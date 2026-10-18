//! Folded lead at the stack: upstands, masonry reglets and overlapping skirts.
use super::weather_sections::{SheetSection, WeatherSide};
use super::{assembly::Assembly, placement::roof_height};
use crate::GenerationResult as Result;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::SignedLength;
use crate::*;
use bevy::math::Vec2;

const UPSTAND_HEIGHT_METRES: f32 = 0.18;
const SHEET_THICKNESS_METRES: f32 = 0.006;
const MASONRY_EMBED_METRES: f32 = 0.035;
const COUNTERFLASHING_OVERHANG_METRES: f32 = 0.016;
const COUNTERFLASHING_DROP_METRES: f32 = 0.08;
const FOOT_LAP_METRES: f32 = 0.02;

pub(super) fn build(
    assembly: &mut Assembly<'_>,
    face: &RoofFace,
    shaft: SpatialBounds<Architectural>,
) -> Result<()> {
    let min = Vec2::new(shaft.min().metres().x, shaft.min().metres().z);
    let max = Vec2::new(shaft.max().metres().x, shaft.max().metres().z);
    let top = [min, Vec2::new(min.x, max.y), max, Vec2::new(max.x, min.y)]
        .into_iter()
        .try_fold(f32::NEG_INFINITY, |high, p| {
            Ok::<_, crate::GenerationError>(
                high.max(roof_height(face, ArchitecturalPlanPoint::from_metres(p)?)?.metres()),
            )
        })?
        + UPSTAND_HEIGHT_METRES;
    for side in WeatherSide::around(shaft)? {
        let [start, end] = side.ends;
        let low = roof_height(face, start)?
            .metres()
            .min(roof_height(face, end)?.metres());
        strip(
            assembly,
            HeatingPartKind::RoofUpstand,
            side,
            SheetSection::new(
                SignedLength::from_metres(-SHEET_THICKNESS_METRES)?,
                SignedLength::from_metres(SHEET_THICKNESS_METRES)?,
                crate::spatial_geometry::Elevation::from_metres(low - FOOT_LAP_METRES)?,
                crate::spatial_geometry::Elevation::from_metres(top)?,
            )?,
        )?;
        strip(
            assembly,
            HeatingPartKind::RoofCounterFlashing,
            side,
            SheetSection::new(
                SignedLength::from_metres(-MASONRY_EMBED_METRES)?,
                SignedLength::from_metres(COUNTERFLASHING_OVERHANG_METRES)?,
                crate::spatial_geometry::Elevation::from_metres(top)?,
                crate::spatial_geometry::Elevation::from_metres(top + SHEET_THICKNESS_METRES)?,
            )?,
        )?;
        strip(
            assembly,
            HeatingPartKind::RoofCounterFlashing,
            side,
            SheetSection::new(
                SignedLength::from_metres(SHEET_THICKNESS_METRES)?,
                SignedLength::from_metres(COUNTERFLASHING_OVERHANG_METRES)?,
                crate::spatial_geometry::Elevation::from_metres(top - COUNTERFLASHING_DROP_METRES)?,
                crate::spatial_geometry::Elevation::from_metres(top + SHEET_THICKNESS_METRES)?,
            )?,
        )?;
    }
    Ok(())
}

fn strip(
    assembly: &mut Assembly<'_>,
    kind: HeatingPartKind,
    side: WeatherSide,
    section: SheetSection,
) -> Result<()> {
    assembly.absolute_part(
        kind,
        BuildingLodMaterial::LeadAlloy,
        section.lapped_bounds(side, SignedLength::from_metres(SHEET_THICKNESS_METRES)?)?,
    )?;
    Ok(())
}
