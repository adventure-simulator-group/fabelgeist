//! Offensive judgement shared by the attack nodes: weapon and measure facts,
//! initiative timing, and the initiative lost to a committed defense.

use super::*;

#[cfg(test)]
pub(super) use facts::ranged_weapon_needs_ammo_lookup;
pub(super) use facts::{OffensiveFacts, compare_target, offensive_facts};
pub(super) use tactics::*;

mod facts;
mod tactics;

/// Marks a bot whose attack was turned into a defense. It must reassess before
/// attacking again; the next assessment removes the marker.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[reflect(Component)]
pub struct InitiativeLost;

pub(super) fn on_attack_committed_to_defense(
    event: On<crate::combat::MeleeAttackCommittedToDefense>,
    mut cmd: Commands,
    bots: Query<(), With<Behaviour>>,
) {
    if bots.contains(event.defender) {
        cmd.entity(event.defender).insert(InitiativeLost);
    }
}

#[cfg(test)]
mod tests {
    use super::{below_preferred_long_weapon_measure, committed_threat_recognition_probability};
    use adventuresim_tactical_core::animation::UnitInterval;

    #[test]
    fn threat_recognition_grows_with_visible_commitment_and_instinct() {
        let early = committed_threat_recognition_probability(
            UnitInterval::from_fraction(0.05).unwrap(),
            3.0,
        );
        let late = committed_threat_recognition_probability(
            UnitInterval::from_fraction(0.4).unwrap(),
            3.0,
        );
        let expert = committed_threat_recognition_probability(
            UnitInterval::from_fraction(0.4).unwrap(),
            5.0,
        );
        assert!(early < late);
        assert!(late < expert);
        assert!((0.0..=1.0).contains(&early));
        assert!((0.0..=1.0).contains(&expert));
    }

    #[test]
    fn polearm_retreats_below_authored_head_band_center_not_generic_seventy_percent() {
        let preferred =
            adventuresim_core::combat::preferred_melee_striking_measure(2.0, 1.9, 0.16, true, 0.7);
        assert!((preferred - 1.92).abs() < 1.0e-6);
        assert!(below_preferred_long_weapon_measure(
            2.0, preferred, 2.6, 1.2
        ));
        assert!(!below_preferred_long_weapon_measure(
            2.0, preferred, 2.8, 1.2
        ));
    }
}
