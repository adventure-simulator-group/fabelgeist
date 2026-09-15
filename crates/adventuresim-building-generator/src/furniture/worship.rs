//! Period worship fittings. Institution selection belongs to the room programme.
use super::{
    FurnitureKey, FurnitureKind,
    builder::{Builder, CollisionPolicy},
};
use crate::BuildingLodMaterial;
use bevy::math::{Quat, Vec3};

pub(super) fn assemble(builder: &mut Builder, key: FurnitureKey) {
    let size = key.interior_spec().unwrap().size_metres;
    match key.kind {
        FurnitureKind::BaptismalFont => font(builder, size),
        FurnitureKind::Pulpit => pulpit(builder, size),
        FurnitureKind::Bima => bima(builder, size),
        FurnitureKind::TorahShrine => shrine(builder, size),
        _ => unreachable!("worship furniture dispatch"),
    }
}

fn font(builder: &mut Builder, size: Vec3) {
    let radius = size.x * 0.5;
    // Octagonal stone bowl, with an inner floor below the rim.
    let profile = [
        (0.0, 0.42),
        (0.11, 0.42),
        (0.14, 0.23),
        (0.43, 0.23),
        (0.56, 0.41),
        (0.86, 0.5),
        (0.89, 0.5),
        (0.89, 0.42),
        (0.67, 0.32),
        (0.62, 0.0),
    ];
    for side in 0..8 {
        let directions = [side, side + 1].map(|i| {
            let angle = std::f32::consts::TAU * i as f32 / 8.0;
            Vec3::new(angle.sin(), 0.0, angle.cos())
        });
        for pair in profile.windows(2) {
            let points = [
                directions[0] * pair[0].1 * size.x + Vec3::Y * pair[0].0,
                directions[1] * pair[0].1 * size.x + Vec3::Y * pair[0].0,
                directions[1] * pair[1].1 * size.x + Vec3::Y * pair[1].0,
                directions[0] * pair[1].1 * size.x + Vec3::Y * pair[1].0,
            ];
            for vertices in [
                [points[0], points[1], points[2]],
                [points[0], points[2], points[3]],
            ] {
                let normal = (vertices[1] - vertices[0])
                    .cross(vertices[2] - vertices[0])
                    .normalize_or_zero();
                if normal != Vec3::ZERO {
                    builder.triangle(BuildingLodMaterial::CarvedSandstone, vertices, normal);
                }
            }
        }
        builder.support(directions[0] * radius * 0.84);
    }
    builder.collider(
        Vec3::Y * 0.31,
        Vec3::new(size.x * 0.45, 0.62, size.x * 0.45),
    );
    builder.collider(
        Vec3::Y * 0.055,
        Vec3::new(size.x * 0.84, 0.11, size.x * 0.84),
    );
    builder.collider(
        Vec3::Y * 0.60,
        Vec3::new(size.x * 0.78, 0.08, size.x * 0.78),
    );
    for side in [-1.0, 1.0] {
        builder.collider(
            Vec3::new(side * size.x * 0.43, 0.73, 0.0),
            Vec3::new(size.x * 0.14, 0.30, size.x * 0.7),
        );
        builder.collider(
            Vec3::new(0.0, 0.73, side * size.x * 0.43),
            Vec3::new(size.x * 0.7, 0.30, size.x * 0.14),
        );
    }
}

