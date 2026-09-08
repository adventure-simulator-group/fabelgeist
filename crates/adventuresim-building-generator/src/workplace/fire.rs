use super::*;

pub(super) fn smithy(a: &mut Assembly<'_>, w: f32, d: f32) {
    let p = Vec2::new(w + 3.6, d * 0.4);
    a.part(
        WorkplaceFeature::Forge,
        WorkplaceMaterial::Masonry,
        Vec3::new(p.x, 0.55, p.y),
        Vec3::new(2.5, 1.1, 2.4),
        true,
    );
    for x in [p.x - 1.0, p.x + 1.0] {
        a.part(
            WorkplaceFeature::Forge,
            WorkplaceMaterial::Masonry,
            Vec3::new(x, 1.95, p.y + 0.85),
            Vec3::new(0.4, 1.7, 0.45),
            true,
        );
    }
    perforated_slab(
        a,
        WorkplaceFeature::Forge,
        Vec3::new(p.x - 1.25, 2.8, p.y + 0.2),
        Vec3::new(p.x + 1.25, 3.1, p.y + 1.2),
        Vec2::new(p.x, p.y + 0.7),
    );
    chimney(a, Vec2::new(p.x, p.y + 0.7), 3.1, 4.1);
    a.passage(
        Vec3::new(p.x - 0.3, 1.12, p.y + 0.4),
        Vec3::new(p.x + 0.3, 7.1, p.y + 1.0),
    );
    a.part(
        WorkplaceFeature::Anvil,
        WorkplaceMaterial::Timber,
        Vec3::new(p.x, 0.4, p.y - 2.2),
        Vec3::new(0.8, 0.8, 0.7),
        false,
    );
    a.part(
        WorkplaceFeature::Anvil,
        WorkplaceMaterial::Iron,
        Vec3::new(p.x, 0.92, p.y - 2.2),
        Vec3::new(1.25, 0.24, 0.55),
        true,
    );
    a.part(
        WorkplaceFeature::Anvil,
        WorkplaceMaterial::Iron,
        Vec3::new(p.x + 0.65, 0.99, p.y - 2.2),
        Vec3::new(0.5, 0.1, 0.25),
        false,
    );
    rack(a, Vec2::new(w + 3.6, d - 2.3));
    bin(a, Vec2::new(1.3, 2.2), Vec2::new(1.8, 2.5), 0.9);
}

pub(super) fn bakehouse(a: &mut Assembly<'_>, w: f32, d: f32) {
    let p = Vec2::new(w + 3.6, d * 0.45);
    a.part(
        WorkplaceFeature::Oven,
        WorkplaceMaterial::Masonry,
        Vec3::new(p.x, 0.3, p.y),
        Vec3::new(2.8, 0.6, 3.6),
        true,
    );
    for x in [p.x - 1.0, p.x + 1.0] {
        a.part(
            WorkplaceFeature::Oven,
            WorkplaceMaterial::Masonry,
            Vec3::new(x, 1.05, p.y),
            Vec3::new(0.65, 0.9, 3.6),
            true,
        );
    }
    a.part(
        WorkplaceFeature::Oven,
        WorkplaceMaterial::Masonry,
        Vec3::new(p.x, 1.05, p.y + 1.55),
        Vec3::new(1.4, 0.9, 0.5),
        true,
    );
    for tier in 0..5 {
        let width = 2.8 - tier as f32 * 0.35;
        let bottom = 1.5 + tier as f32 * 0.2;
        perforated_slab(
            a,
            WorkplaceFeature::Oven,
            Vec3::new(p.x - width * 0.5, bottom, p.y - 1.8),
            Vec3::new(p.x + width * 0.5, bottom + 0.2, p.y + 1.8),
            Vec2::new(p.x, p.y + 1.05),
        );
    }
    chimney(a, Vec2::new(p.x, p.y + 1.05), 2.5, 3.5);
    a.passage(
        Vec3::new(p.x - 0.3, 1.52, p.y + 0.75),
        Vec3::new(p.x + 0.3, 5.9, p.y + 1.35),
    );
    counter(a, Vec2::new(w * 0.5 - 2.2, 2.5), Vec2::new(1.1, 3.0));
    rack(a, Vec2::new(w + 3.6, d - 2.0));
}

pub(super) fn chimney(a: &mut Assembly<'_>, p: Vec2, base: f32, height: f32) {
    for x in [-0.42, 0.42] {
        a.part(
            WorkplaceFeature::Flue,
            WorkplaceMaterial::Masonry,
            Vec3::new(p.x + x, base + height * 0.5, p.y),
            Vec3::new(0.18, height, 1.0),
            true,
        );
    }
    for z in [-0.42, 0.42] {
        a.part(
            WorkplaceFeature::Flue,
            WorkplaceMaterial::Masonry,
            Vec3::new(p.x, base + height * 0.5, p.y + z),
            Vec3::new(0.66, height, 0.18),
            true,
        );
    }
}

pub(super) fn perforated_slab(
    a: &mut Assembly<'_>,
    feature: WorkplaceFeature,
    min: Vec3,
    max: Vec3,
    flue: Vec2,
) {
    let opening_half = 0.33;
    let z0 = flue.y - opening_half;
    let z1 = flue.y + opening_half;
    for (lower, upper) in [
        (
            Vec3::new(min.x, min.y, z0),
            Vec3::new(flue.x - opening_half, max.y, z1),
        ),
        (
            Vec3::new(flue.x + opening_half, min.y, z0),
            Vec3::new(max.x, max.y, z1),
        ),
        (min, Vec3::new(max.x, max.y, z0)),
        (Vec3::new(min.x, min.y, z1), max),
    ] {
        a.part(
            feature,
            WorkplaceMaterial::Masonry,
            (lower + upper) * 0.5,
            upper - lower,
            true,
        );
    }
}
