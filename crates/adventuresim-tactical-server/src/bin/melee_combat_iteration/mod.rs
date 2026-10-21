//! Accelerated, deterministic headless melee combat iteration runner.

use fabelgeist_determinism::Seed;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use adventuresim_core::{
    autoresolve::{
        BattleAttackKind, BattleAttackOutcome, BattleOpening, BattleResolution,
        MeleeIterationBuild, MeleeResponseChoice, MeleeTimelineKind,
        melee_iteration_acceptance_evidence, melee_iteration_roster, resolve_battle,
    },
    combat::{MeleeContactClassification, MeleeContactInvalidationCause},
    item_catalog_schema::EquipmentMaterial,
    skill::{PlayerSkills, Skill},
};
use adventuresim_tactical_server::iteration::{
    TacticalContactOutcome, TacticalCoverageContact, TacticalDecision, TacticalDecisionStatus,
    TacticalDefenseImplement, TacticalDuelResolution, TacticalElapsed, TacticalMeleeOutcome,
    resolve_tactical_server_melee_duel,
};
use clap::Parser;
pub(crate) use error::IterationResult;
use error::{IterationError, MovementObservation, ReportContractError};
use measurement::*;
use report::*;
use reviewer::*;
use serde::Serialize;

mod audit;
mod causal;
mod error;
mod measurement;
mod report;
mod reviewer;
mod runner;

#[derive(Parser, Debug)]
#[command(name = "melee-combat-iteration")]
#[command(about = "Run deterministic tactical-time and autoresolve melee duel batches")]
struct Args {
    #[arg(long, default_value = "target/melee-combat-iteration")]
    output: PathBuf,
    #[arg(long, default_value_t = 32)]
    tactical_seeds: u64,
    #[arg(long, default_value_t = 1_000)]
    autoresolve_seeds: u64,
    #[arg(long, default_value_t = fabelgeist_determinism::Seed::from_u64(1))]
    first_seed: Seed,
}

#[derive(Serialize)]
struct AcceptanceAudit {
    no_tactical_timeouts: bool,
    no_autoresolve_timeouts: bool,
    tactical_energy_conservation_holds: bool,
    side_swap_nonterminal_timeline_equal: bool,
    simultaneous_contacts_preserved: bool,
    canceled_attacks_emit_no_ghost_contacts: bool,
    autoresolve_contacts_respect_scheduled_windup: bool,
    autoresolve_movement_elapsed_matches_distance_delta: bool,
    autoresolve_movement_respects_tick_and_speed_limits: bool,
    polearm_contact_revalidation_holds: bool,
    all_weapon_swept_contact_contract_holds: bool,
    matchups: Vec<MatchupCausalAudit>,
}

#[derive(Serialize)]
struct MatchupCausalAudit {
    opponent: String,
    tactical_first_contacts: [u64; 2],
    autoresolve_first_contacts: [u64; 2],
    autoresolve_attack_starts: [u64; 2],
    autoresolve_resolved_attacks: [u64; 2],
    autoresolve_cancellations: [u64; 2],
    autoresolve_response_events: [u64; 2],
    response_event_for_every_incoming_contact: bool,
}

pub(crate) fn run() -> IterationResult<()> {
    let args = Args::parse();
    if args.tactical_seeds == 0 || args.autoresolve_seeds == 0 {
        return Err(IterationError::InvalidSeedCounts {
            tactical: args.tactical_seeds,
            autoresolve: args.autoresolve_seeds,
        });
    }
    fs::create_dir_all(&args.output)?;
    let acceptance_evidence = melee_iteration_acceptance_evidence()?;
    write_json(
        &args.output.join("acceptance-evidence.json"),
        &acceptance_evidence,
    )?;
    let (john, opponents) = melee_iteration_roster()?;
    let mut summaries = Vec::new();
    for opponent in &opponents {
        summaries.push(run_matchup(&args, &john, opponent)?);
    }
    write_json(&args.output.join("summary.json"), &summaries)?;
    write_json(
        &args.output.join("acceptance-audit.json"),
        &acceptance_audit(&summaries, &acceptance_evidence)?,
    )?;
    write_json(
        &args.output.join("reviewer-index.json"),
        &ReviewerIndex {
            gate: "bounded causal mechanics review before scale acceptance",
            tactical_seeds_per_matchup: args.tactical_seeds,
            autoresolve_seeds_per_matchup: args.autoresolve_seeds,
            summary_file: "summary.json",
            acceptance_evidence_file: "acceptance-evidence.json",
            acceptance_audit_file: "acceptance-audit.json",
            matchup_directories: opponents.iter().map(|build| build.key).collect(),
        },
    )?;
    println!("{}", serde_json::to_string_pretty(&summaries)?);
    Ok(())
}

fn acceptance_audit(
    summaries: &[MatchupSummary],
    evidence: &adventuresim_core::autoresolve::MeleeIterationAcceptanceEvidence,
) -> MeasurementResult<AcceptanceAudit> {
    audit::build_acceptance_audit(summaries, evidence)
}

