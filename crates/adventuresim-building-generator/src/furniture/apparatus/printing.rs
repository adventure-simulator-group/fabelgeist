//! Screw press and compositor's cases; no later iron press or mechanized feed.
use super::*;

pub(super) fn press(builder: &mut Builder, size: Vec3) {
    let bed = 0.78;
    let uprights = size.x * 0.32;
    for side in [-1.0, 1.0] {
        let x = side * uprights;
        builder.timber(Vec3::new(x, 0.065, 0.0), Vec3::new(0.22, 0.13, size.z));
        builder.timber(
            Vec3::new(x, size.y * 0.5, size.z * 0.16),
            Vec3::new(0.15, size.y, 0.18),
        );
        builder.timber(
            Vec3::new(x, bed * 0.5, -size.z * 0.36),
            Vec3::new(0.12, bed, 0.12),
        );
        builder.timber(
            Vec3::new(x, bed - 0.06, 0.0),
            Vec3::new(0.12, 0.12, size.z * 0.88),
        );
    }
    builder.timber(
        Vec3::new(0.0, size.y - 0.12, size.z * 0.16),
        Vec3::new(size.x * 0.85, 0.24, 0.22),
    );
    builder.timber(
        Vec3::new(0.0, bed + 0.035, 0.0),
        Vec3::new(size.x * 0.6, 0.07, size.z * 0.84),
    );
    let platen = Vec3::new(0.0, 1.18, size.z * 0.16);
    builder.timber(platen, Vec3::new(size.x * 0.56, 0.1, size.z * 0.3));
    builder.turned(
        BuildingLodMaterial::InteriorTimber,
        Vec3::new(0.0, platen.y + 0.05, platen.z),
        &[(0.0, 0.065), (size.y - platen.y - 0.06, 0.065)],
    );
    // Raised helical thread follows the screw shaft rather than stacked disc rings.
    let turns = 5;
    let steps = turns * 12;
    for step in 0..steps {
        let point = |i: usize| {
            let fraction = i as f32 / steps as f32;
            let angle = fraction * turns as f32 * std::f32::consts::TAU;
            Vec3::new(
                angle.sin() * 0.067,
                1.24 + fraction * (size.y - 1.48),
                platen.z + angle.cos() * 0.067,
            )
        };
        let a = point(step);
        let b = point(step + 1);
        let delta = b - a;
        builder.cuboid(
            BuildingLodMaterial::InteriorTimber,
            (a + b) * 0.5,
            Vec3::new(0.013, delta.length() + 0.003, 0.013),
            Quat::from_rotation_arc(Vec3::Y, delta.normalize()),
            CollisionPolicy::Decoration,
        );
    }
    builder.timber(
        Vec3::new(0.0, 1.3, platen.z - size.z * 0.24),
        Vec3::new(0.035, 0.035, size.z * 0.52),
    );
    // Extended carriage holds the chase in front of the platen.
    let chase = Vec3::new(0.0, bed + 0.08, -size.z * 0.22);
    for side in [-1.0, 1.0] {
        metal(
            builder,
            chase + Vec3::X * side * size.x * 0.23,
            Vec3::new(0.025, 0.025, size.z * 0.31),
        );
        metal(
            builder,
            chase + Vec3::Z * side * size.z * 0.15,
            Vec3::new(size.x * 0.46, 0.025, 0.025),
        );
    }
    builder.collider(
        Vec3::new(0.0, (1.23 + size.y) * 0.5, platen.z),
        Vec3::new(0.14, size.y - 1.23, 0.14),
    );
}

pub(super) fn type_case(builder: &mut Builder, size: Vec3) {
    let top = size.y - 0.09;
    table(builder, size, top);
    // Open shallow compartments; drawers and the case rest on a joined stand.
    let depth = size.z - 0.03;
    for column in 0..9 {
        builder.timber(
            Vec3::new(
                -size.x * 0.5 + 0.015 + column as f32 * (size.x - 0.03) / 8.0,
                top + 0.04,
                0.0,
            ),
            Vec3::new(0.018, 0.08, depth),
        );
    }
    for row in 0..5 {
        builder.timber(
            Vec3::new(0.0, top + 0.04, -depth * 0.5 + row as f32 * depth / 4.0),
            Vec3::new(size.x, 0.08, 0.018),
        );
    }
    for column in 0..8 {
        for row in 0..4 {
            for piece in 0..2 {
                builder.cuboid(
                    BuildingLodMaterial::LeadAlloy,
                    Vec3::new(
                        -size.x * 0.5 + (column as f32 + 0.3 + piece as f32 * 0.2) * size.x / 8.0,
                        top + 0.025,
                        -depth * 0.5 + (row as f32 + 0.5) * depth / 4.0,
                    ),
                    Vec3::new(size.x / 55.0, 0.05, depth / 8.0),
                    Quat::IDENTITY,
                    CollisionPolicy::Decoration,
                );
            }
        }
    }
}
