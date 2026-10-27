//! Reconcile generated patient chronology with private terminal outcomes.

use adventuresim_core::quest_generation::{OutbreakExposure, OutbreakPatientDeathKind};
use adventuresim_world_schema::calendar::StrategicMinute;

pub(super) fn resolve_patient_death(
    exposure: &OutbreakExposure,
    private_terminal: Option<StrategicMinute>,
    now: StrategicMinute,
) -> OutbreakExposure {
    let mut resolved = exposure.clone();
    match exposure.death_kind {
        Some(OutbreakPatientDeathKind::Disease) => {
            resolved.died_at = private_terminal;
            resolved.death_kind = private_terminal.map(|_| OutbreakPatientDeathKind::Disease);
        }
        Some(OutbreakPatientDeathKind::CarrierAttack) => {
            let latest_attack = private_terminal
                .map(|terminal| terminal.saturating_sub_minutes(1))
                .unwrap_or(now)
                .min(now);
            let attack_at = exposure
                .died_at
                .unwrap_or(latest_attack)
                .min(latest_attack)
                .max(exposure.became_symptomatic_at);
            if attack_at <= latest_attack {
                resolved.died_at = Some(attack_at);
            } else {
                resolved.died_at = None;
                resolved.death_kind = None;
            }
        }
        None => {}
    }
    resolved
}
