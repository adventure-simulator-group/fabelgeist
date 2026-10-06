//! A static lifting tackle: strapped timber block, sheave, hemp fall and open iron hook.
use super::*;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{CuboidDimensions, Position, PositiveLength, RigidRotation};
use bevy::math::Quat;

const ROPE_DIAMETER_METRES: f32 = 0.025;
const HOOK_SECTION_METRES: f32 = 0.027;
const SHEAVE_RADIUS_METRES: f32 = 0.16;
const SHEAVE_CENTRE_METRES: f32 = 2.64;

pub(super) fn rig(
    a: &mut Assembly<'_>,
    centre: ArchitecturalPlanPoint,
) -> Result<(), crate::GenerationError> {
    let centre = centre.metres();
    beam_strap(a, ArchitecturalPlanPoint::from_metres(centre)?)?;
    pulley_block(a, ArchitecturalPlanPoint::from_metres(centre)?)?;
    let cleat = Vec3::new(centre.x + 1.7, 1.20, centre.y - 0.22);
    post_cleat(a, Position::<crate::Architectural>::from_metres(cleat)?)?;
    rope_crown(a, ArchitecturalPlanPoint::from_metres(centre)?)?;
    let load = Vec3::new(centre.x - SHEAVE_RADIUS_METRES, 1.42, centre.y);
    member(
        a,
        WorkplaceMaterial::HempRope,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            load.x,
            SHEAVE_CENTRE_METRES,
            load.z,
        ))?,
        Position::<crate::Architectural>::from_metres(load)?,
        PositiveLength::from_metres(ROPE_DIAMETER_METRES)?,
    )?;
    member(
        a,
        WorkplaceMaterial::HempRope,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            centre.x + SHEAVE_RADIUS_METRES,
            SHEAVE_CENTRE_METRES,
            centre.y,
        ))?,
        Position::<crate::Architectural>::from_metres(cleat - Vec3::X * 0.06)?,
        PositiveLength::from_metres(ROPE_DIAMETER_METRES)?,
    )?;
    open_hook(a, Position::<crate::Architectural>::from_metres(load)?)?;

    Ok(())
}

fn beam_strap(
    a: &mut Assembly<'_>,
    p: ArchitecturalPlanPoint,
) -> Result<(), crate::GenerationError> {
    let p = p.metres();
    for z in [-0.174, 0.174] {
        part(
            a,
            WorkplaceMaterial::Iron,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 3.11, p.y + z))?,
            CuboidDimensions::from_metres(Vec3::new(0.075, 0.37, 0.045))?,
        )?;
    }
    for y in [2.945, 3.275] {
        part(
            a,
            WorkplaceMaterial::Iron,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x, y, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.075, 0.045, 0.39))?,
        )?;
    }
    // An open eye joins the beam band to the wooden cheeks beneath it.
    for x in [-0.055, 0.055] {
        part(
            a,
            WorkplaceMaterial::Iron,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x + x, 2.88, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.032, 0.16, 0.07))?,
        )?;
    }
    part(
        a,
        WorkplaceMaterial::Iron,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 2.805, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(0.14, 0.035, 0.24))?,
    )?;

    Ok(())
}

fn pulley_block(
    a: &mut Assembly<'_>,
    p: ArchitecturalPlanPoint,
) -> Result<(), crate::GenerationError> {
    let p = p.metres();
    for z in [-0.1125, 0.1125] {
        // Stepped shoulders soften the block outline while leaving a central sheave gap.
        for (y, height, width) in [(2.765, 0.10, 0.24), (2.64, 0.15, 0.34), (2.515, 0.10, 0.24)] {
            part(
                a,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(p.x, y, p.y + z))?,
                CuboidDimensions::from_metres(Vec3::new(width, height, 0.065))?,
            )?;
        }
    }
    part(
        a,
        WorkplaceMaterial::Iron,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, SHEAVE_CENTRE_METRES, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(0.07, 0.07, 0.34))?,
    )?;
    // A faceted wheel bears on the axle between the two visible timber cheeks.
    let _: () = for size in [Vec3::new(0.23, 0.31, 0.13), Vec3::new(0.31, 0.23, 0.13)] {
        part(
            a,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                p.x,
                SHEAVE_CENTRE_METRES,
                p.y,
            ))?,
            CuboidDimensions::from_metres(size)?,
        )?;
    };
    Ok(())
}

