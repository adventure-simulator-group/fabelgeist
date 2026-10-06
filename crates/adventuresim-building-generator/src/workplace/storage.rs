use super::*;
use crate::spatial_geometry::{CuboidDimensions, Position};

pub(super) fn barn(
    a: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> Result<(), crate::GenerationError> {
    let w = dimensions.metres().x;
    let d = dimensions.metres().y;
    for x in [1.4, w - 1.4] {
        for z in [d * 0.3, d * 0.65] {
            bin(
                a,
                crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(x, z))?,
                crate::spatial_geometry::PlanDimensions::from_metres(Vec2::new(2.0, 3.0))?,
                crate::spatial_geometry::PositiveLength::from_metres(1.25)?,
            )?;
        }
    }
    // The tall central threshing passage remains uninterrupted from front to rear.
    rack(
        a,
        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(w + 3.0, d - 2.4))?,
    )?;

    Ok(())
}

pub(super) fn stable(
    a: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> Result<(), crate::GenerationError> {
    let w = dimensions.metres().x;
    let d = dimensions.metres().y;
    for bay in 0..(d / 3.0) as u32 {
        let z = 1.5 + bay as f32 * 3.0;
        a.part(
            WorkplaceFeature::Stall,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(1.8, 0.7, z + 1.25))?,
            CuboidDimensions::from_metres(Vec3::new(3.2, 1.4, 0.1))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
        a.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(3.35, 1.05, z + 1.25))?,
            CuboidDimensions::from_metres(Vec3::new(0.16, 2.1, 0.16))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
        trough(
            a,
            crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(0.8, z))?,
        )?;
    }
    rack(
        a,
        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(w + 3.0, d - 2.4))?,
    )?;

    Ok(())
}

pub(super) fn granary(
    a: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> Result<(), crate::GenerationError> {
    let w = dimensions.metres().x;
    let d = dimensions.metres().y;
    for x in [1.5] {
        for z in [2.5, d * 0.5] {
            bin(
                a,
                crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(x, z))?,
                crate::spatial_geometry::PlanDimensions::from_metres(Vec2::new(2.0, 2.4))?,
                crate::spatial_geometry::PositiveLength::from_metres(1.4)?,
            )?;
        }
    }
    // An elevated rear storage gallery is reached by a real flight with 0.18m risers.
    let loft_start = d - 3.0;
    for x in [0.6, w - 0.6] {
        a.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(x, 1.5, loft_start + 0.2))?,
            CuboidDimensions::from_metres(Vec3::new(0.26, 3.0, 0.26))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.part(
        WorkplaceFeature::Beam,
        WorkplaceMaterial::Timber,
        Position::<crate::Architectural>::from_metres(Vec3::new(w * 0.5, 3.1, loft_start + 0.2))?,
        CuboidDimensions::from_metres(Vec3::new(w - 0.6, 0.2, 0.3))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    a.part(
        WorkplaceFeature::Floor,
        WorkplaceMaterial::Timber,
        Position::<crate::Architectural>::from_metres(Vec3::new(w * 0.5, 3.28, d - 1.5))?,
        CuboidDimensions::from_metres(Vec3::new(w - 0.6, 0.16, 2.8))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    for step in 0..18 {
        let rise = (step + 1) as f32 * (3.36 / 18.0);
        a.part(
            WorkplaceFeature::Stair,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                w - 1.0,
                rise * 0.5,
                loft_start - 4.8 + (step as f32 + 0.5) * (4.8 / 18.0),
            ))?,
            CuboidDimensions::from_metres(Vec3::new(1.2, rise, 4.8 / 18.0))?,
            crate::workplace::WorkplacePartVisibility::DetailOnly,
        )?;
        let z = loft_start - 4.8 + step as f32 * (4.8 / 18.0);
        a.passage(
            WorkplacePassagePurpose::UpperCirculation,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                w - 1.55,
                rise + 0.03,
                z + 0.01,
            ))?,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                w - 0.45,
                rise + 2.0,
                z + 4.8 / 18.0 - 0.01,
            ))?,
        )?;
    }
    for x in [1.4, w * 0.5] {
        bin_at(
            a,
            crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(x, d - 1.5))?,
            crate::spatial_geometry::PlanDimensions::from_metres(Vec2::new(1.6, 1.8))?,
            crate::spatial_geometry::PositiveLength::from_metres(1.3)?,
            crate::spatial_geometry::Elevation::<crate::Architectural>::from_metres(3.36)?,
        )?;
    }
    upper_storage_floor(a, dimensions)?;
    // A freestanding loading frame makes the storage use legible without a decorative false door.
    let z = 1.2;
    for x in [0.7, 2.7] {
        a.part(
            WorkplaceFeature::Hoist,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(x, 2.1, z))?,
            CuboidDimensions::from_metres(Vec3::new(0.22, 4.2, 0.22))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.part(
        WorkplaceFeature::Hoist,
        WorkplaceMaterial::Timber,
        Position::<crate::Architectural>::from_metres(Vec3::new(1.7, 4.31, z))?,
        CuboidDimensions::from_metres(Vec3::new(2.5, 0.22, 0.28))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    a.part(
        WorkplaceFeature::Hoist,
        WorkplaceMaterial::Iron,
        Position::<crate::Architectural>::from_metres(Vec3::new(1.7, 3.3, z))?,
        CuboidDimensions::from_metres(Vec3::new(0.055, 2.0, 0.055))?,
        crate::workplace::WorkplacePartVisibility::DetailOnly,
    )?;

    Ok(())
}

pub(super) fn upper_storage_floor(
    a: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> Result<(), crate::GenerationError> {
    let w = dimensions.metres().x;
    let d = dimensions.metres().y;
    for z in [0.6, d * 0.5, d - 0.6] {
        for x in [0.4, w - 2.0] {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Position::<crate::Architectural>::from_metres(Vec3::new(x, 1.5, z))?,
                CuboidDimensions::from_metres(Vec3::new(0.22, 3.0, 0.22))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        a.part(
            WorkplaceFeature::Beam,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new((w - 1.6) * 0.5, 3.1, z))?,
            CuboidDimensions::from_metres(Vec3::new(w - 1.9, 0.2, 0.25))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.part(
        WorkplaceFeature::Floor,
        WorkplaceMaterial::Timber,
        Position::<crate::Architectural>::from_metres(Vec3::new((w - 1.4) * 0.5, 3.28, d * 0.5))?,
        CuboidDimensions::from_metres(Vec3::new(w - 2.0, 0.16, d - 0.6))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}
