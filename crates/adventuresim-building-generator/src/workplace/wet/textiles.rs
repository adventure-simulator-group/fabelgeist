use super::*;
use crate::spatial_geometry::{CuboidDimensions, Position, RigidRotation};
use bevy::math::Quat;

pub(super) fn dye_frames(
    a: &mut Assembly<'_>,
    w: f32,
    d: f32,
) -> Result<(), crate::GenerationError> {
    let count = 2 + a.plan.size.extra_bays().min(1);
    for index in 0..count {
        let z = 2.0 + f32::from(index) * (d - 4.0) / f32::from(count - 1);
        let p = Vec2::new(w + 3.0, z);
        drying_frame(a, p, 2.0, 3.2)?;
        draped_cloth(
            a,
            p,
            3.1,
            if index % 2 == 0 {
                WorkplaceMaterial::DyedCloth
            } else {
                WorkplaceMaterial::UndyedCloth
            },
        )?;
    }
    folded_stock(a, Vec2::new(1.7, d - 2.3))?;
    draining_bench(a, Vec2::new(w - 1.7, 2.2))?;

    Ok(())
}

fn drying_frame(
    a: &mut Assembly<'_>,
    p: Vec2,
    width: f32,
    height: f32,
) -> Result<(), crate::GenerationError> {
    for x in [-width * 0.5, width * 0.5] {
        a.part(
            WorkplaceFeature::DryingFrame,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x + x, height * 0.5, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.18, height, 0.18))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.part(
        WorkplaceFeature::DryingFrame,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, height - 0.1, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(width + 0.3, 0.2, 0.2))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    let _: () = for side in [-1.0, 1.0] {
        let brace = a.part(
            WorkplaceFeature::DryingFrame,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                p.x + side * (width * 0.5 - 0.3),
                height - 0.5,
                p.y,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(0.12, 0.9, 0.12))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
        a.orient_part(
            brace,
            RigidRotation::from_quaternion(Quat::from_rotation_z(
                side * std::f32::consts::FRAC_PI_4,
            ))?,
        )?;
    };
    Ok(())
}

fn draped_cloth(
    a: &mut Assembly<'_>,
    p: Vec2,
    bar: f32,
    material: WorkplaceMaterial,
) -> Result<(), crate::GenerationError> {
    // An over-rail fold physically joins unequal front and back hanging lengths.
    a.part(
        WorkplaceFeature::Cloth,
        material,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, bar + 0.115, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(1.58, 0.03, 0.28))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;
    let _: () = for (z, length) in [(-0.135, 1.75), (0.135, 1.32)] {
        for fold in 0..3 {
            let x = p.x - 0.53 + fold as f32 * 0.53;
            let hanging_length = length - fold as f32 * 0.035;
            // Joined panels give the cloth an uneven lower hem while sharing its upper fold.
            a.part(
                WorkplaceFeature::Cloth,
                material,
                Position::<crate::Architectural>::from_metres(Vec3::new(
                    x,
                    bar + 0.1 - hanging_length * 0.5,
                    p.y + z,
                ))?,
                CuboidDimensions::from_metres(Vec3::new(0.56, hanging_length, 0.03))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    };
    Ok(())
}

pub(super) fn hide_frame(a: &mut Assembly<'_>, p: Vec2) -> Result<(), crate::GenerationError> {
    drying_frame(a, p, 2.8, 2.8)?;
    let _: () = for (x, scale) in [(-0.7, 0.9), (0.7, 1.0)] {
        hanging_hide(a, Vec2::new(p.x + x, p.y), 2.8, scale)?;
    };
    Ok(())
}

fn hanging_hide(
    a: &mut Assembly<'_>,
    p: Vec2,
    top: f32,
    scale: f32,
) -> Result<(), crate::GenerationError> {
    // Neck, broad shoulders, tapered torso and unequal lower tails form a skin silhouette.
    let _: () = for (x, drop, width, height) in [
        (0.0, 0.12, 0.28, 0.24),
        (0.0, 0.45, 0.98, 0.46),
        (0.0, 0.90, 0.72, 0.46),
        (0.0, 1.25, 0.48, 0.30),
        (-0.16, 1.47, 0.18, 0.23),
        (0.16, 1.43, 0.18, 0.15),
    ] {
        a.part(
            WorkplaceFeature::Hide,
            WorkplaceMaterial::Hide,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                p.x + x * scale,
                top - drop * scale,
                p.y - 0.11,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(width * scale, height * scale, 0.04))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}

fn folded_stock(a: &mut Assembly<'_>, p: Vec2) -> Result<(), crate::GenerationError> {
    for x in [-0.7, 0.7] {
        for z in [-0.5, 0.5] {
            a.part(
                WorkplaceFeature::Counter,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(p.x + x, 0.45, p.y + z))?,
                CuboidDimensions::from_metres(Vec3::new(0.15, 0.9, 0.15))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
    }
    a.part(
        WorkplaceFeature::Counter,
        WorkplaceMaterial::UnpaintedTimber,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 0.98, p.y))?,
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
        a.part(
            WorkplaceFeature::Cloth,
            material,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                p.x,
                1.12 + tier as f32 * 0.12,
                p.y,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(1.3, 0.12, 0.9))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    };
    Ok(())
}

fn draining_bench(a: &mut Assembly<'_>, p: Vec2) -> Result<(), crate::GenerationError> {
    for x in [-0.65, 0.65] {
        for z in [-0.7, 0.7] {
            a.part(
                WorkplaceFeature::Counter,
                WorkplaceMaterial::UnpaintedTimber,
                Position::<crate::Architectural>::from_metres(Vec3::new(p.x + x, 0.45, p.y + z))?,
                CuboidDimensions::from_metres(Vec3::new(0.16, 0.9, 0.16))?,
                crate::workplace::WorkplacePartVisibility::Silhouette,
            )?;
        }
        a.part(
            WorkplaceFeature::Counter,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(p.x + x, 0.96, p.y))?,
            CuboidDimensions::from_metres(Vec3::new(0.16, 0.12, 1.8))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    for slat in 0..6 {
        a.part(
            WorkplaceFeature::Counter,
            WorkplaceMaterial::UnpaintedTimber,
            Position::<crate::Architectural>::from_metres(Vec3::new(
                p.x,
                1.06,
                p.y - 0.75 + slat as f32 * 0.3,
            ))?,
            CuboidDimensions::from_metres(Vec3::new(1.8, 0.08, 0.24))?,
            crate::workplace::WorkplacePartVisibility::Silhouette,
        )?;
    }
    a.part(
        WorkplaceFeature::Cloth,
        WorkplaceMaterial::UndyedCloth,
        Position::<crate::Architectural>::from_metres(Vec3::new(p.x, 1.14, p.y))?,
        CuboidDimensions::from_metres(Vec3::new(1.2, 0.08, 1.0))?,
        crate::workplace::WorkplacePartVisibility::Silhouette,
    )?;

    Ok(())
}
