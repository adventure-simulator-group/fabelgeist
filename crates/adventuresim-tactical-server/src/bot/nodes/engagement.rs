//! Choosing, tracking, and assessing the opponent a bot fights.

use super::*;

mod acquire_target;
mod assess;
mod face_target;
mod hold_still;
mod holds_initiative;
mod is_able_to_fight;
mod is_target_still_chosen;

pub(super) fn register(app: &mut App) {
    acquire_target::register(app);
    assess::register(app);
    face_target::register(app);
    hold_still::register(app);
    holds_initiative::register(app);
    is_able_to_fight::register(app);
    is_target_still_chosen::register(app);
}
