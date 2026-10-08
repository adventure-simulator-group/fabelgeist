//! Authored masonry courses and smoke paths in the appliance frame.
use super::*;

// Shared planes keep the masonry shell and smoke openings aligned in HearthLocal.
pub(super) const BORE_HALF_WIDTH_METRES: f32 = 0.18;
pub(super) const HOOD_HALF_WIDTH_METRES: f32 = 0.4;
pub(super) const STOVE_CHAMBER_HALF_WIDTH_METRES: f32 = 0.25;
pub(super) const PLINTH_TOP_METRES: f32 = 0.2;
pub(super) const STOVE_CHAMBER_TOP_METRES: f32 = 1.5;
pub(super) const FIREBOX_BOTTOM_METRES: f32 = 0.45;
pub(super) const FIREBOX_TOP_METRES: f32 = 0.75;
pub(super) const SMOKE_RETURN_BOTTOM_METRES: f32 = 1.15;
pub(super) const SMOKE_RETURN_TOP_METRES: f32 = 1.35;
pub(super) const HOOD_CAP_BOTTOM_METRES: f32 = 2.1;
pub(super) const FLUE_SHELL_BOTTOM_METRES: f32 = 2.2;
pub(super) const FIRE_WALL_BACK_METRES: f32 = -0.15;
pub(super) const FIRE_WALL_FRONT_METRES: f32 = 0.3;
pub(super) const FIREBOX_BACK_METRES: f32 = -0.75;
pub(super) const MOUTH_HALF_WIDTH_METRES: f32 = 0.28;
pub(super) const MOUTH_JAMB_WIDTH_METRES: f32 = 0.12;
pub(super) const HOOD_SIDE_WIDTH_METRES: f32 = 0.22;
pub(super) const HOOD_THROAT_BOTTOM_METRES: f32 = 2.05;
pub(super) const HOOD_THROAT_TOP_METRES: f32 = 2.21;
pub(super) const FLUE_EXIT_OVERLAP_METRES: f32 = 0.02;

struct ApplianceCourse {
    min: Displacement<HearthLocal>,
    max: Displacement<HearthLocal>,
}

pub(super) fn smoke_passages(
    assembly: &mut Assembly<'_>,
    top: Elevation<HearthLocal>,
    bore_back: SignedLength,
    bore_front: SignedLength,
) -> Result<()> {
    let top = top.metres();
    let bore_back = bore_back.metres();
    let bore_front = bore_front.metres();
    assembly.passage(
        Passage::StoveChamber,
        Displacement::from_metres(Vec3::new(
            -STOVE_CHAMBER_HALF_WIDTH_METRES,
            PLINTH_TOP_METRES,
            FIREBOX_BACK_METRES,
        ))?,
        Displacement::from_metres(Vec3::new(
            STOVE_CHAMBER_HALF_WIDTH_METRES,
            STOVE_CHAMBER_TOP_METRES,
            FIRE_WALL_BACK_METRES,
        ))?,
    )?;
    assembly.passage(
        Passage::HearthMouth,
        Displacement::from_metres(Vec3::new(
            -MOUTH_HALF_WIDTH_METRES,
            PLINTH_TOP_METRES,
            FIRE_WALL_FRONT_METRES,
        ))?,
        Displacement::from_metres(Vec3::new(
            MOUTH_HALF_WIDTH_METRES,
            HOOD_CAP_BOTTOM_METRES,
            assembly.placement.site.section.front()?.metres(),
        ))?,
    )?;
    assembly.passage(
        Passage::StoveFirebox,
        Displacement::from_metres(Vec3::new(
            -BORE_HALF_WIDTH_METRES,
            FIREBOX_BOTTOM_METRES,
            FIREBOX_BACK_METRES,
        ))?,
        Displacement::from_metres(Vec3::new(BORE_HALF_WIDTH_METRES, FIREBOX_TOP_METRES, 0.32))?,
    )?;
    assembly.passage(
        Passage::StoveSmokeReturn,
        Displacement::from_metres(Vec3::new(
            -BORE_HALF_WIDTH_METRES,
            SMOKE_RETURN_BOTTOM_METRES,
            FIREBOX_BACK_METRES,
        ))?,
        Displacement::from_metres(Vec3::new(
            BORE_HALF_WIDTH_METRES,
            SMOKE_RETURN_TOP_METRES,
            0.32,
        ))?,
    )?;
    assembly.passage(
        Passage::HoodThroat,
        Displacement::from_metres(Vec3::new(
            -BORE_HALF_WIDTH_METRES,
            HOOD_THROAT_BOTTOM_METRES,
            bore_back,
        ))?,
        Displacement::from_metres(Vec3::new(
            BORE_HALF_WIDTH_METRES,
            HOOD_THROAT_TOP_METRES,
            bore_front,
        ))?,
    )?;
    assembly.passage(
        Passage::FlueBore,
        Displacement::from_metres(Vec3::new(
            -BORE_HALF_WIDTH_METRES,
            FLUE_SHELL_BOTTOM_METRES,
            bore_back,
        ))?,
        Displacement::from_metres(Vec3::new(
            BORE_HALF_WIDTH_METRES,
            top + FLUE_EXIT_OVERLAP_METRES,
            bore_front,
        ))?,
    )?;

    Ok(())
}

