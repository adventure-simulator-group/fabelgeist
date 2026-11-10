//! Causal report records retain their measurements until serialization.

use std::collections::BTreeMap;

use serde::Serialize;

use super::measurement::*;

#[derive(Serialize)]
pub(super) struct MatchupSummary {
    pub(super) matchup: String,
    pub(super) opponent: String,
    pub(super) tactical_runs: u64,
    pub(super) tactical_john_wins: u64,
    pub(super) tactical_opponent_wins: u64,
    pub(super) tactical_mutual_incapacitations: u64,
    pub(super) tactical_timeouts: u64,
    pub(super) tactical_causal: TacticalCausalSummary,
    pub(super) tactical_total_simulated_seconds: Measurement<TacticalSeconds>,
    pub(super) tactical_mean_simulated_seconds: Measurement<TacticalSeconds>,
    pub(super) autoresolve_runs: u64,
    pub(super) autoresolve_john_wins: u64,
    pub(super) autoresolve_opponent_wins: u64,
    pub(super) autoresolve_mutual_incapacitations: u64,
    pub(super) autoresolve_timeouts: u64,
    pub(super) autoresolve_causal: AutoresolveCausalSummary,
    pub(super) john_win_rate_difference: Measurement<WinRateDifference>,
    pub(super) tactical_wall_milliseconds: Measurement<WallMilliseconds>,
    pub(super) autoresolve_wall_milliseconds: Measurement<WallMilliseconds>,
    pub(super) tactical_duels_per_second: Measurement<DuelsPerWallSecond>,
    pub(super) tactical_simulated_seconds_per_wall_second: Measurement<TacticalSecondsPerWallSecond>,
    pub(super) autoresolve_duels_per_second: Measurement<DuelsPerWallSecond>,
}

#[derive(Default, Serialize)]
pub(super) struct TacticalCausalSummary {
    pub(super) john_attack_starts: u64,
    pub(super) opponent_attack_starts: u64,
    pub(super) john_resolved_attacks: u64,
    pub(super) opponent_resolved_attacks: u64,
    pub(super) john_first_contacts: u64,
    pub(super) opponent_first_contacts: u64,
    pub(super) john_canceled_for_defense: u64,
    pub(super) opponent_canceled_for_defense: u64,
    pub(super) john_transformed_by_defense: u64,
    pub(super) opponent_transformed_by_defense: u64,
    pub(super) dodge_attempts: u64,
    pub(super) dodge_avoids: u64,
    pub(super) dodge_contacts: u64,
    pub(super) shield_block_attempts: u64,
    pub(super) shield_blocks_defended: u64,
    pub(super) shield_blocks_failed: u64,
    pub(super) weapon_defense_contacts: u64,
    pub(super) armor_contacts: u64,
    pub(super) armor_stopped: u64,
    pub(super) armor_deflected: u64,
    pub(super) armor_penetrated: u64,
    pub(super) open_wounds: u64,
    pub(super) internal_wounds: u64,
    pub(super) energy_partition_failures: u64,
    pub(super) minimum_contact_separation_metres: Option<Measurement<CenterSeparationMetres>>,
    pub(super) maximum_contact_separation_metres: Option<Measurement<CenterSeparationMetres>>,
    pub(super) maximum_fatigue: Measurement<Fatigue>,
    pub(super) john: TacticalCombatantCausal,
    pub(super) opponent: TacticalCombatantCausal,
}

#[derive(Default, Serialize)]
pub(super) struct TacticalCombatantCausal {
    pub(super) attack_starts: u64,
    pub(super) resolved_attacks: u64,
    pub(super) contacts_dealt: u64,
    pub(super) attacks_received: u64,
    pub(super) block_attempts: u64,
    pub(super) parry_attempts: u64,
    pub(super) dodge_attempts: u64,
    pub(super) effective_weapon_contacts: u64,
    pub(super) shield_block_attempts: u64,
    pub(super) effective_shield_blocks: u64,
    pub(super) dodge_avoids: u64,
    pub(super) dodge_contacts: u64,
    pub(super) committed_attacks_canceled: u64,
    pub(super) committed_attacks_transformed: u64,
    pub(super) armor_surface_contacts_received: u64,
    pub(super) armor_gap_contacts_received: u64,
    pub(super) armor_stopped: u64,
    pub(super) armor_deflected: u64,
    pub(super) armor_penetrated: u64,
    pub(super) anatomical_subregions_received: BTreeMap<String, u64>,
    pub(super) armor_layers_intersected: BTreeMap<String, u64>,
    pub(super) armor_layers_missed: BTreeMap<String, u64>,
    pub(super) open_wounds_received: u64,
    pub(super) internal_wounds_received: u64,
    pub(super) attack_interval_samples: u64,
    pub(super) mean_attack_start_interval_seconds: Measurement<TacticalSeconds>,
    pub(super) contact_energy_samples: u64,
    pub(super) mean_contact_energy_joules: Measurement<ContactJoules>,
    pub(super) maximum_fatigue: Measurement<Fatigue>,
}

