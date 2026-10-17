use super::*;
use crate::GenerationResult as Result;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{CuboidDimensions, Position, RigidRotation};

pub(super) fn tank(assembly: &mut Assembly<'_>, centre: ArchitecturalPlanPoint) -> Result<()> {
    let centre = centre.metres();
    let feature = WorkplaceFeature::SoakingTank;
    assembly.part(
        feature,
        WorkplaceMaterial::Masonry,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.09, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(2.2, 0.18, 2.0))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    for x in [-1.0, 1.0] {
        assembly.part(
            feature,
            WorkplaceMaterial::Masonry,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x + x, 0.57, centre.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.2, 0.96, 2.0))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    for z in [-0.9, 0.9] {
        assembly.part(
            feature,
            WorkplaceMaterial::Masonry,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.57, centre.y + z))?,
            CuboidDimensions::from_metres(Vec3::new(1.8, 0.96, 0.2))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    // Rendered process water rests on the physical basin floor. It is explicitly non-colliding.
    assembly.part(
        WorkplaceFeature::ProcessLiquid,
        WorkplaceMaterial::ProcessLiquid,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.5, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(1.79, 0.64, 1.59))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}

pub(super) fn dye_kettle(
    assembly: &mut Assembly<'_>,
    centre: ArchitecturalPlanPoint,
) -> Result<()> {
    let centre = centre.metres();
    // A masonry heating enclosure supports an open metal-lined vessel above a firing mouth.
    for x in [-0.75, 0.75] {
        assembly.part(
            WorkplaceFeature::DyeKettle,
            WorkplaceMaterial::Masonry,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x + x, 0.4, centre.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.5, 0.8, 2.0))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    assembly.part(
        WorkplaceFeature::DyeKettle,
        WorkplaceMaterial::Masonry,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.4, centre.y + 0.75))?,
        CuboidDimensions::from_metres(Vec3::new(1.0, 0.8, 0.5))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    assembly.part(
        WorkplaceFeature::DyeKettle,
        WorkplaceMaterial::Iron,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.86, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(2.0, 0.12, 2.0))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    for x in [-0.93, 0.93] {
        assembly.part(
            WorkplaceFeature::DyeKettle,
            WorkplaceMaterial::Iron,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x + x, 1.18, centre.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.14, 0.64, 2.0))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    for z in [-0.93, 0.93] {
        assembly.part(
            WorkplaceFeature::DyeKettle,
            WorkplaceMaterial::Iron,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 1.18, centre.y + z))?,
            CuboidDimensions::from_metres(Vec3::new(1.72, 0.64, 0.14))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    assembly.part(
        WorkplaceFeature::ProcessLiquid,
        WorkplaceMaterial::ProcessLiquid,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 1.12, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(1.7, 0.4, 1.7))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    // A timber lifting rail allows cloth to drain back over the heated vessel.
    for x in [-1.1, 1.1] {
        assembly.part(
            WorkplaceFeature::DyeKettle,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x + x,
                1.2,
                centre.y + 0.85,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(0.16, 2.4, 0.16))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    assembly.part(
        WorkplaceFeature::DyeKettle,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 2.48, centre.y + 0.85))?,
        CuboidDimensions::from_metres(Vec3::new(2.5, 0.16, 0.18))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}

pub(super) fn fleshing_beam(
    assembly: &mut Assembly<'_>,
    centre: ArchitecturalPlanPoint,
) -> Result<()> {
    let centre = centre.metres();
    let pitch = 20.0_f32.to_radians();
    for z in [-0.9, 0.9] {
        let bearing_height = 1.15 - pitch.tan() * z - 0.09 / pitch.cos();
        for x in [-0.5, 0.5] {
            assembly.part(
                WorkplaceFeature::FleshingBeam,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    centre.x + x,
                    bearing_height * 0.5,
                    centre.y + z,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.16, bearing_height, 0.2))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        assembly.part(
            WorkplaceFeature::FleshingBeam,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x,
                bearing_height + 0.06,
                centre.y + z,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(1.2, 0.12, 0.2))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    let beam = assembly.part(
        WorkplaceFeature::FleshingBeam,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 1.27, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(0.5, 0.18, 3.0))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    // The assembly rotation API rebuilds support contacts for the sloped working surface.
    assembly.orient_part(
        beam,
        RigidRotation::from_quaternion(bevy::math::Quat::from_rotation_x(pitch))?,
    )?;

    Ok(())
}
