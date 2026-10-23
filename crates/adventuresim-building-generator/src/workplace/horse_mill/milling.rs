use super::*;
use crate::GenerationResult as Result;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{
    CuboidDimensions, Elevation, Position, PositiveLength, RigidRotation,
};
use bevy::math::Quat;
use std::f32::consts::TAU;

const MILLSTONE_HEIGHT_METRES: f32 = 0.26;
const BEDSTONE_CENTRE_METRES: f32 = 0.63;
const RUNNER_CENTRE_METRES: f32 = 0.89;

struct MillstoneSection {
    centre_height: Elevation<crate::Architectural>,
    height: PositiveLength,
}

pub(super) fn stone_and_hopper(
    assembly: &mut Assembly<'_>,
    centre: ArchitecturalPlanPoint,
) -> Result<()> {
    let centre_metres = centre.metres();
    assembly.part(
        WorkplaceFeature::Millstone,
        WorkplaceMaterial::DressedStone,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            centre_metres.x,
            0.25,
            centre_metres.y,
        ))?,
        CuboidDimensions::from_metres(Vec3::new(1.65, 0.5, 1.65))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    // The two 0.26 m stones touch at 0.76 m; keep their authored stations
    // rather than recomputing one centre with differently rounded arithmetic.
    for section in [
        MillstoneSection {
            centre_height: Elevation::from_metres(BEDSTONE_CENTRE_METRES)?,
            height: PositiveLength::from_metres(MILLSTONE_HEIGHT_METRES)?,
        },
        MillstoneSection {
            centre_height: Elevation::from_metres(RUNNER_CENTRE_METRES)?,
            height: PositiveLength::from_metres(MILLSTONE_HEIGHT_METRES)?,
        },
    ] {
        section.assemble(assembly, centre)?;
    }
    // One aligned driven spindle joins the lantern pinion to the upper runner stone.
    assembly.part(
        WorkplaceFeature::MillDrive,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            centre_metres.x,
            1.57,
            centre_metres.y,
        ))?,
        CuboidDimensions::from_metres(Vec3::new(0.18, 2.10, 0.18))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    for y in [2.17, 2.65] {
        assembly.part(
            WorkplaceFeature::MillDrive,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre_metres.x,
                y,
                centre_metres.y,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(0.72, 0.12, 0.72))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    for pin in 0..8 {
        let angle = (pin as f32 + 0.5) * TAU / 8.0;
        let offset = Vec2::new(angle.sin(), angle.cos()) * 0.31;
        assembly.part(
            WorkplaceFeature::MillDrive,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre_metres.x + offset.x,
                2.41,
                centre_metres.y + offset.y,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(0.10, 0.48, 0.10))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    hopper(
        assembly,
        ArchitecturalPlanPoint::from_metres(centre_metres + Vec2::new(0.0, -1.2))?,
    )?;
    assembly.part(
        WorkplaceFeature::Hopper,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            centre_metres.x,
            1.26,
            centre_metres.y - 1.26,
        ))?,
        CuboidDimensions::from_metres(Vec3::new(0.4, 0.1, 0.12))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    let chute = assembly.part(
        WorkplaceFeature::Hopper,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            centre_metres.x,
            1.16,
            centre_metres.y - 0.65,
        ))?,
        CuboidDimensions::from_metres(Vec3::new(0.28, 0.06, 1.35))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    assembly.orient_part(
        chute,
        RigidRotation::from_quaternion(Quat::from_rotation_x(0.2))?,
    )?;
    // The short flour outlet terminates within the working core rather than in the animal track.
    assembly.part(
        WorkplaceFeature::Hopper,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(
            centre_metres.x,
            0.73,
            centre_metres.y + 0.77,
        ))?,
        CuboidDimensions::from_metres(Vec3::new(0.24, 0.12, 0.3))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}

impl MillstoneSection {
    fn assemble(self, assembly: &mut Assembly<'_>, centre: ArchitecturalPlanPoint) -> Result<()> {
        let centre = centre.metres();
        let y = self.centre_height.metres();
        let height = self.height.metres();
        // Grinding surfaces are a single stone, not wall masonry with mortar joints.
        let _: () = for slice in 0..9 {
            let z = (slice as f32 - 4.0) * 0.15;
            let half_width = (0.72_f32.powi(2) - z.powi(2)).sqrt();
            assembly.part(
                WorkplaceFeature::Millstone,
                WorkplaceMaterial::Millstone,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    centre.x,
                    y,
                    centre.y + z,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(half_width * 2.0, height, 0.15))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        };
        Ok(())
    }
}

fn hopper(assembly: &mut Assembly<'_>, centre: ArchitecturalPlanPoint) -> Result<()> {
    let centre = centre.metres();
    for x in [-0.48, 0.48] {
        for z in [-0.32, 0.32] {
            assembly.part(
                WorkplaceFeature::Hopper,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    centre.x + x,
                    0.65,
                    centre.y + z,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.12, 1.3, 0.12))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    }
    for x in [-0.5, 0.5] {
        assembly.part(
            WorkplaceFeature::Hopper,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x + x, 1.58, centre.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.10, 0.6, 0.9))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    for z in [-0.4, 0.4] {
        assembly.part(
            WorkplaceFeature::Hopper,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 1.58, centre.y + z))?,
            CuboidDimensions::from_metres(Vec3::new(0.9, 0.6, 0.10))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    // Paired floor boards leave an actual central feed slot above the inclined chute.
    let _: () = for x in [-0.3, 0.3] {
        assembly.part(
            WorkplaceFeature::Hopper,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x + x, 1.33, centre.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.30, 0.10, 0.8))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
        assembly.part(
            WorkplaceFeature::Hopper,
            WorkplaceMaterial::Grain,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x + x, 1.51, centre.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.29, 0.26, 0.78))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}

pub(super) fn grain_bin(assembly: &mut Assembly<'_>, centre: ArchitecturalPlanPoint) -> Result<()> {
    let centre = centre.metres();
    assembly.part(
        WorkplaceFeature::StorageBin,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.2, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(1.8, 0.12, 1.8))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    for x in [-0.86, 0.86] {
        assembly.part(
            WorkplaceFeature::StorageBin,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x + x, 0.7, centre.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.08, 1.0, 1.8))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    for z in [-0.86, 0.86] {
        assembly.part(
            WorkplaceFeature::StorageBin,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.7, centre.y + z))?,
            CuboidDimensions::from_metres(Vec3::new(1.64, 1.0, 0.08))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    assembly.part(
        WorkplaceFeature::StorageBin,
        WorkplaceMaterial::Grain,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.56, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(1.62, 0.6, 1.62))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}
