use super::{
    builder::{Builder, CollisionPolicy},
    *,
};
use crate::BuildingLodMaterial;
use bevy::math::Quat;

pub(super) fn assemble(builder: &mut Builder, variant: FurnitureVariant) {
    let length = match variant {
        FurnitureVariant::Compact => 2.2,
        FurnitureVariant::Broad => 2.8,
    };
    for x in [-length * 0.5, length * 0.5] {
        builder.timber(Vec3::new(x, 0.65, 0.35), Vec3::new(0.18, 1.3, 0.18));
    }
    builder.timber(
        Vec3::new(0.0, 1.14, 0.35),
        Vec3::new(length + 0.18, 0.18, 0.18),
    );
    for x in [-length * 0.5, length * 0.5] {
        builder.cuboid(
            BuildingLodMaterial::Iron,
            Vec3::new(x, 1.14, 0.35),
            Vec3::new(0.20, 0.045, 0.20),
            Quat::IDENTITY,
            CollisionPolicy::Decoration,
        );
    }
    trough(builder, length - 0.5);
    builder.clearance(
        FurnitureClearanceKind::Access,
        Vec3::new(-length * 0.5, 0.0, -1.65),
        Vec3::new(length * 0.5, 2.0, -0.95),
    );
    builder.clearance(
        FurnitureClearanceKind::WorkingSpace,
        Vec3::new(-length * 0.5, 0.0, 0.60),
        Vec3::new(length * 0.5, 2.2, 1.90),
    );
}

fn trough(builder: &mut Builder, length: f32) {
    let z = -0.50;
    for x in [-length * 0.32, length * 0.32] {
        builder.timber(Vec3::new(x, 0.07, z), Vec3::new(0.20, 0.14, 0.64));
    }
    builder.timber(Vec3::new(0.0, 0.17, z), Vec3::new(length, 0.12, 0.60));
    for side in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(0.0, 0.36, z + side * 0.27),
            Vec3::new(length, 0.38, 0.08),
        );
        builder.timber(
            Vec3::new(side * (length * 0.5 - 0.045), 0.36, z),
            Vec3::new(0.09, 0.38, 0.52),
        );
    }
    // Contained water is rendered only; the real rim, ends and bottom carry physical collision.
    builder.cuboid(
        BuildingLodMaterial::ProcessLiquid,
        Vec3::new(0.0, 0.305, z),
        Vec3::new(length - 0.20, 0.15, 0.43),
        Quat::IDENTITY,
        CollisionPolicy::Decoration,
    );
}
