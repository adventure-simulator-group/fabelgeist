use super::*;
use crate::GenerationResult as Result;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{CuboidDimensions, PlanDimensions, Position};

const STACK_BAY_LENGTH_METRES: f32 = 4.5;
const PLANK_THICKNESS_METRES: f32 = 0.12;
const STICKER_THICKNESS_METRES: f32 = 0.1;
const MAX_RACK_SUPPORT_SPAN_METRES: f32 = 3.0;

pub(super) fn timber_yard(assembly: &mut Assembly<'_>, dimensions: PlanDimensions) -> Result<()> {
    let width = dimensions.metres().x;
    let depth = dimensions.metres().y;
    let bays = ((depth - 3.0) / STACK_BAY_LENGTH_METRES) as u32;
    for x in [2.1, width - 2.1] {
        for bay in 0..bays {
            let z = 0.8 + STACK_BAY_LENGTH_METRES * (bay as f32 + 0.5);
            timber_stack(
                assembly,
                ArchitecturalPlanPoint::from_metres(Vec2::new(x, z))?,
                PlanDimensions::from_metres(Vec2::new(2.8, 3.8))?,
                5 + bay % 3,
            )?;
        }
    }
    // Cross-cutting trestles occupy a rear side bay, never the long handling lane.
    saw_bench(
        assembly,
        ArchitecturalPlanPoint::from_metres(Vec2::new(2.1, depth - 1.25))?,
        PlanDimensions::from_metres(Vec2::new(2.7, 1.0))?,
    )?;

    Ok(())
}

pub(super) fn joinery(assembly: &mut Assembly<'_>, dimensions: PlanDimensions) -> Result<()> {
    let width = dimensions.metres().x;
    let depth = dimensions.metres().y;
    workpiece_on_trestles(
        assembly,
        ArchitecturalPlanPoint::from_metres(Vec2::new(1.7, 2.2))?,
    )?;
    for x in [1.7, width - 1.7] {
        if x > width * 0.5 {
            saw_bench(
                assembly,
                ArchitecturalPlanPoint::from_metres(Vec2::new(x, 2.0))?,
                PlanDimensions::from_metres(Vec2::new(2.2, 2.8))?,
            )?;
        }
        timber_stack(
            assembly,
            ArchitecturalPlanPoint::from_metres(Vec2::new(x, depth - 2.7))?,
            PlanDimensions::from_metres(Vec2::new(2.2, 3.5))?,
            4,
        )?;
    }
    // Wide shallow shelves on grounded standards hold shorter joinery stock.
    let rack_bays = ((depth - 7.0) / MAX_RACK_SUPPORT_SPAN_METRES).ceil() as u32;
    for bay in 0..=rack_bays {
        let z = 6.0 + (depth - 7.0) * bay as f32 / rack_bays as f32;
        for x in [0.45, width - 0.45] {
            assembly.part(
                WorkplaceFeature::Rack,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(x, 1.4, z))?,
                CuboidDimensions::from_metres(Vec3::new(0.18, 2.8, 0.18))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    }
    let _: () = for x in [0.65, width - 0.65] {
        for height in [1.8, 2.5] {
            assembly.part(
                WorkplaceFeature::Rack,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    x,
                    height,
                    (6.0 + depth - 1.0) * 0.5,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.6, 0.12, depth - 7.0))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    };
    Ok(())
}

fn workpiece_on_trestles(
    assembly: &mut Assembly<'_>,
    centre: ArchitecturalPlanPoint,
) -> Result<()> {
    let centre = centre.metres();
    // A long squared beam is raised across two separate trestles for marking and joinery.
    for z in [-1.1, 1.1] {
        for x in [-0.65, 0.65] {
            assembly.part(
                WorkplaceFeature::SawBench,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    centre.x + x,
                    0.55,
                    centre.y + z,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.18, 0.78, 0.4))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        assembly.part(
            WorkplaceFeature::SawBench,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 1.02, centre.y + z))?,
            CuboidDimensions::from_metres(Vec3::new(1.6, 0.16, 0.3))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    assembly.part(
        WorkplaceFeature::SawBench,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 1.26, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(0.42, 0.32, 3.9))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}

fn timber_stack(
    assembly: &mut Assembly<'_>,
    centre: ArchitecturalPlanPoint,
    dimensions: PlanDimensions,
    tiers: u32,
) -> Result<()> {
    let centre = centre.metres();
    let width = dimensions.metres().x;
    let length = dimensions.metres().y;
    for z in [-0.35, 0.35] {
        assembly.part(
            WorkplaceFeature::TimberStack,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x,
                0.24,
                centre.y + z * length,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(width, 0.24, 0.22))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    let _: () = for tier in 0..tiers {
        let bottom = 0.36 + tier as f32 * (PLANK_THICKNESS_METRES + STICKER_THICKNESS_METRES);
        for plank in 0..4 {
            let x = centre.x - width * 0.5 + (plank as f32 + 0.5) * width * 0.25;
            assembly.part(
                WorkplaceFeature::TimberStack,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    x,
                    bottom + PLANK_THICKNESS_METRES * 0.5,
                    centre.y,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(
                    width * 0.25 - 0.04,
                    PLANK_THICKNESS_METRES,
                    length,
                ))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        if tier + 1 < tiers {
            for z in [-0.35, 0.35] {
                assembly.part(
                    WorkplaceFeature::TimberStack,
                    WorkplaceMaterial::UnpaintedTimber,
                    Position::<crate::Architectural>::from_metres(Vec3::new(
                        centre.x,
                        bottom + PLANK_THICKNESS_METRES + STICKER_THICKNESS_METRES * 0.5,
                        centre.y + z * length,
                    ))?,
                    CuboidDimensions::from_metres(Vec3::new(width, STICKER_THICKNESS_METRES, 0.1))?,
                    crate::workplace::WorkplacePartVisibility::Silhouette,
                )?;
            }
        }
    };
    Ok(())
}

fn saw_bench(
    assembly: &mut Assembly<'_>,
    centre: ArchitecturalPlanPoint,
    size: PlanDimensions,
) -> Result<()> {
    let centre = centre.metres();
    let size = size.metres();
    for x in [-0.35, 0.35] {
        for z in [-0.35, 0.35] {
            assembly.part(
                WorkplaceFeature::SawBench,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    centre.x + size.x * x,
                    0.57,
                    centre.y + size.y * z,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.18, 0.82, 0.18))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    }
    assembly.part(
        WorkplaceFeature::SawBench,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 1.06, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(size.x, 0.16, size.y))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    assembly.part(
        WorkplaceFeature::SawBench,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 1.21, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(size.x * 0.65, 0.14, 0.3))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}
