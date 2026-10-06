use super::super::components::RecipeComponent;
use super::*;
use crate::GenerationResult as Result;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::Displacement;
use crate::spatial_geometry::{
    CuboidDimensions, Elevation, Position, PositiveLength, RigidRotation,
};
use bevy::math::Quat;

pub(super) fn dye_frames(
    assembly: &mut Assembly<'_>,
    dimensions: crate::spatial_geometry::PlanDimensions,
) -> Result<()> {
    let width = dimensions.metres().x;
    let depth = dimensions.metres().y;
    let count = 2 + assembly.plan.size.extra_bays().min(1);
    for index in 0..count {
        let z = 2.0 + f32::from(index) * (depth - 4.0) / f32::from(count - 1);
        let centre = Vec2::new(width + 3.0, z);
        drying_frame(
            assembly,
            ArchitecturalPlanPoint::from_metres(centre)?,
            PositiveLength::from_metres(2.0)?,
            PositiveLength::from_metres(3.2)?,
        )?;
        draped_cloth(
            assembly,
            ArchitecturalPlanPoint::from_metres(centre)?,
            Elevation::<crate::Architectural>::from_metres(3.1)?,
            if index % 2 == 0 {
                WorkplaceMaterial::DyedCloth
            } else {
                WorkplaceMaterial::UndyedCloth
            },
        )?;
    }
    folded_stock(
        assembly,
        ArchitecturalPlanPoint::from_metres(Vec2::new(1.7, depth - 2.3))?,
    )?;
    draining_bench(
        assembly,
        ArchitecturalPlanPoint::from_metres(Vec2::new(width - 1.7, 2.2))?,
    )?;

    Ok(())
}

fn drying_frame(
    assembly: &mut Assembly<'_>,
    centre: ArchitecturalPlanPoint,
    width: PositiveLength,
    height: PositiveLength,
) -> Result<()> {
    let centre = centre.metres();
    let width = width.metres();
    let height = height.metres();
    for x in [-width * 0.5, width * 0.5] {
        assembly.part(
            WorkplaceFeature::DryingFrame,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x + x,
                height * 0.5,
                centre.y,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(0.18, height, 0.18))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    assembly.part(
        WorkplaceFeature::DryingFrame,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, height - 0.1, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(width + 0.3, 0.2, 0.2))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    let _: () = for side in [-1.0, 1.0] {
        let brace = assembly.part(
            WorkplaceFeature::DryingFrame,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x + side * (width * 0.5 - 0.3),
                height - 0.5,
                centre.y,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(0.12, 0.9, 0.12))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
        assembly.orient_part(
            brace,
            RigidRotation::from_quaternion(Quat::from_rotation_z(
                side * std::f32::consts::FRAC_PI_4,
            ))?,
        )?;
    };
    Ok(())
}

fn draped_cloth(
    assembly: &mut Assembly<'_>,
    centre: ArchitecturalPlanPoint,
    bar: Elevation<crate::Architectural>,
    material: WorkplaceMaterial,
) -> Result<()> {
    let centre = centre.metres();
    let bar = bar.metres();
    // An over-rail fold physically joins unequal front and back hanging lengths.
    assembly.part(
        WorkplaceFeature::Cloth,
        material,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, bar + 0.115, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(1.58, 0.03, 0.28))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    // Authored cloth panels: offsets are architectural X/Y/Z displacements;
    // cuboid dimensions are width, hanging height and thickness in metres.
    let _: () = for component in [
        RecipeComponent {
            offset: Displacement::from_metres(Vec3::new(0.0, 0.0, -0.135))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(0.56, 1.75, 0.03))?,
        },
        RecipeComponent {
            offset: Displacement::from_metres(Vec3::new(0.0, 0.0, 0.135))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(0.56, 1.32, 0.03))?,
        },
    ] {
        let z = component.offset.metres().z;
        let length = component.dimensions.metres().y;
        for fold in 0..3 {
            let x = centre.x - 0.53 + fold as f32 * 0.53;
            let hanging_length = length - fold as f32 * 0.035;
            // Joined panels give the cloth an uneven lower hem while sharing its upper fold.
            assembly.part(
                WorkplaceFeature::Cloth,
                material,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    x,
                    bar + 0.1 - hanging_length * 0.5,
                    centre.y + z,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.56, hanging_length, 0.03))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    };
    Ok(())
}

