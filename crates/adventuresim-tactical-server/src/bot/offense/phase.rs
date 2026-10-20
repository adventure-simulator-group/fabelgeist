//! Offensive phase transitions and checked native windup/cooldown publication.

use super::*;

struct OffensiveControls<'a> {
    input: &'a mut AuthoritativeMovementIntent,
    controller: &'a mut OffensiveCombatAi,
}

struct MeleePursuit {
    entity: Entity,
    target: Entity,
    distance: f32,
}

#[derive(Debug, Reflect)]
pub(super) enum OffensiveCombatPhase {
    Assessing(Timer),
    GuardingCommittedThreat,
    Pursuing,
    MeleeWindup {
        timer: Timer,
        strike_family: StrikeFamily,
        #[reflect(remote = adventuresim_tactical_core::animation::ReflectedAnimationElapsed)]
        recovery_seconds: AnimationElapsed,
    },
    RangedWindup(Timer),
    Cooldown(Timer),
    WithdrawingUnableToContinue(Timer),
}

impl MeleePursuit {
    fn start(
        self,
        cmd: &mut Commands,
        input: &mut AuthoritativeMovementIntent,
        controller: &mut OffensiveCombatAi,
        skeleton: &SkeletonState,
        facts: &OffensiveFacts,
        config: &AiOffenseConfig,
    ) -> ElapsedResult<()> {
        let Self {
            entity,
            target,
            distance,
        } = self;
        let arrival = match facts.melee_lunge_delay {
            Some(MeleeLungeArrival::ArrivesIn(arrival)) => arrival,
            Some(MeleeLungeArrival::NotNeeded) => AnimationElapsed::ZERO,
            Some(MeleeLungeArrival::CannotArrive) | None => return Ok(()),
        };
        let Some(RecoveryTime::After(recovery)) = facts.melee_recovery_seconds else {
            return Ok(());
        };
        let windup = CombatDuration::from_elapsed(config.windup_seconds)?;
        let timer = Timer::new(
            config.windup_seconds.max(arrival).native_duration()?,
            TimerMode::Once,
        );
        input.0 = long_weapon_windup_movement(
            facts.weapon_reach,
            facts.preferred_melee_measure,
            distance,
            config.long_weapon_measure_threshold_metres,
        );
        let Some(strike_family) = skeleton.available_strike_family(facts.strike_family) else {
            return Ok(());
        };
        cmd.trigger(MeleeAttackStartedIntent {
            attacker: entity,
            target: Some(target),
            windup,
            reported_precision: ReportedPrecision::new(config.hit_precision)
                .expect("AI precision is finite"),
            strike_family,
            hand: AttackHand::Main,
        });
        controller.phase = OffensiveCombatPhase::MeleeWindup {
            timer,
            strike_family,
            recovery_seconds: recovery,
        };
        Ok(())
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "phase execution needs actor, target, authored config, and mutable bot controls"
)]
pub(super) fn drive_offensive_phase(
    cmd: &mut Commands,
    time: &Time<()>,
    entity: Entity,
    target: Entity,
    distance: f32,
    input: &mut AuthoritativeMovementIntent,
    controller: &mut OffensiveCombatAi,
    skeleton: &SkeletonState,
    target_skeleton: &SkeletonState,
    facts: &OffensiveFacts,
    config: &AiOffenseConfig,
    random: &mut CombatRandom,
) -> ElapsedResult<()> {
    if matches!(controller.phase, OffensiveCombatPhase::Pursuing) {
        drive_pursuit_phase(
            cmd,
            entity,
            target,
            distance,
            input,
            controller,
            skeleton,
            target_skeleton,
            facts,
            config,
            random,
        )?;
        return Ok(());
    }
    match &mut controller.phase {
        OffensiveCombatPhase::Pursuing => unreachable!("pursuit was handled above"),
        OffensiveCombatPhase::GuardingCommittedThreat => {
            maintain_committed_threat_guard(
                input,
                controller,
                target_skeleton,
                random,
                facts,
                config,
            )?;
        }
        OffensiveCombatPhase::Assessing(timer) => {
            hold_and_tick(input, timer, time);
            if timer.is_finished() {
                controller.phase = OffensiveCombatPhase::Pursuing;
            }
        }
        OffensiveCombatPhase::MeleeWindup { .. } | OffensiveCombatPhase::RangedWindup(_) => {
            complete_windup(
                MeleePursuit {
                    entity,
                    target,
                    distance,
                },
                cmd,
                time,
                OffensiveControls { input, controller },
                facts,
                config,
                random,
            )?;
        }
        OffensiveCombatPhase::Cooldown(timer) => {
            input.0 = None;
            timer.tick(time.delta());
            if timer.is_finished() {
                controller.phase = OffensiveCombatPhase::Pursuing;
            }
        }
        OffensiveCombatPhase::WithdrawingUnableToContinue(timer) => {
            continue_withdrawal(cmd, time, entity, input, timer);
        }
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "phase execution needs actor, target, authored config, and mutable bot controls"
)]
fn drive_pursuit_phase(
    cmd: &mut Commands,
    entity: Entity,
    target: Entity,
    distance: f32,
    input: &mut AuthoritativeMovementIntent,
    controller: &mut OffensiveCombatAi,
    skeleton: &SkeletonState,
    target_skeleton: &SkeletonState,
    facts: &OffensiveFacts,
    config: &AiOffenseConfig,
    random: &mut CombatRandom,
) -> ElapsedResult<()> {
    if facts.use_ranged {
        drive_ranged_pursuit(
            cmd,
            RangedPursuit {
                entity,
                target,
                distance,
            },
            input,
            controller,
            facts,
            config,
        )?;
    } else if facts.weapon_is_melee
        && matches!(
            facts.melee_lunge_delay,
            Some(MeleeLungeArrival::NotNeeded | MeleeLungeArrival::ArrivesIn(_))
        )
        && target_skeleton.action_kind() == SkeletonAction::Attack
        && target_skeleton.action_phase().fraction() < 0.5
        && random.unit_f32()
            < committed_threat_recognition_probability(
                target_skeleton.action_phase(),
                facts.instinct,
            )
    {
        // Do not knowingly begin a fresh committed strike into an attack that
        // is already on its way. Hold the guard through the threat, then use a
        // seeded perception/initiative delay before looking for a counter.
        // Existing attacks may still overlap; this only prevents deterministic
        // cadence from repeatedly assigning initiative to the same actor.
        input.0 = None;
        controller.phase = OffensiveCombatPhase::GuardingCommittedThreat;
    } else if facts.weapon_is_melee
        && facts.melee_attack_available
        && matches!(facts.melee_recovery_seconds, Some(RecoveryTime::After(_)))
        && facts.dimensions.arm_reach_metres > 0.0
        && matches!(
            facts.melee_lunge_delay,
            Some(MeleeLungeArrival::NotNeeded | MeleeLungeArrival::ArrivesIn(_))
        )
    {
        MeleePursuit {
            entity,
            target,
            distance,
        }
        .start(cmd, input, controller, skeleton, facts, config)?;
    } else if facts.weapon_is_melee && !facts.melee_attack_available {
        input.0 = Some(-Vec2::Y);
        controller.phase = OffensiveCombatPhase::WithdrawingUnableToContinue(Timer::from_seconds(
            1.0,
            TimerMode::Once,
        ));
        cmd.trigger(BotContinuationDecisionEvent {
            combatant: entity,
            decision: BotContinuationDecision::Withdraw,
        });
    } else if facts.weapon_is_melee
        && below_preferred_long_weapon_measure(
            facts.weapon_reach,
            facts.preferred_melee_measure,
            distance,
            config.long_weapon_measure_threshold_metres,
        )
    {
        input.0 = Some(-Vec2::Y);
    } else {
        input.0 = Some(Vec2::Y);
    }
    Ok(())
}

