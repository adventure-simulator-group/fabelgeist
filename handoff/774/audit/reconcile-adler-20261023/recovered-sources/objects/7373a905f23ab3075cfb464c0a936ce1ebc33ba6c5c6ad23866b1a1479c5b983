use super::*;

pub(super) fn resolve_joint_melee_round(
    allies: &mut [Combatant],
    enemies: &mut [Combatant],
    round: usize,
    random: &mut DeterministicRng,
    recorder: &mut BattleRecorder,
    parameters: crate::combat::AutoresolveParameters,
) -> BattleResolutionResult<()> {
    const TACTICAL_SERVER_STEPS_PER_SECOND: f32 = 64.0;
    let step_count = (parameters.combat_round_seconds.seconds() * TACTICAL_SERVER_STEPS_PER_SECOND)
        .round()
        .max(1.0) as usize;
    let window_seconds = AnimationElapsed::from_seconds(
        parameters.combat_round_seconds.seconds() / step_count as f32,
    )?;
    let round_start_seconds = BattleTime::from_seconds(
        round.saturating_sub(1) as f32 * parameters.combat_round_seconds.seconds(),
    )?;
    for step in 0..step_count {
        if side_defeated(allies) || side_defeated(enemies) {
            break;
        }
        resolve_joint_melee_window(
            allies,
            enemies,
            round,
            round_start_seconds.checked_add(window_seconds.checked_scale(step as f32)?)?,
            window_seconds,
            random,
            recorder,
            parameters,
        )?;
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "a fixed window owns both battle sides and recorder"
)]
fn resolve_joint_melee_window(
    allies: &mut [Combatant],
    enemies: &mut [Combatant],
    round: usize,
    window_start_seconds: BattleTime,
    window_seconds: AnimationElapsed,
    random: &mut DeterministicRng,
    recorder: &mut BattleRecorder,
    parameters: crate::combat::AutoresolveParameters,
) -> BattleResolutionResult<()> {
    let window_end_seconds = window_start_seconds.checked_add(window_seconds)?;
    let contact_tolerance = AnimationElapsed::from_seconds(1.0e-6)?;
    let allies_first = allies
        .first()
        .zip(enemies.first())
        .is_none_or(|(a, e)| a.id <= e.id);
    let mut attacks = scheduled_contacts(
        allies,
        enemies,
        window_start_seconds,
        window_end_seconds,
        parameters,
    );
    let movement_until_seconds = attacks
        .iter()
        .map(|(_, attack)| attack.attack_timing.contact_at_seconds)
        .min_by(BattleTime::total_cmp)
        .unwrap_or(window_end_seconds);
    attacks.retain(|(_, attack)| {
        attack
            .attack_timing
            .contact_at_seconds
            .is_within(movement_until_seconds, contact_tolerance)
    });
    advance_joint_melee_movement(
        allies,
        enemies,
        window_start_seconds,
        movement_until_seconds.saturating_elapsed_since(window_start_seconds)?,
        recorder,
        parameters,
    )?;
    sort_contacts(&mut attacks, allies, enemies);
    resolve_contacts(
        attacks, allies, enemies, round, random, recorder, parameters,
    )?;
    if side_defeated(allies) || side_defeated(enemies) {
        return Ok(());
    }
    if movement_until_seconds.precedes_with_margin(window_end_seconds, contact_tolerance) {
        resolve_joint_melee_window(
            allies,
            enemies,
            round,
            movement_until_seconds,
            window_end_seconds.elapsed_since(movement_until_seconds)?,
            random,
            recorder,
            parameters,
        )?;
        return Ok(());
    }
    advance_joint_melee_movement(
        allies,
        enemies,
        movement_until_seconds,
        window_end_seconds.saturating_elapsed_since(movement_until_seconds)?,
        recorder,
        parameters,
    )?;
    schedule_both_sides(
        allies,
        enemies,
        allies_first,
        window_end_seconds,
        random,
        recorder,
        parameters,
    )?;
    Ok(())
}

fn scheduled_contacts(
    allies: &[Combatant],
    enemies: &[Combatant],
    start: BattleTime,
    end: BattleTime,
    parameters: crate::combat::AutoresolveParameters,
) -> Vec<(ScheduledBattleSide, ScheduledMeleeAttack)> {
    scheduled_side_contacts_in_window(allies, enemies, start, end, parameters)
        .into_iter()
        .map(|attack| (ScheduledBattleSide::Allies, attack))
        .chain(
            scheduled_side_contacts_in_window(enemies, allies, start, end, parameters)
                .into_iter()
                .map(|attack| (ScheduledBattleSide::Enemies, attack)),
        )
        .collect()
}

fn attacker_id(
    side: ScheduledBattleSide,
    attack: &ScheduledMeleeAttack,
    allies: &[Combatant],
    enemies: &[Combatant],
) -> u64 {
    match side {
        ScheduledBattleSide::Allies => allies[attack.attacker_index].id,
        ScheduledBattleSide::Enemies => enemies[attack.attacker_index].id,
    }
}

