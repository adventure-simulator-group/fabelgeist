use super::{Builder, FurnitureKind, Vec3, legs};
use crate::BuildingLodMaterial;
use crate::furniture::builder::CollisionPolicy;
use bevy::math::Quat;

pub(super) fn trough(builder: &mut Builder, size: Vec3, kind: FurnitureKind) {
    let bottom = match kind {
        FurnitureKind::GrainBin => 0.1,
        FurnitureKind::KneadingTrough => 0.55,
        _ => 0.18,
    };
    let wall = 0.055;
    let depth = size.y - bottom;
    legs(builder, size, bottom, 0.1);
    builder.timber(Vec3::Y * bottom, Vec3::new(size.x, wall, size.z));
    for sign in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(0.0, bottom + depth * 0.5, sign * (size.z - wall) * 0.5),
            Vec3::new(size.x, depth, wall),
        );
        builder.timber(
            Vec3::new(sign * (size.x - wall) * 0.5, bottom + depth * 0.5, 0.0),
            Vec3::new(wall, depth, size.z),
        );
    }
    if kind == FurnitureKind::GrainBin {
        // A fixed divider gives the bin separate empty storage bays.
        builder.timber(
            Vec3::Y * (bottom + depth * 0.5),
            Vec3::new(wall, depth, size.z),
        );
        for x in [-1.0, 1.0] {
            builder.timber(
                Vec3::new(x * size.x * 0.3, bottom + depth * 0.5, -size.z * 0.5 + wall),
                Vec3::new(0.055, depth, 0.08),
            );
        }
    }
}

pub(super) fn crate_box(builder: &mut Builder, size: Vec3) {
    let wall = 0.045;
    // Separate boards make a closed reusable chest-like packing crate without
    // treating any of its eventual contents as part of the furniture.
    for y in [wall * 0.5, size.y - wall * 0.5] {
        builder.timber(Vec3::Y * y, Vec3::new(size.x, wall, size.z));
    }
    for sign in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(0.0, size.y * 0.5, sign * (size.z - wall - 0.016) * 0.5),
            Vec3::new(size.x, size.y, wall),
        );
        builder.timber(
            Vec3::new(sign * (size.x - wall) * 0.5, size.y * 0.5, 0.0),
            Vec3::new(wall, size.y, size.z),
        );
    }
    for x in [-size.x * 0.32, size.x * 0.32] {
        for z in [-1.0, 1.0] {
            builder.cuboid(
                BuildingLodMaterial::Iron,
                Vec3::new(x, size.y * 0.5, z * (size.z - 0.012) * 0.5),
                Vec3::new(0.035, size.y, 0.012),
                Quat::IDENTITY,
                CollisionPolicy::Solid,
            );
        }
    }
}
