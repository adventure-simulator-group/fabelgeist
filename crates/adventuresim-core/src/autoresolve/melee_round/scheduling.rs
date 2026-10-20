use super::*;

struct MeleeEngagement {
    attacker_id: u64,
    target_id: u64,
    initial_separation: f32,
    distance: f32,
}

struct AttackStartTrace {
    attacker_id: u64,
    target_id: u64,
    distance: f32,
    readiness: f32,
    window_start: f32,
    adaptation: AnimationElapsed,
    timing: ScheduledMeleeTiming,
}

impl MeleeEngagement {
    fn observe(
        attacker: &Combatant,
        defender: &Combatant,
        parameters: crate::combat::AutoresolveParameters,
    ) -> Self {
        let initial_separation =
            maximum_melee_pair_surface_separation(attacker, defender, parameters);
        let attacker_distance = if attacker.melee_engagement_target == Some(defender.id) {
            attacker.melee_engagement_distance_metres
        } else {
            initial_separation
        };
        let defender_distance = if defender.melee_engagement_target == Some(attacker.id) {
            defender.melee_engagement_distance_metres
        } else {
            initial_separation
        };
        Self {
            attacker_id: attacker.id,
            target_id: defender.id,
            initial_separation,
            distance: attacker_distance.min(defender_distance).max(0.0),
        }
    }
    fn commit(self, attacker: &mut Combatant, defender: &mut Combatant) {
        if attacker.melee_engagement_target != Some(self.target_id) {
            attacker.melee_engagement_target = Some(self.target_id);
            attacker.melee_engagement_distance_metres = self.initial_separation;
        }
        if defender.melee_engagement_target != Some(self.attacker_id) {
            defender.melee_engagement_target = Some(self.attacker_id);
            defender.melee_engagement_distance_metres = self.initial_separation;
        }
    }
}

impl AttackStartTrace {
    fn record(self, recorder: &mut BattleRecorder) {
        let Self {
            attacker_id,
            target_id,
            distance,
            readiness,
            window_start,
            adaptation,
            timing,
        } = self;
        let mut event =
            MeleeTimelineEvent::at(MeleeTimelineKind::AttackStarted, timing.started_at_seconds);
        event.combatant_id = Some(attacker_id);
        event.target_id = Some(target_id);
        event.engagement_distance_before_metres = Some(distance);
        event.engagement_distance_after_metres = Some(distance);
        event.readiness_before_seconds = Some(readiness);
        event.readiness_after_seconds = Some(timing.recovery_until_seconds);
        event.attack_id = Some(timing.attack_id(attacker_id));
        event.attack_started_tick = Some(MeleeTimelineEvent::tick_at(timing.started_at_seconds));
        event.attack_contact_tick = Some(MeleeTimelineEvent::tick_at(timing.contact_at_seconds));
        event.attack_recovery_tick =
            Some(MeleeTimelineEvent::tick_at(timing.recovery_until_seconds));
        event.phase_before = Some(if readiness > window_start {
            MeleeTimelinePhase::Recovery
        } else {
            MeleeTimelinePhase::NeutralGuard
        });
        event.phase_after = Some(MeleeTimelinePhase::Windup);
        event.phase_adaptation_delay_seconds = Some(adaptation);
        recorder.record_timeline(event);
    }
}

pub(in crate::autoresolve) fn initialize_melee_phases(
    allies: &mut [Combatant],
    enemies: &mut [Combatant],
    random: &mut DeterministicRng,
    parameters: crate::combat::AutoresolveParameters,
) -> ElapsedResult<()> {
    let mut order = allies
        .iter()
        .enumerate()
        .map(|(index, combatant)| (combatant.id, ScheduledBattleSide::Allies, index))
        .chain(
            enemies
                .iter()
                .enumerate()
                .map(|(index, combatant)| (combatant.id, ScheduledBattleSide::Enemies, index)),
        )
        .collect::<Vec<_>>();
    order.sort_by_key(|(id, _, _)| *id);
    for (_, side, index) in order {
        let combatant = match side {
            ScheduledBattleSide::Allies => &mut allies[index],
            ScheduledBattleSide::Enemies => &mut enemies[index],
        };
        let sampled_delay = parameters.melee_initiative_delay_min_seconds.seconds()
            + random.unit_f32()
                * (parameters.melee_initiative_delay_max_seconds.seconds()
                    - parameters.melee_initiative_delay_min_seconds.seconds());
        let delay = sampled_delay * (3.0 / combatant.attributes.instinct.max(0.5)).clamp(0.6, 2.0);
        let jitter = parameters
            .melee_cadence_jitter_seconds
            .checked_scale(random.unit_f32())?;
        // Both derived spans are checked before publishing this initial schedule.
        let delay = AnimationElapsed::from_seconds(delay)?;
        combatant.melee_recovery_until_seconds = delay.seconds();
        combatant.melee_interval_jitter_seconds = jitter;
    }
    Ok(())
}

