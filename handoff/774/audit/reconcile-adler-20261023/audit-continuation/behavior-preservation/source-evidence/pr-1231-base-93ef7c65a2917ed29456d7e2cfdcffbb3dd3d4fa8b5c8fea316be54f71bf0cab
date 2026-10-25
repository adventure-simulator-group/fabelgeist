//! Guarded bound-roster, launch, autoresolve, and finale receipt checks.
use super::super::*;
use crate::tactical::tactical_server_request_authority;

#[spacetimedb::reducer]
pub fn authority_test_mission_roster(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    require_dev_bootstrap_token(&bootstrap_token)?;
    let actor = 732073;
    let mut party = ctx
        .db
        .character()
        .id()
        .find(actor)
        .and_then(|row| row.party_id)
        .and_then(|id| ctx.db.party_authority().id().find(id.to_owned()))
        .ok_or("Challenge fixture party missing")?;
    let trial = ctx
        .db
        .challenge_authority()
        .party_id()
        .filter(&party.id)
        .next()
        .ok_or("Challenge fixture missing")?;
    let mut site = ctx
        .db
        .case_site_authority()
        .id_key()
        .find(trial.finale_case_site_id.to_string())
        .ok_or("Finale site missing")?;
    site.coordinates_are_geographic = true;
    ctx.db.case_site_authority().id_key().update(site.clone());
    party.current_case_site_id = Some(site.id.clone());
    party.camp_destination = None;
    party.wilderness_canonical_anchor_minute = Some(crate::time::refresh_clock(ctx)?);
    ctx.db.party_authority().id().update(party.clone());
    crate::investigation::set_character_case_site(ctx, actor, Some(site.id.to_string()))?;
    let id = "mission:authority-roster-normal";
    let mission =
        ensure_bound_mission_authority(ctx, id, &party.id, actor, &site, &site.scene_key)?;
    let roster = mission
        .enemy_roster(ctx)
        .map_err(|error| error.to_string())?;
    let mut group = ctx
        .db
        .hostile_group_authority()
        .id()
        .find(
            mission
                .hostile_group_id
                .clone()
                .ok_or("Group binding missing")?,
        )
        .ok_or("Group missing")?;
    group.enemy_count += 1;
    ctx.db.hostile_group_authority().id().update(group.clone());
    if group.capture_enemy_roster(ctx).is_ok()
        || mission
            .enemy_roster(ctx)
            .map_err(|error| error.to_string())?
            .enemy_count()
            != roster.enemy_count()
    {
        return Err("Live planning drift changed the captured roster".into());
    }
    group.enemy_count -= 1;
    ctx.db.hostile_group_authority().id().update(group);
    for invalid in [vec![], vec![roster.enemy_ids()[0]; 2], vec![u64::MAX]] {
        let mut damaged = mission.clone();
        damaged.enemy_character_ids = invalid;
        ctx.db.mission_authority().id().update(damaged);
        if crate::tactical::request_tactical_server(ctx, actor, id.into(), site.scene_key.clone())
            .is_ok()
            || autoresolve_mission(ctx, actor, id.into()).is_ok()
            || ctx
                .db
                .tactical_server_request_authority()
                .mission_id()
                .find(id.to_owned())
                .is_some()
            || ctx
                .db
                .autoresolve_report()
                .battle_id()
                .find(format!("battle:{id}"))
                .is_some()
        {
            return Err("Invalid enemy snapshot launched or resolved".into());
        }
    }
    ctx.db.mission_authority().id().update(mission.clone());
    crate::tactical::request_tactical_server(ctx, actor, id.into(), site.scene_key.clone())?;
    let request = ctx
        .db
        .tactical_server_request_authority()
        .mission_id()
        .find(id.to_owned())
        .ok_or("Tactical projection missing")?;
    if request.enemy_character_ids != mission.enemy_character_ids
        || request.required_enemy_kills != roster.enemy_count().get()
    {
        return Err("Tactical launch disagrees with captured identities".into());
    }
    ctx.db
        .tactical_server_request_authority()
        .mission_id()
        .delete(id.to_owned());
    autoresolve_mission(ctx, actor, id.into())?;
    if ctx
        .db
        .autoresolve_report()
        .battle_id()
        .find(format!("battle:{id}"))
        .is_none()
    {
        return Err("Validated roster did not produce an autoresolve report".into());
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn authority_test_standalone_roster(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    require_dev_bootstrap_token(&bootstrap_token)?;
    let id = "mission:diagnostic-authority-roster";
    let mission = ctx
        .db
        .mission_authority()
        .id()
        .find(id.to_owned())
        .ok_or("Standalone binding missing")?;
    let roster = mission
        .enemy_roster(ctx)
        .map_err(|error| error.to_string())?;
    let request = ctx
        .db
        .tactical_server_request_authority()
        .mission_id()
        .find(id.to_owned())
        .ok_or("Standalone request missing")?;
    if roster.enemy_count().get() != request.required_enemy_kills
        || roster.enemy_ids() != request.enemy_character_ids
    {
        return Err("Standalone launch disagrees with captured identities".into());
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn authority_test_finale_receipts(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    require_dev_bootstrap_token(&bootstrap_token)?;
    let case_id = "case:authority-finales";
    let party_id = "party:authority-finales";
    ctx.db.case_authority().insert(CaseAuthority {
        id: case_id.into(),
        provenance_kind: InvestigationProvenanceKind::Manual,
        generated_case_id: String::new(),
        investigation_case_id: "investigation:authority-finales".into(),
        local_problem_id: None,
        objective_expression_json: "{\"alternatives\":[]}".into(),
        resolution_status: CaseStatus::Resolved,
        resolved_by_party_id: Some(party_id.into()),
    });
    if select_case_finale(ctx, case_id, CaseStatus::Resolved, None)?.is_some() {
        return Err("Absent finale unexpectedly selected".into());
    }
    for (id, status) in [
        ("finale:authority-selected", CaseStatus::Resolved),
        ("finale:authority-ineligible", CaseStatus::Failed),
    ] {
        ctx.db.case_finale_authority().insert(CaseFinaleAuthority {
            id: id.into(),
            case_id: case_id.into(),
            kind: FinaleExecutionKind::RecordResolution,
            resolution_status: status,
            eligible_path_index: None,
            priority: 0,
            status: FinaleStatus::Available,
        });
    }
    let id = select_case_finale(ctx, case_id, CaseStatus::Resolved, None)?
        .ok_or("Finale was not selected")?;
    ctx.db.case_outcome().insert(CaseOutcome {
        case_id: case_id.into(),
        party_id: party_id.into(),
        status: CaseStatus::Resolved,
        winning_path_index: None,
        resolved_at_minute: crate::time::refresh_clock(ctx)?,
        selected_finale_id: id.clone(),
    });
    for _ in 0..2 {
        execute_case_finale(ctx, &id, "finale:authority-source", party_id, None)?;
    }
    let receipt = ctx
        .db
        .case_finale_execution()
        .finale_id()
        .find(&id)
        .ok_or("Finale receipt missing")?;
    if receipt.case_id != case_id
        || receipt.party_id != party_id
        || ctx
            .db
            .case_finale_authority()
            .id()
            .find(&id)
            .ok_or("Finale missing")?
            .status
            != FinaleStatus::Executed
        || ctx
            .db
            .case_finale_authority()
            .id()
            .find("finale:authority-ineligible".to_owned())
            .ok_or("Alternate missing")?
            .status
            != FinaleStatus::Ineligible
        || execute_case_finale(ctx, &id, "different-source", party_id, None).is_ok()
        || execute_case_finale(ctx, &id, "finale:authority-source", "different-party", None).is_ok()
    {
        return Err("Finale execution, selection, or retry ownership disagrees".into());
    }
    Ok(())
}
