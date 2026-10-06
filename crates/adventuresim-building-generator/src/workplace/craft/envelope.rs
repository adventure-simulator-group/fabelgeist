use super::*;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{
    CuboidDimensions, Elevation, PlanDirection, Position, PositiveLength,
};

const POST_SECTION_METRES: f32 = 0.28;
const WALL_PLATE_DEPTH_METRES: f32 = 0.3;
const STRUCTURAL_BAY_METRES: f32 = 3.0;
pub(super) const JOINERY_PORCH_DEPTH_METRES: f32 = 4.5;
const HANDLING_LANE_HALF_WIDTH_METRES: f32 = 1.6;

pub(in super::super) fn build_envelope(
    a: &mut Assembly<'_>,
    program: &BuildingProgram,
) -> Result<()> {
    let (width, depth) = program.footprint.dimensions();
    let width = f32::from(width) * crate::CELL_SIZE_METRES;
    let depth = f32::from(depth) * crate::CELL_SIZE_METRES;
    let h = program.storey_height_metres;
    a.part(
        WorkplaceFeature::Floor,
        WorkplaceMaterial::Masonry,
        Position::<crate::Architectural>::from_metres(Vec3::new(width * 0.5, 0.08, depth * 0.5))?,
        CuboidDimensions::from_metres(Vec3::new(width, 0.16, depth))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    perimeter_frame(a, width, depth, h)?;
    if a.plan.kind == WorkplaceKind::Carpenter {
        joinery_walls(a, width, depth, h)?;
    } else {
        // The low rear windbreak leaves both long sides open for air-drying.
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(0.0, depth))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(width, depth))?,
            PlanDirection::<crate::Architectural>::from_normalized(Vec2::Y)?,
            Elevation::<crate::Architectural>::from_metres(0.0)?,
            PositiveLength::from_metres(1.1)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
    }
    a.passage(
        WorkplacePassagePurpose::GroundFloorCirculation,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            width * 0.5 - HANDLING_LANE_HALF_WIDTH_METRES,
            0.18,
            0.0,
        ))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            width * 0.5 + HANDLING_LANE_HALF_WIDTH_METRES,
            2.7,
            depth - 0.35,
        ))?,
    )?;

    Ok(())
}

fn perimeter_frame(a: &mut Assembly<'_>, width: f32, depth: f32, h: f32) -> Result<()> {
    let bays = (depth / STRUCTURAL_BAY_METRES).ceil() as u32;
    for x in [0.0, width] {
        for bay in 0..=bays {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    x,
                    (h - WALL_PLATE_DEPTH_METRES) * 0.5,
                    depth * bay as f32 / bays as f32,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(
                    POST_SECTION_METRES,
                    h - WALL_PLATE_DEPTH_METRES,
                    POST_SECTION_METRES,
                ))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, 0.0))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, depth))?,
            PlanDirection::<crate::Architectural>::from_normalized(if x == 0.0 {
                Vec2::NEG_X
            } else {
                Vec2::X
            })?,
            Elevation::<crate::Architectural>::from_metres(h - WALL_PLATE_DEPTH_METRES)?,
            PositiveLength::from_metres(WALL_PLATE_DEPTH_METRES)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
    }
    let _: () = for z in [0.0, depth] {
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(0.0, z))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(width, z))?,
            PlanDirection::<crate::Architectural>::from_normalized(if z == 0.0 {
                Vec2::NEG_Y
            } else {
                Vec2::Y
            })?,
            Elevation::<crate::Architectural>::from_metres(h - WALL_PLATE_DEPTH_METRES)?,
            PositiveLength::from_metres(WALL_PLATE_DEPTH_METRES)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
    };
    Ok(())
}

fn joinery_walls(a: &mut Assembly<'_>, width: f32, depth: f32, h: f32) -> Result<()> {
    let porch = JOINERY_PORCH_DEPTH_METRES;
    for x in [0.0, width] {
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, porch))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, depth))?,
            PlanDirection::<crate::Architectural>::from_normalized(if x == 0.0 {
                Vec2::NEG_X
            } else {
                Vec2::X
            })?,
            Elevation::<crate::Architectural>::from_metres(0.0)?,
            PositiveLength::from_metres(h)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
        // Boarding seams express the enclosed shop while the front working bay stays open.
        for board in 0..((depth - porch) / 0.24) as u32 {
            a.part(
                WorkplaceFeature::Boarding,
                WorkplaceMaterial::Timber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    x + if x == 0.0 { -0.1 } else { 0.1 },
                    h * 0.5,
                    porch + (board as f32 + 0.5) * 0.24,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.035, h, 0.21))?,
                crate::workplace::WorkplacePartVisibility::DetailOnly,
            )?;
        }
    }
    a.wall(
        ArchitecturalPlanPoint::try_from(Vec2::new(0.0, depth))?,
        ArchitecturalPlanPoint::try_from(Vec2::new(width, depth))?,
        PlanDirection::<crate::Architectural>::from_normalized(Vec2::Y)?,
        Elevation::<crate::Architectural>::from_metres(0.0)?,
        PositiveLength::from_metres(h)?,
        crate::workplace::assembly::WallConstruction::TimberBoards,
    )?;
    let left = width * 0.5 - 1.85;
    let right = width - left;
    for (start, end) in [(0.0, left), (right, width)] {
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(start, porch))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(end, porch))?,
            PlanDirection::<crate::Architectural>::from_normalized(Vec2::NEG_Y)?,
            Elevation::<crate::Architectural>::from_metres(0.0)?,
            PositiveLength::from_metres(h)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
    }
    a.wall(
        ArchitecturalPlanPoint::try_from(Vec2::new(left, porch))?,
        ArchitecturalPlanPoint::try_from(Vec2::new(right, porch))?,
        PlanDirection::<crate::Architectural>::from_normalized(Vec2::NEG_Y)?,
        Elevation::<crate::Architectural>::from_metres(2.9)?,
        PositiveLength::from_metres(h - 2.9)?,
        crate::workplace::assembly::WallConstruction::TimberBoards,
    )?;
    // An aisle-aligned doorway connects the covered work bay to the enclosed rear shop.
    let _: () = for x in [left, right] {
        a.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(x, 1.45, porch))?,
            CuboidDimensions::from_metres(Vec3::new(0.24, 2.9, 0.28))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}
