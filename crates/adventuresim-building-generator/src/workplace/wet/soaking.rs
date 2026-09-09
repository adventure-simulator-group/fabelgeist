use super::*;

pub(super) fn tank(a: &mut Assembly<'_>, p: Vec2) {
    let feature = WorkplaceFeature::SoakingTank;
    a.part(
        feature,
        WorkplaceMaterial::Masonry,
        Vec3::new(p.x, 0.09, p.y),
        Vec3::new(2.2, 0.18, 2.0),
        true,
    );
    for x in [-1.0, 1.0] {
        a.part(
            feature,
            WorkplaceMaterial::Masonry,
            Vec3::new(p.x + x, 0.57, p.y),
            Vec3::new(0.2, 0.96, 2.0),
            true,
        );
    }
    for z in [-0.9, 0.9] {
        a.part(
            feature,
            WorkplaceMaterial::Masonry,
            Vec3::new(p.x, 0.57, p.y + z),
            Vec3::new(1.8, 0.96, 0.2),
            true,
        );
    }
    // Rendered process water rests on the physical basin floor. It is explicitly non-colliding.
    a.part(
        WorkplaceFeature::ProcessLiquid,
        WorkplaceMaterial::ProcessLiquid,
        Vec3::new(p.x, 0.5, p.y),
        Vec3::new(1.79, 0.64, 1.59),
        true,
    );
}

pub(super) fn dye_kettle(a: &mut Assembly<'_>, p: Vec2) {
    // A masonry heating enclosure supports an open metal-lined vessel above a firing mouth.
    for x in [-0.75, 0.75] {
        a.part(
            WorkplaceFeature::DyeKettle,
            WorkplaceMaterial::Masonry,
            Vec3::new(p.x + x, 0.4, p.y),
            Vec3::new(0.5, 0.8, 2.0),
            true,
        );
    }
    a.part(
        WorkplaceFeature::DyeKettle,
        WorkplaceMaterial::Masonry,
        Vec3::new(p.x, 0.4, p.y + 0.75),
        Vec3::new(1.0, 0.8, 0.5),
        true,
    );
    a.part(
        WorkplaceFeature::DyeKettle,
        WorkplaceMaterial::Iron,
        Vec3::new(p.x, 0.86, p.y),
        Vec3::new(2.0, 0.12, 2.0),
        true,
    );
    for x in [-0.93, 0.93] {
        a.part(
            WorkplaceFeature::DyeKettle,
            WorkplaceMaterial::Iron,
            Vec3::new(p.x + x, 1.18, p.y),
            Vec3::new(0.14, 0.64, 2.0),
            true,
        );
    }
    for z in [-0.93, 0.93] {
        a.part(
            WorkplaceFeature::DyeKettle,
            WorkplaceMaterial::Iron,
            Vec3::new(p.x, 1.18, p.y + z),
            Vec3::new(1.72, 0.64, 0.14),
            true,
        );
    }
    a.part(
        WorkplaceFeature::ProcessLiquid,
        WorkplaceMaterial::ProcessLiquid,
        Vec3::new(p.x, 1.12, p.y),
        Vec3::new(1.7, 0.4, 1.7),
        true,
    );
    // A timber lifting rail allows cloth to drain back over the heated vessel.
    for x in [-1.1, 1.1] {
        a.part(
            WorkplaceFeature::DyeKettle,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x + x, 1.2, p.y + 0.85),
            Vec3::new(0.16, 2.4, 0.16),
            true,
        );
    }
    a.part(
        WorkplaceFeature::DyeKettle,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 2.48, p.y + 0.85),
        Vec3::new(2.5, 0.16, 0.18),
        true,
    );
}

pub(super) fn fleshing_beam(a: &mut Assembly<'_>, p: Vec2) {
    let pitch = 20.0_f32.to_radians();
    for z in [-0.9, 0.9] {
        let bearing_height = 1.15 - pitch.tan() * z - 0.09 / pitch.cos();
        for x in [-0.5, 0.5] {
            a.part(
                WorkplaceFeature::FleshingBeam,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(p.x + x, bearing_height * 0.5, p.y + z),
                Vec3::new(0.16, bearing_height, 0.2),
                true,
            );
        }
        a.part(
            WorkplaceFeature::FleshingBeam,
            WorkplaceMaterial::UnpaintedTimber,
            Vec3::new(p.x, bearing_height + 0.06, p.y + z),
            Vec3::new(1.2, 0.12, 0.2),
            true,
        );
    }
    let beam = a.part(
        WorkplaceFeature::FleshingBeam,
        WorkplaceMaterial::UnpaintedTimber,
        Vec3::new(p.x, 1.27, p.y),
        Vec3::new(0.5, 0.18, 3.0),
        true,
    );
    // The assembly rotation API rebuilds support contacts for the sloped working surface.
    a.orient_part(beam, bevy::math::Quat::from_rotation_x(pitch));
}
