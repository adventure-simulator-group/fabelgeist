use super::*;

/// Movement intent toward the bot's facing, which follows its target.
pub(in crate::bot) const ADVANCE: Vec2 = Vec2::Y;
/// Movement intent away from the bot's facing.
pub(in crate::bot) const RETREAT: Vec2 = Vec2::NEG_Y;

pub(in crate::bot) fn initiative_delay_seconds(
    random: &mut CombatRandom,
    instinct: f32,
    config: &AiOffenseConfig,
) -> ElapsedResult<AnimationElapsed> {
    let sampled = random.range_f32(
        config.initiative_delay_min_seconds.seconds(),
        config.initiative_delay_max_seconds.seconds(),
    );
    AnimationElapsed::from_seconds(sampled * (3.0 / instinct.max(0.5)).clamp(0.6, 2.0))
}

pub(in crate::bot) fn committed_threat_recognition_probability(
    attack_phase: adventuresim_tactical_core::animation::UnitInterval,
    instinct: f32,
) -> f32 {
    let visible_windup = (attack_phase.fraction().clamp(0.0, 0.5) * 2.0).sqrt();
    (visible_windup * instinct.max(0.0) / 5.0).clamp(0.0, 1.0)
}

pub(in crate::bot) fn below_preferred_long_weapon_measure(
    reach_metres: f32,
    preferred_measure_metres: f32,
    center_distance_metres: f32,
    long_weapon_threshold_metres: f32,
) -> bool {
    let preferred_center_distance_metres = preferred_measure_metres
        + adventuresim_core::combat::HUMANOID_MELEE_MINIMUM_CENTER_SEPARATION_METRES;
    reach_metres >= long_weapon_threshold_metres
        && center_distance_metres < preferred_center_distance_metres
}

pub(in crate::bot) fn long_weapon_windup_movement(
    reach_metres: f32,
    preferred_measure_metres: f32,
    center_distance_metres: f32,
    long_weapon_threshold_metres: f32,
) -> Option<Vec2> {
    below_preferred_long_weapon_measure(
        reach_metres,
        preferred_measure_metres,
        center_distance_metres,
        long_weapon_threshold_metres,
    )
    .then_some(RETREAT)
}

/// Movement that brings a ranged attacker into its standoff band, or `None`
/// once it is in position to shoot.
pub(in crate::bot) fn ranged_standoff_movement(
    distance_metres: f32,
    weapon_reach_metres: f32,
    config: &AiOffenseConfig,
) -> Option<Vec2> {
    let standoff = (weapon_reach_metres * config.ranged_reach_fraction)
        .clamp(
            config.ranged_standoff_min_metres,
            config.ranged_standoff_max_metres,
        )
        .min(weapon_reach_metres);
    if distance_metres > weapon_reach_metres
        || distance_metres > standoff + config.ranged_standoff_slop_metres
    {
        Some(ADVANCE)
    } else if distance_metres + config.ranged_standoff_slop_metres < standoff {
        Some(RETREAT)
    } else {
        None
    }
}
