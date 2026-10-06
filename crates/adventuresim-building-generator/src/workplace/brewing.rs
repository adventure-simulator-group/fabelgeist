//! Small preindustrial brewing yards and ventilated malt-drying houses.
use super::{assembly::Assembly, *};
use crate::GenerationResult;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{
    CuboidDimensions, Elevation, PlanDirection, Position, PositiveLength,
};
use crate::{GableProfile, RidgeAxis, RoofKind, RoofPiece};

mod drying;
mod kiln;
#[cfg(test)]
mod tests;
mod vessels;
pub(super) use drying::{drying_wall, malthouse};

/// The service shelter occupies the reserved side yard, leaving its rear hearth in open air.
pub(super) fn service_roof(main: Vec2) -> RoofPiece {
    RoofPiece {
        kind: RoofKind::Gable,
        centre: Vec2::new(main.x + 3.6, (main.y - 2.8) * 0.5),
        size: Vec2::new(3.2, main.y - 4.0),
        base_height_metres: 2.8,
        pitch_degrees: 25.0,
        ridge_axis: RidgeAxis::Z,
        eave_metres: 0.18,
        gable_profile: GableProfile::Plain,
    }
}

pub(super) fn brewery(
    assembly: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> GenerationResult<()> {
    let width = dimensions.metres().x;
    let depth = dimensions.metres().y;
    service_frame(assembly, dimensions)?;
    for z in [2.1, depth * 0.5 - 0.7] {
        vessels::vat(
            assembly,
            crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(width + 3.6, z))?,
            crate::spatial_geometry::PositiveLength::from_metres(0.95)?,
            crate::spatial_geometry::PositiveLength::from_metres(1.25)?,
        )?;
    }
    vessels::vat(
        assembly,
        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(2.0, 2.2))?,
        crate::spatial_geometry::PositiveLength::from_metres(1.05)?,
        crate::spatial_geometry::PositiveLength::from_metres(1.4)?,
    )?;
    for z in [depth * 0.55, depth - 2.2] {
        vessels::vat(
            assembly,
            crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(2.0, z))?,
            crate::spatial_geometry::PositiveLength::from_metres(1.05)?,
            crate::spatial_geometry::PositiveLength::from_metres(1.4)?,
        )?;
    }
    brewing_bench(
        assembly,
        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(
            width - 2.1,
            depth - 3.0,
        ))?,
    )?;
    // Broad masonry shoulders around an open firing mouth, with a continuous rear flue.
    hearth(
        assembly,
        crate::plan_geometry::ArchitecturalPlanPoint::from_metres(Vec2::new(
            width + 3.6,
            depth - 1.65,
        ))?,
        crate::spatial_geometry::Elevation::<crate::Architectural>::from_metres(4.25)?,
    )?;
    assembly.passage(
        WorkplacePassagePurpose::OutdoorRoute,
        Position::<crate::Architectural>::from_metres(Vec3::new(width + 2.2, 0.05, depth - 3.0))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(width + 5.0, 2.3, depth - 2.85))?,
    )?;

    Ok(())
}

