use super::{
    builder::{Builder, CollisionPolicy},
    *,
};
use crate::BuildingLodMaterial;
use bevy::math::{Quat, Vec2};
use std::f32::consts::TAU;

const STAVES: usize = 12;

pub(super) fn barrel(builder: &mut Builder, variant: FurnitureVariant) {
    let (radius, height) = match variant {
        FurnitureVariant::Compact => (0.38, 0.86),
        FurnitureVariant::Broad => (0.47, 1.08),
    };
    let rings = [
        (0.0, 0.84),
        (0.16, 0.94),
        (0.5, 1.0),
        (0.84, 0.94),
        (1.0, 0.84),
    ];
    for stave in 0..STAVES {
        let angles = [
            stave as f32 * TAU / STAVES as f32,
            (stave + 1) as f32 * TAU / STAVES as f32,
        ];
        let direction = |angle: f32| Vec3::new(angle.sin(), 0.0, angle.cos());
        for pair in rings.windows(2) {
            let [(y0, r0), (y1, r1)] = [pair[0], pair[1]];
            let points = [
                direction(angles[0]) * radius * r0 + Vec3::Y * y0 * height,
                direction(angles[1]) * radius * r0 + Vec3::Y * y0 * height,
                direction(angles[1]) * radius * r1 + Vec3::Y * y1 * height,
                direction(angles[0]) * radius * r1 + Vec3::Y * y1 * height,
            ];
            let normal = (points[1] - points[0])
                .cross(points[3] - points[0])
                .normalize();
            builder.quad(BuildingLodMaterial::InteriorTimber, points, normal);
        }
        for (y, normal) in [(0.0, -Vec3::Y), (height, Vec3::Y)] {
            builder.triangle(
                BuildingLodMaterial::InteriorTimber,
                [
                    Vec3::Y * y,
                    direction(angles[0]) * radius * 0.84 + Vec3::Y * y,
                    direction(angles[1]) * radius * 0.84 + Vec3::Y * y,
                ],
                normal,
            );
        }
        builder.support(direction(angles[0]) * radius * 0.84);
    }
    for fraction in [0.16, 0.84] {
        hoop(builder, radius * 0.94, height * fraction);
    }
    builder.collider(
        Vec3::Y * height * 0.5,
        Vec3::new(radius * 2.0, height, radius * 2.0),
    );
    builder.clearance(
        FurnitureClearanceKind::Access,
        Vec3::new(-radius, 0.0, -radius - 0.65),
        Vec3::new(radius, 1.8, -radius - 0.08),
    );
}

fn hoop(builder: &mut Builder, radius: f32, y: f32) {
    for segment in 0..STAVES {
        let angle = segment as f32 * TAU / STAVES as f32;
        let next = (segment + 1) as f32 * TAU / STAVES as f32;
        let point = |a: f32, r: f32, h: f32| Vec3::new(a.sin() * r, h, a.cos() * r);
        let outer = radius + 0.018;
        let inner = radius - 0.010;
        let bottom = y - 0.045;
        let top = y + 0.045;
        builder.quad(
            BuildingLodMaterial::Iron,
            [
                point(angle, outer, bottom),
                point(next, outer, bottom),
                point(next, outer, top),
                point(angle, outer, top),
            ],
            Vec3::new(
                ((angle + next) * 0.5).sin(),
                0.0,
                ((angle + next) * 0.5).cos(),
            ),
        );
        for (height, normal) in [(bottom, -Vec3::Y), (top, Vec3::Y)] {
            builder.quad(
                BuildingLodMaterial::Iron,
                [
                    point(angle, inner, height),
                    point(next, inner, height),
                    point(next, outer, height),
                    point(angle, outer, height),
                ],
                normal,
            );
        }
    }
}

pub(super) fn cargo(builder: &mut Builder, variant: FurnitureVariant) {
    let wide = variant == FurnitureVariant::Broad;
    crate_box(
        builder,
        Vec3::new(-0.45, 0.0, 0.0),
        Vec3::new(0.85, 0.65, 0.8),
    );
    crate_box(
        builder,
        Vec3::new(0.46, 0.0, 0.10),
        Vec3::new(0.72, 0.5, 0.7),
    );
    if wide {
        crate_box(
            builder,
            Vec3::new(-0.45, 0.65, 0.03),
            Vec3::new(0.7, 0.48, 0.65),
        );
    }
    sack(builder, Vec2::new(0.44, 0.06), 0.5, 0.30, 0.46);
    builder.clearance(
        FurnitureClearanceKind::WorkingSpace,
        Vec3::new(-0.92, 0.0, -1.2),
        Vec3::new(0.92, 1.9, -0.5),
    );
}

