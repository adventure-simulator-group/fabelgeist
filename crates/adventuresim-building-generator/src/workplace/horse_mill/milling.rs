use super::*;
use bevy::math::Quat;
use std::f32::consts::TAU;

pub(super) fn stone_and_hopper(a: &mut Assembly<'_>, p: Vec2) {
    a.part(
        WorkplaceFeature::Millstone,
        WorkplaceMaterial::DressedStone,
        Vec3::new(p.x, 0.25, p.y),
        Vec3::new(1.65, 0.5, 1.65),
        true,
    );
    stone_disc(a, p, 0.63, 0.26);
    stone_disc(a, p, 0.89, 0.26);
    // One aligned driven spindle joins the lantern pinion to the upper runner stone.
    a.part(
        WorkplaceFeature::MillDrive,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 1.57, p.y),
        Vec3::new(0.18, 2.10, 0.18),
        true,
    );
    for y in [2.17, 2.65] {
        a.part(
            WorkplaceFeature::MillDrive,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x, y, p.y),
            Vec3::new(0.72, 0.12, 0.72),
            true,
        );
    }
    for pin in 0..8 {
        let angle = (pin as f32 + 0.5) * TAU / 8.0;
        let offset = Vec2::new(angle.sin(), angle.cos()) * 0.31;
        a.part(
            WorkplaceFeature::MillDrive,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x + offset.x, 2.41, p.y + offset.y),
            Vec3::new(0.10, 0.48, 0.10),
            true,
        );
    }
    hopper(a, p + Vec2::new(0.0, -1.2));
    a.part(
        WorkplaceFeature::Hopper,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 1.26, p.y - 1.26),
        Vec3::new(0.4, 0.1, 0.12),
        true,
    );
    let chute = a.part(
        WorkplaceFeature::Hopper,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 1.16, p.y - 0.65),
        Vec3::new(0.28, 0.06, 1.35),
        true,
    );
    a.orient_part(chute, Quat::from_rotation_x(0.2));
    // The short flour outlet terminates within the working core rather than in the animal track.
    a.part(
        WorkplaceFeature::Hopper,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 0.73, p.y + 0.77),
        Vec3::new(0.24, 0.12, 0.3),
        true,
    );
}

fn stone_disc(a: &mut Assembly<'_>, p: Vec2, y: f32, height: f32) {
    // Faceted dressed stone uses the same authoritative cuboids in detail, LOD and collision.
    for slice in 0..9 {
        let z = (slice as f32 - 4.0) * 0.15;
        let half_width = (0.72_f32.powi(2) - z.powi(2)).sqrt();
        a.part(
            WorkplaceFeature::Millstone,
            WorkplaceMaterial::DressedStone,
            Vec3::new(p.x, y, p.y + z),
            Vec3::new(half_width * 2.0, height, 0.15),
            true,
        );
    }
}

fn hopper(a: &mut Assembly<'_>, p: Vec2) {
    for x in [-0.48, 0.48] {
        for z in [-0.32, 0.32] {
            a.part(
                WorkplaceFeature::Hopper,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(p.x + x, 0.65, p.y + z),
                Vec3::new(0.12, 1.3, 0.12),
                true,
            );
        }
    }
    for x in [-0.5, 0.5] {
        a.part(
            WorkplaceFeature::Hopper,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x + x, 1.58, p.y),
            Vec3::new(0.10, 0.6, 0.9),
            true,
        );
    }
    for z in [-0.4, 0.4] {
        a.part(
            WorkplaceFeature::Hopper,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x, 1.58, p.y + z),
            Vec3::new(0.9, 0.6, 0.10),
            true,
        );
    }
    // Paired floor boards leave an actual central feed slot above the inclined chute.
    for x in [-0.3, 0.3] {
        a.part(
            WorkplaceFeature::Hopper,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x + x, 1.33, p.y),
            Vec3::new(0.30, 0.10, 0.8),
            true,
        );
        a.part(
            WorkplaceFeature::Hopper,
            WorkplaceMaterial::Grain,
            Vec3::new(p.x + x, 1.51, p.y),
            Vec3::new(0.29, 0.26, 0.78),
            true,
        );
    }
}

pub(super) fn grain_bin(a: &mut Assembly<'_>, p: Vec2) {
    a.part(
        WorkplaceFeature::StorageBin,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 0.2, p.y),
        Vec3::new(1.8, 0.12, 1.8),
        true,
    );
    for x in [-0.86, 0.86] {
        a.part(
            WorkplaceFeature::StorageBin,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x + x, 0.7, p.y),
            Vec3::new(0.08, 1.0, 1.8),
            true,
        );
    }
    for z in [-0.86, 0.86] {
        a.part(
            WorkplaceFeature::StorageBin,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x, 0.7, p.y + z),
            Vec3::new(1.64, 1.0, 0.08),
            true,
        );
    }
    a.part(
        WorkplaceFeature::StorageBin,
        WorkplaceMaterial::Grain,
        Vec3::new(p.x, 0.56, p.y),
        Vec3::new(1.62, 0.6, 1.62),
        true,
    );
}
