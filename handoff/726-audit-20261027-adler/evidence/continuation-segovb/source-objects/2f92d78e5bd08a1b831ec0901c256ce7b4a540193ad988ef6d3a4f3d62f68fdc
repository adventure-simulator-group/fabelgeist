use super::*;

#[derive(Clone, Copy)]
pub(super) struct RangedPursuit {
    pub entity: Entity,
    pub target: Entity,
    pub distance: SignedLength,
}

pub(super) fn drive_ranged_pursuit(
    cmd: &mut Commands,
    pursuit: RangedPursuit,
    input: &mut AuthoritativeMovementIntent,
    controller: &mut OffensiveCombatAi,
    facts: &OffensiveFacts,
    config: &AiOffenseConfig,
) -> ElapsedResult<()> {
    let standoff = (facts.weapon_reach.metres() * config.ranged_reach_fraction)
        .clamp(
            config.ranged_standoff_min_metres.metres(),
            config.ranged_standoff_max_metres.metres(),
        )
        .min(facts.weapon_reach.metres());
    if pursuit.distance.metres() > facts.weapon_reach.metres()
        || pursuit.distance.metres() > standoff + config.ranged_standoff_slop_metres.metres()
    {
        input.0 = Some(Vec2::Y);
    } else if pursuit.distance.metres() + config.ranged_standoff_slop_metres.metres() < standoff {
        input.0 = Some(-Vec2::Y);
    } else {
        input.0 = None;
        let windup = CombatDuration::from_elapsed(config.windup_seconds)?;
        let timer = Timer::new(config.windup_seconds.native_duration()?, TimerMode::Once);
        cmd.trigger(RangedAttackStartedIntent {
            attacker: pursuit.entity,
            target: Some(pursuit.target),
            animation_windup: windup,
            minimum_windup: windup,
        });
        controller.phase = OffensiveCombatPhase::RangedWindup(timer);
    }
    Ok(())
}
