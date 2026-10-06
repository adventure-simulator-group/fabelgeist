//! Coopered vessels assembled from the same oriented timber and iron cuboids as the building.
use super::*;
use crate::GenerationResult as Result;
use crate::spatial_geometry::{CuboidDimensions, Position, RigidRotation};
use std::f32::consts::TAU;

const STAVE_COUNT: u32 = 12;
const STAVE_THICKNESS_METRES: f32 = 0.12;
const HOOP_HEIGHT_METRES: f32 = 0.09;
const HOOP_THICKNESS_METRES: f32 = 0.035;
const BOTTOM_PLANK_COUNT: u32 = 12;
const BOTTOM_THICKNESS_METRES: f32 = 0.16;

pub(super) fn vat(
    assembly: &mut Assembly<'_>,
    centre: crate::plan_geometry::ArchitecturalPlanPoint,
    radius: crate::spatial_geometry::PositiveLength,
    height: crate::spatial_geometry::PositiveLength,
) -> Result<()> {
    let centre = centre.metres();
    let radius = radius.metres();
    let height = height.metres();
    let plank_depth = radius * 2.0 / BOTTOM_PLANK_COUNT as f32;
    for index in 0..BOTTOM_PLANK_COUNT {
        let z = -radius + (index as f32 + 0.5) * plank_depth;
        let half_width = (radius * radius - z * z).sqrt();
        assembly.part(
            WorkplaceFeature::Vat,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x,
                BOTTOM_THICKNESS_METRES * 0.5,
                centre.y + z,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(
                half_width * 2.0,
                BOTTOM_THICKNESS_METRES,
                plank_depth,
            ))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    let step = TAU / STAVE_COUNT as f32;
    let stave_width = 2.0 * radius * (step * 0.5).tan();
    for index in 0..STAVE_COUNT {
        let angle = index as f32 * step;
        let direction = Vec2::new(angle.sin(), angle.cos());
        let centre = centre + direction * radius;
        oriented_part(
            assembly,
            WorkplaceMaterial::UnpaintedTimber,
            crate::spatial_geometry::Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x,
                height * 0.5,
                centre.y,
            ))?,
            crate::spatial_geometry::CuboidDimensions::from_metres(Vec3::new(
                stave_width,
                height,
                STAVE_THICKNESS_METRES,
            ))?,
            crate::spatial_geometry::Radians::new(angle)?,
        )?;
    }
    let _: () = for elevation in [height * 0.18, height * 0.8] {
        for index in 0..STAVE_COUNT {
            let angle = index as f32 * step;
            let hoop_radius = radius + (STAVE_THICKNESS_METRES + HOOP_THICKNESS_METRES) * 0.5;
            let centre = centre + Vec2::new(angle.sin(), angle.cos()) * hoop_radius;
            oriented_part(
                assembly,
                WorkplaceMaterial::Iron,
                crate::spatial_geometry::Position::<crate::Architectural>::from_metres(Vec3::new(
                    centre.x, elevation, centre.y,
                ))?,
                crate::spatial_geometry::CuboidDimensions::from_metres(Vec3::new(
                    2.0 * hoop_radius * (step * 0.5).tan(),
                    HOOP_HEIGHT_METRES,
                    HOOP_THICKNESS_METRES,
                ))?,
                crate::spatial_geometry::Radians::new(angle)?,
            )?;
        }
    };
    Ok(())
}

fn oriented_part(
    assembly: &mut Assembly<'_>,
    material: WorkplaceMaterial,
    centre: crate::spatial_geometry::Position<crate::Architectural>,
    size: crate::spatial_geometry::CuboidDimensions,
    yaw: crate::spatial_geometry::Radians,
) -> Result<()> {
    let yaw = yaw.radians();
    let id = assembly.part(
        WorkplaceFeature::Vat,
        material,
        centre,
        size,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    assembly.orient_part(
        id,
        RigidRotation::from_quaternion(bevy::math::Quat::from_rotation_y(yaw))?,
    )?;

    Ok(())
}
