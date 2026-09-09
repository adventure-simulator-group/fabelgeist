use super::*;
use bevy::math::Quat;
use std::f32::consts::{FRAC_PI_4, TAU};

pub(super) fn assemble(a: &mut Assembly<'_>, p: Vec2) {
    a.part(
        WorkplaceFeature::MillDrive,
        WorkplaceMaterial::DressedStone,
        Vec3::new(p.x, 0.2, p.y),
        Vec3::new(0.9, 0.4, 0.9),
        true,
    );
    a.part(
        WorkplaceFeature::MillDrive,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 2.075, p.y),
        Vec3::new(0.32, 3.75, 0.32),
        true,
    );
    for x in [1.8, 11.2] {
        a.part(
            WorkplaceFeature::Post,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(x, 1.975, p.y),
            Vec3::new(0.18, 3.95, 0.3),
            true,
        );
    }
    a.part(
        WorkplaceFeature::Beam,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(6.5, 4.075, p.y),
        Vec3::new(9.6, 0.25, 0.35),
        true,
    );
    cogwheel(a, p);
    overhead_sweep(a, p);
}

fn cogwheel(a: &mut Assembly<'_>, p: Vec2) {
    for spoke in 0..4 {
        oriented(
            a,
            WorkplaceFeature::MillDrive,
            Vec3::new(p.x, 2.65, p.y),
            Vec3::new(0.16, 0.2, 3.7),
            Quat::from_rotation_y(spoke as f32 * FRAC_PI_4),
        );
    }
    for tooth in 0..16 {
        let angle = tooth as f32 * TAU / 16.0;
        let direction = Vec2::new(angle.sin(), angle.cos());
        let point = p + direction * 1.8;
        oriented(
            a,
            WorkplaceFeature::MillDrive,
            Vec3::new(point.x, 2.65, point.y),
            Vec3::new(0.74, 0.24, 0.22),
            Quat::from_rotation_y(angle),
        );
        oriented(
            a,
            WorkplaceFeature::MillDrive,
            Vec3::new(point.x, 2.39, point.y),
            Vec3::new(0.14, 0.3, 0.14),
            Quat::from_rotation_y(angle),
        );
    }
}

fn overhead_sweep(a: &mut Assembly<'_>, p: Vec2) {
    // The arm turns above every fixed mill component; only its unhitched draw rope descends.
    a.part(
        WorkplaceFeature::MillSweep,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 3.4, p.y - 1.95),
        Vec3::new(0.18, 0.22, 4.1),
        true,
    );
    let brace = a.part(
        WorkplaceFeature::MillSweep,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 3.63, p.y - 1.0),
        Vec3::new(0.12, 0.12, 2.05),
        true,
    );
    a.orient_part(brace, Quat::from_rotation_x(-0.2));
    a.part(
        WorkplaceFeature::MillSweep,
        WorkplaceMaterial::HempRope,
        Vec3::new(p.x, 2.49, p.y - 3.9),
        Vec3::new(0.045, 1.6, 0.045),
        true,
    );
    a.part(
        WorkplaceFeature::MillSweep,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 1.65, p.y - 3.9),
        Vec3::new(0.65, 0.08, 0.1),
        true,
    );
}

fn oriented(
    a: &mut Assembly<'_>,
    feature: WorkplaceFeature,
    centre: Vec3,
    size: Vec3,
    rotation: Quat,
) {
    let id = a.part(
        feature,
        WorkplaceMaterial::UnpaintedTimber,
        centre,
        size,
        true,
    );
    a.orient_part(id, rotation);
}
