use super::*;
use crate::GenerationResult as Result;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{CuboidDimensions, Position, RigidRotation};
use bevy::math::Quat;
use std::f32::consts::{FRAC_PI_4, TAU};

pub(super) fn assemble(assembly: &mut Assembly<'_>, centre: ArchitecturalPlanPoint) -> Result<()> {
    let centre = centre.metres();
    assembly.part(
        WorkplaceFeature::MillDrive,
        WorkplaceMaterial::DressedStone,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.2, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(0.9, 0.4, 0.9))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    assembly.part(
        WorkplaceFeature::MillDrive,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 2.075, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(0.32, 3.75, 0.32))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    for x in [1.8, 11.2] {
        assembly.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(x, 1.975, centre.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.18, 3.95, 0.3))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    assembly.part(
        WorkplaceFeature::Beam,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(6.5, 4.075, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(9.6, 0.25, 0.35))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    cogwheel(assembly, ArchitecturalPlanPoint::from_metres(centre)?)?;
    overhead_sweep(assembly, ArchitecturalPlanPoint::from_metres(centre)?)?;

    Ok(())
}

fn cogwheel(assembly: &mut Assembly<'_>, centre: ArchitecturalPlanPoint) -> Result<()> {
    let centre = centre.metres();
    for spoke in 0..4 {
        oriented(
            assembly,
            WorkplaceFeature::MillDrive,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 2.65, centre.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.16, 0.2, 3.7))?,
            RigidRotation::from_quaternion(Quat::from_rotation_y(spoke as f32 * FRAC_PI_4))?,
        )?;
    }
    let _: () = for tooth in 0..16 {
        let angle = tooth as f32 * TAU / 16.0;
        let direction = Vec2::new(angle.sin(), angle.cos());
        let point = centre + direction * 1.8;
        oriented(
            assembly,
            WorkplaceFeature::MillDrive,
            Position::<crate::Architectural>::from_metres(Vec3::new(point.x, 2.65, point.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.74, 0.24, 0.22))?,
            RigidRotation::from_quaternion(Quat::from_rotation_y(angle))?,
        )?;
        oriented(
            assembly,
            WorkplaceFeature::MillDrive,
            Position::<crate::Architectural>::from_metres(Vec3::new(point.x, 2.39, point.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.14, 0.3, 0.14))?,
            RigidRotation::from_quaternion(Quat::from_rotation_y(angle))?,
        )?;
    };
    Ok(())
}

fn overhead_sweep(assembly: &mut Assembly<'_>, centre: ArchitecturalPlanPoint) -> Result<()> {
    let centre = centre.metres();
    // The arm turns above every fixed mill component; only its unhitched draw rope descends.
    assembly.part(
        WorkplaceFeature::MillSweep,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 3.4, centre.y - 1.95))?,
        CuboidDimensions::from_metres(Vec3::new(0.18, 0.22, 4.1))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    let brace = assembly.part(
        WorkplaceFeature::MillSweep,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 3.63, centre.y - 1.0))?,
        CuboidDimensions::from_metres(Vec3::new(0.12, 0.12, 2.05))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    assembly.orient_part(
        brace,
        RigidRotation::from_quaternion(Quat::from_rotation_x(-0.2))?,
    )?;
    assembly.part(
        WorkplaceFeature::MillSweep,
        WorkplaceMaterial::HempRope,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 2.49, centre.y - 3.9))?,
        CuboidDimensions::from_metres(Vec3::new(0.045, 1.6, 0.045))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    assembly.part(
        WorkplaceFeature::MillSweep,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 1.65, centre.y - 3.9))?,
        CuboidDimensions::from_metres(Vec3::new(0.65, 0.08, 0.1))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}

fn oriented(
    assembly: &mut Assembly<'_>,
    feature: WorkplaceFeature,
    centre: Position<crate::Architectural>,
    size: CuboidDimensions,
    rotation: RigidRotation,
) -> Result<()> {
    let id = assembly.part(
        feature,
        WorkplaceMaterial::UnpaintedTimber,
        centre,
        size,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    assembly.orient_part(id, rotation)?;

    Ok(())
}
