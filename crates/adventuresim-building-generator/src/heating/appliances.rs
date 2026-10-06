//! A shared masonry fire wall separates cooking hearth and rear-fed tiled stove.
use super::assembly::Assembly;
use super::placement::HearthLocal;
use crate::spatial_geometry::{Displacement, Elevation, SignedLength};
use crate::{Architectural, StoreyIndex};
use crate::{
    BuildingLodMaterial as Material, HeatingPartKind as Part, HeatingPassageKind as Passage,
    WallMaterialClass,
};
use bevy::math::Vec3;
const MASONRY: Material = Material::Wall(WallMaterialClass::CivilianMasonry);

pub(super) fn build(
    a: &mut Assembly<'_>,
    top: Elevation<Architectural>,
) -> Result<(), crate::GenerationError> {
    let top = top.metres() - a.placement.site.floor_height.metres();
    let front = a.placement.site.section.front()?.metres();
    let shaft_back = a.placement.site.section.shaft_offset()?.metres() - 0.3;
    let shaft_front = a.placement.site.section.shaft_offset()?.metres() + 0.3;
    let bore_back = a.placement.site.section.shaft_offset()?.metres() - 0.18;
    let bore_front = a.placement.site.section.shaft_offset()?.metres() + 0.18;
    // Upper appliances transfer their weight directly through continuous masonry.
    if a.placement.site.storey_level > StoreyIndex::GROUND {
        let support = a.absolute_part(Part::SupportPier, MASONRY, a.placement.site.support()?)?;
        a.plan.ground_support = a.bearing(support)?;
    }
    // The hearth, fire wall and stove bear on one plinth.
    let footing = a.part(
        Part::Footing,
        MASONRY,
        Displacement::from_metres(Vec3::new(-0.48, 0.0, -0.9))?,
        Displacement::from_metres(Vec3::new(0.48, 0.2, front))?,
    )?;
    if a.placement.site.storey_level == StoreyIndex::GROUND {
        a.plan.ground_support = a.bearing(footing)?;
    }
    fire_wall(a)?;
    a.part(
        Part::Hood,
        MASONRY,
        Displacement::from_metres(Vec3::new(
            -0.4,
            super::placement::FIRE_WALL_PATCH_HEIGHT_METRES,
            0.15,
        ))?,
        Displacement::from_metres(Vec3::new(0.4, 2.1, 0.3))?,
    )?;
    stove_shell(a)?;
    // Masonry jambs support the hood on either side of its open cooking mouth.
    for x in [-0.4, 0.28] {
        a.part(
            Part::Hearth,
            MASONRY,
            Displacement::from_metres(Vec3::new(x, 0.2, 0.15))?,
            Displacement::from_metres(Vec3::new(x + 0.12, 2.1, front))?,
        )?;
    }
    // Continuous rear shell above the fire wall; hood cap leaves an actual bore.
    a.part(
        Part::Hood,
        MASONRY,
        Displacement::from_metres(Vec3::new(-0.4, 2.1, 0.15))?,
        Displacement::from_metres(Vec3::new(0.4, 2.2, bore_back))?,
    )?;
    a.part(
        Part::Hood,
        MASONRY,
        Displacement::from_metres(Vec3::new(-0.4, 2.1, bore_front))?,
        Displacement::from_metres(Vec3::new(0.4, 2.2, front))?,
    )?;
    for x in [-0.4, 0.18] {
        a.part(
            Part::Hood,
            MASONRY,
            Displacement::from_metres(Vec3::new(x, 2.1, bore_back))?,
            Displacement::from_metres(Vec3::new(x + 0.22, 2.2, bore_front))?,
        )?;
    }
    flue_shell(
        a,
        Elevation::from_metres(top)?,
        SignedLength::from_metres(shaft_back)?,
        SignedLength::from_metres(shaft_front)?,
        SignedLength::from_metres(bore_back)?,
        SignedLength::from_metres(bore_front)?,
    )?;
    if let Some(shoulder) = a.placement.site.shaft_shoulder()? {
        let shaft = a
            .placement
            .site
            .shaft(crate::spatial_geometry::Elevation::from_metres(
                shoulder.max().metres().y,
            )?)?;
        for bounds in super::floors::pieces(shoulder, shaft)? {
            a.absolute_part(Part::FlueShoulder, MASONRY, bounds)?;
        }
    }
    smoke_passages(
        a,
        Elevation::from_metres(top)?,
        SignedLength::from_metres(bore_back)?,
        SignedLength::from_metres(bore_front)?,
    )?;

    Ok(())
}