fn run_matchup(
    args: &Args,
    john: &MeleeIterationBuild,
    opponent: &MeleeIterationBuild,
) -> IterationResult<MatchupSummary> {
    let directory = args.output.join(opponent.key);
    fs::create_dir_all(&directory)?;
    let tactical = runner::run_tactical_batch(args, john, opponent, &directory)?;

    let autoresolve = runner::run_autoresolve_batch(args, john, opponent, &directory)?;
    let tactical_win_rate =
        Measurement::<WinRate>::from_counts(tactical.john_wins, args.tactical_seeds)?;
    let autoresolve_win_rate =
        Measurement::<WinRate>::from_counts(autoresolve.john_wins, args.autoresolve_seeds)?;
    let summary = MatchupSummary {
        matchup: format!("{} vs {}", john.name, opponent.name),
        opponent: opponent.key.to_owned(),
        tactical_runs: args.tactical_seeds,
        tactical_john_wins: tactical.john_wins,
        tactical_opponent_wins: tactical.opponent_wins,
        tactical_mutual_incapacitations: tactical.mutual_incapacitations,
        tactical_timeouts: tactical.timeouts,
        tactical_causal: tactical.causal,
        tactical_total_simulated_seconds: tactical.simulated_seconds,
        tactical_mean_simulated_seconds: tactical.simulated_seconds.mean(args.tactical_seeds)?,
        autoresolve_runs: args.autoresolve_seeds,
        autoresolve_john_wins: autoresolve.john_wins,
        autoresolve_opponent_wins: autoresolve.opponent_wins,
        autoresolve_mutual_incapacitations: autoresolve.mutual_incapacitations,
        autoresolve_timeouts: autoresolve.timeouts,
        autoresolve_causal: autoresolve.causal,
        john_win_rate_difference: tactical_win_rate.difference(autoresolve_win_rate)?,
        tactical_wall_milliseconds: tactical.wall_seconds.milliseconds()?,
        autoresolve_wall_milliseconds: autoresolve.wall_seconds.milliseconds()?,
        tactical_duels_per_second: tactical.wall_seconds.duels_per_second(args.tactical_seeds)?,
        tactical_simulated_seconds_per_wall_second: tactical
            .simulated_seconds
            .per_wall_second(tactical.wall_seconds)?,
        autoresolve_duels_per_second: autoresolve
            .wall_seconds
            .duels_per_second(args.autoresolve_seeds)?,
    };
    write_json(&directory.join("summary.json"), &summary)?;
    let packet = ReviewerPacket {
        balance_concept: "Physically based gameplay: equipment, anatomy, force, skill, positioning, pain, blood loss, imbalance, and exertion should produce plausible causal outcomes rather than symmetric game abstractions.",
        scale_context: "Attributes and skill ranks use a 0-5 practical scale. About 3 describes a healthy adult or competent practitioner; 4 is exceptional or expert; 5 approaches Olympic or world-class human performance.",
        matchup: summary.matchup.clone(),
        combatants: [reviewer_combatant(john), reviewer_combatant(opponent)],
        tactical_trace_file: "tactical-trace.json",
        all_tactical_traces_file: "tactical-traces.ndjson",
        autoresolve_trace_file: "autoresolve-trace.json",
        all_autoresolve_traces_file: "autoresolve-traces.ndjson",
        aggregate_summary_file: "summary.json",
        acceptance_evidence_file: "../acceptance-evidence.json",
        acceptance_audit_file: "../acceptance-audit.json",
    };
    write_json(&directory.join("reviewer-packet.json"), &packet)?;
    Ok(summary)
}

fn record_autoresolve_combatant_causal(
    attacker: &mut AutoresolveCombatantCausal,
    defender: &mut AutoresolveCombatantCausal,
    entry: &adventuresim_core::autoresolve::BattleLogEntry,
) -> IterationResult<()> {
    attacker.attacks += 1;
    attacker.hits_dealt += u64::from(matches!(
        entry.outcome,
        BattleAttackOutcome::HitHealth | BattleAttackOutcome::HitArmor
    ));
    defender.attacks_received += 1;
    match entry.defender_response {
        MeleeResponseChoice::Block => defender.block_attempts += 1,
        MeleeResponseChoice::Parry => defender.parry_attempts += 1,
        MeleeResponseChoice::Dodge => defender.dodge_attempts += 1,
        _ => {}
    }
    if entry.defender_response == MeleeResponseChoice::Dodge {
        let contacted = entry.outcome != BattleAttackOutcome::Missed;
        defender.dodge_avoids += u64::from(!contacted);
        defender.dodge_contacts += u64::from(contacted);
    }
    if matches!(
        entry.defender_response,
        MeleeResponseChoice::Block | MeleeResponseChoice::Parry
    ) && entry.outcome == BattleAttackOutcome::Blocked
    {
        defender.effective_weapon_contacts += 1;
    }
    if let Some(telemetry) = &entry.melee_telemetry {
        causal::record_autoresolve_telemetry(attacker, defender, telemetry)?;
    }
    if let Some(impact) = entry.armor_impact {
        defender.armor_surface_contacts_received += 1;
        use adventuresim_core::combat::ArmorImpactOutcome;
        match impact.outcome {
            ArmorImpactOutcome::Stopped => defender.armor_stopped += 1,
            ArmorImpactOutcome::Deflected => defender.armor_deflected += 1,
            ArmorImpactOutcome::Penetrated => defender.armor_penetrated += 1,
        }
    } else if matches!(
        entry.outcome,
        BattleAttackOutcome::HitHealth | BattleAttackOutcome::HitArmor
    ) {
        defender.armor_gap_contacts_received += 1;
    }
    Ok(())
}

fn record_tactical_causal(
    summary: &mut TacticalCausalSummary,
    outcome: &TacticalMeleeOutcome,
    john_name: &str,
) -> IterationResult<()> {
    causal::record_tactical_first_contact(summary, outcome, john_name);
    causal::record_tactical_decisions(summary, outcome, john_name)?;
    for event in &outcome.events {
        causal::record_tactical_event(summary, event, john_name)?;
    }
    causal::record_tactical_wounds(summary, outcome, john_name);
    Ok(())
}

fn write_json(path: &Path, value: &impl Serialize) -> IterationResult<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).map_err(|source| IterationError::WriteReport {
        path: path.to_owned(),
        source,
    })
}
