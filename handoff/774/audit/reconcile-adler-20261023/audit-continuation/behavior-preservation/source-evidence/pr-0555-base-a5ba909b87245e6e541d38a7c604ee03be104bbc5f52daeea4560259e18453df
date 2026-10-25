use super::*;

pub(super) fn balance(builder: &mut Builder, size: Vec3) {
    let top = 0.78;
    table(builder, size, top);
    let beam_y = size.y - 0.09;
    builder.timber(
        Vec3::Y * ((beam_y + top) * 0.5),
        Vec3::new(0.065, beam_y - top, 0.065),
    );
    metal(
        builder,
        Vec3::Y * beam_y,
        Vec3::new(size.x * 0.82, 0.028, 0.024),
    );
    metal(
        builder,
        Vec3::Y * (beam_y + 0.035),
        Vec3::new(0.012, 0.11, 0.012),
    );
    for side in [-1.0, 1.0] {
        metal(
            builder,
            Vec3::new(0.0, beam_y, side * 0.038),
            Vec3::new(0.065, 0.08, 0.012),
        );
    }
    metal(builder, Vec3::Y * beam_y, Vec3::new(0.015, 0.015, 0.09));
    let pan_y = top + 0.13;
    let radius = size.x * 0.135;
    for side in [-1.0, 1.0] {
        let x = side * size.x * 0.32;
        builder.turned(
            BuildingLodMaterial::Bronze,
            Vec3::new(x, pan_y, 0.0),
            &[
                (0.0, radius * 0.6),
                (0.045, radius),
                (0.055, radius),
                (0.018, radius * 0.57),
            ],
        );
        for attachment in 0..3 {
            let angle = attachment as f32 * std::f32::consts::TAU / 3.0;
            let start = Vec3::new(
                x + angle.sin() * radius * 0.86,
                pan_y + 0.045,
                angle.cos() * radius * 0.86,
            );
            let end = Vec3::new(x, beam_y, 0.0);
            let delta = end - start;
            builder.cuboid(
                BuildingLodMaterial::Iron,
                (start + end) * 0.5,
                Vec3::new(0.008, delta.length(), 0.008),
                Quat::from_rotation_arc(Vec3::Y, delta.normalize()),
                CollisionPolicy::Decoration,
            );
        }
    }
    // Nesting weights on the table remain distinct from the suspended pans.
    for index in 0..3 {
        let radius = 0.024 + index as f32 * 0.01;
        builder.turned(
            BuildingLodMaterial::Bronze,
            Vec3::new((index as f32 - 1.0) * 0.11, top, -size.z * 0.32),
            &[(0.0, radius), (0.055, radius), (0.063, radius * 0.8)],
        );
    }
    builder.collider(
        Vec3::Y * ((top + size.y) * 0.5),
        Vec3::new(0.065, size.y - top, 0.065),
    );
}

pub(super) fn reckoning(builder: &mut Builder, size: Vec3) {
    let top = size.y - 0.018;
    table(builder, size, top);
    // Inlaid lines represent successive powers of ten; counters sit on lines and spaces.
    for row in 0..4 {
        builder.cuboid(
            BuildingLodMaterial::UndyedCloth,
            Vec3::new(0.0, top + 0.001, (row as f32 - 1.5) * 0.13),
            Vec3::new(size.x * 0.78, 0.002, 0.006),
            Quat::IDENTITY,
            CollisionPolicy::Decoration,
        );
    }
    builder.cuboid(
        BuildingLodMaterial::UndyedCloth,
        Vec3::new(0.0, top + 0.001, 0.0),
        Vec3::new(0.006, 0.002, 0.46),
        Quat::IDENTITY,
        CollisionPolicy::Decoration,
    );
    for (column, row) in [
        (-2.0, -1.5),
        (-1.0, -1.5),
        (-1.5, -0.5),
        (1.0, -1.5),
        (2.0, -1.5),
        (3.0, -1.5),
        (1.0, -0.5),
        (2.0, -0.5),
    ] {
        builder.turned(
            BuildingLodMaterial::Bronze,
            Vec3::new(column * 0.09, top, row * 0.13),
            &[(0.0, 0.018), (0.004, 0.018)],
        );
    }
}
