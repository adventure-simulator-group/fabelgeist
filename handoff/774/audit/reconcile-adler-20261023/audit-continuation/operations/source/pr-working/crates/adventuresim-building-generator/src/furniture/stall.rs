use super::{
    builder::{Builder, CollisionPolicy},
    *,
};
use crate::BuildingLodMaterial;
use bevy::math::Quat;

mod stock;

const COUNTER_HEIGHT_METRES: f32 = 0.85;
const COUNTER_CENTRE_Z_METRES: f32 = -0.37;

pub(super) fn canopy(builder: &mut Builder, variant: FurnitureVariant) {
    let half_width = match variant {
        FurnitureVariant::Compact => 1.2,
        FurnitureVariant::Broad => 1.65,
    };
    let half_depth = 0.85;
    let eave = 2.18;
    let ridge = 2.60;
    for x in [-half_width, half_width] {
        for z in [-half_depth, half_depth] {
            builder.timber(Vec3::new(x, eave * 0.5, z), Vec3::new(0.10, eave, 0.10));
        }
        builder.timber(
            Vec3::new(x, eave - 0.04, 0.0),
            Vec3::new(0.12, 0.12, half_depth * 2.0 + 0.1),
        );
    }
    for z in [-half_depth, half_depth] {
        builder.timber(
            Vec3::new(0.0, eave - 0.04, z),
            Vec3::new(half_width * 2.0 + 0.1, 0.12, 0.12),
        );
        builder.timber(
            Vec3::new(0.0, (eave + ridge) * 0.5, z),
            Vec3::new(0.10, ridge - eave, 0.10),
        );
        for sign in [-1.0, 1.0] {
            beam(
                builder,
                Vec3::new(sign * half_width, eave, z),
                Vec3::new(0.0, ridge, z),
                0.06,
            );
            // Corner knee braces attach below the eave while staying above the working route.
            beam(
                builder,
                Vec3::new(sign * half_width, eave - 0.32, z),
                Vec3::new(sign * (half_width - 0.35), eave, z),
                0.06,
            );
        }
    }
    builder.timber(
        Vec3::new(0.0, ridge, 0.0),
        Vec3::new(0.09, 0.09, half_depth * 2.0 + 0.1),
    );
    canvas(builder, half_width, half_depth, eave + 0.035, ridge + 0.045);
    super::seating::table(
        builder,
        Vec3::new(0.0, 0.0, COUNTER_CENTRE_Z_METRES),
        half_width * 2.0 - 0.2,
        0.55,
        COUNTER_HEIGHT_METRES,
    );
    stock::display(builder, variant);
    builder.clearance(
        FurnitureClearanceKind::Access,
        Vec3::new(-half_width + 0.1, 0.0, -1.8),
        Vec3::new(half_width - 0.1, 1.95, -0.98),
    );
    builder.clearance(
        FurnitureClearanceKind::WorkingSpace,
        Vec3::new(-half_width + 0.3, 0.0, 0.05),
        Vec3::new(half_width - 0.3, 1.95, 1.5),
    );
}

fn beam(builder: &mut Builder, start: Vec3, end: Vec3, width: f32) {
    let delta = end - start;
    builder.cuboid(
        BuildingLodMaterial::InteriorTimber,
        (start + end) * 0.5,
        Vec3::new(width, delta.length() + width, width),
        Quat::from_rotation_arc(Vec3::Y, delta.normalize()),
        CollisionPolicy::Solid,
    );
}

fn canvas(builder: &mut Builder, half_width: f32, half_depth: f32, eave: f32, ridge: f32) {
    let point = |x: usize, z: usize| {
        let x_fraction = x as f32 / 4.0;
        let z_fraction = z as f32 / 4.0;
        let along = x_fraction * 2.0 - 1.0;
        let pitch = eave + (ridge - eave) * (1.0 - along.abs());
        // Slack between supported ridge/eaves and front/rear rafters, with seams following the fall.
        let sag = (std::f32::consts::PI * x_fraction * 2.0).sin().abs()
            * (std::f32::consts::PI * z_fraction).sin()
            * 0.10;
        Vec3::new(
            along * half_width,
            pitch - sag,
            (z_fraction * 2.0 - 1.0) * half_depth,
        )
    };
    for x in 0..4 {
        for z in 0..4 {
            let positions = [
                point(x, z),
                point(x + 1, z),
                point(x + 1, z + 1),
                point(x, z + 1),
            ];
            for triangle in [
                [positions[0], positions[1], positions[2]],
                [positions[0], positions[2], positions[3]],
            ] {
                let normal = (triangle[2] - triangle[0])
                    .cross(triangle[1] - triangle[0])
                    .normalize();
                builder.triangle(BuildingLodMaterial::UndyedCloth, triangle, normal);
                builder.triangle(BuildingLodMaterial::UndyedCloth, triangle, -normal);
            }
        }
    }
}
