use super::*;
use crate::BuildingLodMaterial;
use crate::furniture::builder::CollisionPolicy;
use bevy::math::Quat;

pub(super) fn assemble(builder: &mut Builder, kind: FurnitureKind, size: Vec3) {
    let post = 0.075;
    legs(builder, size, size.y, post);
    let beds: &[f32] = if kind == FurnitureKind::BunkBed {
        &[0.38, 1.35]
    } else {
        &[0.38]
    };
    for &deck_height in beds {
        deck(builder, size, deck_height);
    }
    for end in [-1.0, 1.0] {
        let z = end * (size.z - post) * 0.5;
        let rail_top = if end > 0.0 { size.y } else { size.y * 0.8 };
        builder.timber(
            Vec3::new(0.0, rail_top - 0.05, z),
            Vec3::new(size.x, 0.1, post),
        );
        if kind == FurnitureKind::Bed {
            builder.timber(
                Vec3::new(0.0, (rail_top + 0.48) * 0.5, z),
                Vec3::new(size.x - post, rail_top - 0.48, 0.045),
            );
        } else {
            // Ward furniture has an open, easily cleaned spindle frame.
            for fraction in [-0.3, 0.0, 0.3] {
                builder.timber(
                    Vec3::new(size.x * fraction, (rail_top + 0.4) * 0.5, z),
                    Vec3::new(0.035, rail_top - 0.4, 0.035),
                );
            }
        }
    }
    if kind == FurnitureKind::BunkBed {
        let ladder_x = -size.x * 0.5 + 0.05;
        for z in [-0.28, 0.28] {
            builder.timber(Vec3::new(ladder_x, 0.71, z), Vec3::new(0.065, 1.42, 0.065));
        }
        for rung in 1..5 {
            builder.timber(
                Vec3::new(ladder_x, rung as f32 * 0.28, 0.0),
                Vec3::new(0.065, 0.055, 0.62),
            );
        }
        builder.timber(
            Vec3::new((size.x - post) * 0.5, size.y - 0.14, 0.0),
            Vec3::new(post, 0.1, size.z),
        );
    }
}

fn deck(builder: &mut Builder, size: Vec3, height: f32) {
    for side in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(side * (size.x - 0.075) * 0.5, height, 0.0),
            Vec3::new(0.075, 0.15, size.z),
        );
    }
    builder.timber(
        Vec3::Y * height,
        Vec3::new(size.x - 0.075, 0.06, size.z - 0.15),
    );
    builder.cuboid(
        BuildingLodMaterial::UndyedCloth,
        Vec3::Y * (height + 0.115),
        Vec3::new(size.x - 0.16, 0.17, size.z - 0.19),
        Quat::IDENTITY,
        CollisionPolicy::Solid,
    );
}
