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
    a.part(
        WorkplaceFeature::Floor,
        WorkplaceMaterial::Masonry,
        Position::<crate::Architectural>::from_metres(Vec3::new(w * 0.5, 0.08, d * 0.5))?,
        CuboidDimensions::from_metres(Vec3::new(w, 0.16, d))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    let bays = (d / 3.0).ceil() as u32;
    for x in [0.0, w] {
        for bay in 0..=bays {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    x,
                    (h - 0.25) * 0.5,
                    d * bay as f32 / bays as f32,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.28, h - 0.25, 0.28))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, 0.0))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, d))?,
            PlanDirection::<crate::Architectural>::from_normalized(if x == 0.0 {
                Vec2::NEG_X
            } else {
                Vec2::X
            })?,
            Elevation::<crate::Architectural>::from_metres(h - 0.25)?,
            PositiveLength::from_metres(0.25)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
        // The front two bays expose the drive; rear weatherboarding shelters grain.
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, 8.5))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, d))?,
            PlanDirection::<crate::Architectural>::from_normalized(if x == 0.0 {
                Vec2::NEG_X
            } else {
                Vec2::X
            })?,
            Elevation::<crate::Architectural>::from_metres(0.0)?,
            PositiveLength::from_metres(h)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
    }
    for x in [4.0, 8.0] {
        a.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(x, (h - 0.25) * 0.5, 0.0))?,
            CuboidDimensions::from_metres(Vec3::new(0.3, h - 0.25, 0.3))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.wall(
        ArchitecturalPlanPoint::try_from(Vec2::ZERO)?,
        ArchitecturalPlanPoint::try_from(Vec2::new(w, 0.0))?,
        PlanDirection::<crate::Architectural>::from_normalized(Vec2::NEG_Y)?,
        Elevation::<crate::Architectural>::from_metres(h - 0.25)?,
        PositiveLength::from_metres(0.25)?,
        crate::workplace::assembly::WallConstruction::TimberBoards,
    )?;
    a.wall(
        ArchitecturalPlanPoint::try_from(Vec2::new(0.0, d))?,
        ArchitecturalPlanPoint::try_from(Vec2::new(w, d))?,
        PlanDirection::<crate::Architectural>::from_normalized(Vec2::Y)?,
        Elevation::<crate::Architectural>::from_metres(0.0)?,
        PositiveLength::from_metres(h)?,
        crate::workplace::assembly::WallConstruction::TimberBoards,
    )?;
    a.passage(
        WorkplacePassagePurpose::GroundFloorCirculation,
        Position::<crate::Architectural>::from_metres(Vec3::new(0.2, 0.18, 0.0))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(1.7, 2.4, d - 0.3))?,
    )?;
    a.passage(
        WorkplacePassagePurpose::GroundFloorCirculation,
        Position::<crate::Architectural>::from_metres(Vec3::new(1.7, 0.18, 11.0))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(10.8, 2.4, 12.0))?,
    )?;
    a.passage(
        WorkplacePassagePurpose::GroundFloorCirculation,
        Position::<crate::Architectural>::from_metres(Vec3::new(5.2, 0.18, 11.0))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(7.6, 2.4, d - 0.3))?,
    )?;

    Ok(())
}
