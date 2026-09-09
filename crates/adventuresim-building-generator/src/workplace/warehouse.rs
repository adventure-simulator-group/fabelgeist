//! Long masonry storehouses with two usable storage levels and a covered loading apron.
use super::{assembly::Assembly, *};
use crate::{GableProfile, RidgeAxis, RoofKind, RoofPiece};

mod envelope;
mod hoist;
mod loading;
mod storage;
#[cfg(test)]
mod tests;

pub(super) use envelope::build_envelope;

pub(super) fn loading_roof(main: Vec2) -> RoofPiece {
    RoofPiece {
        kind: RoofKind::Gable,
        centre: Vec2::new(main.x + 2.8, main.y * 0.25),
        size: Vec2::new(4.0, 4.4),
        base_height_metres: 3.7,
        pitch_degrees: 28.0,
        ridge_axis: RidgeAxis::X,
        eave_metres: 0.18,
        gable_profile: GableProfile::Plain,
    }
}

pub(super) fn fit_workplace(a: &mut Assembly<'_>, w: f32, d: f32) {
    storage::storage_floors(a, w, d);
    loading::loading_hood(a, w, d);
}