pub(super) fn fire_wall(assembly: &mut Assembly<'_>) -> Result<()> {
    for course in [
        ApplianceCourse {
            min: Displacement::from_metres(Vec3::new(
                -CORE_HALF_WIDTH_METRES,
                PLINTH_TOP_METRES,
                FIRE_WALL_BACK_METRES,
            ))?,
            max: Displacement::from_metres(Vec3::new(
                -BORE_HALF_WIDTH_METRES,
                crate::heating::placement::FIRE_WALL_PATCH_HEIGHT_METRES,
                FIRE_WALL_FRONT_METRES,
            ))?,
        },
        ApplianceCourse {
            min: Displacement::from_metres(Vec3::new(
                BORE_HALF_WIDTH_METRES,
                PLINTH_TOP_METRES,
                FIRE_WALL_BACK_METRES,
            ))?,
            max: Displacement::from_metres(Vec3::new(
                CORE_HALF_WIDTH_METRES,
                crate::heating::placement::FIRE_WALL_PATCH_HEIGHT_METRES,
                FIRE_WALL_FRONT_METRES,
            ))?,
        },
        ApplianceCourse {
            min: Displacement::from_metres(Vec3::new(
                -BORE_HALF_WIDTH_METRES,
                PLINTH_TOP_METRES,
                FIRE_WALL_BACK_METRES,
            ))?,
            max: Displacement::from_metres(Vec3::new(
                BORE_HALF_WIDTH_METRES,
                FIREBOX_BOTTOM_METRES,
                FIRE_WALL_FRONT_METRES,
            ))?,
        },
        ApplianceCourse {
            min: Displacement::from_metres(Vec3::new(
                -BORE_HALF_WIDTH_METRES,
                FIREBOX_TOP_METRES,
                FIRE_WALL_BACK_METRES,
            ))?,
            max: Displacement::from_metres(Vec3::new(
                BORE_HALF_WIDTH_METRES,
                SMOKE_RETURN_BOTTOM_METRES,
                FIRE_WALL_FRONT_METRES,
            ))?,
        },
        ApplianceCourse {
            min: Displacement::from_metres(Vec3::new(
                -BORE_HALF_WIDTH_METRES,
                SMOKE_RETURN_TOP_METRES,
                FIRE_WALL_BACK_METRES,
            ))?,
            max: Displacement::from_metres(Vec3::new(
                BORE_HALF_WIDTH_METRES,
                crate::heating::placement::FIRE_WALL_PATCH_HEIGHT_METRES,
                FIRE_WALL_FRONT_METRES,
            ))?,
        },
    ] {
        assembly.part(Part::FireWall, MASONRY, course.min, course.max)?;
    }
    Ok(())
}

