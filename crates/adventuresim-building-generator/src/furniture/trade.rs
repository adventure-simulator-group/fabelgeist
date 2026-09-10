//! Empty trade furnishings: construction, storage and continuous shop counters.
use super::{FurnitureKey, FurnitureKind, builder::Builder};
use bevy::math::Vec3;

mod counters;
mod racks;
mod storage;
#[cfg(test)]
mod tests;
mod work;

pub(super) fn assemble(builder: &mut Builder, key: FurnitureKey) {
    let size = key
        .interior_spec()
        .expect("interior trade dimensions")
        .size_metres;
    use FurnitureKind::*;
    match key.kind {
        Counter | CounterLeftEnd | CounterRightEnd | CounterCorner => {
            counters::counter(builder, size, key.kind);
        }
        DisplayCounter => counters::display(builder, size),
        Workbench | CuttingTable | ButchersBlock => work::table(builder, size, key.kind),
        ToolRack | WeaponRack | DryingRack | CaskRack | HayRack => {
            racks::rack(builder, size, key.kind);
        }
        ArmourStand => racks::armour_stand(builder, size),
        GrainBin | KneadingTrough | FeedTrough => storage::trough(builder, size, key.kind),
        StorageCrate => storage::crate_box(builder, size),
        _ => unreachable!("non-trade furniture dispatched to trade recipes"),
    }
}

/// Four legs remain within the authored envelope; the top touches each leg.
fn legs(builder: &mut Builder, size: Vec3, height: f32, thickness: f32) {
    for x in [-1.0, 1.0] {
        for z in [-1.0, 1.0] {
            builder.timber(
                Vec3::new(
                    x * (size.x - thickness) * 0.5,
                    height * 0.5,
                    z * (size.z - thickness) * 0.5,
                ),
                Vec3::new(thickness, height, thickness),
            );
        }
    }
}

fn beam(builder: &mut Builder, from: Vec3, to: Vec3, thickness: f32) {
    let direction = to - from;
    builder.cuboid(
        crate::BuildingLodMaterial::InteriorTimber,
        (from + to) * 0.5,
        Vec3::new(thickness, direction.length(), thickness),
        bevy::math::Quat::from_rotation_arc(Vec3::Y, direction.normalize()),
        super::builder::CollisionPolicy::Solid,
    );
}
