use super::*;

pub(in crate::melee_combat_iteration) fn record_autoresolve_telemetry(
    attacker: &mut AutoresolveCombatantCausal,
    defender: &mut AutoresolveCombatantCausal,
    telemetry: &adventuresim_core::autoresolve::MeleeContactTelemetry,
) -> IterationResult<()> {
    let interval = telemetry
        .attack_interval_seconds
        .available()
        .ok_or(ReportContractError::UnavailableRecoveryInterval)?;
    *attacker
        .contact_classifications
        .entry(format!("{:?}", telemetry.contact_classification))
        .or_default() += 1;
    record_contact_measures(attacker, telemetry)?;
    attacker.full_energy_intended_contacts_inside_ten_centimetres += u64::from(
        telemetry.actual_contact_measure_metres < 0.1
            && telemetry.contact_classification == MeleeContactClassification::IntendedSurface
            && telemetry.contact_energy_fraction >= 0.999,
    );
    *defender
        .anatomical_subregions_received
        .entry(format!("{:?}", telemetry.anatomical_subregion))
        .or_default() += 1;
    for layer in &telemetry.armor_layer_chain {
        let key = format!("{:?}:{:?}", layer.inventory_item_id, layer.material);
        let distribution = if layer.intersected {
            &mut defender.armor_layers_intersected
        } else {
            &mut defender.armor_layers_missed
        };
        *distribution.entry(key).or_default() += 1;
    }
    attacker.attack_samples += 1;
    attacker
        .mean_attack_interval_seconds
        .checked_add(Measurement::<AutoresolveSeconds>::from_elapsed(interval)?)?;
    attacker.mean_attack_power_multiplier.checked_add(
        Measurement::<AttackPowerMultiplier>::from_native(telemetry.attack_power_multiplier)?,
    )?;
    let performance = Measurement::<IncapacitationPerformance>::from_performance(
        telemetry.attacker_incapacitation_performance,
    )?;
    attacker.minimum_attack_incapacitation_performance = Some(
        attacker
            .minimum_attack_incapacitation_performance
            .map_or(performance, |minimum| minimum.min(performance)),
    );
    Ok(())
}

pub(in crate::melee_combat_iteration) fn record_timeline_event(
    combatant: &mut AutoresolveCombatantCausal,
    event: &adventuresim_core::autoresolve::MeleeTimelineEvent,
) -> IterationResult<()> {
    match event.kind {
        MeleeTimelineKind::Movement => record_movement_event(combatant, event)?,
        MeleeTimelineKind::AttackStarted => combatant.attack_starts += 1,
        MeleeTimelineKind::Response => record_response_event(combatant, event)?,
        MeleeTimelineKind::AttackCanceled => combatant.committed_attacks_canceled += 1,
        MeleeTimelineKind::AttackTransformed => combatant.committed_attacks_transformed += 1,
        MeleeTimelineKind::Contact if event.simultaneous_members.len() > 1 => {
            combatant.simultaneous_contacts += 1;
        }
        MeleeTimelineKind::Contact | MeleeTimelineKind::Terminal => {}
    }
    Ok(())
}