#[derive(Default, Serialize)]
pub(super) struct AutoresolveCausalSummary {
    pub(super) total_rounds: u64,
    pub(super) melee_attacks: u64,
    pub(super) hits: u64,
    pub(super) blocks_or_parries: u64,
    pub(super) block_attempts: u64,
    pub(super) parry_attempts: u64,
    pub(super) misses: u64,
    pub(super) dodge_attempts: u64,
    pub(super) dodge_avoids: u64,
    pub(super) dodge_contacts: u64,
    pub(super) shield_contacts: u64,
    pub(super) weapon_defense_contacts: u64,
    pub(super) armor_contacts: u64,
    pub(super) armor_stopped: u64,
    pub(super) armor_deflected: u64,
    pub(super) armor_penetrated: u64,
    pub(super) john_attacks: u64,
    pub(super) opponent_attacks: u64,
    pub(super) john_first_contacts: u64,
    pub(super) opponent_first_contacts: u64,
    pub(super) john_weapon_reach_metres: Measurement<WeaponReachMetres>,
    pub(super) opponent_weapon_reach_metres: Measurement<WeaponReachMetres>,
    pub(super) total_contact_energy_joules: Measurement<ContactJoules>,
    pub(super) total_health_damage: Measurement<HealthDamage>,
    pub(super) final_blood_loss_fraction: Measurement<BloodLossFraction>,
    pub(super) final_wound_count: u64,
    pub(super) final_open_wound_count: u64,
    pub(super) final_internal_wound_count: u64,
    pub(super) final_wound_flow_fraction_per_second: Measurement<WoundFlowFractionPerSecond>,
    pub(super) final_fatigue: Measurement<Fatigue>,
    pub(super) final_acute_trauma: Measurement<AcuteTrauma>,
    pub(super) final_imbalance: Measurement<Imbalance>,
    pub(super) john_yields: u64,
    pub(super) opponent_yields: u64,
    pub(super) john: AutoresolveCombatantCausal,
    pub(super) opponent: AutoresolveCombatantCausal,
}

#[derive(Default, Serialize)]
pub(super) struct AutoresolveCombatantCausal {
    pub(super) attack_starts: u64,
    pub(super) attacks: u64,
    pub(super) hits_dealt: u64,
    pub(super) attacks_received: u64,
    pub(super) block_attempts: u64,
    pub(super) parry_attempts: u64,
    pub(super) dodge_attempts: u64,
    pub(super) effective_weapon_contacts: u64,
    pub(super) dodge_avoids: u64,
    pub(super) dodge_contacts: u64,
    pub(super) committed_attacks_canceled: u64,
    pub(super) committed_attacks_transformed: u64,
    pub(super) movement_actions: BTreeMap<String, u64>,
    pub(super) movement_segments: u64,
    pub(super) movement_elapsed_seconds: Measurement<AutoresolveSeconds>,
    pub(super) movement_absolute_displacement_metres: Measurement<MovementMetres>,
    pub(super) maximum_movement_speed_metres_per_second: Measurement<MovementSpeed>,
    pub(super) maximum_movement_segment_seconds: Measurement<AutoresolveSeconds>,
    pub(super) movement_displacement_limit_failures: u64,
    pub(super) movement_nonzero_delta_zero_elapsed: u64,
    pub(super) premature_contact_events: u64,
    pub(super) response_availability: BTreeMap<String, u64>,
    pub(super) response_choices: BTreeMap<String, u64>,
    pub(super) phase_adaptation_events: u64,
    pub(super) phase_adaptation_delay_seconds: Measurement<AutoresolveSeconds>,
    pub(super) simultaneous_contacts: u64,
    pub(super) armor_surface_contacts_received: u64,
    pub(super) armor_gap_contacts_received: u64,
    pub(super) armor_stopped: u64,
    pub(super) armor_deflected: u64,
    pub(super) armor_penetrated: u64,
    pub(super) anatomical_subregions_received: BTreeMap<String, u64>,
    pub(super) armor_layers_intersected: BTreeMap<String, u64>,
    pub(super) armor_layers_missed: BTreeMap<String, u64>,
    pub(super) attack_samples: u64,
    pub(super) contact_classifications: BTreeMap<String, u64>,
    pub(super) contact_measure_samples: u64,
    pub(super) mean_scheduled_contact_measure_metres: Measurement<ContactMetres>,
    pub(super) mean_actual_contact_measure_metres: Measurement<ContactMetres>,
    pub(super) minimum_actual_contact_measure_metres: Option<Measurement<ContactMetres>>,
    pub(super) maximum_actual_contact_measure_metres: Option<Measurement<ContactMetres>>,
    pub(super) minimum_actual_center_separation_metres: Option<Measurement<CenterSeparationMetres>>,
    pub(super) full_energy_intended_contacts_inside_ten_centimetres: u64,
    pub(super) mean_attack_interval_seconds: Measurement<AutoresolveSeconds>,
    pub(super) mean_attack_power_multiplier: Measurement<AttackPowerMultiplier>,
    pub(super) minimum_attack_incapacitation_performance: Option<Measurement<IncapacitationPerformance>>,
    pub(super) final_pain_incapacitation: Measurement<PainIncapacitation>,
    pub(super) final_blood_loss_fraction: Measurement<BloodLossFraction>,
    pub(super) final_acute_trauma: Measurement<AcuteTrauma>,
    pub(super) final_fatigue: Measurement<Fatigue>,
    pub(super) final_imbalance: Measurement<Imbalance>,
    pub(super) final_open_wounds: u64,
    pub(super) final_internal_wounds: u64,
    pub(super) incapacitated_runs: u64,
    pub(super) yielded_runs: u64,
    pub(super) terminal_causes: BTreeMap<String, u64>,
}

