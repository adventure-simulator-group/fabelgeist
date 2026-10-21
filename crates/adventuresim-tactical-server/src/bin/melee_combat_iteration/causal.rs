use super::*;

pub(super) use autoresolve::{record_autoresolve_telemetry, record_timeline_event};

mod autoresolve;

pub(super) fn record_tactical_first_contact(
    summary: &mut TacticalCausalSummary,
    outcome: &TacticalMeleeOutcome,
    john_name: &str,
) {
    if let Some(first_contact) = outcome.events.first() {
        if first_contact.attacker == john_name {
            summary.john_first_contacts += 1;
        } else {
            summary.opponent_first_contacts += 1;
        }
    }
}

pub(super) fn record_tactical_decisions(
    summary: &mut TacticalCausalSummary,
    outcome: &TacticalMeleeOutcome,
    john_name: &str,
) -> IterationResult<()> {
    let mut previous_starts = [None, None];
    for decision in &outcome.decision_events {
        let john = decision.combatant == john_name;
        record_summary_decision(summary, decision.decision, decision.status, john);
        let combatant = if john {
            &mut summary.john
        } else {
            &mut summary.opponent
        };
        let previous = &mut previous_starts[usize::from(!john)];
        record_combatant_decision(combatant, decision, previous)?;
    }
    Ok(())
}

fn record_summary_decision(
    summary: &mut TacticalCausalSummary,
    decision: TacticalDecision,
    status: TacticalDecisionStatus,
    john: bool,
) {
    let pair = if john {
        (
            &mut summary.john_attack_starts,
            &mut summary.john_canceled_for_defense,
            &mut summary.john_transformed_by_defense,
        )
    } else {
        (
            &mut summary.opponent_attack_starts,
            &mut summary.opponent_canceled_for_defense,
            &mut summary.opponent_transformed_by_defense,
        )
    };
    match (decision, status) {
        (TacticalDecision::Attack, TacticalDecisionStatus::Started) => *pair.0 += 1,
        (TacticalDecision::Attack, TacticalDecisionStatus::CanceledForDefense) => *pair.1 += 1,
        (TacticalDecision::Attack, TacticalDecisionStatus::TransformedByDefense) => *pair.2 += 1,
        _ => {}
    }
}