pub(super) fn hide_frame(
    assembly: &mut Assembly<'_>,
    centre: ArchitecturalPlanPoint,
) -> Result<()> {
    let centre = centre.metres();
    drying_frame(
        assembly,
        ArchitecturalPlanPoint::from_metres(centre)?,
        PositiveLength::from_metres(2.8)?,
        PositiveLength::from_metres(2.8)?,
    )?;
    struct HidePlacement {
        offset: Displacement<crate::Architectural>,
        scale: f32,
    }
    for placement in [
        HidePlacement {
            offset: Displacement::from_metres(Vec3::new(-0.7, 0.0, 0.0))?,
            scale: 0.9,
        },
        HidePlacement {
            offset: Displacement::from_metres(Vec3::new(0.7, 0.0, 0.0))?,
            scale: 1.0,
        },
    ] {
        let x = placement.offset.metres().x;
        let scale = placement.scale;
        hanging_hide(
            assembly,
            ArchitecturalPlanPoint::from_metres(Vec2::new(centre.x + x, centre.y))?,
            Elevation::<crate::Architectural>::from_metres(2.8)?,
            scale,
        )?;
    }
    Ok(())
}

fn hanging_hide(
    assembly: &mut Assembly<'_>,
    centre: ArchitecturalPlanPoint,
    top: Elevation<crate::Architectural>,
    scale: f32,
) -> Result<()> {
    let centre = centre.metres();
    let top = top.metres();
    // Neck, broad shoulders, tapered torso and unequal lower tails form a skin silhouette.
    for component in [
        RecipeComponent {
            offset: Displacement::from_metres(Vec3::new(0.0, 0.12, 0.0))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(0.28, 0.24, 0.04))?,
        },
        RecipeComponent {
            offset: Displacement::from_metres(Vec3::new(0.0, 0.45, 0.0))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(0.98, 0.46, 0.04))?,
        },
        RecipeComponent {
            offset: Displacement::from_metres(Vec3::new(0.0, 0.90, 0.0))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(0.72, 0.46, 0.04))?,
        },
        RecipeComponent {
            offset: Displacement::from_metres(Vec3::new(0.0, 1.25, 0.0))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(0.48, 0.30, 0.04))?,
        },
        RecipeComponent {
            offset: Displacement::from_metres(Vec3::new(-0.16, 1.47, 0.0))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(0.18, 0.23, 0.04))?,
        },
        RecipeComponent {
            offset: Displacement::from_metres(Vec3::new(0.16, 1.43, 0.0))?,
            dimensions: CuboidDimensions::from_metres(Vec3::new(0.18, 0.15, 0.04))?,
        },
    ] {
        let x = component.offset.metres().x;
        let drop = component.offset.metres().y;
        let width = component.dimensions.metres().x;
        let height = component.dimensions.metres().y;
        assembly.part(
            WorkplaceFeature::Hide,
            WorkplaceMaterial::Hide,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x + x * scale,
                top - drop * scale,
                centre.y - 0.11,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(
                width * scale,
                height * scale,
                component.dimensions.metres().z,
            ))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    Ok(())
}

fn folded_stock(assembly: &mut Assembly<'_>, centre: ArchitecturalPlanPoint) -> Result<()> {
    let centre = centre.metres();
    for x in [-0.7, 0.7] {
        for z in [-0.5, 0.5] {
            assembly.part(
                WorkplaceFeature::Counter,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    centre.x + x,
                    0.45,
                    centre.y + z,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.15, 0.9, 0.15))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    }
    assembly.part(
        WorkplaceFeature::Counter,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 0.98, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(1.8, 0.16, 1.4))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    let _: () = for (tier, material) in [
        WorkplaceMaterial::UndyedCloth,
        WorkplaceMaterial::DyedCloth,
        WorkplaceMaterial::UndyedCloth,
    ]
    .into_iter()
    .enumerate()
    {
        assembly.part(
            WorkplaceFeature::Cloth,
            material,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x,
                1.12 + tier as f32 * 0.12,
                centre.y,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(1.3, 0.12, 0.9))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}

fn draining_bench(assembly: &mut Assembly<'_>, centre: ArchitecturalPlanPoint) -> Result<()> {
    let centre = centre.metres();
    for x in [-0.65, 0.65] {
        for z in [-0.7, 0.7] {
            assembly.part(
                WorkplaceFeature::Counter,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    centre.x + x,
                    0.45,
                    centre.y + z,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.16, 0.9, 0.16))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        assembly.part(
            WorkplaceFeature::Counter,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(centre.x + x, 0.96, centre.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.16, 0.12, 1.8))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    for slat in 0..6 {
        assembly.part(
            WorkplaceFeature::Counter,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                centre.x,
                1.06,
                centre.y - 0.75 + slat as f32 * 0.3,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(1.8, 0.08, 0.24))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    assembly.part(
        WorkplaceFeature::Cloth,
        WorkplaceMaterial::UndyedCloth,
        Position::<crate::Architectural>::from_metres(Vec3::new(centre.x, 1.14, centre.y))?,
        CuboidDimensions::from_metres(Vec3::new(1.2, 0.08, 1.0))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}
