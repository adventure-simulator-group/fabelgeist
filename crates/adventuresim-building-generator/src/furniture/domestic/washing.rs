use super::*;
use crate::BuildingLodMaterial;
use crate::furniture::builder::CollisionPolicy;
use bevy::math::Quat;
use std::f32::consts::TAU;

const TUB_STAVE_COUNT: usize = 16;

pub(super) fn tub(builder: &mut Builder, size: Vec3) {
    // Thick segment walls leave a genuinely open interior; there is no fill collider.
    let radius = Vec3::new(size.x * 0.5 - 0.04, 0.0, size.z * 0.5 - 0.04);
    for segment in 0..TUB_STAVE_COUNT {
        let angle = segment as f32 * TAU / TUB_STAVE_COUNT as f32;
        let next = (segment + 1) as f32 * TAU / TUB_STAVE_COUNT as f32;
        let at = |a: f32| Vec3::new(a.sin() * radius.x, 0.0, a.cos() * radius.z);
        let a = at(angle);
        let b = at(next);
        let delta = b - a;
        let yaw = -delta.z.atan2(delta.x);
        let rotation = Quat::from_rotation_y(yaw);
        builder.cuboid(
            BuildingLodMaterial::InteriorTimber,
            (a + b) * 0.5 + Vec3::Y * size.y * 0.5,
            Vec3::new(delta.length() + 0.012, size.y, 0.06),
            rotation,
            CollisionPolicy::Solid,
        );
        for fraction in [0.18, 0.82] {
            builder.cuboid(
                BuildingLodMaterial::Iron,
                (a + b) * 0.5 + Vec3::Y * size.y * fraction,
                Vec3::new(delta.length() + 0.014, 0.045, 0.072),
                rotation,
                CollisionPolicy::Decoration,
            );
        }
        // A fan of horizontal bottom planks meets every stave, including the rounded ends.
        let points = [Vec3::Y * 0.065, a + Vec3::Y * 0.065, b + Vec3::Y * 0.065];
        builder.triangle(BuildingLodMaterial::InteriorTimber, points, Vec3::Y);
    }
    // The polygon floor is represented by inset overlapping bars, leaving the basin open above it.
    for row in -3_i32..=3 {
        let x = row as f32 * radius.x / 4.0;
        let depth = radius.z * 2.0 * (1.0 - (x / radius.x).powi(2)).sqrt();
        builder.timber(
            Vec3::new(x, 0.03, 0.0),
            Vec3::new(radius.x * 0.3, 0.06, depth * 0.93),
        );
    }
}

pub(super) fn stand(builder: &mut Builder, size: Vec3) {
    legs(builder, size, size.y - 0.1, 0.065);
    boards(
        builder,
        Vec3::Y * (size.y - 0.12),
        Vec3::new(size.x, 0.06, size.z),
    );
    boards(builder, Vec3::Y * 0.22, Vec3::new(size.x, 0.045, size.z));
    // A rear splash board and low side rails identify the stand without adding a loose basin.
    builder.timber(
        Vec3::new(0.0, size.y - 0.075, (size.z - 0.04) * 0.5),
        Vec3::new(size.x, 0.15, 0.04),
    );
    for side in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(side * (size.x - 0.04) * 0.5, size.y - 0.095, 0.0),
            Vec3::new(0.04, 0.09, size.z),
        );
    }
}
