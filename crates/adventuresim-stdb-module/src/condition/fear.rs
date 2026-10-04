//! Enemy fear projection after stored threat vocabulary has been admitted.

use adventuresim_core::bestiary::ThreatId;

pub(super) fn enemy_fear_multiplier(enemy_type: ThreatId) -> f32 {
    1.0 + f32::from(enemy_type.profile().combat.fear) / 50.0
}
