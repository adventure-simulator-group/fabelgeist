//! A shared masonry fire wall separates cooking hearth and rear-fed tiled stove.
use super::assembly::Assembly;
use super::placement::{
    CORE_HALF_DEPTH_METRES, CORE_HALF_WIDTH_METRES, HearthLocal, SHAFT_HALF_WIDTH_METRES,
};
use crate::GenerationResult as Result;
use crate::spatial_geometry::{Displacement, Elevation, SignedLength};
use crate::{Architectural, StoreyIndex};
use crate::{
    BuildingLodMaterial as Material, HeatingPartKind as Part, HeatingPassageKind as Passage,
    WallMaterialClass,
};
use bevy::math::Vec3;
const MASONRY: Material = Material::Wall(WallMaterialClass::CivilianMasonry);

mod sections;
use sections::*;

pub(super) fn build(assembly: &mut Assembly<'_>, top: Elevation<Architectural>) -> Result<()> {
    let top = top.metres() - assembly.placement.site.floor_height.metres();
    let front = assembly.placement.site.section.front()?.metres();
    let shaft_back =
        assembly.placement.site.section.shaft_offset()?.metres() - SHAFT_HALF_WIDTH_METRES;
    let shaft_front =
        assembly.placement.site.section.shaft_offset()?.metres() + SHAFT_HALF_WIDTH_METRES;
    let bore_back =
        assembly.placement.site.section.shaft_offset()?.metres() - BORE_HALF_WIDTH_METRES;
    let bore_front =
        assembly.placement.site.section.shaft_offset()?.metres() + BORE_HALF_WIDTH_METRES;
    // Upper appliances transfer their weight directly through continuous masonry.
    if assembly.placement.site.storey_level > StoreyIndex::GROUND {
        let support = assembly.absolute_part(
            Part::SupportPier,
            MASONRY,
            assembly.placement.site.support()?,
        )?;
        assembly.plan.ground_support = assembly.bearing(support)?;
    }
    // The hearth, fire wall and stove bear on one plinth.
    let footing = assembly.part(
        Part::Footing,
        MASONRY,
        Displacement::from_metres(Vec3::new(
            -CORE_HALF_WIDTH_METRES,
            0.0,
            -CORE_HALF_DEPTH_METRES,
        ))?,
        Displacement::from_metres(Vec3::new(CORE_HALF_WIDTH_METRES, PLINTH_TOP_METRES, front))?,
    )?;
    if assembly.placement.site.storey_level == StoreyIndex::GROUND {
        assembly.plan.ground_support = assembly.bearing(footing)?;
    }
    fire_wall(assembly)?;
    assembly.part(
        Part::Hood,
        MASONRY,
        Displacement::from_metres(Vec3::new(
            -HOOD_HALF_WIDTH_METRES,
            super::placement::FIRE_WALL_PATCH_HEIGHT_METRES,
            0.15,
        ))?,
        Displacement::from_metres(Vec3::new(
            HOOD_HALF_WIDTH_METRES,
            HOOD_CAP_BOTTOM_METRES,
            FIRE_WALL_FRONT_METRES,
        ))?,
    )?;
    stove_shell(assembly)?;
    hood_mouth(
        assembly,
        SignedLength::from_metres(front)?,
        SignedLength::from_metres(bore_back)?,
        SignedLength::from_metres(bore_front)?,
    )?;
    flue_shell(
        assembly,
        Elevation::from_metres(top)?,
        SignedLength::from_metres(shaft_back)?,
        SignedLength::from_metres(shaft_front)?,
        SignedLength::from_metres(bore_back)?,
        SignedLength::from_metres(bore_front)?,
    )?;
    if let Some(shoulder) = assembly.placement.site.shaft_shoulder()? {
        let shaft =
            assembly
                .placement
                .site
                .shaft(crate::spatial_geometry::Elevation::from_metres(
                    shoulder.max().metres().y,
                )?)?;
        for bounds in super::floors::pieces(shoulder, shaft)? {
            assembly.absolute_part(Part::FlueShoulder, MASONRY, bounds)?;
        }
    }
    smoke_passages(
        assembly,
        Elevation::from_metres(top)?,
        SignedLength::from_metres(bore_back)?,
        SignedLength::from_metres(bore_front)?,
    )?;

    Ok(())
}

fn hood_mouth(
    assembly: &mut Assembly<'_>,
    front: SignedLength,
    bore_back: SignedLength,
    bore_front: SignedLength,
) -> Result<()> {
    let front = front.metres();
    let bore_back = bore_back.metres();
    let bore_front = bore_front.metres();
    // Masonry jambs support the hood on either side of its open cooking mouth.
    for x in [-HOOD_HALF_WIDTH_METRES, MOUTH_HALF_WIDTH_METRES] {
        assembly.part(
            Part::Hearth,
            MASONRY,
            Displacement::from_metres(Vec3::new(x, PLINTH_TOP_METRES, 0.15))?,
            Displacement::from_metres(Vec3::new(
                x + MOUTH_JAMB_WIDTH_METRES,
                HOOD_CAP_BOTTOM_METRES,
                front,
            ))?,
        )?;
    }
    // Continuous rear shell above the fire wall; hood cap leaves an actual bore.
    assembly.part(
        Part::Hood,
        MASONRY,
        Displacement::from_metres(Vec3::new(
            -HOOD_HALF_WIDTH_METRES,
            HOOD_CAP_BOTTOM_METRES,
            0.15,
        ))?,
        Displacement::from_metres(Vec3::new(
            HOOD_HALF_WIDTH_METRES,
            FLUE_SHELL_BOTTOM_METRES,
            bore_back,
        ))?,
    )?;
    assembly.part(
        Part::Hood,
        MASONRY,
        Displacement::from_metres(Vec3::new(
            -HOOD_HALF_WIDTH_METRES,
            HOOD_CAP_BOTTOM_METRES,
            bore_front,
        ))?,
        Displacement::from_metres(Vec3::new(
            HOOD_HALF_WIDTH_METRES,
            FLUE_SHELL_BOTTOM_METRES,
            front,
        ))?,
    )?;
    for x in [-HOOD_HALF_WIDTH_METRES, BORE_HALF_WIDTH_METRES] {
        assembly.part(
            Part::Hood,
            MASONRY,
            Displacement::from_metres(Vec3::new(x, HOOD_CAP_BOTTOM_METRES, bore_back))?,
            Displacement::from_metres(Vec3::new(
                x + HOOD_SIDE_WIDTH_METRES,
                FLUE_SHELL_BOTTOM_METRES,
                bore_front,
            ))?,
        )?;
    }
    Ok(())
}