fn rope_crown(
    a: &mut Assembly<'_>,
    p: ArchitecturalPlanPoint,
) -> Result<(), crate::GenerationError> {
    let p = p.metres();
    let arc_segments = 8;
    let _: () = for segment in 0..arc_segments {
        let point = |index: u32| {
            let angle = std::f32::consts::PI * index as f32 / arc_segments as f32;
            Vec3::new(
                p.x + SHEAVE_RADIUS_METRES * angle.cos(),
                SHEAVE_CENTRE_METRES + SHEAVE_RADIUS_METRES * angle.sin(),
                p.y,
            )
        };
        member(
            a,
            WorkplaceMaterial::HempRope,
            Position::<crate::Architectural>::from_metres(point(segment))?,
            Position::<crate::Architectural>::from_metres(point(segment + 1))?,
            PositiveLength::from_metres(ROPE_DIAMETER_METRES)?,
        )?;
    };
    Ok(())
}

fn post_cleat(
    a: &mut Assembly<'_>,
    p: Position<crate::Architectural>,
) -> Result<(), crate::GenerationError> {
    let p = p.metres();
    part(
        a,
        WorkplaceMaterial::Iron,
        Position::<crate::Architectural>::from_metres(p + Vec3::new(0.0, -0.055, 0.08))?,
        CuboidDimensions::from_metres(Vec3::new(0.14, 0.22, 0.06))?,
    )?;
    part(
        a,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(p + Vec3::new(0.0, -0.035, 0.035))?,
        CuboidDimensions::from_metres(Vec3::new(0.34, 0.055, 0.075))?,
    )?;
    for y in [0.0, -0.05] {
        part(
            a,
            WorkplaceMaterial::HempRope,
            Position::<crate::Architectural>::from_metres(p + Vec3::Y * y)?,
            CuboidDimensions::from_metres(Vec3::new(0.21, ROPE_DIAMETER_METRES, 0.05))?,
        )?;
    }
    member(
        a,
        WorkplaceMaterial::HempRope,
        Position::<crate::Architectural>::from_metres(p - Vec3::X * 0.06)?,
        Position::<crate::Architectural>::from_metres(p + Vec3::new(-0.085, -0.28, 0.0))?,
        PositiveLength::from_metres(ROPE_DIAMETER_METRES)?,
    )?;

    Ok(())
}

fn open_hook(
    a: &mut Assembly<'_>,
    eye: Position<crate::Architectural>,
) -> Result<(), crate::GenerationError> {
    let eye = eye.metres();
    let bend = [
        Vec3::ZERO,
        Vec3::new(0.0, -0.24, 0.0),
        Vec3::new(0.075, -0.31, 0.0),
        Vec3::new(0.19, -0.31, 0.0),
        Vec3::new(0.22, -0.21, 0.0),
    ];
    let _: () = for ends in bend.windows(2) {
        member(
            a,
            WorkplaceMaterial::Iron,
            Position::<crate::Architectural>::from_metres(eye + ends[0])?,
            Position::<crate::Architectural>::from_metres(eye + ends[1])?,
            PositiveLength::from_metres(HOOK_SECTION_METRES)?,
        )?;
    };
    Ok(())
}

fn part(
    a: &mut Assembly<'_>,
    material: WorkplaceMaterial,
    centre: Position<crate::Architectural>,
    size: CuboidDimensions,
) -> Result<(), crate::GenerationError> {
    a.part(
        WorkplaceFeature::LoadingHoist,
        material,
        centre,
        size,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}

fn member(
    a: &mut Assembly<'_>,
    material: WorkplaceMaterial,
    from: Position<crate::Architectural>,
    to: Position<crate::Architectural>,
    section: PositiveLength,
) -> Result<(), crate::GenerationError> {
    let from = from.metres();
    let to = to.metres();
    let section = section.metres();
    let direction = to - from;
    let id = a.part(
        WorkplaceFeature::LoadingHoist,
        material,
        Position::from_metres((from + to) * 0.5)?,
        CuboidDimensions::from_metres(Vec3::new(section, direction.length(), section))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    a.orient_part(
        id,
        RigidRotation::from_quaternion(Quat::from_rotation_arc(Vec3::Y, direction.normalize()))?,
    )?;

    Ok(())
}
