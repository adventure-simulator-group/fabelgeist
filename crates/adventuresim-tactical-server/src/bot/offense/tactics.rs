use super::*;

pub(super) fn initiative_delay_seconds(
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

pub(super) fn committed_threat_recognition_probability(
    attack_phase: adventuresim_tactical_core::animation::UnitInterval,
    instinct: f32,
) -> f32 {
    let visible_windup = (attack_phase.fraction().clamp(0.0, 0.5) * 2.0).sqrt();
    (visible_windup * instinct.max(0.0) / 5.0).clamp(0.0, 1.0)
}

pub(super) fn below_preferred_long_weapon_measure(
    reach_metres: SignedLength,
    preferred_measure_metres: SignedLength,
    center_distance_metres: SignedLength,
    long_weapon_threshold_metres: SignedLength,
) -> bool {
    let preferred_center_distance_metres = preferred_measure_metres.metres()
        + adventuresim_core::combat::HUMANOID_MELEE_MINIMUM_CENTER_SEPARATION_METRES;
    reach_metres.metres() >= long_weapon_threshold_metres.metres()
        && center_distance_metres.metres() < preferred_center_distance_metres
}

pub(super) fn long_weapon_windup_movement(
    reach_metres: SignedLength,
    preferred_measure_metres: SignedLength,
    center_distance_metres: SignedLength,
    long_weapon_threshold_metres: SignedLength,
) -> Option<Vec2> {
    below_preferred_long_weapon_measure(
        reach_metres,
        preferred_measure_metres,
        center_distance_metres,
        long_weapon_threshold_metres,
    )
    .then_some(-Vec2::Y)
}

pub(super) fn should_abort_windup(
    phase: &OffensiveCombatPhase,
    facts: &OffensiveFacts,
    distance: SignedLength,
) -> bool {
    let melee_target_reachable = matches!(
        facts.melee_lunge_delay,
        Some(MeleeLungeArrival::NotNeeded | MeleeLungeArrival::ArrivesIn(_))
    );

    matches!(
        phase,
        OffensiveCombatPhase::MeleeWindup { .. }
            if !facts.weapon_is_melee
                || !facts.melee_attack_available
                || facts.dimensions.arm_reach_metres.metres() <= 0.0
                || !melee_target_reachable
                || !matches!(facts.melee_recovery_seconds, Some(RecoveryTime::After(_)))
    ) || matches!(
        phase,
        OffensiveCombatPhase::RangedWindup(_) if !facts.use_ranged || distance.metres() > facts.weapon_reach.metres()
    )
}
