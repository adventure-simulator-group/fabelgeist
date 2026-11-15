//! The guard: fists up while [`GUARD`] is held, unless a strike is under way or the fighter
//! can't act. A strike that reaches a guarded fighter from the front is only a shove
//! ([`crate::server::resolve_strikes`]); the attacker judges the guard where it saw it.

use bevy::prelude::*;
use bevy_netahoy::UserCmd;

use crate::{Fighter, input::GUARD};

/// The guard covers strikes from within this cosine of where the fighter faces.
pub const GUARD_COVER_COS: f32 = 0.3;

/// Raises or drops the guard for `command`.
pub fn step(fighter: &mut Fighter, command: &UserCmd) {
    fighter.guarding =
        command.buttons.contains(GUARD) && fighter.strike.is_none() && fighter.can_act();
}

/// Whether a fighter facing `yaw` and standing at `at` covers a strike from `attacker`.
pub fn covers(yaw: f32, at: Vec3, attacker: Vec3) -> bool {
    let facing = Quat::from_rotation_y(yaw) * Vec3::NEG_Z;
    let toward = (attacker - at).with_y(0.0).normalize_or_zero();
    facing.dot(toward) >= GUARD_COVER_COS
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_netahoy::Buttons;

    fn command(buttons: Buttons) -> UserCmd {
        UserCmd {
            buttons,
            ..default()
        }
    }

    #[test]
    fn the_guard_is_up_while_held_and_free() {
        let mut fighter = Fighter::default();
        step(&mut fighter, &command(GUARD));
        assert!(fighter.guarding);
        step(&mut fighter, &command(Buttons::empty()));
        assert!(!fighter.guarding);
        fighter.stagger = 3;
        step(&mut fighter, &command(GUARD));
        assert!(!fighter.guarding, "a reeling fighter can't guard");
    }

    #[test]
    fn the_guard_covers_the_front_only() {
        // Facing -Z (yaw 0): an attacker in front is covered, one behind is not.
        assert!(covers(0.0, Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0)));
        assert!(!covers(0.0, Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0)));
        assert!(!covers(0.0, Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0)));
    }
}
