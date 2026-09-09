use super::*;

pub(super) fn loading_hood(a: &mut Assembly<'_>, w: f32, d: f32) {
    let z = d * 0.25;
    for x in [w + 0.8, w + 4.8] {
        for end in [z - 2.2, z + 2.2] {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Vec3::new(x, 1.7, end),
                Vec3::new(0.3, 3.4, 0.3),
                true,
            );
        }
    }
    for end in [z - 2.2, z + 2.2] {
        a.wall(
            Vec2::new(w + 0.8, end),
            Vec2::new(w + 4.8, end),
            if end < z { Vec2::NEG_Y } else { Vec2::Y },
            3.4,
            0.3,
            true,
        );
    }
    for x in [w + 0.8, w + 4.8] {
        a.wall(
            Vec2::new(x, z - 2.2),
            Vec2::new(x, z + 2.2),
            if x < w + 2.8 { Vec2::NEG_X } else { Vec2::X },
            3.4,
            0.3,
            true,
        );
    }
    // A grounded lifting frame occupies the rear edge of the hood, beside the clear cart path.
    for x in [w + 1.1, w + 4.5] {
        a.part(
            WorkplaceFeature::LoadingHoist,
            WorkplaceMaterial::Timber,
            Vec3::new(x, 1.5, z + 1.55),
            Vec3::new(0.24, 3.0, 0.24),
            true,
        );
    }
    a.part(
        WorkplaceFeature::LoadingHoist,
        WorkplaceMaterial::Timber,
        Vec3::new(w + 2.8, 3.12, z + 1.55),
        Vec3::new(3.8, 0.24, 0.3),
        true,
    );
    super::hoist::rig(a, Vec2::new(w + 2.8, z + 1.55));
}