fn maintain_committed_threat_guard(
    input: &mut AuthoritativeMovementIntent,
    controller: &mut OffensiveCombatAi,
    target_skeleton: &SkeletonState,
    random: &mut CombatRandom,
    facts: &OffensiveFacts,
    config: &AiOffenseConfig,
) -> ElapsedResult<()> {
    input.0 = None;
    if target_skeleton.action_kind() != SkeletonAction::Attack
        || target_skeleton.action_phase().fraction() >= 0.5
    {
        controller.phase = OffensiveCombatPhase::Assessing(Timer::new(
            initiative_delay_seconds(random, facts.instinct, config)?.native_duration()?,
            TimerMode::Once,
        ));
    }
    Ok(())
}

fn hold_and_tick(input: &mut AuthoritativeMovementIntent, timer: &mut Timer, time: &Time<()>) {
    input.0 = None;
    timer.tick(time.delta());
}

fn continue_withdrawal(
    cmd: &mut Commands,
    time: &Time<()>,
    entity: Entity,
    input: &mut AuthoritativeMovementIntent,
    timer: &mut Timer,
) {
    input.0 = Some(-Vec2::Y);
    timer.tick(time.delta());
    if timer.is_finished() {
        input.0 = None;
        cmd.entity(entity).insert(CombatantYielded);
        cmd.trigger(BotContinuationDecisionEvent {
            combatant: entity,
            decision: BotContinuationDecision::Yield,
        });
    }
}