fn crate_box(builder: &mut Builder, origin: Vec3, size: Vec3) {
    let width = size.x;
    let depth = size.z;
    let height = size.y;
    // Actual slat gaps and strong corner standards make an open cargo box instead of a solid cube.
    for x in [-width * 0.5 + 0.04, width * 0.5 - 0.04] {
        for z in [-depth * 0.5 + 0.04, depth * 0.5 - 0.04] {
            builder.timber(
                origin + Vec3::new(x, height * 0.5, z),
                Vec3::new(0.08, height, 0.08),
            );
        }
    }
    for row in 0..3 {
        let y = 0.11 + row as f32 * (height - 0.18) * 0.5;
        for z in [-depth * 0.5 + 0.03, depth * 0.5 - 0.03] {
            builder.cuboid(
                BuildingLodMaterial::InteriorTimber,
                origin + Vec3::new(0.0, y, z),
                Vec3::new(width, 0.12, 0.06),
                Quat::IDENTITY,
                CollisionPolicy::Decoration,
            );
        }
        for x in [-width * 0.5 + 0.03, width * 0.5 - 0.03] {
            builder.cuboid(
                BuildingLodMaterial::InteriorTimber,
                origin + Vec3::new(x, y, 0.0),
                Vec3::new(0.06, 0.12, depth),
                Quat::IDENTITY,
                CollisionPolicy::Decoration,
            );
        }
    }
    for plank in 0..4 {
        let z = -depth * 0.5 + (plank as f32 + 0.5) * depth / 4.0;
        builder.cuboid(
            BuildingLodMaterial::InteriorTimber,
            origin + Vec3::new(0.0, 0.035, z),
            Vec3::new(width, 0.07, depth / 4.0 - 0.012),
            Quat::IDENTITY,
            CollisionPolicy::Decoration,
        );
        builder.cuboid(
            BuildingLodMaterial::InteriorTimber,
            origin + Vec3::new(0.0, height - 0.035, z),
            Vec3::new(width, 0.07, depth / 4.0 - 0.012),
            Quat::IDENTITY,
            CollisionPolicy::Decoration,
        );
    }
    // One bounded box is the collision proxy; the visible slat gaps remain sub-hand-width.
    builder.collider(origin + Vec3::Y * height * 0.5, size);
}

fn sack(builder: &mut Builder, p: Vec2, base: f32, radius: f32, height: f32) {
    let levels = [
        (base, radius * 0.78),
        (base + height * 0.18, radius),
        (base + height * 0.68, radius * 0.88),
        (base + height * 0.86, radius * 0.35),
        (base + height, radius * 0.18),
    ];
    for segment in 0..8 {
        let angle = segment as f32 * TAU / 8.0;
        let next = (segment + 1) as f32 * TAU / 8.0;
        let at = |a: f32, y: f32, r: f32| Vec3::new(p.x + a.sin() * r, y, p.y + a.cos() * r * 0.8);
        for pair in levels.windows(2) {
            let points = [
                at(angle, pair[0].0, pair[0].1),
                at(next, pair[0].0, pair[0].1),
                at(next, pair[1].0, pair[1].1),
                at(angle, pair[1].0, pair[1].1),
            ];
            let normal = (points[1] - points[0])
                .cross(points[3] - points[0])
                .normalize();
            builder.quad(BuildingLodMaterial::UndyedCloth, points, normal);
        }
        builder.triangle(
            BuildingLodMaterial::UndyedCloth,
            [
                Vec3::new(p.x, base + height, p.y),
                at(angle, base + height, radius * 0.18),
                at(next, base + height, radius * 0.18),
            ],
            Vec3::Y,
        );
    }
    builder.cuboid(
        BuildingLodMaterial::HempRope,
        Vec3::new(p.x, base + height * 0.85, p.y),
        Vec3::new(radius * 0.74, 0.045, radius * 0.62),
        Quat::IDENTITY,
        CollisionPolicy::Decoration,
    );
    builder.collider(
        Vec3::new(p.x, base + height * 0.5, p.y),
        Vec3::new(radius * 2.0, height, radius * 1.6),
    );
}
