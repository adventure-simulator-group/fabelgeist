//! Wet trades combine roofed working rooms with real vessels and supported drying equipment.
use super::{assembly::Assembly, *};
use crate::{GableProfile, RidgeAxis, RoofKind, RoofPiece};

mod envelope;
mod soaking;
#[cfg(test)]
mod tests;
mod textiles;
pub(super) use envelope::build_envelope;

pub(super) fn service_roof(main: Vec2) -> RoofPiece {
    RoofPiece {
        kind: RoofKind::Gable,
        centre: Vec2::new(main.x + 3.8, main.y * 0.5),
        size: Vec2::new(3.2, main.y - 1.2),
        base_height_metres: 3.0,
        pitch_degrees: 24.0,
        ridge_axis: RidgeAxis::Z,
        eave_metres: 0.16,
        gable_profile: GableProfile::Plain,
    }
}

pub(super) fn fit_workplace(a: &mut Assembly<'_>, w: f32, d: f32) {
    match a.plan.kind {
        WorkplaceKind::Dyer => {
            soaking::dye_kettle(a, Vec2::new(1.7, 2.2));
            textiles::dye_frames(a, w, d);
        }
        WorkplaceKind::Tannery => {
            drying_canopy(a, w, d);
            let count = 2 + a.plan.size.extra_bays();
            for bay in 0..count {
                soaking::tank(a, Vec2::new(w + 3.8, 2.2 + f32::from(bay) * 3.0));
            }
            textiles::hide_frame(a, Vec2::new(w + 3.8, d - 1.2));
            soaking::fleshing_beam(a, Vec2::new(1.7, 2.2));
            soaking::fleshing_beam(a, Vec2::new(w - 1.7, d - 2.4));
        }
        _ => unreachable!("only dyeing and tanning use the wet-trade programme"),
    }
}

fn drying_canopy(a: &mut Assembly<'_>, w: f32, d: f32) {
    let front = 0.6;
    let back = d - 0.6;
    let bays = ((back - front) / 3.0).ceil() as u32;
    for x in [w + 2.2, w + 5.4] {
        for bay in 0..=bays {
            let z = front + (back - front) * bay as f32 / bays as f32;
            a.part(
                WorkplaceFeature::Post,
                WorkplaceMaterial::UnpaintedTimber,
                Vec3::new(x, 1.4, z),
                Vec3::new(0.24, 2.8, 0.24),
                true,
            );
        }
        a.wall(
            Vec2::new(x, front),
            Vec2::new(x, back),
            if x < w + 3.8 { Vec2::NEG_X } else { Vec2::X },
            2.8,
            0.2,
            true,
        );
    }
    for z in [front, back] {
        a.wall(
            Vec2::new(w + 2.2, z),
            Vec2::new(w + 5.4, z),
            if z == front { Vec2::NEG_Y } else { Vec2::Y },
            2.8,
            0.2,
            true,
        );
    }
}
