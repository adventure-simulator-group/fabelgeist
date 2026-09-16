//! A shared masonry fire wall separates cooking hearth and rear-fed tiled stove.
use super::assembly::Assembly;
use crate::{
    BuildingLodMaterial as Material, HeatingPartKind as Part, HeatingPassageKind as Passage,
    WallMaterialClass,
};
use bevy::math::Vec3;
const MASONRY: Material = Material::Wall(WallMaterialClass::CivilianMasonry);

pub(super) fn build(a: &mut Assembly<'_>, top: f32) {
    let top = top - a.placement.floor_height;
    let front = a.placement.section.front();
    let shaft_back = a.placement.section.shaft_offset() - 0.3;
    let shaft_front = a.placement.section.shaft_offset() + 0.3;
    let bore_back = a.placement.section.shaft_offset() - 0.18;
    let bore_front = a.placement.section.shaft_offset() + 0.18;
    // Upper appliances transfer their weight directly through continuous masonry.
    if a.placement.storey_level > 0 {
        a.absolute_part(Part::SupportPier, MASONRY, a.placement.support());
        a.plan.ground_support = a.geometry.solids.last().unwrap().supported_by[0];
    }
    // The hearth, fire wall and stove bear on one plinth.
    a.part(
        Part::Footing,
        MASONRY,
        Vec3::new(-0.48, 0.0, -0.9),
        Vec3::new(0.48, 0.2, front),
    );
    if a.placement.storey_level == 0 {
        a.plan.ground_support = a.geometry.solids.last().unwrap().supported_by[0];
    }
    fire_wall(a);
    a.part(
        Part::Hood,
        MASONRY,
        Vec3::new(-0.4, super::placement::FIRE_WALL_PATCH_HEIGHT_METRES, 0.15),
        Vec3::new(0.4, 2.1, 0.3),
    );
    // Stove shell, with the firebox and smoke return opening toward the kitchen.
    for (min, max) in [
        (Vec3::new(-0.4, 0.2, -0.9), Vec3::new(-0.25, 1.5, -0.15)),
        (Vec3::new(0.25, 0.2, -0.9), Vec3::new(0.4, 1.5, -0.15)),
        (Vec3::new(-0.25, 0.2, -0.9), Vec3::new(0.25, 1.5, -0.75)),
        (Vec3::new(-0.4, 1.5, -0.9), Vec3::new(0.4, 1.65, -0.15)),
    ] {
        a.part(Part::TiledStove, Material::GlazedTile, min, max);
    }
    // Masonry jambs support the hood on either side of its open cooking mouth.
    for x in [-0.4, 0.28] {
        a.part(
            Part::Hearth,
            MASONRY,
            Vec3::new(x, 0.2, 0.15),
            Vec3::new(x + 0.12, 2.1, front),
        );
    }
    // Continuous rear shell above the fire wall; hood cap leaves an actual bore.
    a.part(
        Part::Hood,
        MASONRY,
        Vec3::new(-0.4, 2.1, 0.15),
        Vec3::new(0.4, 2.2, bore_back),
    );
    a.part(
        Part::Hood,
        MASONRY,
        Vec3::new(-0.4, 2.1, bore_front),
        Vec3::new(0.4, 2.2, front),
    );
    for x in [-0.4, 0.18] {
        a.part(
            Part::Hood,
            MASONRY,
            Vec3::new(x, 2.1, bore_back),
            Vec3::new(x + 0.22, 2.2, bore_front),
        );
    }
    for (min, max) in [
        (
            Vec3::new(-0.3, 2.2, shaft_back),
            Vec3::new(-0.18, top, shaft_front),
        ),
        (
            Vec3::new(0.18, 2.2, shaft_back),
            Vec3::new(0.3, top, shaft_front),
        ),
        (
            Vec3::new(-0.18, 2.2, shaft_back),
            Vec3::new(0.18, top, bore_back),
        ),
        (
            Vec3::new(-0.18, 2.2, bore_front),
            Vec3::new(0.18, top, shaft_front),
        ),
    ] {
        a.part(Part::Flue, MASONRY, min, max);
    }
    if let Some(shoulder) = a.placement.shaft_shoulder() {
        let shaft = a.placement.shaft(shoulder.max.y);
        for bounds in super::floors::pieces(shoulder, shaft) {
            a.absolute_part(Part::FlueShoulder, MASONRY, bounds);
        }
    }
    smoke_passages(a, top, bore_back, bore_front);
}

fn smoke_passages(a: &mut Assembly<'_>, top: f32, bore_back: f32, bore_front: f32) {
    a.passage(
        Passage::StoveChamber,
        Vec3::new(-0.25, 0.2, -0.75),
        Vec3::new(0.25, 1.5, -0.15),
    );
    a.passage(
        Passage::HearthMouth,
        Vec3::new(-0.28, 0.2, 0.3),
        Vec3::new(0.28, 2.1, a.placement.section.front()),
    );
    a.passage(
        Passage::StoveFirebox,
        Vec3::new(-0.18, 0.45, -0.75),
        Vec3::new(0.18, 0.75, 0.32),
    );
    a.passage(
        Passage::StoveSmokeReturn,
        Vec3::new(-0.18, 1.15, -0.75),
        Vec3::new(0.18, 1.35, 0.32),
    );
    a.passage(
        Passage::HoodThroat,
        Vec3::new(-0.18, 2.05, bore_back),
        Vec3::new(0.18, 2.21, bore_front),
    );
    a.passage(
        Passage::FlueBore,
        Vec3::new(-0.18, 2.2, bore_back),
        Vec3::new(0.18, top + 0.02, bore_front),
    );
}

fn fire_wall(a: &mut Assembly<'_>) {
    for (min, max) in [
        (
            Vec3::new(-0.48, 0.2, -0.15),
            Vec3::new(-0.18, super::placement::FIRE_WALL_PATCH_HEIGHT_METRES, 0.3),
        ),
        (
            Vec3::new(0.18, 0.2, -0.15),
            Vec3::new(0.48, super::placement::FIRE_WALL_PATCH_HEIGHT_METRES, 0.3),
        ),
        (Vec3::new(-0.18, 0.2, -0.15), Vec3::new(0.18, 0.45, 0.3)),
        (Vec3::new(-0.18, 0.75, -0.15), Vec3::new(0.18, 1.15, 0.3)),
        (
            Vec3::new(-0.18, 1.35, -0.15),
            Vec3::new(0.18, super::placement::FIRE_WALL_PATCH_HEIGHT_METRES, 0.3),
        ),
    ] {
        a.part(Part::FireWall, MASONRY, min, max);
    }
}
