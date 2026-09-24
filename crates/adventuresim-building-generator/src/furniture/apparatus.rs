//! Trade and household apparatus. Dimensions are design envelopes, not artifact measurements.
use super::{
    FurnitureKey, FurnitureKind,
    builder::{Builder, CollisionPolicy},
};
use crate::BuildingLodMaterial;
use bevy::math::{Quat, Vec3};

mod commerce;
mod household;
mod printing;
mod weaving;

pub(super) fn assemble(builder: &mut Builder, key: FurnitureKey) {
    let size = key.interior_spec().unwrap().size_metres;
    match key.kind {
        FurnitureKind::CandleStand => household::candle_stand(builder, size),
        FurnitureKind::SpinningStool => household::spinning(builder, size),
        FurnitureKind::BalanceTable => commerce::balance(builder, size),
        FurnitureKind::ReckoningTable => commerce::reckoning(builder, size),
        FurnitureKind::TreadleLoom => weaving::loom(builder, size),
        FurnitureKind::PrintingPress => printing::press(builder, size),
        FurnitureKind::TypeCase => printing::type_case(builder, size),
        _ => unreachable!("apparatus recipe dispatch"),
    }
}

fn table(builder: &mut Builder, size: Vec3, height: f32) {
    for x in [-1.0, 1.0] {
        for z in [-1.0, 1.0] {
            builder.timber(
                Vec3::new(
                    x * (size.x - 0.08) * 0.5,
                    height * 0.5,
                    z * (size.z - 0.08) * 0.5,
                ),
                Vec3::new(0.08, height, 0.08),
            );
        }
        builder.timber(
            Vec3::new(x * (size.x - 0.08) * 0.5, 0.22, 0.0),
            Vec3::new(0.065, 0.08, size.z),
        );
    }
    builder.timber(Vec3::Y * 0.22, Vec3::new(size.x, 0.08, 0.065));
    builder.timber(Vec3::Y * (height - 0.03), Vec3::new(size.x, 0.06, size.z));
}

fn metal(builder: &mut Builder, centre: Vec3, size: Vec3) {
    builder.cuboid(
        BuildingLodMaterial::Iron,
        centre,
        size,
        Quat::IDENTITY,
        CollisionPolicy::Decoration,
    );
}