fn pulpit(builder: &mut Builder, size: Vec3) {
    let deck = 0.9;
    let front = -size.z * 0.5;
    let platform_z = size.z * 0.5 - 0.5;
    builder.timber(
        Vec3::new(0.0, deck * 0.5, platform_z),
        Vec3::new(size.x, deck, 1.0),
    );
    let run = size.z - 1.0;
    for tread in 0..5 {
        let height = (tread + 1) as f32 * deck / 5.0;
        builder.timber(
            Vec3::new(0.0, height * 0.5, front + (tread as f32 + 0.5) * run / 5.0),
            Vec3::new(size.x * 0.68, height, run / 5.0),
        );
    }
    for side in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(side * (size.x - 0.06) * 0.5, deck + 0.47, platform_z),
            Vec3::new(0.06, 0.94, 1.0),
        );
        // Stair-side handholds rise with the real treads.
        let start = Vec3::new(side * size.x * 0.43, 0.75, front + 0.05);
        let end = Vec3::new(side * (size.x - 0.06) * 0.5, deck + 0.75, platform_z - 0.46);
        builder.timber(
            start * Vec3::new(1.0, 0.5, 1.0),
            Vec3::new(0.05, start.y, 0.05),
        );
        let delta = end - start;
        builder.cuboid(
            BuildingLodMaterial::InteriorTimber,
            (start + end) * 0.5,
            Vec3::new(0.05, delta.length(), 0.05),
            Quat::from_rotation_arc(Vec3::Y, delta.normalize()),
            CollisionPolicy::Solid,
        );
    }
    builder.timber(
        Vec3::new(0.0, deck + 0.47, size.z * 0.5 - 0.03),
        Vec3::new(size.x, 0.94, 0.06),
    );
    builder.cuboid(
        BuildingLodMaterial::InteriorTimber,
        Vec3::new(0.0, deck + 0.98, size.z * 0.5 - 0.16),
        Vec3::new(size.x, 0.05, 0.3),
        Quat::from_rotation_x(0.15),
        CollisionPolicy::Solid,
    );
}

fn bima(builder: &mut Builder, size: Vec3) {
    builder.timber(
        Vec3::new(0.0, 0.14, 0.12),
        Vec3::new(size.x, 0.28, size.z - 0.24),
    );
    builder.timber(
        Vec3::new(0.0, 0.07, -size.z * 0.5 + 0.12),
        Vec3::new(size.x * 0.6, 0.14, 0.24),
    );
    for x in [-0.45, 0.45] {
        for z in [0.0, 0.48] {
            builder.timber(Vec3::new(x, 0.69, z), Vec3::new(0.07, 0.82, 0.07));
        }
    }
    builder.cuboid(
        BuildingLodMaterial::InteriorTimber,
        Vec3::new(0.0, 1.12, 0.24),
        Vec3::new(1.1, 0.06, 0.7),
        Quat::from_rotation_x(-0.1),
        CollisionPolicy::Solid,
    );
}

fn shrine(builder: &mut Builder, size: Vec3) {
    let board = 0.055;
    builder.timber(
        Vec3::new(0.0, board * 0.5, 0.0),
        Vec3::new(size.x, board, size.z),
    );
    builder.timber(
        Vec3::new(0.0, size.y - board * 0.5, 0.0),
        Vec3::new(size.x, board, size.z),
    );
    builder.timber(
        Vec3::new(0.0, size.y * 0.5, size.z * 0.5 - board * 0.5),
        Vec3::new(size.x, size.y, board),
    );
    for side in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(side * (size.x - board) * 0.5, size.y * 0.5, 0.0),
            Vec3::new(board, size.y, size.z),
        );
    }
    // Soft hanging folds remain a separate textile surface below the supporting rod.
    let panels = 12;
    let width = size.x - board * 2.0;
    let point = |i: usize, top: bool| {
        let phase = i as f32 * std::f32::consts::PI;
        Vec3::new(
            width * (i as f32 / panels as f32 - 0.5),
            if top {
                size.y * 0.94
            } else {
                0.10 + 0.015 * phase.cos()
            },
            -size.z * 0.5 + board + 0.025 * phase.cos(),
        )
    };
    for panel in 0..panels {
        let face = [
            point(panel, false),
            point(panel + 1, false),
            point(panel + 1, true),
            point(panel, true),
        ];
        let normal = (face[1] - face[0]).cross(face[3] - face[0]).normalize();
        builder.quad(BuildingLodMaterial::DyedCloth, face, normal);
        builder.quad(
            BuildingLodMaterial::DyedCloth,
            [face[3], face[2], face[1], face[0]],
            -normal,
        );
    }
    builder.timber(
        Vec3::new(0.0, size.y * 0.94, -size.z * 0.5 + board),
        Vec3::new(size.x, 0.035, 0.035),
    );
}
