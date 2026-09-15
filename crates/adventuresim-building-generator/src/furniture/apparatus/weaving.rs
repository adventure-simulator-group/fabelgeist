//! Hand-operated horizontal loom: warp and cloth beams, heddles, beater, and foot treadles.
use super::*;
mod motion;

pub(super) fn loom(builder: &mut Builder, size: Vec3) {
    let width = size.x - 0.16;
    let depth = size.z - 0.76;
    motion::seat(builder, width, depth);
    for side in [-1.0, 1.0] {
        for end in [-1.0, 1.0] {
            builder.timber(
                Vec3::new(side * width * 0.5, size.y * 0.5, end * depth * 0.5),
                Vec3::new(0.1, size.y, 0.1),
            );
        }
        for y in [0.19, size.y - 0.08] {
            builder.timber(
                Vec3::new(side * width * 0.5, y, 0.0),
                Vec3::new(0.08, 0.1, depth),
            );
        }
    }
    motion::beams(builder, width, depth);
    builder.timber(
        Vec3::new(0.0, size.y - 0.08, 0.0),
        Vec3::new(width, 0.1, 0.1),
    );
    // Cloth on the front beam and a taut warp behind the heddles.
    builder.cuboid(
        BuildingLodMaterial::UndyedCloth,
        Vec3::new(0.0, 0.85, -depth * 0.32),
        Vec3::new(width * 0.8, 0.008, depth * 0.34),
        Quat::IDENTITY,
        CollisionPolicy::Decoration,
    );
    for thread in 0..12 {
        let x = (thread as f32 / 11.0 - 0.5) * width * 0.8;
        fiber(
            builder,
            BuildingLodMaterial::UndyedCloth,
            Vec3::new(x, 0.85, depth * 0.16),
            Vec3::new(0.004, 0.004, depth * 0.64),
        );
        let z = if thread % 2 == 0 { -0.07 } else { 0.07 };
        for (y, length) in [(0.745, 0.19), (1.015, 0.25)] {
            fiber(
                builder,
                BuildingLodMaterial::HempRope,
                Vec3::new(x, y, z),
                Vec3::new(0.006, length, 0.006),
            );
        }
        // A loop around each selected warp strand joins upper and lower heddle cords.
        for side in [-1.0, 1.0] {
            fiber(
                builder,
                BuildingLodMaterial::HempRope,
                Vec3::new(x + side * 0.009, 0.86, z),
                Vec3::new(0.006, 0.06, 0.006),
            );
        }
        for y in [0.83, 0.89] {
            fiber(
                builder,
                BuildingLodMaterial::HempRope,
                Vec3::new(x, y, z),
                Vec3::new(0.024, 0.006, 0.006),
            );
        }
        builder.timber(Vec3::new(x, 0.89, -0.24), Vec3::new(0.006, 0.30, 0.012));
    }
    for z in [-0.07, 0.07] {
        let treadle_x = if z < 0.0 { -0.13 } else { 0.13 };
        builder.cuboid(
            BuildingLodMaterial::HempRope,
            Vec3::new(treadle_x, 0.425, z),
            Vec3::new(0.009, 0.45, 0.009),
            Quat::IDENTITY,
            CollisionPolicy::Decoration,
        );
        for side in [-1.0, 1.0] {
            builder.cuboid(
                BuildingLodMaterial::HempRope,
                Vec3::new(side * width * 0.35, 0.895, z),
                Vec3::new(0.008, 0.49, 0.008),
                Quat::IDENTITY,
                CollisionPolicy::Decoration,
            );
        }
        for y in [0.65, 1.14] {
            builder.timber(Vec3::new(0.0, y, z), Vec3::new(width * 0.86, 0.025, 0.025));
        }
    }
    motion::suspension(builder, width, size.y);
    // Beater hangs from the top rails, independently of the two heddle frames.
    builder.timber(
        Vec3::new(0.0, size.y - 0.08, -0.24),
        Vec3::new(width, 0.1, 0.1),
    );
    builder.timber(
        Vec3::new(0.0, 0.19, depth * 0.15),
        Vec3::new(width, 0.08, 0.08),
    );
    for side in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(side * width * 0.43, (size.y + 0.65) * 0.5, -0.24),
            Vec3::new(0.03, size.y - 0.65, 0.035),
        );
        builder.timber(
            Vec3::new(side * 0.13, 0.19, -depth * 0.12),
            Vec3::new(0.1, 0.04, depth * 0.64),
        );
    }
    builder.timber(
        Vec3::new(0.0, 0.73, -0.24),
        Vec3::new(width * 0.9, 0.035, 0.05),
    );
    builder.timber(
        Vec3::new(0.0, 1.05, -0.24),
        Vec3::new(width * 0.9, 0.035, 0.05),
    );
    builder.collider(
        Vec3::new(0.0, 0.85, 0.0),
        Vec3::new(width * 0.8, 0.05, depth),
    );
}

// Crossed narrow ribbons retain the representative yarn silhouette without capped boxes.
fn fiber(builder: &mut Builder, material: BuildingLodMaterial, centre: Vec3, size: Vec3) {
    let axis = crate::member_uv::grain_axis(size);
    let half_length = size.dot(axis) * 0.5;
    let across = if axis == Vec3::Y { Vec3::X } else { Vec3::Y };
    for side in [across, axis.cross(across)] {
        let radius = size.dot(side.abs()) * 0.5;
        let points = [
            centre - axis * half_length - side * radius,
            centre + axis * half_length - side * radius,
            centre + axis * half_length + side * radius,
            centre - axis * half_length + side * radius,
        ];
        builder.quad(material, points, axis.cross(side));
    }
}
