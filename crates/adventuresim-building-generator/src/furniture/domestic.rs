//! Joined timber furniture: empty usable structures scaled to placement envelopes.
use super::{FurnitureKey, FurnitureKind, builder::Builder};
use bevy::math::Vec3;

mod beds;
mod cabinets;
mod seating;
mod tables;
#[cfg(test)]
mod tests;
mod washing;

pub(super) fn assemble(builder: &mut Builder, key: FurnitureKey) {
    let size = key.interior_spec().unwrap().size_metres;
    match key.kind {
        FurnitureKind::DiningTable => tables::trestle(builder, size),
        FurnitureKind::Bench
        | FurnitureKind::Chair
        | FurnitureKind::Stool
        | FurnitureKind::ChurchBench => seating::assemble(builder, key.kind, size),
        FurnitureKind::Bed | FurnitureKind::BunkBed | FurnitureKind::WardBed => {
            beds::assemble(builder, key.kind, size);
        }
        FurnitureKind::StorageChest => cabinets::chest(builder, size),
        FurnitureKind::Cupboard | FurnitureKind::Shelving => {
            cabinets::standing(builder, key.kind, size);
        }
        FurnitureKind::WritingDesk => tables::desk(builder, size),
        FurnitureKind::Lectern => tables::lectern(builder, size),
        FurnitureKind::Altar => tables::altar(builder, size),
        FurnitureKind::BathTub => washing::tub(builder, size),
        FurnitureKind::WashStand => washing::stand(builder, size),
        _ => unreachable!("non-domestic furniture routed to domestic assembler"),
    }
}

/// Four standards remain inset within the envelope; their feet define real ground contacts.
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

/// Close-jointed boards make the surface readable without adding tabletop contents.
fn boards(builder: &mut Builder, centre: Vec3, size: Vec3) {
    let count = (size.z / 0.18).ceil() as usize;
    let pitch = size.z / count as f32;
    for index in 0..count {
        builder.timber(
            centre + Vec3::Z * (-size.z * 0.5 + (index as f32 + 0.5) * pitch),
            Vec3::new(size.x, size.y, pitch - 0.003),
        );
    }
}
