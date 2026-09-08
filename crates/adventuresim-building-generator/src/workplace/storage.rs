use super::*;

pub(super) fn barn(a: &mut Assembly<'_>, w: f32, d: f32) {
    for x in [1.4, w - 1.4] {
        for z in [d * 0.3, d * 0.65] {
            bin(a, Vec2::new(x, z), Vec2::new(2.0, 3.0), 1.25);
        }
    }
    // The tall central threshing passage remains uninterrupted from front to rear.
    rack(a, Vec2::new(w + 3.0, d - 2.4));
}

pub(super) fn stable(a: &mut Assembly<'_>, w: f32, d: f32) {
    for bay in 0..(d / 3.0) as u32 {
        let z = 1.5 + bay as f32 * 3.0;
        a.part(
            WorkplaceFeature::Stall,
            WorkplaceMaterial::Timber,
            Vec3::new(1.8, 0.7, z + 1.25),
            Vec3::new(3.2, 1.4, 0.1),
            true,
        );
        a.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::Timber,
            Vec3::new(3.35, 1.05, z + 1.25),
            Vec3::new(0.16, 2.1, 0.16),
            true,
        );
        trough(a, Vec2::new(0.8, z));
    }
    rack(a, Vec2::new(w + 3.0, d - 2.4));
}

pub(super) fn granary(a: &mut Assembly<'_>, w: f32, d: f32) {
    for x in [1.5] {
        for z in [2.5, d * 0.5] {
            bin(a, Vec2::new(x, z), Vec2::new(2.0, 2.4), 1.4);
        }
    }
    // An elevated rear storage gallery is reached by a real flight with 0.18m risers.
    let loft_start = d - 3.0;
    for x in [0.6, w - 0.6] {
        a.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::Timber,
            Vec3::new(x, 1.5, loft_start + 0.2),
            Vec3::new(0.26, 3.0, 0.26),
            true,
        );
    }
    a.part(
        WorkplaceFeature::Beam,
        WorkplaceMaterial::Timber,
        Vec3::new(w * 0.5, 3.1, loft_start + 0.2),
        Vec3::new(w - 0.6, 0.2, 0.3),
        true,
    );
    a.part(
        WorkplaceFeature::Floor,
        WorkplaceMaterial::Timber,
        Vec3::new(w * 0.5, 3.28, d - 1.5),
        Vec3::new(w - 0.6, 0.16, 2.8),
        true,
    );
    for step in 0..18 {
        let rise = (step + 1) as f32 * (3.36 / 18.0);
        a.part(
            WorkplaceFeature::Stair,
            WorkplaceMaterial::Timber,
            Vec3::new(
                w - 1.0,
                rise * 0.5,
                loft_start - 4.8 + (step as f32 + 0.5) * (4.8 / 18.0),
            ),
            Vec3::new(1.2, rise, 4.8 / 18.0),
            false,
        );
        let z = loft_start - 4.8 + step as f32 * (4.8 / 18.0);
        a.passage(
            Vec3::new(w - 1.55, rise + 0.03, z + 0.01),
            Vec3::new(w - 0.45, rise + 2.0, z + 4.8 / 18.0 - 0.01),
        );
    }
    for x in [1.4, w * 0.5] {
        bin_at(a, Vec2::new(x, d - 1.5), Vec2::new(1.6, 1.8), 1.3, 3.36);
    }
    upper_storage_floor(a, w, d);
    // A freestanding loading frame makes the storage use legible without a decorative false door.
    let z = 1.2;
    for x in [0.7, 2.7] {
        a.part(
            WorkplaceFeature::Hoist,
            WorkplaceMaterial::Timber,
            Vec3::new(x, 2.1, z),
            Vec3::new(0.22, 4.2, 0.22),
            true,
        );
    }
    a.part(
        WorkplaceFeature::Hoist,
        WorkplaceMaterial::Timber,
        Vec3::new(1.7, 4.31, z),
        Vec3::new(2.5, 0.22, 0.28),
        true,
    );
    a.part(
        WorkplaceFeature::Hoist,
        WorkplaceMaterial::Iron,
        Vec3::new(1.7, 3.3, z),
        Vec3::new(0.055, 2.0, 0.055),
        false,
    );
}

pub(super) fn upper_storage_floor(a: &mut Assembly<'_>, w: f32, d: f32) {
    for z in [0.6, d * 0.5, d - 0.6] {
        for x in [0.4, w - 2.0] {
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::Timber,
                Vec3::new(x, 1.5, z),
                Vec3::new(0.22, 3.0, 0.22),
                true,
            );
        }
        a.part(
            WorkplaceFeature::Beam,
            WorkplaceMaterial::Timber,
            Vec3::new((w - 1.6) * 0.5, 3.1, z),
            Vec3::new(w - 1.9, 0.2, 0.25),
            true,
        );
    }
    a.part(
        WorkplaceFeature::Floor,
        WorkplaceMaterial::Timber,
        Vec3::new((w - 1.4) * 0.5, 3.28, d * 0.5),
        Vec3::new(w - 2.0, 0.16, d - 0.6),
        true,
    );
}