fn record_movement_event(
    combatant: &mut AutoresolveCombatantCausal,
    event: &adventuresim_core::autoresolve::MeleeTimelineEvent,
) -> IterationResult<()> {
    let missing = |observation| ReportContractError::MissingMovementObservation {
        sequence: event.sequence,
        observation,
    };
    let before = event
        .movement_velocity_before_metres_per_second
        .ok_or_else(|| missing(MovementObservation::VelocityBefore))?;
    let after = event
        .movement_velocity_after_metres_per_second
        .ok_or_else(|| missing(MovementObservation::VelocityAfter))?;
    let speed_limit = event
        .movement_speed_limit_metres_per_second
        .ok_or_else(|| missing(MovementObservation::SpeedLimit))?;
    let elapsed = event
        .movement_elapsed_seconds
        .ok_or_else(|| missing(MovementObservation::Elapsed))?;
    if let Some(action) = event.movement_action {
        *combatant
            .movement_actions
            .entry(format!("{action:?}"))
            .or_default() += 1;
    }
    let displacement = event.movement_displacement_metres.unwrap_or_default().abs();
    let velocity = before.magnitude().max(after.magnitude());
    let elapsed_measurement = Measurement::<AutoresolveSeconds>::from_elapsed(elapsed)?;
    combatant
        .movement_elapsed_seconds
        .checked_add(elapsed_measurement)?;
    combatant.movement_segments += 1;
    combatant
        .movement_absolute_displacement_metres
        .checked_add(Measurement::<MovementMetres>::from_native(displacement)?)?;
    combatant.maximum_movement_speed_metres_per_second = combatant
        .maximum_movement_speed_metres_per_second
        .max(Measurement::<MovementSpeed>::from_speed(velocity)?);
    combatant.maximum_movement_segment_seconds = combatant
        .maximum_movement_segment_seconds
        .max(elapsed_measurement);
    // This f32 inequality is the original movement-contract kernel: metres
    // versus metres/second times seconds, with the existing metre tolerance.
    combatant.movement_displacement_limit_failures +=
        u64::from(displacement > speed_limit.metres_per_second() * elapsed.seconds() + 1.0e-5);
    let distance_delta = event
        .engagement_distance_before_metres
        .zip(event.engagement_distance_after_metres)
        .map_or(0.0, |(before, after)| (after - before).abs());
    combatant.movement_nonzero_delta_zero_elapsed +=
        u64::from(distance_delta > 0.0 && elapsed.is_zero());
    Ok(())
}

fn record_response_event(
    combatant: &mut AutoresolveCombatantCausal,
    event: &adventuresim_core::autoresolve::MeleeTimelineEvent,
) -> MeasurementResult<()> {
    if let Some(availability) = event.response_availability {
        *combatant
            .response_availability
            .entry(format!("{availability:?}"))
            .or_default() += 1;
    }
    if let Some(choice) = event.response_choice {
        *combatant
            .response_choices
            .entry(format!("{choice:?}"))
            .or_default() += 1;
    }
    if let Some(delay) = event.phase_adaptation_delay_seconds
        && !delay.is_zero()
    {
        combatant.phase_adaptation_events += 1;
        combatant
            .phase_adaptation_delay_seconds
            .checked_add(Measurement::<AutoresolveSeconds>::from_elapsed(delay)?)?;
    }
    Ok(())
}

fn record_contact_measures(
    attacker: &mut AutoresolveCombatantCausal,
    telemetry: &adventuresim_core::autoresolve::MeleeContactTelemetry,
) -> MeasurementResult<()> {
    attacker.contact_measure_samples += 1;
    attacker.mean_scheduled_contact_measure_metres.checked_add(
        Measurement::<ContactMetres>::from_native(telemetry.scheduled_contact_measure_metres)?,
    )?;
    let actual =
        Measurement::<ContactMetres>::from_native(telemetry.actual_contact_measure_metres)?;
    attacker
        .mean_actual_contact_measure_metres
        .checked_add(actual)?;
    let separation = Measurement::<CenterSeparationMetres>::from_native(
        telemetry.actual_center_separation_metres,
    )?;
    attacker.minimum_actual_contact_measure_metres = Some(
        attacker
            .minimum_actual_contact_measure_metres
            .map_or(actual, |minimum| minimum.min(actual)),
    );
    attacker.maximum_actual_contact_measure_metres = Some(
        attacker
            .maximum_actual_contact_measure_metres
            .map_or(actual, |maximum| maximum.max(actual)),
    );
    attacker.minimum_actual_center_separation_metres = Some(
        attacker
            .minimum_actual_center_separation_metres
            .map_or(separation, |minimum| minimum.min(separation)),
    );
    Ok(())
}
