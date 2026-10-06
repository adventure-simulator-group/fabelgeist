//! Small preindustrial brewing yards and ventilated malt-drying houses.
use super::{assembly::Assembly, *};
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

pub(super) fn brewery(a: &mut Assembly<'_>, w: f32, d: f32) -> Result<(), crate::GenerationError> {
    service_frame(a, w, d)?;
    for z in [2.1, d * 0.5 - 0.7] {
        vessels::vat(a, Vec2::new(w + 3.6, z), 0.95, 1.25)?;
    }
    vessels::vat(a, Vec2::new(2.0, 2.2), 1.05, 1.4)?;
    for z in [d * 0.55, d - 2.2] {
        vessels::vat(a, Vec2::new(2.0, z), 1.05, 1.4)?;
    }
    brewing_bench(a, Vec2::new(w - 2.1, d - 3.0))?;
    // Broad masonry shoulders around an open firing mouth, with a continuous rear flue.
    hearth(a, Vec2::new(w + 3.6, d - 1.65), 4.25)?;
    a.passage(
        WorkplacePassagePurpose::OutdoorRoute,
        Position::<crate::Architectural>::from_metres(Vec3::new(w + 2.2, 0.05, d - 3.0))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(w + 5.0, 2.3, d - 2.85))?,
    )?;

    Ok(())
}

fn brewing_bench(a: &mut Assembly<'_>, p: Vec2) -> Result<(), crate::GenerationError> {
    for x in [-0.8, 0.8] {
        for z in [-1.2, 1.2] {
            a.part(
                WorkplaceFeature::Counter,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(p.x + x, 0.45, p.y + z))?,
                CuboidDimensions::from_metres(Vec3::new(0.18, 0.9, 0.18))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    }
    a.part(
        WorkplaceFeature::Counter,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 0.98, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(2.0, 0.16, 2.8))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    // A small open rinsing vessel on the working bench gives the surface a clear use.
    let _: () = for (offset, size) in [
        (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.3, 0.08, 1.3)),
        (Vec3::new(-0.61, 0.2, 0.0), Vec3::new(0.08, 0.4, 1.3)),
        (Vec3::new(0.61, 0.2, 0.0), Vec3::new(0.08, 0.4, 1.3)),
        (Vec3::new(0.0, 0.2, -0.61), Vec3::new(1.3, 0.4, 0.08)),
        (Vec3::new(0.0, 0.2, 0.61), Vec3::new(1.3, 0.4, 0.08)),
    ] {
        a.part(
            WorkplaceFeature::Trough,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 1.1, p.y) + offset)?,
            CuboidDimensions::from_metres(size)?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}

fn service_frame(a: &mut Assembly<'_>, w: f32, d: f32) -> Result<(), crate::GenerationError> {
    let front = 0.6;
    let back = d - 3.4;
    let bays = ((back - front) / 3.0).ceil() as u32;
    for x in [w + 2.0, w + 5.2] {
        for bay in 0..=bays {
            let z = front + (back - front) * bay as f32 / bays as f32;
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Position::<crate::Architectural>::from_metres(Vec3::new(x, 1.3, z))?,
                CuboidDimensions::from_metres(Vec3::new(0.24, 2.6, 0.24))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(x, front))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(x, back))?,
            PlanDirection::<crate::Architectural>::from_normalized(if x < w + 3.0 {
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
        a.wall(
            ArchitecturalPlanPoint::try_from(Vec2::new(w + 2.0, z))?,
            ArchitecturalPlanPoint::try_from(Vec2::new(w + 5.2, z))?,
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

fn hearth(a: &mut Assembly<'_>, p: Vec2, flue_top: f32) -> Result<(), crate::GenerationError> {
    for x in [-0.9, 0.9] {
        a.part(
            WorkplaceFeature::Kiln,
            WorkplaceMaterial::Masonry,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x + x, 0.7, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.5, 1.4, 2.2))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.part(
        WorkplaceFeature::Kiln,
        WorkplaceMaterial::Masonry,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 0.7, p.y + 0.85))?,
        CuboidDimensions::from_metres(Vec3::new(1.3, 1.4, 0.5))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    a.part(
        WorkplaceFeature::Kiln,
        WorkplaceMaterial::Iron,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 1.5, p.y - 0.35))?,
        CuboidDimensions::from_metres(Vec3::new(2.3, 0.2, 1.5))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    // Hollow square shaft directly over the rear masonry. Its opening is never capped.
    for x in [-0.34, 0.34] {
        a.part(
            WorkplaceFeature::Flue,
            WorkplaceMaterial::Masonry,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                p.x + x,
                (1.4 + flue_top) * 0.5,
                p.y + 0.75,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(0.18, flue_top - 1.4, 0.86))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    let _: () = for z in [-0.34, 0.34] {
        a.part(
            WorkplaceFeature::Flue,
            WorkplaceMaterial::Masonry,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                p.x,
                (1.4 + flue_top) * 0.5,
                p.y + 0.75 + z,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(0.5, flue_top - 1.4, 0.18))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}
