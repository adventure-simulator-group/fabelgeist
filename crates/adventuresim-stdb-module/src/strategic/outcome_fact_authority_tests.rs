//! Guarded payload chronology, index attribution, and outcome replay checks.
use super::*;
use adventuresim_core::case::{
    Objective, ObjectiveExpression, ObjectiveId, ObjectivePath, ObjectiveRequirement, OutcomeFact,
    OutcomeFactKind,
};

#[spacetimedb::reducer]
pub fn authority_test_outcome_fact_payload(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    require_dev_bootstrap_token(&bootstrap_token)?;
    let case_id = "case:authority-fact-payload";
    let party_id = "party:authority-fact-payload";
    let now = crate::time::refresh_clock(ctx)
        .map_err(|error: crate::time::WorldClockError| error.to_string())?;
    let deadline = now.saturating_add_minutes(10);
    let expression = ObjectiveExpression {
        alternatives: vec![ObjectivePath {
            objectives: vec![Objective {
                id: ObjectiveId::new("objective:authority-window").map_err(|e| format!("{e:?}"))?,
                requirement: ObjectiveRequirement::SurviveWindow {
                    site_id: "site:authority-window".into(),
                    through_minute: deadline,
                },
            }],
        }],
    };
    ctx.db.case_authority().insert(CaseAuthority {
        id: case_id.into(),
        provenance_kind: InvestigationProvenanceKind::Manual,
        generated_case_id: String::new(),
        investigation_case_id: "investigation:authority-fact-payload".into(),
        local_problem_id: None,
        objective_expression_json: serde_json::to_string(&expression).map_err(|e| e.to_string())?,
        resolution_status: CaseStatus::Open,
        resolved_by_party_id: None,
    });
    let early = OutcomeFactKind::WindowSurvived {
        site_id: "site:authority-window".into(),
        through_minute: deadline.saturating_sub_minutes(1),
    };
    for _ in 0..2 {
        ingest_case_outcome_fact(
            ctx,
            "authority:early-window",
            case_id,
            party_id,
            early.clone(),
        )?;
    }
    let row = ctx
        .db
        .case_outcome_fact()
        .source_id()
        .find("authority:early-window".to_owned())
        .ok_or("Fact missing")?;
    let payload: OutcomeFact = serde_json::from_str(&row.fact_json).map_err(|e| e.to_string())?;
    if row.id != payload.id.as_str()
        || row.case_id != payload.case_id.as_str()
        || row.party_id != payload.party_id
        || row.source_id != payload.source_id
        || payload.happened_at != now
        || payload.kind != early
        || ctx
            .db
            .case_authority()
            .id()
            .find(case_id.to_owned())
            .ok_or("Case missing")?
            .resolution_status
            != CaseStatus::Open
    {
        return Err("Fact index attribution, payload chronology, or early window disagrees".into());
    }
    let complete = OutcomeFactKind::WindowSurvived {
        site_id: "site:authority-window".into(),
        through_minute: deadline,
    };
    if ingest_case_outcome_fact(
        ctx,
        "authority:early-window",
        case_id,
        party_id,
        complete.clone(),
    )
    .is_ok()
        || ingest_case_outcome_fact(
            ctx,
            "authority:early-window",
            case_id,
            "different-party",
            early,
        )
        .is_ok()
    {
        return Err("Conflicting source retry accepted".into());
    }
    for _ in 0..2 {
        ingest_case_outcome_fact(
            ctx,
            "authority:complete-window",
            case_id,
            party_id,
            complete.clone(),
        )?;
    }
    if ctx
        .db
        .case_outcome_fact()
        .case_id()
        .filter(&case_id.to_owned())
        .count()
        != 2
        || ctx
            .db
            .case_outcome()
            .case_id()
            .find(case_id.to_owned())
            .ok_or("Outcome missing")?
            .status
            != CaseStatus::Resolved
    {
        return Err("Payload window evaluation or exact terminal replay disagrees".into());
    }
    Ok(())
}