pub(super) fn stove_shell(assembly: &mut Assembly<'_>) -> Result<()> {
    // Stove shell, with the firebox and smoke return opening toward the kitchen.
    for course in [
        ApplianceCourse {
            min: Displacement::from_metres(Vec3::new(
                -HOOD_HALF_WIDTH_METRES,
                PLINTH_TOP_METRES,
                -CORE_HALF_DEPTH_METRES,
            ))?,
            max: Displacement::from_metres(Vec3::new(
                -STOVE_CHAMBER_HALF_WIDTH_METRES,
                STOVE_CHAMBER_TOP_METRES,
                FIRE_WALL_BACK_METRES,
            ))?,
        },
        ApplianceCourse {
            min: Displacement::from_metres(Vec3::new(
                STOVE_CHAMBER_HALF_WIDTH_METRES,
                PLINTH_TOP_METRES,
                -CORE_HALF_DEPTH_METRES,
            ))?,
            max: Displacement::from_metres(Vec3::new(
                HOOD_HALF_WIDTH_METRES,
                STOVE_CHAMBER_TOP_METRES,
                FIRE_WALL_BACK_METRES,
            ))?,
        },
        ApplianceCourse {
            min: Displacement::from_metres(Vec3::new(
                -STOVE_CHAMBER_HALF_WIDTH_METRES,
                PLINTH_TOP_METRES,
                -CORE_HALF_DEPTH_METRES,
            ))?,
            max: Displacement::from_metres(Vec3::new(
                STOVE_CHAMBER_HALF_WIDTH_METRES,
                STOVE_CHAMBER_TOP_METRES,
                FIREBOX_BACK_METRES,
            ))?,
        },
        ApplianceCourse {
            min: Displacement::from_metres(Vec3::new(
                -HOOD_HALF_WIDTH_METRES,
                STOVE_CHAMBER_TOP_METRES,
                -CORE_HALF_DEPTH_METRES,
            ))?,
            max: Displacement::from_metres(Vec3::new(
                HOOD_HALF_WIDTH_METRES,
                1.65,
                FIRE_WALL_BACK_METRES,
            ))?,
        },
    ] {
        assembly.part(
            Part::TiledStove,
            Material::GlazedTile,
            course.min,
            course.max,
        )?;
    }
    Ok(())
}

// All arguments originate in this appliance's admitted local set-out. Shell
// subtraction is a private native kernel; parts are admitted in HearthLocal.
pub(super) fn flue_shell(
    assembly: &mut Assembly<'_>,
    top: Elevation<HearthLocal>,
    shaft_back: SignedLength,
    shaft_front: SignedLength,
    bore_back: SignedLength,
    bore_front: SignedLength,
) -> Result<()> {
    let top = top.metres();
    let shaft_back = shaft_back.metres();
    let shaft_front = shaft_front.metres();
    let bore_back = bore_back.metres();
    let bore_front = bore_front.metres();
    for course in [
        ApplianceCourse {
            min: Displacement::from_metres(Vec3::new(
                -SHAFT_HALF_WIDTH_METRES,
                FLUE_SHELL_BOTTOM_METRES,
                shaft_back,
            ))?,
            max: Displacement::from_metres(Vec3::new(-BORE_HALF_WIDTH_METRES, top, shaft_front))?,
        },
        ApplianceCourse {
            min: Displacement::from_metres(Vec3::new(
                BORE_HALF_WIDTH_METRES,
                FLUE_SHELL_BOTTOM_METRES,
                shaft_back,
            ))?,
            max: Displacement::from_metres(Vec3::new(SHAFT_HALF_WIDTH_METRES, top, shaft_front))?,
        },
        ApplianceCourse {
            min: Displacement::from_metres(Vec3::new(
                -BORE_HALF_WIDTH_METRES,
                FLUE_SHELL_BOTTOM_METRES,
                shaft_back,
            ))?,
            max: Displacement::from_metres(Vec3::new(BORE_HALF_WIDTH_METRES, top, bore_back))?,
        },
        ApplianceCourse {
            min: Displacement::from_metres(Vec3::new(
                -BORE_HALF_WIDTH_METRES,
                FLUE_SHELL_BOTTOM_METRES,
                bore_front,
            ))?,
            max: Displacement::from_metres(Vec3::new(BORE_HALF_WIDTH_METRES, top, shaft_front))?,
        },
    ] {
        assembly.part(Part::Flue, MASONRY, course.min, course.max)?;
    }
    Ok(())
}
