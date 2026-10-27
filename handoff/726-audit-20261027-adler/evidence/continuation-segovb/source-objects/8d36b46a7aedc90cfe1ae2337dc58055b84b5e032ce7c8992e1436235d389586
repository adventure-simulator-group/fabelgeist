use super::{
    builder::{Builder, CollisionPolicy},
    *,
};
use crate::BuildingLodMaterial;
use bevy::math::Quat;

pub(super) fn table_and_benches(builder: &mut Builder, variant: FurnitureVariant) {
    let length = match variant {
        FurnitureVariant::Compact => 1.8,
        FurnitureVariant::Broad => 2.4,
    };
    table(builder, Vec3::ZERO, length, 0.8, 0.80);
    for z in [-0.85, 0.85] {
        bench(builder, Vec3::new(0.0, 0.0, z), length, 0.34, 0.46);
    }
    builder.clearance(
        FurnitureClearanceKind::Access,
        Vec3::new(length * 0.5 + 0.08, 0.0, -1.05),
        Vec3::new(length * 0.5 + 0.8, 1.9, 1.05),
    );
    for z in [-1.0_f32, 1.0] {
        let a = z * 1.12;
        let b = z * 1.72;
        builder.clearance(
            FurnitureClearanceKind::WorkingSpace,
            Vec3::new(-length * 0.5, 0.0, a.min(b)),
            Vec3::new(length * 0.5, 1.9, a.max(b)),
        );
    }
}

pub(super) fn table(builder: &mut Builder, origin: Vec3, length: f32, depth: f32, height: f32) {
    let leg_x = length * 0.5 - 0.22;
    let leg_z = depth * 0.5 - 0.12;
    for x in [-leg_x, leg_x] {
        for z in [-leg_z, leg_z] {
            builder.timber(
                origin + Vec3::new(x, (height - 0.07) * 0.5, z),
                Vec3::new(0.12, height - 0.07, 0.12),
            );
        }
        builder.timber(
            origin + Vec3::new(x, height - 0.12, 0.0),
            Vec3::new(0.14, 0.12, depth),
        );
    }
    for z in [-leg_z, leg_z] {
        builder.timber(
            origin + Vec3::new(0.0, height - 0.15, z),
            Vec3::new(length - 0.3, 0.16, 0.10),
        );
    }
    slatted_top(builder, origin + Vec3::Y * (height - 0.035), length, depth);
}

fn bench(builder: &mut Builder, origin: Vec3, length: f32, depth: f32, height: f32) {
    for x in [-length * 0.5 + 0.25, length * 0.5 - 0.25] {
        builder.timber(
            origin + Vec3::new(x, (height - 0.07) * 0.5, 0.0),
            Vec3::new(0.16, height - 0.07, depth * 0.72),
        );
        builder.timber(
            origin + Vec3::new(x, 0.055, 0.0),
            Vec3::new(0.22, 0.11, depth + 0.08),
        );
    }
    builder.timber(
        origin + Vec3::new(0.0, height * 0.45, 0.0),
        Vec3::new(length - 0.36, 0.09, 0.09),
    );
    slatted_top(builder, origin + Vec3::Y * (height - 0.035), length, depth);
}

fn slatted_top(builder: &mut Builder, centre: Vec3, length: f32, depth: f32) {
    let count = (depth / 0.16).ceil() as usize;
    let pitch = depth / count as f32;
    for plank in 0..count {
        let z = -depth * 0.5 + (plank as f32 + 0.5) * pitch;
        builder.cuboid(
            BuildingLodMaterial::InteriorTimber,
            centre + Vec3::Z * z,
            Vec3::new(length, 0.07, pitch - 0.01),
            Quat::IDENTITY,
            CollisionPolicy::Decoration,
        );
    }
    builder.collider(centre, Vec3::new(length, 0.07, depth));
}
