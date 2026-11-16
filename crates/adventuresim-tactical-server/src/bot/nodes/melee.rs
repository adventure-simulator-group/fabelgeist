//! Melee attack decisions: guarding against incoming strikes, striking,
//! closing to striking measure, and withdrawing when the bot cannot strike.

use super::*;

mod can_start_melee_strike;
mod guard_against_incoming_strike;
mod melee_strike;
mod melee_strike_unavailable;
mod recognizes_incoming_strike;
mod step_to_melee_measure;
mod withdraw;
mod yield_combat;

pub(super) fn register(app: &mut App) {
    can_start_melee_strike::register(app);
    guard_against_incoming_strike::register(app);
    melee_strike::register(app);
    melee_strike_unavailable::register(app);
    recognizes_incoming_strike::register(app);
    step_to_melee_measure::register(app);
    withdraw::register(app);
    yield_combat::register(app);
}