fn smoke_passages(
    a: &mut Assembly<'_>,
    top: Elevation<HearthLocal>,
    bore_back: SignedLength,
    bore_front: SignedLength,
) -> Result<(), crate::GenerationError> {
    let top = top.metres();
    let bore_back = bore_back.metres();
    let bore_front = bore_front.metres();
    a.passage(
        Passage::StoveChamber,
        Displacement::from_metres(Vec3::new(-0.25, 0.2, -0.75))?,
        Displacement::from_metres(Vec3::new(0.25, 1.5, -0.15))?,
    )?;
    a.passage(
        Passage::HearthMouth,
        Displacement::from_metres(Vec3::new(-0.28, 0.2, 0.3))?,
        Displacement::from_metres(Vec3::new(
            0.28,
            2.1,
            a.placement.site.section.front()?.metres(),
        ))?,
    )?;
    a.passage(
        Passage::StoveFirebox,
        Displacement::from_metres(Vec3::new(-0.18, 0.45, -0.75))?,
        Displacement::from_metres(Vec3::new(0.18, 0.75, 0.32))?,
    )?;
    a.passage(
        Passage::StoveSmokeReturn,
        Displacement::from_metres(Vec3::new(-0.18, 1.15, -0.75))?,
        Displacement::from_metres(Vec3::new(0.18, 1.35, 0.32))?,
    )?;
    a.passage(
        Passage::HoodThroat,
        Displacement::from_metres(Vec3::new(-0.18, 2.05, bore_back))?,
        Displacement::from_metres(Vec3::new(0.18, 2.21, bore_front))?,
    )?;
    a.passage(
        Passage::FlueBore,
        Displacement::from_metres(Vec3::new(-0.18, 2.2, bore_back))?,
        Displacement::from_metres(Vec3::new(0.18, top + 0.02, bore_front))?,
    )?;

    Ok(())
}

fn fire_wall(a: &mut Assembly<'_>) -> Result<(), crate::GenerationError> {
    let _: () = for (min, max) in [
        (
            Vec3::new(-0.48, 0.2, -0.15),
            Vec3::new(-0.18, super::placement::FIRE_WALL_PATCH_HEIGHT_METRES, 0.3),
        ),
        (
            Vec3::new(0.18, 0.2, -0.15),
            Vec3::new(0.48, super::placement::FIRE_WALL_PATCH_HEIGHT_METRES, 0.3),
        ),
        (Vec3::new(-0.18, 0.2, -0.15), Vec3::new(0.18, 0.45, 0.3)),
        (Vec3::new(-0.18, 0.75, -0.15), Vec3::new(0.18, 1.15, 0.3)),
        (
            Vec3::new(-0.18, 1.35, -0.15),
            Vec3::new(0.18, super::placement::FIRE_WALL_PATCH_HEIGHT_METRES, 0.3),
        ),
    ] {
        a.part(
            Part::FireWall,
            MASONRY,
            Displacement::from_metres(min)?,
            Displacement::from_metres(max)?,
        )?;
    };
    Ok(())
}

fn stove_shell(a: &mut Assembly<'_>) -> Result<(), crate::GenerationError> {
    // Stove shell, with the firebox and smoke return opening toward the kitchen.
    for (min, max) in [
        (Vec3::new(-0.4, 0.2, -0.9), Vec3::new(-0.25, 1.5, -0.15)),
        (Vec3::new(0.25, 0.2, -0.9), Vec3::new(0.4, 1.5, -0.15)),
        (Vec3::new(-0.25, 0.2, -0.9), Vec3::new(0.25, 1.5, -0.75)),
        (Vec3::new(-0.4, 1.5, -0.9), Vec3::new(0.4, 1.65, -0.15)),
    ] {
        a.part(
            Part::TiledStove,
            Material::GlazedTile,
            Displacement::from_metres(min)?,
            Displacement::from_metres(max)?,
        )?;
    }
    Ok(())
}

// All arguments originate in this appliance's admitted local set-out. Shell
// subtraction is a private native kernel; parts are admitted in HearthLocal.
fn flue_shell(
    a: &mut Assembly<'_>,
    top: Elevation<HearthLocal>,
    shaft_back: SignedLength,
    shaft_front: SignedLength,
    bore_back: SignedLength,
    bore_front: SignedLength,
) -> Result<(), crate::GenerationError> {
    let top = top.metres();
    let shaft_back = shaft_back.metres();
    let shaft_front = shaft_front.metres();
    let bore_back = bore_back.metres();
    let bore_front = bore_front.metres();
    for (min, max) in [
        (
            Vec3::new(-0.3, 2.2, shaft_back),
            Vec3::new(-0.18, top, shaft_front),
        ),
        (
            Vec3::new(0.18, 2.2, shaft_back),
            Vec3::new(0.3, top, shaft_front),
        ),
        (
            Vec3::new(-0.18, 2.2, shaft_back),
            Vec3::new(0.18, top, bore_back),
        ),
        (
            Vec3::new(-0.18, 2.2, bore_front),
            Vec3::new(0.18, top, shaft_front),
        ),
    ] {
        a.part(
            Part::Flue,
            MASONRY,
            Displacement::from_metres(min)?,
            Displacement::from_metres(max)?,
        )?;
    }
    Ok(())
}
