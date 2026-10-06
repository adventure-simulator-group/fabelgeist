use super::*;
use crate::spatial_geometry::{CuboidDimensions, Position};

const UPPER_FLOOR_METRES: f32 = 3.36;
const STAIR_RUN_METRES: f32 = 4.8;
const STAIR_STEP_COUNT: u32 = 18;

pub(super) fn storage_floors(
    a: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> Result<(), crate::GenerationError> {
    let w = dimensions.metres().x;
    let d = dimensions.metres().y;
    let bays = (d / 3.0).ceil() as u32;
    for bay in 0..=bays {
        let z = 0.6 + (d - 1.2) * bay as f32 / bays as f32;
        if [d * 0.25, d * 0.5]
            .iter()
            .any(|opening| (z - opening).abs() < 1.4)
        {
            continue;
        }
        for x in [0.45, 3.25, w - 3.25, w - 2.0] {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Position::<crate::Architectural>::from_metres(Vec3::new(x, 1.4, z))?,
                CuboidDimensions::from_metres(Vec3::new(0.3, 2.8, 0.3))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        a.part(
            WorkplaceFeature::Beam,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new((w - 1.55) * 0.5, 3.0, z))?,
            CuboidDimensions::from_metres(Vec3::new(w - 1.7, 0.4, 0.3))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.part(
        WorkplaceFeature::Floor,
        WorkplaceMaterial::Timber,
        Position::<crate::Architectural>::from_metres(Vec3::new((w - 1.4) * 0.5, 3.28, d * 0.5))?,
        CuboidDimensions::from_metres(Vec3::new(w - 2.0, 0.16, d - 0.6))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    // The full-width rear landing bridges the side stair to the main upper storage floor.
    a.part(
        WorkplaceFeature::Floor,
        WorkplaceMaterial::Timber,
        Position::<crate::Architectural>::from_metres(Vec3::new(w * 0.5, 3.28, d - 1.3))?,
        CuboidDimensions::from_metres(Vec3::new(w - 0.6, 0.16, 2.0))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    let landing = d - 2.3;
    let step_depth = STAIR_RUN_METRES / STAIR_STEP_COUNT as f32;
    for step in 0..STAIR_STEP_COUNT {
        let rise = UPPER_FLOOR_METRES * (step + 1) as f32 / STAIR_STEP_COUNT as f32;
        let front = landing - STAIR_RUN_METRES + step as f32 * step_depth;
        a.part(
            WorkplaceFeature::Stair,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                w - 1.0,
                rise * 0.5,
                front + step_depth * 0.5,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(1.2, rise, step_depth))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
        a.passage(
            WorkplacePassagePurpose::UpperCirculation,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                w - 1.55,
                rise + 0.03,
                front + 0.01,
            ))?,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                w - 0.45,
                rise + 2.0,
                front + step_depth - 0.01,
            ))?,
        )?;
    }
    a.passage(
        WorkplacePassagePurpose::UpperCirculation,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            w * 0.5 - 1.5,
            UPPER_FLOOR_METRES + 0.04,
            0.5,
        ))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(w * 0.5 + 1.5, 5.5, d - 0.4))?,
    )?;
    a.passage(
        WorkplacePassagePurpose::UpperCirculation,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            w * 0.5,
            UPPER_FLOOR_METRES + 0.04,
            landing + 0.05,
        ))?,
        Position::<crate::Architectural>::from_metres(Vec3::new(w - 0.4, 5.5, d - 0.4))?,
    )?;
    let _: () = for z in [2.4, d * 0.4, d - 3.0] {
        for base in [0.16, UPPER_FLOOR_METRES] {
            crate_stack(a, Vec3::new(1.8, base, z))?;
        }
    };
    Ok(())
}

fn crate_stack(a: &mut Assembly<'_>, base: Vec3) -> Result<(), crate::GenerationError> {
    for offset in [-0.55, 0.55] {
        a.part(
            WorkplaceFeature::StorageBin,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(base + Vec3::new(0.0, 0.5, offset))?,
            CuboidDimensions::from_metres(Vec3::new(1.8, 1.0, 1.0))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.part(
        WorkplaceFeature::StorageBin,
        WorkplaceMaterial::Timber,
        Position::<crate::Architectural>::from_metres(base + Vec3::new(0.0, 1.35, 0.0))?,
        CuboidDimensions::from_metres(Vec3::new(1.4, 0.7, 1.0))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    let _: () = for offset in [-0.55, 0.55] {
        a.part(
            WorkplaceFeature::Boarding,
            WorkplaceMaterial::Timber,
            Position::<crate::Architectural>::from_metres(base + Vec3::new(0.915, 0.5, offset))?,
            CuboidDimensions::from_metres(Vec3::new(0.04, 1.0, 0.09))?,
            crate::workplace::WorkplacePartVisibility::DetailOnly,
        )?;
    };
    Ok(())
}