pub(in crate::autoresolve) fn schedule_side_melee_attacks(
    attackers: &mut [Combatant],
    defenders: &mut [Combatant],
    round: usize,
    random: &mut DeterministicRng,
    recorder: &mut BattleRecorder,
    parameters: crate::combat::AutoresolveParameters,
) -> BattleResolutionResult<Vec<ScheduledMeleeAttack>> {
    let start = round.saturating_sub(1) as f32 * parameters.combat_round_seconds.seconds();
    schedule_side_melee_attacks_in_window(
        attackers,
        defenders,
        start,
        parameters.combat_round_seconds,
        random,
        recorder,
        parameters,
    )
}

pub(in crate::autoresolve) fn schedule_side_melee_attacks_in_window(
    attackers: &mut [Combatant],
    defenders: &mut [Combatant],
    window_start: f32,
    window_seconds: AnimationElapsed,
    random: &mut DeterministicRng,
    recorder: &mut BattleRecorder,
    parameters: crate::combat::AutoresolveParameters,
) -> BattleResolutionResult<Vec<ScheduledMeleeAttack>> {
    let mut scheduled = Vec::new();
    for index in 0..attackers.len() {
        if let Some(attack) = schedule_attacker(
            index,
            attackers,
            defenders,
            window_start,
            window_seconds,
            random,
            recorder,
            parameters,
        )? {
            scheduled.push(attack);
        }
    }
    Ok(scheduled)
}

#[expect(
    clippy::too_many_arguments,
    reason = "one actor scheduling joins both sides and deterministic state"
)]
fn schedule_attacker(
    index: usize,
    attackers: &mut [Combatant],
    defenders: &mut [Combatant],
    window_start: f32,
    window_seconds: AnimationElapsed,
    random: &mut DeterministicRng,
    recorder: &mut BattleRecorder,
    parameters: crate::combat::AutoresolveParameters,
) -> BattleResolutionResult<Option<ScheduledMeleeAttack>> {
    if attackers[index].is_defeated()
        || side_defeated(defenders)
        || preferred_attack_mode(&attackers[index]) != BattleAttackKind::Melee
    {
        return Ok(None);
    }
    if !attackers[index].can_attack_melee() {
        attackers[index].yielded = true;
        return Ok(None);
    }
    let (target_index, flanking) = melee_assignment(index, attackers, defenders, parameters);
    let window_end = window_start + window_seconds.seconds();
    if let (Some(started_at_seconds), Some(contact_at_seconds)) = (
        attackers[index].melee_attack_started_at_seconds,
        attackers[index].melee_attack_contact_at_seconds,
    ) {
        return Ok(
            (contact_at_seconds <= window_end).then(|| ScheduledMeleeAttack {
                attacker_index: index,
                target_index,
                flanking,
                attack_timing: ScheduledMeleeTiming {
                    started_at_seconds,
                    contact_at_seconds,
                    recovery_until_seconds: attackers[index].melee_recovery_until_seconds,
                },
            }),
        );
    }
    let engagement =
        MeleeEngagement::observe(&attackers[index], &defenders[target_index], parameters);
    let MeleeEngagement {
        attacker_id,
        target_id,
        distance,
        ..
    } = engagement;
    let reach = melee_effective_reach(&attackers[index]);
    if distance > reach + parameters.melee_lunge_maximum_travel_metres {
        engagement.commit(&mut attackers[index], &mut defenders[target_index]);
        return Ok(None);
    }
    let readiness = attackers[index].melee_recovery_until_seconds
        + attackers[index]
            .melee_phase_adaptation_delay_seconds
            .seconds();
    let Some(started) = available_attack_start(window_start, window_end, window_start, readiness)
    else {
        engagement.commit(&mut attackers[index], &mut defenders[target_index]);
        return Ok(None);
    };
    let Some(interval) = attack_interval(&attackers[index], parameters)?.available() else {
        engagement.commit(&mut attackers[index], &mut defenders[target_index]);
        return Ok(None);
    };
    engagement.commit(&mut attackers[index], &mut defenders[target_index]);
    let timing = ScheduledMeleeTiming {
        started_at_seconds: started,
        contact_at_seconds: started + parameters.melee_windup_seconds.seconds(),
        recovery_until_seconds: started + interval.seconds(),
    };
    let adaptation = attackers[index].melee_phase_adaptation_delay_seconds;
    let jitter = parameters
        .melee_cadence_jitter_seconds
        .checked_scale(random.unit_f32())?;
    timing.commit(&mut attackers[index], distance, jitter);
    AttackStartTrace {
        attacker_id,
        target_id,
        distance,
        readiness,
        window_start,
        adaptation,
        timing,
    }
    .record(recorder);
    Ok(
        (timing.contact_at_seconds <= window_end).then_some(ScheduledMeleeAttack {
            attacker_index: index,
            target_index,
            flanking,
            attack_timing: timing,
        }),
    )
}

