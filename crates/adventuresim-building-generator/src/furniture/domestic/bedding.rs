//! Stuffed linen bedding with chamfered edges and a separate folded-back cover.
use bevy::math::Vec3;

use crate::{BuildingLodMaterial, furniture::builder::Builder};

pub(super) fn dress(builder: &mut Builder, size: Vec3, height: f32) {
    cushion(
        builder,
        Vec3::Y * (height + 0.035),
        Vec3::new(size.x - 0.16, 0.17, size.z - 0.19),
        BuildingLodMaterial::UndyedCloth,
    );
    cushion(
        builder,
        Vec3::new(0.0, height + 0.205, size.z * 0.32),
        Vec3::new((size.x - 0.28).min(0.65), 0.13, 0.42),
        BuildingLodMaterial::UndyedCloth,
    );
    cushion(
        builder,
        Vec3::new(0.0, height + 0.205, -size.z * 0.13),
        Vec3::new(size.x - 0.14, 0.035, size.z * 0.61),
        BuildingLodMaterial::DyedCloth,
    );
    cushion(
        builder,
        Vec3::new(0.0, height + 0.24, size.z * 0.14),
        Vec3::new(size.x - 0.14, 0.025, 0.13),
        BuildingLodMaterial::DyedCloth,
    );
}

fn cushion(builder: &mut Builder, base: Vec3, size: Vec3, material: BuildingLodMaterial) {
    let corner = size.x.min(size.z) * 0.09;
    let outline = [
        [-size.x * 0.5 + corner, -size.z * 0.5],
        [size.x * 0.5 - corner, -size.z * 0.5],
        [size.x * 0.5, -size.z * 0.5 + corner],
        [size.x * 0.5, size.z * 0.5 - corner],
        [size.x * 0.5 - corner, size.z * 0.5],
        [-size.x * 0.5 + corner, size.z * 0.5],
        [-size.x * 0.5, size.z * 0.5 - corner],
        [-size.x * 0.5, -size.z * 0.5 + corner],
    ];
    let rings = [(0.0, 0.86), (0.3, 1.0), (0.65, 0.94), (0.9, 0.72)];
    let point = |i: usize, (y, scale): (f32, f32)| {
        let tuck = 1.0;
        base + Vec3::new(
            outline[i][0] * scale * tuck,
            size.y * y,
            outline[i][1] * scale,
        )
    };
    for i in 0..outline.len() {
        let next = (i + 1) % outline.len();
        for pair in rings.windows(2) {
            let points = [
                point(i, pair[0]),
                point(next, pair[0]),
                point(next, pair[1]),
                point(i, pair[1]),
            ];
            let normal = (points[3] - points[0])
                .cross(points[1] - points[0])
                .normalize();
            builder.quad(material, points, normal);
        }
        for (ring, normal) in [(rings[0], -Vec3::Y), (rings[3], Vec3::Y)] {
            builder.triangle(
                material,
                [
                    base + Vec3::Y * (if normal.y > 0.0 { size.y } else { 0.0 }),
                    point(i, ring),
                    point(next, ring),
                ],
                normal,
            );
        }
    }
    // The central mattress volume supplies support without covering chamfered corners.
    builder.collider(
        base + Vec3::Y * size.y * 0.5,
        size * Vec3::new(0.86, 1.0, 0.86),
    );
}
