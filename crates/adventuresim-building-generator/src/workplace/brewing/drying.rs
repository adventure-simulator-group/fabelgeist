use super::*;
use crate::GenerationResult as Result;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{
    CuboidDimensions, Elevation, PlanDirection, Position, PositiveLength,
};

pub(in super::super) fn drying_wall(
    assembly: &mut Assembly<'_>,
    x: f32,
    depth: f32,
    h: f32,
    outward: Vec2,
) -> Result<()> {
    let bays = (depth / 2.5) as u32;
    let length = depth / bays as f32;
    let _: () = for bay in 0..bays {
        let start = bay as f32 * length;
        let left = start + length * 0.5 - 0.7;
        let right = left + 1.4;
        for (from, to) in [(start, left), (right, start + length)] {
            assembly.wall(
                ArchitecturalPlanPoint::try_from(Vec2::new(x, from))?,
                ArchitecturalPlanPoint::try_from(Vec2::new(x, to))?,
                PlanDirection::<crate::Architectural>::from_normalized(outward)?,
                Elevation::<crate::Architectural>::from_metres(0.0)?,
                PositiveLength::from_metres(h)?,
                crate::workplace::assembly::WallConstruction::Masonry,
            )?;
        }
        for (base, height) in [(0.0, 1.6), (2.6, h - 2.6)] {
            assembly.wall(
                ArchitecturalPlanPoint::try_from(Vec2::new(x, left))?,
                ArchitecturalPlanPoint::try_from(Vec2::new(x, right))?,
                PlanDirection::<crate::Architectural>::from_normalized(outward)?,
                Elevation::<crate::Architectural>::from_metres(base)?,
                PositiveLength::from_metres(height)?,
                crate::workplace::assembly::WallConstruction::Masonry,
            )?;
        }
        // Framed horizontal slats leave real ventilation slots through the masonry wall.
        for z in [left, right] {
            assembly.part(
                WorkplaceFeature::Louver,
                WorkplaceMaterial::Timber,
                Position::<crate::Architectural>::from_metres(Vec3::new(x, 2.1, z))?,
                CuboidDimensions::from_metres(Vec3::new(0.5, 1.04, 0.09))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        for row in 0..4 {
            assembly.part(
                WorkplaceFeature::Louver,
                WorkplaceMaterial::Timber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    x,
                    1.71 + row as f32 * 0.25,
                    (left + right) * 0.5,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.5, 0.12, 1.4))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    };
    Ok(())
}

pub(in super::super) fn malthouse(
    assembly: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> Result<()> {
    let width = dimensions.metres().x;
    let depth = dimensions.metres().y;
    super::kiln::drying_kiln(
        assembly,
        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(
            width + 3.0,
            depth - 2.6,
        ))?,
    )?;
    let _: () = for x in [2.2, width - 2.2] {
        for z in [2.4, depth * 0.5, depth - 2.4] {
            drying_bed(
                assembly,
                crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(x, z))?,
            )?;
        }
    };
    Ok(())
}

fn drying_bed(
    assembly: &mut Assembly<'_>,
    centre: crate::plan_geometry::ArchitecturalPlanPoint,
) -> Result<()> {
    let centre = centre.metres();
    for x in [-0.8, 0.8] {
        assembly.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x + x, 0.3, centre.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.16, 0.6, 2.0))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    assembly.part(
        WorkplaceFeature::Rack,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.68, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(2.0, 0.16, 2.0))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    for x in [-0.98, 0.98] {
        assembly.part(
            WorkplaceFeature::Rack,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x + x, 0.85, centre.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.08, 0.18, 2.0))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    for z in [-0.98, 0.98] {
        assembly.part(
            WorkplaceFeature::Rack,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.85, centre.y + z))?,
            CuboidDimensions::from_metres(Vec3::new(1.88, 0.18, 0.08))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    assembly.part(
        WorkplaceFeature::StorageBin,
        WorkplaceMaterial::Grain,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.79, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(1.84, 0.06, 1.84))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}
