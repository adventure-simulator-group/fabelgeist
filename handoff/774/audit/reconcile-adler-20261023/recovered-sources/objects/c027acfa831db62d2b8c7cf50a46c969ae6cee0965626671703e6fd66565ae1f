use super::*;

pub(super) fn candle_stand(builder: &mut Builder, size: Vec3) {
    // Cruciform iron feet and a socket candle; no modern lantern or electric light.
    for footprint in [Vec3::new(size.x, 0.04, 0.07), Vec3::new(0.07, 0.04, size.z)] {
        builder.cuboid(
            BuildingLodMaterial::Iron,
            Vec3::Y * 0.02,
            footprint,
            Quat::IDENTITY,
            CollisionPolicy::Solid,
        );
    }
    builder.turned(
        BuildingLodMaterial::Iron,
        Vec3::ZERO,
        &[
            (0.035, 0.06),
            (0.11, 0.025),
            (size.y - 0.23, 0.019),
            (size.y - 0.21, 0.07),
        ],
    );
    candle(builder, Vec3::Y * (size.y - 0.21), 0.21);
    builder.collider(Vec3::Y * size.y * 0.5, Vec3::new(0.04, size.y, 0.04));
}

pub(super) fn candle(builder: &mut Builder, base: Vec3, height: f32) {
    builder.turned(
        BuildingLodMaterial::Iron,
        base,
        &[(0.0, 0.07), (0.008, 0.07), (0.013, 0.021), (0.04, 0.021)],
    );
    builder.turned(
        BuildingLodMaterial::CandleWax,
        base,
        &[(0.025, 0.015), (height - 0.008, 0.014)],
    );
    metal(
        builder,
        base + Vec3::Y * (height - 0.006),
        Vec3::new(0.004, 0.012, 0.004),
    );
}

pub(super) fn spinning(builder: &mut Builder, size: Vec3) {
    let seat = Vec3::new(size.x * 0.7, 0.45, size.z);
    table(builder, seat, seat.y);
    let distaff = Vec3::new(size.x * 0.37, 0.0, size.z * 0.25);
    builder.timber(
        distaff + Vec3::Y * size.y * 0.5,
        Vec3::new(0.025, size.y, 0.025),
    );
    builder.timber(
        Vec3::new(size.x * 0.22, 0.42, distaff.z),
        Vec3::new(size.x * 0.4, 0.04, 0.045),
    );
    builder.turned(
        BuildingLodMaterial::UndyedCloth,
        distaff,
        &[
            (size.y - 0.27, 0.022),
            (size.y - 0.19, 0.055),
            (size.y - 0.07, 0.055),
            (size.y, 0.016),
        ],
    );
    // A bracket outside the seat parks the spindle without occupying sitting space.
    builder.timber(
        Vec3::new(-size.x * 0.28, 0.28, 0.0),
        Vec3::new(size.x * 0.26, 0.035, 0.09),
    );
    builder.timber(
        Vec3::new(-size.x * 0.28, 0.36, 0.0),
        Vec3::new(0.035, 0.18, 0.05),
    );
    let spindle = Vec3::new(-size.x * 0.38, 0.298, 0.0);
    builder.turned(
        BuildingLodMaterial::InteriorTimber,
        spindle,
        &[(0.0, 0.009), (0.2, 0.007)],
    );
    builder.turned(
        BuildingLodMaterial::Earthenware,
        spindle,
        &[(0.014, 0.033), (0.03, 0.038), (0.046, 0.025)],
    );
    builder.turned(
        BuildingLodMaterial::HempRope,
        distaff,
        &[(size.y - 0.14, 0.057), (size.y - 0.128, 0.058)],
    );
}
