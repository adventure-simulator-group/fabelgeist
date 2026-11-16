//! Ranged attack decisions: keeping a standoff distance and shooting.

use super::*;

mod can_shoot_target;
mod is_at_ranged_standoff;
mod ranged_shot;
mod step_to_ranged_standoff;

pub(super) fn register(app: &mut App) {
    can_shoot_target::register(app);
    is_at_ranged_standoff::register(app);
    ranged_shot::register(app);
    step_to_ranged_standoff::register(app);
}