fn sort_contacts(
    attacks: &mut [(ScheduledBattleSide, ScheduledMeleeAttack)],
    allies: &[Combatant],
    enemies: &[Combatant],
) {
    attacks.sort_by(|(left_side, left), (right_side, right)| {
        left.attack_timing
            .contact_at_seconds
            .total_cmp(&right.attack_timing.contact_at_seconds)
            .then_with(|| {
                attacker_id(*left_side, left, allies, enemies).cmp(&attacker_id(
                    *right_side,
                    right,
                    allies,
                    enemies,
                ))
            })
    });
}

fn contact_batch(
    attacks: &[(ScheduledBattleSide, ScheduledMeleeAttack)],
    index: usize,
    allies: &[Combatant],
    enemies: &[Combatant],
) -> MeleeContactBatch {
    let (side, attack) = &attacks[index];
    let contact_seconds = attack.attack_timing.contact_at_seconds;
    let members = attacks
        .iter()
        .filter(|(_, candidate)| candidate.attack_timing.contact_at_seconds == contact_seconds)
        .map(|(candidate_side, candidate)| {
            candidate.attack_timing.attack_id(attacker_id(
                *candidate_side,
                candidate,
                allies,
                enemies,
            ))
        })
        .collect::<Vec<_>>();
    let attack_id = attack
        .attack_timing
        .attack_id(attacker_id(*side, attack, allies, enemies));
    let order = members
        .iter()
        .position(|member| *member == attack_id)
        .unwrap_or_default() as u32;
    MeleeContactBatch {
        id: members.first().copied().unwrap_or(attack_id),
        members,
        order,
    }
}

fn resolve_contacts(
    attacks: Vec<(ScheduledBattleSide, ScheduledMeleeAttack)>,
    allies: &mut [Combatant],
    enemies: &mut [Combatant],
    round: usize,
    random: &mut DeterministicRng,
    recorder: &mut BattleRecorder,
    parameters: crate::combat::AutoresolveParameters,
) -> BattleResolutionResult<()> {
    let batches = (0..attacks.len())
        .map(|index| contact_batch(&attacks, index, allies, enemies))
        .collect::<Vec<_>>();
    for ((side, attack), batch) in attacks.into_iter().zip(batches) {
        let attacker = match side {
            ScheduledBattleSide::Allies => &allies[attack.attacker_index],
            ScheduledBattleSide::Enemies => &enemies[attack.attacker_index],
        };
        if !scheduled_attack_is_current(attacker, attack.attack_timing)
            || (attacker.is_defeated() && batch.members.len() == 1)
        {
            continue;
        }
        match side {
            ScheduledBattleSide::Allies => resolve_melee_turn(
                attack.attacker_index,
                attack.target_index,
                attack.flanking,
                allies,
                enemies,
                round,
                random,
                recorder,
                parameters,
                attack.attack_timing,
                batch,
            ),
            ScheduledBattleSide::Enemies => resolve_melee_turn(
                attack.attacker_index,
                attack.target_index,
                attack.flanking,
                enemies,
                allies,
                round,
                random,
                recorder,
                parameters,
                attack.attack_timing,
                batch,
            ),
        }?;
        let attacker = match side {
            ScheduledBattleSide::Allies => &mut allies[attack.attacker_index],
            ScheduledBattleSide::Enemies => &mut enemies[attack.attacker_index],
        };
        attacker.melee_attack_started_at_seconds = None;
        attacker.melee_attack_contact_at_seconds = None;
        attacker.melee_attack_scheduled_measure_metres = None;
    }
    Ok(())
}

fn schedule_both_sides(
    allies: &mut [Combatant],
    enemies: &mut [Combatant],
    allies_first: bool,
    at_seconds: BattleTime,
    random: &mut DeterministicRng,
    recorder: &mut BattleRecorder,
    parameters: crate::combat::AutoresolveParameters,
) -> BattleResolutionResult<()> {
    let (first, second) = if allies_first {
        (allies, enemies)
    } else {
        (enemies, allies)
    };
    schedule_side_melee_attacks_in_window(
        first,
        second,
        at_seconds,
        AnimationElapsed::ZERO,
        random,
        recorder,
        parameters,
    )?;
    schedule_side_melee_attacks_in_window(
        second,
        first,
        at_seconds,
        AnimationElapsed::ZERO,
        random,
        recorder,
        parameters,
    )?;
    Ok(())
}

fn advance_joint_melee_movement(
    allies: &mut [Combatant],
    enemies: &mut [Combatant],
    interval_start_seconds: BattleTime,
    elapsed_seconds: AnimationElapsed,
    recorder: &mut BattleRecorder,
    parameters: crate::combat::AutoresolveParameters,
) -> BattleResolutionResult<()> {
    for index in 0..allies.len().min(enemies.len()) {
        let (first, second) = if allies[index].id <= enemies[index].id {
            (&mut allies[index], &mut enemies[index])
        } else {
            (&mut enemies[index], &mut allies[index])
        };
        advance_melee_pair_movement(
            first,
            second,
            interval_start_seconds,
            elapsed_seconds,
            recorder,
            parameters,
        )?;
    }
    Ok(())
}
