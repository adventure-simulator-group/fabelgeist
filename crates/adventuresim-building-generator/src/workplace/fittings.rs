use super::*;
use crate::spatial_geometry::{CuboidDimensions, Position};

pub(super) fn market(a: &mut Assembly<'_>, w: f32, d: f32) -> Result<(), crate::GenerationError> {
    let _: () = for x in [2.0, w - 2.0] {
        for bay in 0..(d / 3.0) as u32 {
            counter(a, Vec2::new(x, 1.5 + bay as f32 * 3.0), Vec2::new(2.2, 1.4))?;
        }
    };
    Ok(())
}

pub(super) fn counter(
    a: &mut Assembly<'_>,
    p: Vec2,
    size: Vec2,
) -> Result<(), crate::GenerationError> {
    for x in [-0.4, 0.4] {
        for z in [-0.4, 0.4] {
            a.part(
                WorkplaceFeature::Counter,
                WorkplaceMaterial::Timber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    p.x + x * size.x,
                    0.45,
                    p.y + z * size.y,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.12, 0.9, 0.12))?,
                crate::workplace::WorkplacePartVisibility::DetailOnly,
            )?;
        }
    }
    a.part(
        WorkplaceFeature::Counter,
        WorkplaceMaterial::Timber,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 0.96, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(size.x, 0.12, size.y))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}

pub(super) fn trough(a: &mut Assembly<'_>, p: Vec2) -> Result<(), crate::GenerationError> {
    bin(a, p, Vec2::new(0.7, 1.4), 0.55)?;

    Ok(())
}
pub(super) fn bin(
    a: &mut Assembly<'_>,
    p: Vec2,
    size: Vec2,
    height: f32,
) -> Result<(), crate::GenerationError> {
    bin_at(a, p, size, height, 0.0)?;

    Ok(())
}
pub(super) fn bin_at(
    a: &mut Assembly<'_>,
    p: Vec2,
    size: Vec2,
    height: f32,
    base: f32,
) -> Result<(), crate::GenerationError> {
    let feature = if height < 0.6 {
        WorkplaceFeature::Trough
    } else {
        WorkplaceFeature::StorageBin
    };
    for x in [-0.5, 0.5] {
        a.part(
            feature,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                p.x + x * size.x,
                base + height * 0.5,
                p.y,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(0.09, height, size.y))?,
            crate::workplace::WorkplacePartVisibility::DetailOnly,
        )?;
    }
    let _: () = for z in [-0.5, 0.5] {
        a.part(
            feature,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                p.x,
                base + height * 0.5,
                p.y + z * size.y,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(size.x, height, 0.09))?,
            crate::workplace::WorkplacePartVisibility::DetailOnly,
        )?;
    };
    Ok(())
}

pub(super) fn rack(a: &mut Assembly<'_>, p: Vec2) -> Result<(), crate::GenerationError> {
    for x in [-0.8, 0.8] {
        a.part(
            WorkplaceFeature::Rack,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x + x, 0.85, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.16, 1.7, 0.16))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    let _: () = for y in [0.35, 0.9, 1.5] {
        a.part(
            WorkplaceFeature::Rack,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x, y, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(1.8, 0.12, 1.3))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}
