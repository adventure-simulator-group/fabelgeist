//! Enemy fear projection for strategic condition.

pub(super) fn enemy_fear_multiplier(enemy_type: &str) -> Result<f32, String> {
    let threat = enemy_type.parse::<adventuresim_core::bestiary::ThreatId>();
    threat
        .map(|id| 1.0 + f32::from(id.profile().combat.fear) / 50.0)
        .map_err(|_| format!("Unknown threat ID in quest: {enemy_type}"))
}