impl TacticalCausalSummary {
    pub(super) fn finish_means(&mut self) -> MeasurementResult<()> {
        self.john.finish_means()?;
        self.opponent.finish_means()?;
        Ok(())
    }
}

impl TacticalCombatantCausal {
    pub(super) fn finish_means(&mut self) -> MeasurementResult<()> {
        self.mean_attack_start_interval_seconds = self
            .mean_attack_start_interval_seconds
            .mean(self.attack_interval_samples)?;
        self.mean_contact_energy_joules = self
            .mean_contact_energy_joules
            .mean(self.contact_energy_samples)?;
        Ok(())
    }
}

impl AutoresolveCombatantCausal {
    pub(super) fn record_final(
        &mut self,
        outcome: &adventuresim_core::autoresolve::CombatantOutcome,
    ) -> MeasurementResult<()> {
        self.final_pain_incapacitation.checked_add(
            Measurement::<PainIncapacitation>::from_native(outcome.pain_incapacitation)?,
        )?;
        self.final_blood_loss_fraction.checked_add(
            Measurement::<BloodLossFraction>::from_native(outcome.blood_loss_fraction)?,
        )?;
        self.final_acute_trauma.checked_add(
            Measurement::<AcuteTrauma>::from_native(outcome.acute_trauma)?,
        )?;
        self.final_fatigue.checked_add(
            Measurement::<Fatigue>::from_native(outcome.fatigue)?,
        )?;
        self.final_imbalance.checked_add(
            Measurement::<Imbalance>::from_native(outcome.imbalance)?,
        )?;
        self.final_open_wounds += outcome.open_wound_count as u64;
        self.final_internal_wounds += outcome.internal_wound_count as u64;
        self.incapacitated_runs += u64::from(outcome.incapacitated);
        self.yielded_runs += u64::from(outcome.yielded);
        if let Some(cause) = outcome.terminal_cause {
            *self
                .terminal_causes
                .entry(format!("{cause:?}"))
                .or_default() += 1;
        }
        Ok(())
    }

    pub(super) fn finish_means(&mut self) -> MeasurementResult<()> {
        self.mean_attack_interval_seconds = self
            .mean_attack_interval_seconds
            .mean(self.attack_samples)?;
        self.mean_attack_power_multiplier = self
            .mean_attack_power_multiplier
            .mean(self.attack_samples)?;
        self.mean_scheduled_contact_measure_metres = self
            .mean_scheduled_contact_measure_metres
            .mean(self.contact_measure_samples)?;
        self.mean_actual_contact_measure_metres = self
            .mean_actual_contact_measure_metres
            .mean(self.contact_measure_samples)?;
        Ok(())
    }
}
