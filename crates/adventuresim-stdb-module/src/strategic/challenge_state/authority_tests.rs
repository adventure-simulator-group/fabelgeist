//! Guarded puzzle and road-choice lifecycle acceptance checks.
use super::super::*;

fn puzzle_lifecycle(ctx: &ReducerContext, character_id: u64) -> Result<(), String> {
    let issuance = materialize_order_errantry(
        ctx,
        character_id,
        None,
        ErrantryLaunch::DirectDemoCamp(PuzzleKind::OrderedSigils),
    )?;
    let initial = ctx
        .db
        .challenge_authority()
        .case_id()
        .filter(&issuance.case_id)
        .next()
        .ok_or("Puzzle missing")?;
    let puzzle: adventuresim_puzzles::PuzzleAuthority =
        serde_json::from_str(&initial.puzzle_json).map_err(|error| error.to_string())?;
    let adventuresim_puzzles::PuzzleAuthority::OrderedSigils(puzzle) = puzzle else {
        return Err("Unexpected puzzle kind".into());
    };
    let correct = serde_json::to_string(&adventuresim_puzzles::PuzzleSubmission::OrderedSigils {
        ordering: puzzle.solution,
    })
    .map_err(|error| error.to_string())?;
    let mut wrong_order = puzzle.solution;
    wrong_order.swap(0, 1);
    let wrong = serde_json::to_string(&adventuresim_puzzles::PuzzleSubmission::OrderedSigils {
        ordering: wrong_order,
    })
    .map_err(|error| error.to_string())?;
    if !initial.is_open() || initial.revision != 0 {
        return Err("New puzzle is not open".into());
    }
    for _ in 0..2 {
        submit_puzzle_challenge(
            ctx,
            character_id,
            issuance.case_id.clone(),
            initial.id.clone(),
            0,
            wrong.clone(),
        )?;
    }
    let attempted = ctx
        .db
        .challenge_authority()
        .id()
        .find(&initial.id)
        .ok_or("Puzzle missing")?;
    if !attempted.is_open() || attempted.revision != 1 {
        return Err("Wrong answer or exact retry changed puzzle completion".into());
    }
    for _ in 0..2 {
        submit_puzzle_challenge(
            ctx,
            character_id,
            issuance.case_id.clone(),
            initial.id.clone(),
            1,
            correct.clone(),
        )?;
    }
    let solved = ctx
        .db
        .challenge_authority()
        .id()
        .find(&initial.id)
        .ok_or("Puzzle missing")?;
    if solved.is_open()
        || solved.revision != 2
        || solved.journey_departure_minute != initial.journey_departure_minute
        || solved.camp_movement_minute != initial.camp_movement_minute
        || ctx
            .db
            .challenge_attempt_receipt()
            .challenge_id()
            .filter(&initial.id)
            .count()
            != 2
        || submit_puzzle_challenge(
            ctx,
            character_id,
            issuance.case_id.clone(),
            initial.id.clone(),
            1,
            wrong,
        )
        .is_ok()
        || submit_puzzle_challenge(ctx, character_id, issuance.case_id, initial.id, 2, correct)
            .is_ok()
    {
        return Err("Solve, camp binding, closure, or conflicting retry disagrees".into());
    }
    Ok(())
}

fn road_lifecycle(ctx: &ReducerContext, character_id: u64) -> Result<(), String> {
    let id =
        materialize_development_road_encounter(ctx, character_id, "unlawful_bridge_custom_v1")?;
    let initial = ctx
        .db
        .road_challenge_authority()
        .id()
        .find(&id)
        .ok_or("Road trial missing")?;
    if !initial.is_open() {
        return Err("New road trial is not open".into());
    }
    for _ in 0..2 {
        resolve_errantry_road_challenge(
            ctx,
            character_id,
            id.clone(),
            0,
            "challenge_to_arms".into(),
            "combat-choice".into(),
        )?;
    }
    let closed = ctx
        .db
        .road_challenge_authority()
        .id()
        .find(&id)
        .ok_or("Road trial missing")?;
    if closed.is_open()
        || closed.revision != 1
        || closed.resolved_choice.as_deref() != Some("challenge_to_arms")
        || resolve_errantry_road_challenge(
            ctx,
            character_id,
            id.clone(),
            0,
            "ignore".into(),
            "combat-choice".into(),
        )
        .is_ok()
    {
        return Err("Road choice closure or conflicting retry disagrees".into());
    }
    let mut encounter = ctx
        .db
        .strategic_encounter()
        .party_id()
        .find(&closed.party_id)
        .ok_or("Combat transition missing")?;
    // Inject only the strategic outcome of transient combat, never tactical tick state.
    encounter.status = StrategicEncounterStatus::Resolved;
    encounter.outcome = Some("victory".into());
    ctx.db
        .strategic_encounter()
        .party_id()
        .update(encounter.clone());
    resolve_narrative_combat_followup(ctx, &encounter)?;
    let completed = ctx
        .db
        .road_challenge_authority()
        .id()
        .find(&id)
        .ok_or("Road trial missing")?;
    resolve_narrative_combat_followup(ctx, &encounter)?;
    let replayed = ctx
        .db
        .road_challenge_authority()
        .id()
        .find(&id)
        .ok_or("Road trial missing")?;
    if completed.is_open()
        || completed.resolved_choice != closed.resolved_choice
        || completed.result_transcript == closed.result_transcript
        || replayed.result_transcript != completed.result_transcript
        || replayed.revision != completed.revision
    {
        return Err("Delayed combat result reopened or duplicated road resolution".into());
    }
    encounter.outcome = Some("defeat".into());
    if resolve_narrative_combat_followup(ctx, &encounter).is_ok() {
        return Err("Conflicting combat retry was accepted".into());
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn authority_test_challenge_lifecycle(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    require_dev_bootstrap_token(&bootstrap_token)?;
    let character_id = 732073;
    crate::character::create_named_character_with_id(
        ctx,
        character_id,
        "Challenge Fixture".into(),
    )?;
    puzzle_lifecycle(ctx, character_id)?;
    road_lifecycle(ctx, character_id)
}
