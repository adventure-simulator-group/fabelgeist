use super::*;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{
    CuboidDimensions, Elevation, Position, PositiveLength, RigidRotation,
};
use bevy::math::Quat;
use std::f32::consts::TAU;

pub(super) fn stone_and_hopper(
    a: &mut Assembly<'_>,
    p: ArchitecturalPlanPoint,
) -> Result<(), crate::GenerationError> {
    let p = p.metres();
    a.part(
        WorkplaceFeature::Millstone,
        WorkplaceMaterial::DressedStone,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 0.25, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(1.65, 0.5, 1.65))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    stone_disc(
        a,
        ArchitecturalPlanPoint::from_metres(p)?,
        Elevation::<crate::Architectural>::from_metres(0.63)?,
        PositiveLength::from_metres(0.26)?,
    )?;
    stone_disc(
        a,
        ArchitecturalPlanPoint::from_metres(p)?,
        Elevation::<crate::Architectural>::from_metres(0.89)?,
        PositiveLength::from_metres(0.26)?,
    )?;
    // One aligned driven spindle joins the lantern pinion to the upper runner stone.
    a.part(
        WorkplaceFeature::MillDrive,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 1.57, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(0.18, 2.10, 0.18))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    for y in [2.17, 2.65] {
        a.part(
            WorkplaceFeature::MillDrive,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x, y, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.72, 0.12, 0.72))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    for pin in 0..8 {
        let angle = (pin as f32 + 0.5) * TAU / 8.0;
        let offset = Vec2::new(angle.sin(), angle.cos()) * 0.31;
        a.part(
            WorkplaceFeature::MillDrive,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                p.x + offset.x,
                2.41,
                p.y + offset.y,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(0.10, 0.48, 0.10))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    hopper(
        a,
        ArchitecturalPlanPoint::from_metres(p + Vec2::new(0.0, -1.2))?,
    )?;
    a.part(
        WorkplaceFeature::Hopper,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 1.26, p.y - 1.26))?,
        CuboidDimensions::from_metres(Vec3::new(0.4, 0.1, 0.12))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    let chute = a.part(
        WorkplaceFeature::Hopper,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 1.16, p.y - 0.65))?,
        CuboidDimensions::from_metres(Vec3::new(0.28, 0.06, 1.35))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    a.orient_part(
        chute,
        RigidRotation::from_quaternion(Quat::from_rotation_x(0.2))?,
    )?;
    // The short flour outlet terminates within the working core rather than in the animal track.
    a.part(
        WorkplaceFeature::Hopper,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 0.73, p.y + 0.77))?,
        CuboidDimensions::from_metres(Vec3::new(0.24, 0.12, 0.3))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}

fn stone_disc(
    a: &mut Assembly<'_>,
    p: ArchitecturalPlanPoint,
    y: Elevation<crate::Architectural>,
    height: PositiveLength,
) -> Result<(), crate::GenerationError> {
    let p = p.metres();
    let y = y.metres();
    let height = height.metres();
    // Grinding surfaces are a single stone, not wall masonry with mortar joints.
    let _: () = for slice in 0..9 {
        let z = (slice as f32 - 4.0) * 0.15;
        let half_width = (0.72_f32.powi(2) - z.powi(2)).sqrt();
        a.part(
            WorkplaceFeature::Millstone,
            WorkplaceMaterial::Millstone,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x, y, p.y + z))?,
            CuboidDimensions::from_metres(Vec3::new(half_width * 2.0, height, 0.15))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}

fn hopper(a: &mut Assembly<'_>, p: ArchitecturalPlanPoint) -> Result<(), crate::GenerationError> {
    let p = p.metres();
    for x in [-0.48, 0.48] {
        for z in [-0.32, 0.32] {
            a.part(
                WorkplaceFeature::Hopper,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(p.x + x, 0.65, p.y + z))?,
                CuboidDimensions::from_metres(Vec3::new(0.12, 1.3, 0.12))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    }
    for x in [-0.5, 0.5] {
        a.part(
            WorkplaceFeature::Hopper,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x + x, 1.58, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.10, 0.6, 0.9))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    for z in [-0.4, 0.4] {
        a.part(
            WorkplaceFeature::Hopper,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 1.58, p.y + z))?,
            CuboidDimensions::from_metres(Vec3::new(0.9, 0.6, 0.10))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    // Paired floor boards leave an actual central feed slot above the inclined chute.
    let _: () = for x in [-0.3, 0.3] {
        a.part(
            WorkplaceFeature::Hopper,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x + x, 1.33, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.30, 0.10, 0.8))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
        a.part(
            WorkplaceFeature::Hopper,
            WorkplaceMaterial::Grain,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x + x, 1.51, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.29, 0.26, 0.78))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}

pub(super) fn grain_bin(
    a: &mut Assembly<'_>,
    p: ArchitecturalPlanPoint,
) -> Result<(), crate::GenerationError> {
    let p = p.metres();
    a.part(
        WorkplaceFeature::StorageBin,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 0.2, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(1.8, 0.12, 1.8))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    for x in [-0.86, 0.86] {
        a.part(
            WorkplaceFeature::StorageBin,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x + x, 0.7, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.08, 1.0, 1.8))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    for z in [-0.86, 0.86] {
        a.part(
            WorkplaceFeature::StorageBin,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 0.7, p.y + z))?,
            CuboidDimensions::from_metres(Vec3::new(1.64, 1.0, 0.08))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.part(
        WorkplaceFeature::StorageBin,
        WorkplaceMaterial::Grain,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 0.56, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(1.62, 0.6, 1.62))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}
