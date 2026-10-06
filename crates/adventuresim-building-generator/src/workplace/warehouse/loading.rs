use super::*;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{
    CuboidDimensions, Elevation, PlanDirection, Position, PositiveLength,
};

pub(super) fn loading_hood(
    a: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> Result<(), crate::GenerationError> {
    let w = dimensions.metres().x;
    let d = dimensions.metres().y;
    let z = d * 0.25;
    for x in [w + 0.8, w + 4.8] {
        for end in [z - 2.2, z + 2.2] {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Position::<crate::Architectural>::from_metres(Vec3::new(x, 1.7, end))?,
                CuboidDimensions::from_metres(Vec3::new(0.3, 3.4, 0.3))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    }
    for end in [z - 2.2, z + 2.2] {
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(w + 0.8, end))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(w + 4.8, end))?,
            PlanDirection::<crate::Architectural>::from_normalized(if end < z {
                Vec2::NEG_Y
            } else {
                Vec2::Y
            })?,
            Elevation::<crate::Architectural>::from_metres(3.4)?,
            PositiveLength::from_metres(0.3)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
    }
    for x in [w + 0.8, w + 4.8] {
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, z - 2.2))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, z + 2.2))?,
            PlanDirection::<crate::Architectural>::from_normalized(if x < w + 2.8 {
                Vec2::NEG_X
            } else {
                Vec2::X
            })?,
            Elevation::<crate::Architectural>::from_metres(3.4)?,
            PositiveLength::from_metres(0.3)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
    }
    // A grounded lifting frame occupies the rear edge of the hood, beside the clear cart path.
    for x in [w + 1.1, w + 4.5] {
        a.part(
            WorkplaceFeature::LoadingHoist,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(x, 1.5, z + 1.55))?,
            CuboidDimensions::from_metres(Vec3::new(0.24, 3.0, 0.24))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.part(
        WorkplaceFeature::LoadingHoist,
        WorkplaceMaterial::Timber,
        Position::<crate::Architectural>::from_metres(Vec3::new(w + 2.8, 3.12, z + 1.55))?,
        CuboidDimensions::from_metres(Vec3::new(3.8, 0.24, 0.3))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    super::hoist::rig(
        a,
        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(w + 2.8, z + 1.55))?,
    )?;

    Ok(())
}