fn record_combatant_decision(
    combatant: &mut TacticalCombatantCausal,
    decision: &adventuresim_tactical_server::iteration::TacticalDecisionLogEntry,
    previous: &mut Option<TacticalElapsed>,
) -> IterationResult<()> {
    match (decision.decision, decision.status) {
        (TacticalDecision::Attack, TacticalDecisionStatus::Started) => {
            combatant.attack_starts += 1;
            if let Some(previous_seconds) = previous.replace(decision.elapsed_seconds) {
                combatant.attack_interval_samples += 1;
                combatant.mean_attack_start_interval_seconds.checked_add(
                    Measurement::<TacticalSeconds>::from_interval(
                        decision.elapsed_seconds.elapsed_since(previous_seconds)?,
                    )?,
                )?;
            }
        }
        (TacticalDecision::Attack, TacticalDecisionStatus::CanceledForDefense) => {
            combatant.committed_attacks_canceled += 1;
        }
        (TacticalDecision::Attack, TacticalDecisionStatus::TransformedByDefense) => {
            combatant.committed_attacks_transformed += 1;
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn record_tactical_wounds(
    summary: &mut TacticalCausalSummary,
    outcome: &TacticalMeleeOutcome,
    john_name: &str,
) {
    for wound in &outcome.wound_events {
        let combatant = if wound.combatant == john_name {
            &mut summary.john
        } else {
            &mut summary.opponent
        };
        match wound.kind {
            adventuresim_core::combat::CombatWoundKind::Open => {
                summary.open_wounds += 1;
                combatant.open_wounds_received += 1;
            }
            adventuresim_core::combat::CombatWoundKind::Internal => {
                summary.internal_wounds += 1;
                combatant.internal_wounds_received += 1;
            }
        }
    }
}

pub(super) fn record_tactical_event(
    summary: &mut TacticalCausalSummary,
    event: &adventuresim_tactical_server::iteration::TacticalMeleeLogEntry,
    john_name: &str,
) -> MeasurementResult<()> {
    let attacker_is_john = event.attacker == john_name;
    record_tactical_attacker(summary, event, attacker_is_john)?;
    record_tactical_defender(summary, event, attacker_is_john)?;
    record_tactical_totals(summary, event)?;
    Ok(())
}

fn record_tactical_attacker(
    summary: &mut TacticalCausalSummary,
    event: &adventuresim_tactical_server::iteration::TacticalMeleeLogEntry,
    john: bool,
) -> MeasurementResult<()> {
    let attacker = if john {
        summary.john_resolved_attacks += 1;
        &mut summary.john
    } else {
        summary.opponent_resolved_attacks += 1;
        &mut summary.opponent
    };
    attacker.resolved_attacks += 1;
    attacker.contacts_dealt += u64::from(event.contact_energy_joules > 0.0);
    attacker.contact_energy_samples += 1;
    attacker.mean_contact_energy_joules.checked_add(
        Measurement::<ContactJoules>::from_native(event.contact_energy_joules)?,
    )?;
    attacker.maximum_fatigue = attacker
        .maximum_fatigue
        .max(Measurement::<Fatigue>::from_native(event.attacker_incapacitation.fatigue)?);
    Ok(())
}

fn record_tactical_defender(
    summary: &mut TacticalCausalSummary,
    event: &adventuresim_tactical_server::iteration::TacticalMeleeLogEntry,
    attacker_is_john: bool,
) -> MeasurementResult<()> {
    let defender = if attacker_is_john {
        &mut summary.opponent
    } else {
        &mut summary.john
    };
    defender.attacks_received += 1;
    defender.maximum_fatigue = defender
        .maximum_fatigue
        .max(Measurement::<Fatigue>::from_native(event.defender_incapacitation.fatigue)?);

    *defender
        .anatomical_subregions_received
        .entry(event.anatomical_subregion.clone())
        .or_default() += 1;
    match event.defender_decision {
        TacticalDecision::Block => defender.block_attempts += 1,
        TacticalDecision::Parry => defender.parry_attempts += 1,
        TacticalDecision::Dodge => defender.dodge_attempts += 1,
        _ => {}
    }
    if event.defender_decision == TacticalDecision::Dodge {
        defender.dodge_avoids += u64::from(event.outcome == TacticalContactOutcome::Avoided);
        defender.dodge_contacts += u64::from(event.outcome != TacticalContactOutcome::Avoided);
    }
    record_defensive_implement(defender, event);
    record_defender_armor(defender, event);
    Ok(())
}

fn record_defensive_implement(
    defender: &mut TacticalCombatantCausal,
    event: &adventuresim_tactical_server::iteration::TacticalMeleeLogEntry,
) {
    if event.defensive_implement_kind == Some(TacticalDefenseImplement::Shield) {
        defender.shield_block_attempts += 1;
        defender.effective_shield_blocks +=
            u64::from(event.outcome == TacticalContactOutcome::Defended);
    } else if matches!(
        event.defender_decision,
        TacticalDecision::Block | TacticalDecision::Parry
    ) && event.outcome == TacticalContactOutcome::Defended
    {
        defender.effective_weapon_contacts += 1;
    }
}

fn record_defender_armor(
    defender: &mut TacticalCombatantCausal,
    event: &adventuresim_tactical_server::iteration::TacticalMeleeLogEntry,
) {
    if event.coverage_contact == TacticalCoverageContact::ArmorSurface {
        defender.armor_surface_contacts_received += 1;
        increment_armor_outcome(
            event.armor_outcome,
            &mut defender.armor_stopped,
            &mut defender.armor_deflected,
            &mut defender.armor_penetrated,
        );
    } else if event.coverage_contact == TacticalCoverageContact::Gap {
        defender.armor_gap_contacts_received += 1;
    }
    for layer in &event.armor_layer_chain {
        let key = format!("{}:{:?}", layer.item_id, layer.material);
        let distribution = if layer.intersected {
            &mut defender.armor_layers_intersected
        } else {
            &mut defender.armor_layers_missed
        };
        *distribution.entry(key).or_default() += 1;
    }
}

fn record_tactical_totals(
    summary: &mut TacticalCausalSummary,
    event: &adventuresim_tactical_server::iteration::TacticalMeleeLogEntry,
) -> MeasurementResult<()> {
    let separation =
        Measurement::<CenterSeparationMetres>::from_native(event.center_separation_metres)?;
    summary.minimum_contact_separation_metres = Some(
        summary
            .minimum_contact_separation_metres
            .map_or(separation, |minimum| minimum.min(separation)),
    );
    summary.maximum_contact_separation_metres = Some(
        summary
            .maximum_contact_separation_metres
            .map_or(separation, |maximum| maximum.max(separation)),
    );
    summary.maximum_fatigue = summary
        .maximum_fatigue
        .max(Measurement::<Fatigue>::from_native(event.attacker_incapacitation.fatigue)?)
        .max(Measurement::<Fatigue>::from_native(event.defender_incapacitation.fatigue)?);
    record_dodge_totals(summary, event);
    record_defense_totals(summary, event);
    record_armor_totals(summary, event);
    Ok(())
}

fn record_dodge_totals(
    summary: &mut TacticalCausalSummary,
    event: &adventuresim_tactical_server::iteration::TacticalMeleeLogEntry,
) {
    if event.defender_decision != TacticalDecision::Dodge {
        return;
    }
    summary.dodge_attempts += 1;
    summary.dodge_avoids += u64::from(event.outcome == TacticalContactOutcome::Avoided);
    summary.dodge_contacts += u64::from(event.outcome != TacticalContactOutcome::Avoided);
}

fn record_defense_totals(
    summary: &mut TacticalCausalSummary,
    event: &adventuresim_tactical_server::iteration::TacticalMeleeLogEntry,
) {
    if event.defensive_implement_kind == Some(TacticalDefenseImplement::Shield) {
        summary.shield_block_attempts += 1;
        summary.shield_blocks_defended +=
            u64::from(event.outcome == TacticalContactOutcome::Defended);
        summary.shield_blocks_failed +=
            u64::from(event.outcome != TacticalContactOutcome::Defended);
    } else {
        summary.weapon_defense_contacts +=
            u64::from(event.outcome == TacticalContactOutcome::Defended);
    }
}

fn record_armor_totals(
    summary: &mut TacticalCausalSummary,
    event: &adventuresim_tactical_server::iteration::TacticalMeleeLogEntry,
) {
    if event.coverage_contact != TacticalCoverageContact::ArmorSurface {
        return;
    }
    summary.armor_contacts += 1;
    increment_armor_outcome(
        event.armor_outcome,
        &mut summary.armor_stopped,
        &mut summary.armor_deflected,
        &mut summary.armor_penetrated,
    );
    let partition = event.resisted_energy_joules
        + event.transmitted_energy_joules
        + event.penetrated_energy_joules;
    summary.energy_partition_failures +=
        u64::from((partition - event.contact_energy_joules).abs() > 0.01);
}

fn increment_armor_outcome(
    outcome: Option<adventuresim_core::combat::ArmorImpactOutcome>,
    stopped: &mut u64,
    deflected: &mut u64,
    penetrated: &mut u64,
) {
    match outcome {
        Some(adventuresim_core::combat::ArmorImpactOutcome::Stopped) => *stopped += 1,
        Some(adventuresim_core::combat::ArmorImpactOutcome::Deflected) => *deflected += 1,
        Some(adventuresim_core::combat::ArmorImpactOutcome::Penetrated) => *penetrated += 1,
        None => {}
    }
}
