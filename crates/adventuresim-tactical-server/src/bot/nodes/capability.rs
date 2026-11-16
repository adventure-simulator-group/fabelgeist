//! Open-ended capabilities that run alongside a bot's other behaviour and
//! never finish on their own.

use super::*;

mod face_nearest_opponent;
mod raise_guard;
mod reactive_defense;
mod recover_to_upright;

pub(super) fn register(app: &mut App) {
    face_nearest_opponent::register(app);
    raise_guard::register(app);
    reactive_defense::register(app);
    recover_to_upright::register(app);
}
