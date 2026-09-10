use super::{Builder, FurnitureKind, Vec3, legs};
use crate::BuildingLodMaterial;
use crate::furniture::builder::CollisionPolicy;
use bevy::math::Quat;

pub(super) fn table(builder: &mut Builder, size: Vec3, kind: FurnitureKind) {
    let thickness = match kind {
        FurnitureKind::ButchersBlock => 0.28,
        FurnitureKind::Workbench => 0.1,
        _ => 0.055,
    };
    let leg_height = size.y - thickness;
    legs(builder, size, leg_height, 0.12);
    let top_depth = if kind == FurnitureKind::ButchersBlock {
        size.z - 0.024
    } else {
        size.z
    };
    builder.timber(
        Vec3::Y * (size.y - thickness * 0.5),
        Vec3::new(size.x, thickness, top_depth),
    );
    for sign in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(0.0, leg_height - 0.065, sign * (size.z - 0.12) * 0.5),
            Vec3::new(size.x - 0.12, 0.13, 0.075),
        );
        builder.timber(
            Vec3::new(sign * (size.x - 0.12) * 0.5, 0.2, 0.0),
            Vec3::new(0.075, 0.09, size.z - 0.12),
        );
    }
    match kind {
        FurnitureKind::Workbench => {
            builder.timber(Vec3::Y * 0.23, Vec3::new(size.x - 0.1, 0.05, size.z - 0.1));
            // A fixed wooden vice jaw sits below the working surface, within
            // the top's footprint, joined to the apron by its iron screw.
            builder.timber(
                Vec3::new(-size.x * 0.28, size.y - 0.2, -size.z * 0.5 + 0.055),
                Vec3::new(0.3, 0.25, 0.11),
            );
            builder.cuboid(
                BuildingLodMaterial::Iron,
                Vec3::new(-size.x * 0.28, size.y - 0.23, -size.z * 0.5 + 0.08),
                Vec3::new(0.035, 0.035, 0.16),
                Quat::IDENTITY,
                CollisionPolicy::Solid,
            );
        }
        FurnitureKind::ButchersBlock => {
            for z in [-1.0, 1.0] {
                builder.cuboid(
                    BuildingLodMaterial::Iron,
                    Vec3::new(0.0, size.y - thickness * 0.75, z * (size.z - 0.012) * 0.5),
                    Vec3::new(size.x, 0.035, 0.012),
                    Quat::IDENTITY,
                    CollisionPolicy::Solid,
                );
            }
        }
        _ => {
            builder.timber(Vec3::Y * 0.2, Vec3::new(size.x - 0.12, 0.09, 0.07));
        }
    }
}