fn brewing_bench(
    assembly: &mut Assembly<'_>,
    centre: crate::plan_geometry::ArchitecturalPlanPoint,
) -> GenerationResult<()> {
    let centre = centre.metres();
    for x in [-0.8, 0.8] {
        for z in [-1.2, 1.2] {
            assembly.part(
                WorkplaceFeature::Counter,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    centre.x + x,
                    0.45,
                    centre.y + z,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.18, 0.9, 0.18))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    }
    assembly.part(
        WorkplaceFeature::Counter,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.98, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(2.0, 0.16, 2.8))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    // A small open rinsing vessel on the working bench gives the surface a clear use.
    let _: () = for component in [
        crate::workplace::components::RecipeComponent {
            offset: crate::spatial_geometry::Displacement::from_metres(Vec3::new(0.0, 0.0, 0.0))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(1.3, 0.08, 1.3))?,
        },
        crate::workplace::components::RecipeComponent {
            offset: crate::spatial_geometry::Displacement::from_metres(Vec3::new(-0.61, 0.2, 0.0))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(0.08, 0.4, 1.3))?,
        },
        crate::workplace::components::RecipeComponent {
            offset: crate::spatial_geometry::Displacement::from_metres(Vec3::new(0.61, 0.2, 0.0))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(0.08, 0.4, 1.3))?,
        },
        crate::workplace::components::RecipeComponent {
            offset: crate::spatial_geometry::Displacement::from_metres(Vec3::new(0.0, 0.2, -0.61))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(1.3, 0.4, 0.08))?,
        },
        crate::workplace::components::RecipeComponent {
            offset: crate::spatial_geometry::Displacement::from_metres(Vec3::new(0.0, 0.2, 0.61))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(1.3, 0.4, 0.08))?,
        },
    ] {
        assembly.part(
            WorkplaceFeature::Trough,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(
                Vec3::new(centre.x, 1.1, centre.y) + component.offset.metres(),
            )?,
            component.dimensions,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}

fn service_frame(
    assembly: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> GenerationResult<()> {
    let width = dimensions.metres().x;
    let depth = dimensions.metres().y;
    let front = 0.6;
    let back = depth - 3.4;
    let bays = ((back - front) / 3.0).ceil() as u32;
    for x in [width + 2.0, width + 5.2] {
        for bay in 0..=bays {
            let z = front + (back - front) * bay as f32 / bays as f32;
            assembly.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Position::<crate::Architectural>::from_metres(Vec3::new(x, 1.3, z))?,
                CuboidDimensions::from_metres(Vec3::new(0.24, 2.6, 0.24))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        assembly.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, front))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, back))?,
            PlanDirection::<crate::Architectural>::from_normalized(if x < width + 3.0 {
                Vec2::NEG_X
            } else {
                Vec2::X
            })?,
            Elevation::<crate::Architectural>::from_metres(2.6)?,
            PositiveLength::from_metres(0.2)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
    }
    let _: () = for z in [front, back] {
        assembly.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(width + 2.0, z))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(width + 5.2, z))?,
            PlanDirection::<crate::Architectural>::from_normalized(if z == front {
                Vec2::NEG_Y
            } else {
                Vec2::Y
            })?,
            Elevation::<crate::Architectural>::from_metres(2.6)?,
            PositiveLength::from_metres(0.2)?,
            crate::workplace::assembly::WallConstruction::TimberBoards,
        )?;
    };
    Ok(())
}

fn hearth(
    assembly: &mut Assembly<'_>,
    centre: crate::plan_geometry::ArchitecturalPlanPoint,
    flue_top: crate::spatial_geometry::Elevation<crate::Architectural>,
) -> GenerationResult<()> {
    let centre = centre.metres();
    let flue_top = flue_top.metres();
    for x in [-0.9, 0.9] {
        assembly.part(
            WorkplaceFeature::Kiln,
            WorkplaceMaterial::Masonry,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x + x, 0.7, centre.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.5, 1.4, 2.2))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    assembly.part(
        WorkplaceFeature::Kiln,
        WorkplaceMaterial::Masonry,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.7, centre.y + 0.85))?,
        CuboidDimensions::from_metres(Vec3::new(1.3, 1.4, 0.5))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    assembly.part(
        WorkplaceFeature::Kiln,
        WorkplaceMaterial::Iron,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 1.5, centre.y - 0.35))?,
        CuboidDimensions::from_metres(Vec3::new(2.3, 0.2, 1.5))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    // Hollow square shaft directly over the rear masonry. Its opening is never capped.
    for x in [-0.34, 0.34] {
        assembly.part(
            WorkplaceFeature::Flue,
            WorkplaceMaterial::Masonry,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x + x,
                (1.4 + flue_top) * 0.5,
                centre.y + 0.75,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(0.18, flue_top - 1.4, 0.86))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    let _: () = for z in [-0.34, 0.34] {
        assembly.part(
            WorkplaceFeature::Flue,
            WorkplaceMaterial::Masonry,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x,
                (1.4 + flue_top) * 0.5,
                centre.y + 0.75 + z,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(0.5, flue_top - 1.4, 0.18))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}
