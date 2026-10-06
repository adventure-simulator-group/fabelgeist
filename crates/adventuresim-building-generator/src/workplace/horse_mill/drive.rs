use super::*;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{CuboidDimensions, Position, RigidRotation};
use bevy::math::Quat;
use std::f32::consts::{FRAC_PI_4, TAU};

pub(super) fn assemble(
    a: &mut Assembly<'_>,
    p: ArchitecturalPlanPoint,
) -> Result<(), crate::GenerationError> {
    let p = p.metres();
    a.part(
        WorkplaceFeature::MillDrive,
        WorkplaceMaterial::DressedStone,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 0.2, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(0.9, 0.4, 0.9))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    a.part(
        WorkplaceFeature::MillDrive,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 2.075, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(0.32, 3.75, 0.32))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    for x in [1.8, 11.2] {
        a.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(x, 1.975, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.18, 3.95, 0.3))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.part(
        WorkplaceFeature::Beam,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(6.5, 4.075, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(9.6, 0.25, 0.35))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    cogwheel(a, ArchitecturalPlanPoint::from_metres(p)?)?;
    overhead_sweep(a, ArchitecturalPlanPoint::from_metres(p)?)?;

    Ok(())
}

fn cogwheel(a: &mut Assembly<'_>, p: ArchitecturalPlanPoint) -> Result<(), crate::GenerationError> {
    let p = p.metres();
    for spoke in 0..4 {
        oriented(
            a,
            WorkplaceFeature::MillDrive,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 2.65, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.16, 0.2, 3.7))?,
            RigidRotation::from_quaternion(Quat::from_rotation_y(spoke as f32 * FRAC_PI_4))?,
        )?;
    }
    let _: () = for tooth in 0..16 {
        let angle = tooth as f32 * TAU / 16.0;
        let direction = Vec2::new(angle.sin(), angle.cos());
        let point = p + direction * 1.8;
        oriented(
            a,
            WorkplaceFeature::MillDrive,
            Position::<crate::Architectural>::from_metres(Vec3::new(point.x, 2.65, point.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.74, 0.24, 0.22))?,
            RigidRotation::from_quaternion(Quat::from_rotation_y(angle))?,
        )?;
        oriented(
            a,
            WorkplaceFeature::MillDrive,
            Position::<crate::Architectural>::from_metres(Vec3::new(point.x, 2.39, point.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.14, 0.3, 0.14))?,
            RigidRotation::from_quaternion(Quat::from_rotation_y(angle))?,
        )?;
    };
    Ok(())
}

fn overhead_sweep(
    a: &mut Assembly<'_>,
    p: ArchitecturalPlanPoint,
) -> Result<(), crate::GenerationError> {
    let p = p.metres();
    // The arm turns above every fixed mill component; only its unhitched draw rope descends.
    a.part(
        WorkplaceFeature::MillSweep,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 3.4, p.y - 1.95))?,
        CuboidDimensions::from_metres(Vec3::new(0.18, 0.22, 4.1))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    let brace = a.part(
        WorkplaceFeature::MillSweep,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 3.63, p.y - 1.0))?,
        CuboidDimensions::from_metres(Vec3::new(0.12, 0.12, 2.05))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    a.orient_part(
        brace,
        RigidRotation::from_quaternion(Quat::from_rotation_x(-0.2))?,
    )?;
    a.part(
        WorkplaceFeature::MillSweep,
        WorkplaceMaterial::HempRope,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 2.49, p.y - 3.9))?,
        CuboidDimensions::from_metres(Vec3::new(0.045, 1.6, 0.045))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    a.part(
        WorkplaceFeature::MillSweep,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 1.65, p.y - 3.9))?,
        CuboidDimensions::from_metres(Vec3::new(0.65, 0.08, 0.1))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}

fn oriented(
    a: &mut Assembly<'_>,
    feature: WorkplaceFeature,
    centre: Position<crate::Architectural>,
    size: CuboidDimensions,
    rotation: RigidRotation,
) -> Result<(), crate::GenerationError> {
    let id = a.part(
        feature,
        WorkplaceMaterial::UnpaintedTimber,
        centre,
        size,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    a.orient_part(id, rotation)?;

    Ok(())
}
