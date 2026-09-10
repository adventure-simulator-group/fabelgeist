use super::*;
use crate::BuildingLodMaterial;
use crate::furniture::builder::CollisionPolicy;
use bevy::math::Quat;

fn iron(builder: &mut Builder, centre: Vec3, size: Vec3) {
    builder.cuboid(
        BuildingLodMaterial::Iron,
        centre,
        size,
        Quat::IDENTITY,
        CollisionPolicy::Decoration,
    );
}

pub(super) fn chest(builder: &mut Builder, size: Vec3) {
    legs(builder, size, 0.14, 0.09);
    builder.timber(
        Vec3::Y * (size.y + 0.06) * 0.5,
        Vec3::new(size.x - 0.045, size.y - 0.12, size.z - 0.045),
    );
    boards(
        builder,
        Vec3::Y * (size.y - 0.043),
        Vec3::new(size.x, 0.07, size.z),
    );
    for fraction in [-0.32, 0.32] {
        iron(
            builder,
            Vec3::new(size.x * fraction, size.y - 0.008, 0.0),
            Vec3::new(0.035, 0.016, size.z),
        );
        for end in [-1.0, 1.0] {
            iron(
                builder,
                Vec3::new(
                    size.x * fraction,
                    size.y * 0.58,
                    end * (size.z - 0.022) * 0.5,
                ),
                Vec3::new(0.035, size.y * 0.56, 0.022),
            );
        }
    }
    iron(
        builder,
        Vec3::new(0.0, size.y * 0.69, -(size.z - 0.024) * 0.5),
        Vec3::new(0.09, 0.13, 0.024),
    );
}

pub(super) fn standing(builder: &mut Builder, kind: FurnitureKind, size: Vec3) {
    legs(builder, size, size.y, 0.065);
    for side in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(side * (size.x - 0.05) * 0.5, size.y * 0.5, 0.0),
            Vec3::new(0.05, size.y, size.z),
        );
    }
    for shelf in 0..5 {
        let y = 0.13 + shelf as f32 * (size.y - 0.16) / 4.0;
        builder.timber(Vec3::Y * y, Vec3::new(size.x, 0.06, size.z));
    }
    builder.timber(
        Vec3::new(0.0, (size.y + 0.1) * 0.5, (size.z - 0.025) * 0.5),
        Vec3::new(size.x, size.y - 0.1, 0.025),
    );
    if kind == FurnitureKind::Cupboard {
        for side in [-1.0, 1.0] {
            door(builder, size, side);
        }
    }
}

fn door(builder: &mut Builder, size: Vec3, side: f32) {
    let width = size.x * 0.5 - 0.04;
    let x = side * size.x * 0.25;
    let z = -size.z * 0.5 + 0.024;
    let bottom = 0.16;
    let height = size.y - bottom - 0.06;
    builder.timber(
        Vec3::new(x, bottom + height * 0.5, z + 0.012),
        Vec3::new(width, height, 0.024),
    );
    for edge in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(x + edge * (width - 0.045) * 0.5, bottom + height * 0.5, z),
            Vec3::new(0.045, height, 0.035),
        );
    }
    for fraction in [0.025, 0.5, 0.975] {
        builder.timber(
            Vec3::new(x, bottom + height * fraction, z),
            Vec3::new(width, 0.045, 0.035),
        );
    }
    iron(
        builder,
        Vec3::new(side * 0.065, size.y * 0.5, z - 0.012),
        Vec3::new(0.027, 0.08, 0.022),
    );
}
