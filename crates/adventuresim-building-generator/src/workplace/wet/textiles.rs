use super::*;
use bevy::math::Quat;

pub(super) fn dye_frames(a: &mut Assembly<'_>, w: f32, d: f32) {
    let count = 2 + a.plan.size.extra_bays().min(1);
    for index in 0..count {
        let z = 2.0 + f32::from(index) * (d - 4.0) / f32::from(count - 1);
        let p = Vec2::new(w + 3.0, z);
        drying_frame(a, p, 2.0, 3.2);
        draped_cloth(
            a,
            p,
            3.1,
            if index % 2 == 0 {
                WorkplaceMaterial::DyedCloth
            } else {
                WorkplaceMaterial::UndyedCloth
            },
        );
    }
    folded_stock(a, Vec2::new(1.7, d - 2.3));
    draining_bench(a, Vec2::new(w - 1.7, 2.2));
}

fn drying_frame(a: &mut Assembly<'_>, p: Vec2, width: f32, height: f32) {
    for x in [-width * 0.5, width * 0.5] {
        a.part(
            WorkplaceFeature::DryingFrame,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x + x, height * 0.5, p.y),
            Vec3::new(0.18, height, 0.18),
            true,
        );
    }
    a.part(
        WorkplaceFeature::DryingFrame,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, height - 0.1, p.y),
        Vec3::new(width + 0.3, 0.2, 0.2),
        true,
    );
    for side in [-1.0, 1.0] {
        let brace = a.part(
            WorkplaceFeature::DryingFrame,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x + side * (width * 0.5 - 0.3), height - 0.5, p.y),
            Vec3::new(0.12, 0.9, 0.12),
            true,
        );
        a.orient_part(
            brace,
            Quat::from_rotation_z(side * std::f32::consts::FRAC_PI_4),
        );
    }
}

fn draped_cloth(a: &mut Assembly<'_>, p: Vec2, bar: f32, material: WorkplaceMaterial) {
    // An over-rail fold physically joins unequal front and back hanging lengths.
    a.part(
        WorkplaceFeature::Cloth,
        material,
        Vec3::new(p.x, bar + 0.115, p.y),
        Vec3::new(1.58, 0.03, 0.28),
        true,
    );
    for (z, length) in [(-0.135, 1.75), (0.135, 1.32)] {
        for fold in 0..3 {
            let x = p.x - 0.53 + fold as f32 * 0.53;
            let hanging_length = length - fold as f32 * 0.035;
            // Joined panels give the cloth an uneven lower hem while sharing its upper fold.
            a.part(
                WorkplaceFeature::Cloth,
                material,
                Vec3::new(x, bar + 0.1 - hanging_length * 0.5, p.y + z),
                Vec3::new(0.56, hanging_length, 0.03),
                true,
            );
        }
    }
}

pub(super) fn hide_frame(a: &mut Assembly<'_>, p: Vec2) {
    drying_frame(a, p, 2.8, 2.8);
    for (x, scale) in [(-0.7, 0.9), (0.7, 1.0)] {
        hanging_hide(a, Vec2::new(p.x + x, p.y), 2.8, scale);
    }
}

fn hanging_hide(a: &mut Assembly<'_>, p: Vec2, top: f32, scale: f32) {
    // Neck, broad shoulders, tapered torso and unequal lower tails form a skin silhouette.
    for (x, drop, width, height) in [
        (0.0, 0.12, 0.28, 0.24),
        (0.0, 0.45, 0.98, 0.46),
        (0.0, 0.90, 0.72, 0.46),
        (0.0, 1.25, 0.48, 0.30),
        (-0.16, 1.47, 0.18, 0.23),
        (0.16, 1.43, 0.18, 0.15),
    ] {
        a.part(
            WorkplaceFeature::Hide,
            WorkplaceMaterial::Hide,
            Vec3::new(p.x + x * scale, top - drop * scale, p.y - 0.11),
            Vec3::new(width * scale, height * scale, 0.04),
            true,
        );
    }
}

fn folded_stock(a: &mut Assembly<'_>, p: Vec2) {
    for x in [-0.7, 0.7] {
        for z in [-0.5, 0.5] {
            a.part(
                WorkplaceFeature::Counter,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(p.x + x, 0.45, p.y + z),
                Vec3::new(0.15, 0.9, 0.15),
                true,
            );
        }
    }
    a.part(
        WorkplaceFeature::Counter,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 0.98, p.y),
        Vec3::new(1.8, 0.16, 1.4),
        true,
    );
    for (tier, material) in [
        WorkplaceMaterial::UndyedCloth,
        WorkplaceMaterial::DyedCloth,
        WorkplaceMaterial::UndyedCloth,
    ]
    .into_iter()
    .enumerate()
    {
        a.part(
            WorkplaceFeature::Cloth,
            material,
            Vec3::new(p.x, 1.12 + tier as f32 * 0.12, p.y),
            Vec3::new(1.3, 0.12, 0.9),
            true,
        );
    }
}

fn draining_bench(a: &mut Assembly<'_>, p: Vec2) {
    for x in [-0.65, 0.65] {
        for z in [-0.7, 0.7] {
            a.part(
                WorkplaceFeature::Counter,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(p.x + x, 0.45, p.y + z),
                Vec3::new(0.16, 0.9, 0.16),
                true,
            );
        }
        a.part(
            WorkplaceFeature::Counter,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x + x, 0.96, p.y),
            Vec3::new(0.16, 0.12, 1.8),
            true,
        );
    }
    for slat in 0..6 {
        a.part(
            WorkplaceFeature::Counter,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x, 1.06, p.y - 0.75 + slat as f32 * 0.3),
            Vec3::new(1.8, 0.08, 0.24),
            true,
        );
    }
    a.part(
        WorkplaceFeature::Cloth,
        WorkplaceMaterial::UndyedCloth,
        Vec3::new(p.x, 1.14, p.y),
        Vec3::new(1.2, 0.08, 1.0),
        true,
    );
}
