use super::*;
use crate::BuildingLodMaterial;
use crate::furniture::builder::CollisionPolicy;
use bevy::math::Quat;

pub(super) fn trestle(builder: &mut Builder, size: Vec3) {
    let top_thickness = 0.065;
    for side in [-1.0, 1.0] {
        let x = side * size.x * 0.32;
        builder.timber(
            Vec3::new(x, 0.045, 0.0),
            Vec3::new(0.24, 0.09, size.z * 0.86),
        );
        builder.timber(
            Vec3::new(x, (size.y - top_thickness) * 0.5, 0.0),
            Vec3::new(0.13, size.y - top_thickness, 0.24),
        );
        builder.timber(
            Vec3::new(x, size.y - 0.105, 0.0),
            Vec3::new(0.18, 0.08, size.z * 0.92),
        );
    }
    builder.timber(Vec3::Y * 0.24, Vec3::new(size.x * 0.78, 0.1, 0.09));
    boards(
        builder,
        Vec3::Y * (size.y - top_thickness * 0.5),
        Vec3::new(size.x, top_thickness, size.z),
    );
}

pub(super) fn desk(builder: &mut Builder, size: Vec3) {
    let writing_front = size.y - 0.13;
    legs(builder, size, writing_front - 0.04, 0.08);
    // A wedge of joined side boards supports the sloping writing leaf.
    for side in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(side * (size.x - 0.08) * 0.5, writing_front - 0.08, 0.0),
            Vec3::new(0.08, 0.16, size.z),
        );
    }
    builder.timber(
        Vec3::new(0.0, size.y - 0.13, (size.z - 0.06) * 0.5),
        Vec3::new(size.x, 0.16, 0.06),
    );
    slope(builder, size.x, size.z, writing_front, size.y);
}

pub(super) fn lectern(builder: &mut Builder, size: Vec3) {
    builder.timber(Vec3::Y * 0.045, Vec3::new(size.x, 0.09, size.z * 0.28));
    builder.timber(Vec3::Y * 0.045, Vec3::new(size.x * 0.24, 0.09, size.z));
    builder.timber(
        Vec3::Y * (size.y - 0.225) * 0.5,
        Vec3::new(0.15, size.y - 0.225, 0.15),
    );
    builder.timber(
        Vec3::Y * (size.y - 0.26),
        Vec3::new(size.x * 0.8, 0.1, 0.15),
    );
    slope(builder, size.x, size.z, size.y - 0.3, size.y);
}

fn slope(builder: &mut Builder, width: f32, depth: f32, front: f32, back: f32) {
    let thickness = 0.045;
    let angle = -((back - front) / depth).atan();
    let length = (depth - thickness * angle.sin().abs()) / angle.cos();
    let centre_y = (front + back) * 0.5 - thickness;
    builder.cuboid(
        BuildingLodMaterial::InteriorTimber,
        Vec3::Y * centre_y,
        Vec3::new(width, thickness, length),
        Quat::from_rotation_x(angle),
        CollisionPolicy::Solid,
    );
    // A retaining lip belongs to the leaf, and keeps the otherwise empty slope legible.
    builder.timber(
        Vec3::new(0.0, front - 0.005, -depth * 0.5 + 0.04),
        Vec3::new(width, 0.04, 0.055),
    );
}

pub(super) fn altar(builder: &mut Builder, size: Vec3) {
    builder.timber(Vec3::Y * 0.065, Vec3::new(size.x, 0.13, size.z));
    builder.timber(
        Vec3::Y * (size.y - 0.08) * 0.5,
        Vec3::new(size.x * 0.86, size.y - 0.08, size.z * 0.82),
    );
    builder.timber(Vec3::Y * (size.y - 0.045), Vec3::new(size.x, 0.09, size.z));
    for x in [-1.0, 0.0, 1.0] {
        builder.timber(
            Vec3::new(x * size.x * 0.38, size.y * 0.49, -size.z * 0.43),
            Vec3::new(0.065, size.y * 0.72, 0.06),
        );
    }
}