fn attack_interval(
    attacker: &Combatant,
    parameters: crate::combat::AutoresolveParameters,
) -> BattleResolutionResult<crate::combat::RecoveryTime> {
    let base = attacker
        .equipment
        .melee_weapon
        .map_or(parameters.reference_melee_attack_seconds, |weapon| {
            weapon.attack_interval_seconds
        })
        .max(parameters.minimum_attack_interval_seconds)
        .checked_add(attacker.melee_interval_jitter_seconds)?;
    Ok(crate::combat::incapacitation_adjusted_recovery_seconds(
        base,
        attacker.incapacitation_performance()?,
    )?)
}

pub(in crate::autoresolve) fn scheduled_side_contacts_in_window(
    attackers: &[Combatant],
    defenders: &[Combatant],
    start: f32,
    end: f32,
    parameters: crate::combat::AutoresolveParameters,
) -> Vec<ScheduledMeleeAttack> {
    attackers
        .iter()
        .enumerate()
        .filter_map(|(index, attacker)| {
            let started = attacker.melee_attack_started_at_seconds?;
            let contact = attacker.melee_attack_contact_at_seconds?;
            if contact < start || contact > end {
                return None;
            }
            let (assigned, flanking) = melee_assignment(index, attackers, defenders, parameters);
            let target = attacker
                .melee_engagement_target
                .and_then(|id| defenders.iter().position(|candidate| candidate.id == id))
                .unwrap_or(assigned);
            Some(ScheduledMeleeAttack {
                attacker_index: index,
                target_index: target,
                flanking,
                attack_timing: ScheduledMeleeTiming {
                    started_at_seconds: started,
                    contact_at_seconds: contact,
                    recovery_until_seconds: attacker.melee_recovery_until_seconds,
                },
            })
        })
        .collect()
}

pub(in crate::autoresolve) fn take_side_turns(
    attackers: &mut [Combatant],
    defenders: &mut [Combatant],
    round: usize,
    random: &mut DeterministicRng,
    recorder: &mut BattleRecorder,
    parameters: crate::combat::AutoresolveParameters,
) -> BattleResolutionResult<()> {
    for attack in
        schedule_side_melee_attacks(attackers, defenders, round, random, recorder, parameters)?
    {
        if !scheduled_attack_is_current(&attackers[attack.attacker_index], attack.attack_timing) {
            continue;
        }
        let id = attack
            .attack_timing
            .attack_id(attackers[attack.attacker_index].id);
        resolve_melee_turn(
            attack.attacker_index,
            attack.target_index,
            attack.flanking,
            attackers,
            defenders,
            round,
            random,
            recorder,
            parameters,
            attack.attack_timing,
            MeleeContactBatch {
                id,
                members: vec![id],
                order: 0,
            },
        )?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub(in crate::autoresolve) struct ScheduledMeleeAttack {
    pub(in crate::autoresolve) attacker_index: usize,
    pub(in crate::autoresolve) target_index: usize,
    pub(in crate::autoresolve) flanking: f32,
    pub(in crate::autoresolve) attack_timing: ScheduledMeleeTiming,
}

#[derive(Clone, Debug)]
pub(in crate::autoresolve) struct MeleeContactBatch {
    pub(in crate::autoresolve) id: u64,
    pub(in crate::autoresolve) members: Vec<u64>,
    pub(in crate::autoresolve) order: u32,
}

pub(in crate::autoresolve) fn scheduled_attack_is_current(
    attacker: &Combatant,
    timing: ScheduledMeleeTiming,
) -> bool {
    attacker.melee_attack_started_at_seconds == Some(timing.started_at_seconds)
        && attacker.melee_attack_contact_at_seconds == Some(timing.contact_at_seconds)
}
