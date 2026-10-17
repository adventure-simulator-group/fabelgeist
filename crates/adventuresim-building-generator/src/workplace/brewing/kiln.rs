//! Enclosed malt kiln: fire chamber, open drying grate, loading hatch and hollow low hood.
use super::super::components::RecipeComponent;
use super::*;
use crate::GenerationResult as Result;
use crate::spatial_geometry::Displacement;
use crate::spatial_geometry::{CuboidDimensions, Position};

pub(super) fn drying_kiln(
    assembly: &mut Assembly<'_>,
    centre: crate::plan_geometry::ArchitecturalPlanPoint,
) -> Result<()> {
    let centre = centre.metres();
    for x in [-1.0, 1.0] {
        masonry(
            assembly,
            crate::plan_geometry::ArchitecturalPlanPoint::from_metres(centre)?,
            crate::spatial_geometry::Displacement::<crate::Architectural>::from_metres(Vec3::new(
                x, 1.2, 0.0,
            ))?,
            crate::spatial_geometry::CuboidDimensions::from_metres(Vec3::new(0.3, 2.4, 2.6))?,
        )?;
    }
    masonry(
        assembly,
        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(centre)?,
        crate::spatial_geometry::Displacement::<crate::Architectural>::from_metres(Vec3::new(
            0.0, 1.2, 1.15,
        ))?,
        crate::spatial_geometry::CuboidDimensions::from_metres(Vec3::new(1.7, 2.4, 0.3))?,
    )?;
    // Separate firing and malt-loading apertures are real holes, not painted rectangles.
    for x in [-0.75, 0.75] {
        masonry(
            assembly,
            crate::plan_geometry::ArchitecturalPlanPoint::from_metres(centre)?,
            crate::spatial_geometry::Displacement::<crate::Architectural>::from_metres(Vec3::new(
                x, 0.45, -1.15,
            ))?,
            crate::spatial_geometry::CuboidDimensions::from_metres(Vec3::new(0.8, 0.9, 0.3))?,
        )?;
    }
    for component in [
        RecipeComponent {
            offset: Displacement::from_metres(Vec3::new(0.0, 1.15, -1.15))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(2.3, 0.5, 0.3))?,
        },
        RecipeComponent {
            offset: Displacement::from_metres(Vec3::new(0.0, 2.25, -1.15))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(2.3, 0.3, 0.3))?,
        },
    ] {
        masonry(
            assembly,
            crate::plan_geometry::ArchitecturalPlanPoint::from_metres(centre)?,
            component.offset,
            component.dimensions,
        )?;
    }
    for x in [-0.85, 0.85] {
        masonry(
            assembly,
            crate::plan_geometry::ArchitecturalPlanPoint::from_metres(centre)?,
            crate::spatial_geometry::Displacement::<crate::Architectural>::from_metres(Vec3::new(
                x, 1.75, -1.15,
            ))?,
            crate::spatial_geometry::CuboidDimensions::from_metres(Vec3::new(0.6, 0.7, 0.3))?,
        )?;
    }
    drying_grate(
        assembly,
        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(centre)?,
    )?;
    // Each inward-stepped course overlaps the one beneath; the centre stays open to the sky.
    for tier in 0..4 {
        let outer = Vec2::new(2.3, 2.6) - Vec2::splat(tier as f32 * 0.28);
        hood_course(
            assembly,
            crate::plan_geometry::ArchitecturalPlanPoint::from_metres(centre)?,
            crate::spatial_geometry::PlanDimensions::from_metres(outer)?,
            crate::spatial_geometry::Elevation::<crate::Architectural>::from_metres(
                2.4 + tier as f32 * 0.2,
            )?,
            crate::spatial_geometry::PositiveLength::from_metres(0.2)?,
        )?;
    }
    hood_course(
        assembly,
        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(centre)?,
        crate::spatial_geometry::PlanDimensions::from_metres(Vec2::new(1.18, 1.48))?,
        crate::spatial_geometry::Elevation::<crate::Architectural>::from_metres(3.2)?,
        crate::spatial_geometry::PositiveLength::from_metres(0.6)?,
    )?;
    assembly.passage(
        WorkplacePassagePurpose::ServiceClearance,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            centre.x - 0.25,
            0.05,
            centre.y - 1.32,
        ))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            centre.x + 0.25,
            0.8,
            centre.y + 0.85,
        ))?,
    )?;
    // A reserved continuous vertical vent between the grate bars also guards later roof edits.
    assembly.passage(
        WorkplacePassagePurpose::ServiceClearance,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            centre.x - 0.12,
            0.8,
            centre.y - 0.1,
        ))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            centre.x + 0.12,
            4.0,
            centre.y + 0.1,
        ))?,
    )?;

    Ok(())
}

fn drying_grate(
    assembly: &mut Assembly<'_>,
    centre: crate::plan_geometry::ArchitecturalPlanPoint,
) -> Result<()> {
    let centre = centre.metres();
    let _: () = for z in [-0.9, -0.6, -0.3, 0.3, 0.6, 0.9] {
        assembly.part(
            WorkplaceFeature::Kiln,
            WorkplaceMaterial::Iron,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 1.44, centre.y + z))?,
            CuboidDimensions::from_metres(Vec3::new(2.0, 0.12, 0.12))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
        assembly.part(
            WorkplaceFeature::StorageBin,
            WorkplaceMaterial::Grain,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 1.54, centre.y + z))?,
            CuboidDimensions::from_metres(Vec3::new(1.65, 0.08, 0.1))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}

fn hood_course(
    assembly: &mut Assembly<'_>,
    centre: crate::plan_geometry::ArchitecturalPlanPoint,
    outer: crate::spatial_geometry::PlanDimensions,
    base: crate::spatial_geometry::Elevation<crate::Architectural>,
    height: crate::spatial_geometry::PositiveLength,
) -> Result<()> {
    let centre = centre.metres();
    let outer = outer.metres();
    let base = base.metres();
    let height = height.metres();
    let thickness = 0.26;
    for x in [-0.5, 0.5] {
        assembly.part(
            WorkplaceFeature::Flue,
            WorkplaceMaterial::Masonry,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x + x * (outer.x - thickness),
                base + height * 0.5,
                centre.y,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(thickness, height, outer.y))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    let _: () = for z in [-0.5, 0.5] {
        assembly.part(
            WorkplaceFeature::Flue,
            WorkplaceMaterial::Masonry,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x,
                base + height * 0.5,
                centre.y + z * (outer.y - thickness),
            ))?,
            CuboidDimensions::from_metres(Vec3::new(outer.x - thickness * 2.0, height, thickness))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}

fn masonry(
    assembly: &mut Assembly<'_>,
    centre: crate::plan_geometry::ArchitecturalPlanPoint,
    offset: crate::spatial_geometry::Displacement<crate::Architectural>,
    size: crate::spatial_geometry::CuboidDimensions,
) -> Result<()> {
    let centre = centre.metres();
    let offset = offset.metres();
    assembly.part(
        WorkplaceFeature::Kiln,
        WorkplaceMaterial::Masonry,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.0, centre.y) + offset)?,
        size,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}
