use super::*;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{
    CuboidDimensions, Elevation, PlanDirection, Position, PositiveLength,
};

pub(in super::super) fn build_envelope(
    a: &mut Assembly<'_>,
    program: &BuildingProgram,
) -> Result<(), crate::GenerationError> {
    let (width, depth) = program.footprint.dimensions();
    let w = f32::from(width) * crate::CELL_SIZE_METRES;
    let d = f32::from(depth) * crate::CELL_SIZE_METRES;
    let h = program.storey_height_metres;
    let timber = a.plan.kind == WorkplaceKind::Tannery;
    a.part(
        WorkplaceFeature::Floor,
        WorkplaceMaterial::Masonry,
        Position::<crate::Architectural>::from_metres(Vec3::new(w * 0.5, 0.08, d * 0.5))?,
        CuboidDimensions::from_metres(Vec3::new(w, 0.16, d))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    // An open front bay admits working light while the rear room remains sheltered.
    let porch = 4.2;
    for x in [0.0, w] {
        for z in [0.0, porch] {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(x, (h - 0.24) * 0.5, z))?,
                CuboidDimensions::from_metres(Vec3::new(0.26, h - 0.24, 0.26))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, 0.0))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, porch))?,
            PlanDirection::<crate::Architectural>::from_normalized(if x == 0.0 {
                Vec2::NEG_X
            } else {
                Vec2::X
            })?,
            Elevation::<crate::Architectural>::from_metres(h - 0.24)?,
            PositiveLength::from_metres(0.24)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, porch))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, d))?,
            PlanDirection::<crate::Architectural>::from_normalized(if x == 0.0 {
                Vec2::NEG_X
            } else {
                Vec2::X
            })?,
            Elevation::<crate::Architectural>::from_metres(0.0)?,
            PositiveLength::from_metres(h)?,
            if timber {
                crate::workplace::assembly::WallConstruction::TimberBoards
            } else {
                crate::workplace::assembly::WallConstruction::Masonry
            },
        )?;
    }
    a.wall(
        ArchitecturalPlanPoint::try_from(Vec2::ZERO)?,
        ArchitecturalPlanPoint::try_from(Vec2::new(w, 0.0))?,
        PlanDirection::<crate::Architectural>::from_normalized(Vec2::NEG_Y)?,
        Elevation::<crate::Architectural>::from_metres(h - 0.24)?,
        PositiveLength::from_metres(0.24)?,
        crate::workplace::assembly::WallConstruction::TimberBoards,
    )?;
    a.wall(
        ArchitecturalPlanPoint::try_from(Vec2::new(0.0, d))?,
        ArchitecturalPlanPoint::try_from(Vec2::new(w, d))?,
        PlanDirection::<crate::Architectural>::from_normalized(Vec2::Y)?,
        Elevation::<crate::Architectural>::from_metres(0.0)?,
        PositiveLength::from_metres(h)?,
        if timber {
            crate::workplace::assembly::WallConstruction::TimberBoards
        } else {
            crate::workplace::assembly::WallConstruction::Masonry
        },
    )?;
    for (start, end) in [(0.0, w * 0.5 - 1.7), (w * 0.5 + 1.7, w)] {
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(start, porch))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(end, porch))?,
            PlanDirection::<crate::Architectural>::from_normalized(Vec2::NEG_Y)?,
            Elevation::<crate::Architectural>::from_metres(0.0)?,
            PositiveLength::from_metres(h)?,
            if timber {
                crate::workplace::assembly::WallConstruction::TimberBoards
            } else {
                crate::workplace::assembly::WallConstruction::Masonry
            },
        )?;
    }
    a.wall(
        ArchitecturalPlanPoint::try_from(Vec2::new(w * 0.5 - 1.7, porch))?,
        ArchitecturalPlanPoint::try_from(Vec2::new(w * 0.5 + 1.7, porch))?,
        PlanDirection::<crate::Architectural>::from_normalized(Vec2::NEG_Y)?,
        Elevation::<crate::Architectural>::from_metres(2.55)?,
        PositiveLength::from_metres(h - 2.55)?,
        if timber {
            crate::workplace::assembly::WallConstruction::TimberBoards
        } else {
            crate::workplace::assembly::WallConstruction::Masonry
        },
    )?;
    a.passage(
        WorkplacePassagePurpose::GroundFloorCirculation,
        Position::<crate::Architectural>::from_metres(Vec3::new(w * 0.5 - 1.5, 0.18, 0.0))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(w * 0.5 + 1.5, 2.4, d - 0.35))?,
    )?;
    a.passage(
        WorkplacePassagePurpose::OutdoorRoute,
        Position::<crate::Architectural>::from_metres(Vec3::new(w + 0.35, 0.05, 0.0))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(w + 1.85, 2.4, d))?,
    )?;

    Ok(())
}
