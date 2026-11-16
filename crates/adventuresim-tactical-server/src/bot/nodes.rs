//! Game-specific behaviour-tree nodes for server-controlled combatants.
//!
//! Condition nodes decide as soon as their task starts. Action nodes either
//! finish at once or work every frame until they are done; a parent that
//! finishes first interrupts them. Every node acts on the bot that runs the
//! tree (the task's actor).
//!
//! - [`capability`]: open-ended behaviours that run alongside everything else.
//! - [`engagement`]: choosing, tracking, and assessing an opponent.
//! - [`ranged`] and [`melee`]: the attack decisions and attacks themselves.

use std::time::Duration;

use super::offense::{
    ADVANCE, InitiativeLost, RETREAT, below_preferred_long_weapon_measure,
    committed_threat_recognition_probability, initiative_delay_seconds, ranged_standoff_movement,
};
use super::senses::{BotSenses, StrikePlan, strike_is_incoming};
use super::*;

mod capability;
mod engagement;
mod fallback;
mod melee;
mod ranged;

type MovementQuery<'w, 's> = Query<'w, 's, &'static mut AuthoritativeMovementIntent>;

pub(super) struct BotNodesPlugin;

impl Plugin for BotNodesPlugin {
    fn build(&self, app: &mut App) {
        fallback::register(app);
        capability::register(app);
        engagement::register(app);
        ranged::register(app);
        melee::register(app);
    }
}

/// Finishes a condition: success when it holds.
fn status_for(holds: bool) -> TaskStatus {
    if holds {
        TaskStatus::Success
    } else {
        TaskStatus::Failure
    }
}

/// Sets the bot's movement intent, relative to its facing.
fn steer(movement: &mut MovementQuery, actor: Entity, intent: Option<Vec2>) {
    if let Ok(mut current) = movement.get_mut(actor) {
        current.0 = intent;
    }
}

/// Turns the bot to `yaw`.
fn face(looks: &mut Query<&mut CharacterLook>, actor: Entity, yaw: f32) {
    if let Ok(mut look) = looks.get_mut(actor) {
        look.yaw = yaw;
    }
}

/// A one-shot timer that finishes after `duration`.
fn timer_after(duration: Duration) -> Timer {
    Timer::new(duration, TimerMode::Once)
}