fn complete_windup(
    actor: MeleePursuit,
    cmd: &mut Commands,
    time: &Time<()>,
    controls: OffensiveControls<'_>,
    facts: &OffensiveFacts,
    config: &AiOffenseConfig,
    random: &mut CombatRandom,
) -> ElapsedResult<()> {
    let MeleePursuit {
        entity,
        target,
        distance,
    } = actor;
    let OffensiveControls { input, controller } = controls;
    match &mut controller.phase {
        OffensiveCombatPhase::MeleeWindup {
            timer,
            strike_family: _,
            recovery_seconds,
        } => {
            input.0 = long_weapon_windup_movement(
                facts.weapon_reach,
                facts.preferred_melee_measure,
                distance,
                config.long_weapon_measure_threshold_metres,
            );
            let mut advanced = timer.clone();
            advanced.tick(time.delta());
            if advanced.is_finished() {
                let jitter = AnimationElapsed::from_seconds(
                    random.range_f32(0.0, config.cadence_jitter_seconds.seconds()),
                )?;
                let cooldown = recovery_seconds.checked_add(jitter)?.native_duration()?;
                controller.phase =
                    OffensiveCombatPhase::Cooldown(Timer::new(cooldown, TimerMode::Once));
            } else {
                *timer = advanced;
            }
        }
        OffensiveCombatPhase::RangedWindup(timer) => {
            input.0 = None;
            let mut advanced = timer.clone();
            advanced.tick(time.delta());
            if advanced.is_finished() {
                let jitter = AnimationElapsed::from_seconds(
                    random.range_f32(0.0, config.cadence_jitter_seconds.seconds()),
                )?;
                let cooldown = config
                    .cooldown_seconds
                    .checked_add(jitter)?
                    .native_duration()?;
                cmd.trigger(RangedAttackIntent {
                    attacker: entity,
                    target: Some(target),
                    body_part: config.target_body_part,
                    reported_precision: ReportedPrecision::new(config.hit_precision)
                        .expect("AI precision is finite"),
                });
                controller.phase =
                    OffensiveCombatPhase::Cooldown(Timer::new(cooldown, TimerMode::Once));
            } else {
                *timer = advanced;
            }
        }
        _ => {}
    }
    Ok(())
}
