//! Enclosed malt kiln: fire chamber, open drying grate, loading hatch and hollow low hood.
use super::*;

pub(super) fn drying_kiln(a: &mut Assembly<'_>, p: Vec2) {
    for x in [-1.0, 1.0] {
        masonry(a, p, Vec3::new(x, 1.2, 0.0), Vec3::new(0.3, 2.4, 2.6));
    }
    masonry(a, p, Vec3::new(0.0, 1.2, 1.15), Vec3::new(1.7, 2.4, 0.3));
    // Separate firing and malt-loading apertures are real holes, not painted rectangles.
    for x in [-0.75, 0.75] {
        masonry(a, p, Vec3::new(x, 0.45, -1.15), Vec3::new(0.8, 0.9, 0.3));
    }
    for (y, height) in [(1.15, 0.5), (2.25, 0.3)] {
        masonry(a, p, Vec3::new(0.0, y, -1.15), Vec3::new(2.3, height, 0.3));
    }
    for x in [-0.85, 0.85] {
        masonry(a, p, Vec3::new(x, 1.75, -1.15), Vec3::new(0.6, 0.7, 0.3));
    }
    drying_grate(a, p);
    // Each inward-stepped course overlaps the one beneath; the centre stays open to the sky.
    for tier in 0..4 {
        let outer = Vec2::new(2.3, 2.6) - Vec2::splat(tier as f32 * 0.28);
        hood_course(a, p, outer, 2.4 + tier as f32 * 0.2, 0.2);
    }
    hood_course(a, p, Vec2::new(1.18, 1.48), 3.2, 0.6);
    a.passage(
        Vec3::new(p.x - 0.25, 0.05, p.y - 1.32),
        Vec3::new(p.x + 0.25, 0.8, p.y + 0.85),
    );
    // A reserved continuous vertical vent between the grate bars also guards later roof edits.
    a.passage(
        Vec3::new(p.x - 0.12, 0.8, p.y - 0.1),
        Vec3::new(p.x + 0.12, 4.0, p.y + 0.1),
    );
}

fn drying_grate(a: &mut Assembly<'_>, p: Vec2) {
    for z in [-0.9, -0.6, -0.3, 0.3, 0.6, 0.9] {
        a.part(
            WorkplaceFeature::Kiln,
            WorkplaceMaterial::Iron,
            Vec3::new(p.x, 1.44, p.y + z),
            Vec3::new(2.0, 0.12, 0.12),
            true,
        );
        a.part(
            WorkplaceFeature::StorageBin,
            WorkplaceMaterial::Grain,
            Vec3::new(p.x, 1.54, p.y + z),
            Vec3::new(1.65, 0.08, 0.1),
            true,
        );
    }
}

fn hood_course(a: &mut Assembly<'_>, p: Vec2, outer: Vec2, base: f32, height: f32) {
    let thickness = 0.26;
    for x in [-0.5, 0.5] {
        a.part(
            WorkplaceFeature::Flue,
            WorkplaceMaterial::Masonry,
            Vec3::new(p.x + x * (outer.x - thickness), base + height * 0.5, p.y),
            Vec3::new(thickness, height, outer.y),
            true,
        );
    }
    for z in [-0.5, 0.5] {
        a.part(
            WorkplaceFeature::Flue,
            WorkplaceMaterial::Masonry,
            Vec3::new(p.x, base + height * 0.5, p.y + z * (outer.y - thickness)),
            Vec3::new(outer.x - thickness * 2.0, height, thickness),
            true,
        );
    }
}

fn masonry(a: &mut Assembly<'_>, p: Vec2, offset: Vec3, size: Vec3) {
    a.part(
        WorkplaceFeature::Kiln,
        WorkplaceMaterial::Masonry,
        Vec3::new(p.x, 0.0, p.y) + offset,
        size,
        true,
    );
}
