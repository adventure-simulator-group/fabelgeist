use super::*;

pub(super) fn assemble(builder: &mut Builder, kind: FurnitureKind, size: Vec3) {
    let backed = matches!(kind, FurnitureKind::Chair | FurnitureKind::ChurchBench);
    let seat_top = if backed { 0.47 } else { size.y };
    legs(builder, size, seat_top - 0.06, 0.07);
    boards(
        builder,
        Vec3::Y * (seat_top - 0.03),
        Vec3::new(size.x, 0.06, size.z),
    );
    for z in [-1.0, 1.0] {
        builder.timber(
            Vec3::new(0.0, 0.19, z * (size.z - 0.07) * 0.5),
            Vec3::new(size.x - 0.07, 0.06, 0.06),
        );
    }
    if backed {
        let back_z = (size.z - 0.07) * 0.5;
        for x in [-1.0, 1.0] {
            builder.timber(
                Vec3::new(x * (size.x - 0.07) * 0.5, size.y * 0.5, back_z),
                Vec3::new(0.07, size.y, 0.07),
            );
        }
        let back_height = size.y - seat_top - 0.12;
        if kind == FurnitureKind::Chair {
            // Joined chair with two vertical splats and an upper crest rail.
            for x in [-0.17, 0.17] {
                builder.timber(
                    Vec3::new(x * size.x, seat_top + back_height * 0.5, back_z),
                    Vec3::new(size.x * 0.17, back_height, 0.05),
                );
            }
        } else {
            builder.timber(
                Vec3::new(0.0, seat_top + back_height * 0.5, back_z),
                Vec3::new(size.x - 0.1, back_height, 0.05),
            );
        }
        builder.timber(
            Vec3::new(0.0, size.y - 0.06, back_z),
            Vec3::new(size.x, 0.12, 0.07),
        );
    }
}
