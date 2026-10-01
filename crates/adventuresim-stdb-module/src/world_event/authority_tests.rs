//! Guarded offense identity, legal settlement, and consequence replay checks.
use super::*;
use crate::character::character;
use crate::local_problem::{LocalProblemAuthority, local_problem_authority};
use crate::reputation::{settle_offenses, snapshot_arrest_charges, unsettled_arrest_charges};

#[spacetimedb::reducer]
pub fn authority_test_offense_policy(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let actor = 732083;
    crate::character::create_named_character_with_id(ctx, actor, "Offense Fixture".into())?;
    let now = crate::time::refresh_clock(ctx)?;
    for _ in 0..2 {
        commit_noticed_illegal_foraging(ctx, actor, "riverdale", "authority-offense", 100, now)?;
    }
    let offense_id = format!("offense:forage:{actor}:authority-offense");
    let offense = ctx
        .db
        .discovered_offense()
        .id()
        .find(&offense_id)
        .ok_or("Offense missing")?;
    let consequence = WorldEventConsequence::DiscoveredOffense {
        offense_id,
        character_id: actor,
        settlement_id: "riverdale".into(),
        kind: ExistingOffenseKind::IllegalForaging,
        severity: 1,
        minute: now,
    };
    preflight_consequences(ctx, std::slice::from_ref(&consequence))?;
    let mut conflicting = consequence.clone();
    if let WorldEventConsequence::DiscoveredOffense { severity, .. } = &mut conflicting {
        *severity = 2;
    }
    if preflight_consequences(ctx, &[conflicting]).is_ok()
        || commit_noticed_illegal_foraging(ctx, actor, "riverdale", "authority-offense", 200, now)
            .is_ok()
        || snapshot_arrest_charges(ctx, "incident:authority-offense", actor, "riverdale") != 1
    {
        return Err("Offense identity or charge snapshot disagrees".into());
    }
    let charges = unsettled_arrest_charges(ctx, "incident:authority-offense", actor, "riverdale");
    if adventuresim_core::reputation::authority_fine_for_charges(
        &charges
            .iter()
            .map(|row| (row.severity, row.settled))
            .collect::<Vec<_>>(),
    )
    .is_none()
    {
        return Err("Implemented offense lost its fine policy".into());
    }
    settle_offenses(ctx, charges);
    preflight_consequences(ctx, &[consequence])?;
    commit_noticed_illegal_foraging(ctx, actor, "riverdale", "authority-offense", 100, now)?;
    let stored = ctx
        .db
        .discovered_offense()
        .id()
        .find(&offense.id)
        .ok_or("Offense missing")?;
    if !stored.settled
        || stored.severity != offense.severity
        || !unsettled_arrest_charges(ctx, "incident:authority-offense", actor, "riverdale")
            .is_empty()
    {
        return Err("Offense replay overwrote downstream legal settlement".into());
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn authority_test_finale_world_effects(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let now = crate::time::refresh_clock(ctx)?;
    let actor = 732083;
    let party_id = ctx
        .db
        .character()
        .id()
        .find(actor)
        .and_then(|row| row.party_id)
        .ok_or("Party missing")?;
    let problem_id = "local-problem:authority-finales";
    ctx.db
        .local_problem_authority()
        .insert(LocalProblemAuthority {
            id: problem_id.into(),
            gateway_bucket: 0,
            scope_key: "authority-finales".into(),
            scope_json: "{}".into(),
            consequence_mechanism: "fixture".into(),
            symptom: "fixture".into(),
            buy_bps: 0,
            sell_penalty_bps: 0,
            encounter_frequency_bps: 0,
            encounter_archetype: None,
            disease_intensity: 0,
            disease_id: String::new(),
            starts_at: now,
            ends_at: now.saturating_add_minutes(60),
            mitigation_bps: 0,
            incident_count: 1,
            recurring_hostile: false,
            public_awareness_bps: 0,
            public_since_minute: None,
            resolved_at: None,
            opaque_case_ref: "case:authority-world-finales".into(),
        });
    for _ in 0..2 {
        commit_generated_case_resolution(
            ctx,
            "finale:authority-world-effects",
            "authority-world-source",
            "case:authority-world-finales",
            "case:public-authority-world-finales",
            &party_id,
            "riverdale",
            Some(problem_id),
            500,
            now,
        )?;
    }
    let problem = ctx
        .db
        .local_problem_authority()
        .id()
        .find(problem_id.to_owned())
        .ok_or("Problem missing")?;
    if problem.resolved_at != Some(now)
        || ctx
            .db
            .local_problem_outcome_receipt()
            .id()
            .find(format!("{problem_id}:authority-world-source"))
            .is_none()
        || ctx
            .db
            .reputation_event()
            .character_id()
            .filter(actor)
            .filter(|row| row.source_kind == "case_resolution")
            .count()
            != 1
        || commit_generated_case_resolution(
            ctx,
            "finale:authority-world-effects",
            "changed-source",
            "case:authority-world-finales",
            "case:public-authority-world-finales",
            &party_id,
            "riverdale",
            Some(problem_id),
            500,
            now,
        )
        .is_ok()
    {
        return Err(
            "Finale world consequences were missing, duplicated, or accepted a conflicting retry"
                .into(),
        );
    }
    Ok(())
}
