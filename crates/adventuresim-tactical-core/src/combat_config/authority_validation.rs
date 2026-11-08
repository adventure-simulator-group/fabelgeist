use super::*;

pub(super) fn validate_realtime_authority(
    authority: &RealtimeAuthorityConfig,
) -> Result<(), TacticalCombatConfigError> {
    let finite_nonnegative = |value: f32| value.is_finite() && value >= 0.0;
    let authority_values = [
        authority.defense.roll_dodge_effectiveness,
        authority.melee.range_latency_tolerance_metres.metres(),
        authority.melee.windup_jitter_fraction,
        authority.melee.lunge_range_window_metres.metres(),
        authority.melee.lunge_quickstep_threshold_metres.metres(),
        authority.ranged.range_latency_tolerance_metres.metres(),
        authority.ranged.aim_half_angle_degrees,
        authority.impact.whole_body_velocity_scale,
    ];
    if !authority_values.into_iter().all(finite_nonnegative) {
        return Err(TacticalCombatConfigError::Validation(
            "authority values must be finite and non-negative",
        ));
    }
    if authority.defense.roll_dodge_effectiveness > 1.0
        || authority.melee.windup_jitter_fraction > 1.0
        || authority.ranged.aim_half_angle_degrees > 180.0
        || authority.melee.completion_allowance_seconds.seconds() > 5.0
        || authority.ranged.completion_allowance_seconds.seconds() > 5.0
    {
        return Err(TacticalCombatConfigError::Validation(
            "authority values exceed compiled safety bounds",
        ));
    }
    Ok(())
}
