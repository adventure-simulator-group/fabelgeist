//! Authorize, revalidate and commit an investigation attempt in the native transaction.

use super::*;

pub(crate) fn perform_investigation_action_authorized(
    ctx: &ReducerContext,
    actor_id: u64,
    action_id: String,
    method: String,
    expected_version: u32,
    leader_approved: bool,
) -> Result<(), InvestigationExecutionError> {
    require_strategic_character_authority(ctx, (actor_id).into())?;
    let attempt_id = inv::compound_id(&[
        "attempt",
        &action_id,
        &actor_id.to_string(),
        &expected_version.to_string(),
    ]);
    if let Some(attempt) = ctx.db.investigation_action_attempt().id().find(&attempt_id) {
        return if attempt.owner_character_id == actor_id
            && attempt.capability_id == action_id
            && attempt.method == method
            && attempt.expected_version == expected_version
        {
            Ok(())
        } else {
            Err(InvestigationExecutionError::AttemptConflict)
        };
    }
    let actor = crate::character::require_living_character(ctx, (actor_id).into())?;
    let party_id = actor
        .party_id
        .clone()
        .ok_or(InvestigationExecutionError::PartyRequired)?;
    let party = ctx
        .db
        .party_authority()
        .id()
        .find(&party_id)
        .ok_or(InvestigationExecutionError::PartyMissing)?;
    let capability_row = ctx
        .db
        .investigation_action_capability()
        .id()
        .find(&action_id);
    let rights_place = capability_row
        .as_ref()
        .filter(|capability| {
            capability.owner_character_id == actor_id
                && capability.target_kind == action::InvestigationTargetKind::Site
                && capability.method == InvestigationActionKind::InspectSite.stable_id()
        })
        .and_then(|capability| exact_action_case_site_for_observer(ctx, capability))
        .map(|matched| matched.site.id.to_place());
    let rights_question = action::investigation_rights_question(
        adventuresim_core::physical_object::CustodyCharacterId::try_from(
            adventuresim_core::identity::CharacterId::from(actor_id),
        )
        .map_err(InvestigationExecutionError::ActorIdentity)?,
        rights_place,
    )
    .map_err(InvestigationExecutionError::RightsQuestion)?;
    let rights = action::decide_investigation_rights(
        &rights_question,
        party.leader_id == actor_id,
        leader_approved,
        expected_version,
    );
    if rights.kind() != adventuresim_core::rights::RightsDecisionKind::Allowed {
        return Err(InvestigationExecutionError::LeaderApprovalRequired);
    }
    let mut capability = capability_row.ok_or(InvestigationExecutionError::ActionUnavailable)?;
    if capability.owner_character_id != actor_id
        || !capability.active
        || capability.method != method
        || capability.version != expected_version
    {
        return Err(InvestigationExecutionError::ActionStale);
    }
    if reissue_stale_custody_capability(ctx, &mut capability, &party_id)? {
        return Ok(());
    }
    let route_admission::ValidatedActionRoute {
        kind,
        target_terrain,
    } = route_admission::ValidatedActionRoute::from_capability(ctx, &capability)?;
    let admitted_party_id = adventuresim_core::identity::PartyId::try_new(party_id.clone())
        .map_err(InvestigationAdmissionError::PartyIdentity)?;
    let members =
        validate_live_action_prerequisites(ctx, &actor, &admitted_party_id, &capability, kind)?;
    let Some(started_at) =
        synchronize_party_activity_time(ctx, &members, (party.leader_id).into())?
    else {
        return Ok(());
    };
    validate_generated_pattern_condition(ctx, &capability, kind, started_at)?;
    let mut route_skills = party_action_skills(ctx, &party_id, (actor_id).into(), target_terrain)?;
    if let Some(investigability) = generated_investigability(ctx, &capability) {
        route_skills = apply_investigability_to_route_skills(route_skills, investigability);
    }
    let resolution_input = action::ResolutionInput {
        seed: capability.seed,
        attempt_index: expected_version,
        kind,
        terrain: actor_action_terrain(ctx, &actor),
        target_terrain,
        time_of_day: if (360..1_200).contains(&started_at.minute_of_day()) {
            action::TimeOfDay::Day
        } else {
            action::TimeOfDay::Night
        },
        evidence_age_minutes: started_at.elapsed_since(capability.evidence_age_origin_minute),
        current_uncertainty_bps: capability.uncertainty_bps,
        skills: route_skills,
        weather: actor_action_weather(ctx, &actor, started_at),
    };
    let bounded_progress =
        capability_uses_bounded_progress(capability.provenance_kind, kind).then(|| {
            let prior_failures = contiguous_failed_attempts(
                &capability.id,
                capability.owner_character_id,
                &capability.method,
                capability.version,
                ctx.db
                    .investigation_action_attempt()
                    .capability_id()
                    .filter(&capability.id),
            );
            action::resolve_with_bounded_progress(resolution_input, prior_failures)
        });
    let resolution = bounded_progress
        .map(|progress| progress.resolution)
        .unwrap_or_else(|| action::resolve(resolution_input));
    let planned_site_action = site_bound_investigation_plan(
        ctx,
        (actor_id).into(),
        party.leader_id,
        &rights,
        &rights_question,
        &capability,
        &attempt_id,
        started_at,
        &members,
        resolution,
        resolution_input,
    )?;
    // This is the final mutation-boundary validation. Browser previews and
    // party votes are UX; only this transaction authorizes the shared time.
    validate_live_action_prerequisites(ctx, &actor, &admitted_party_id, &capability, kind)?;
    validate_generated_pattern_condition(ctx, &capability, kind, started_at)?;
    let planned_site_action = match planned_site_action {
        Some(adventuresim_core::strategic_action::PlanningOutcome::Ready(planned)) => {
            let replanned = site_bound_investigation_plan(
                ctx,
                (actor_id).into(),
                party.leader_id,
                &rights,
                &rights_question,
                &capability,
                &attempt_id,
                started_at,
                &members,
                resolution,
                resolution_input,
            )?
            .ok_or(InvestigationExecutionError::SiteChanged)?;
            let current_snapshot = match &replanned {
                adventuresim_core::strategic_action::PlanningOutcome::Ready(plan) => {
                    plan.snapshot()
                }
                adventuresim_core::strategic_action::PlanningOutcome::Rejected(_) => {
                    return Err(InvestigationExecutionError::PrerequisitesChanged);
                }
            };
            let provenance = planned.provenance();
            adventuresim_core::strategic_action::validate_commit(
                &planned,
                &replanned,
                current_snapshot,
                &adventuresim_core::strategic_action::CommitAttempt {
                    request_id: provenance.request_id.clone(),
                    action_id: provenance.action_id.clone(),
                    authority_binding: provenance.authority_binding,
                },
                None,
            )
            .map_err(InvestigationExecutionError::Commit)?;
            Some(planned)
        }
        Some(adventuresim_core::strategic_action::PlanningOutcome::Rejected(_)) => {
            return Err(InvestigationExecutionError::ActionUnavailable);
        }
        None => None,
    };

    let (effect_members, effect_minutes, permits_resolution) =
        if let Some(plan) = &planned_site_action {
            let mut interval = None;
            let mut commit = None;
            for effect in plan.effects() {
                match effect {
                    adventuresim_core::strategic_action::ActionEffect::Domain(
                        action::InvestigationPlanEffect::AttemptPartyInterval {
                            member_ids,
                            requested_minutes,
                        },
                    ) => interval = Some((member_ids, *requested_minutes)),
                    adventuresim_core::strategic_action::ActionEffect::Domain(
                        action::InvestigationPlanEffect::CommitResolution(value),
                    ) => commit = Some(*value),
                    _ => return Err(InvestigationExecutionError::UnsupportedEffect),
                }
            }
            let (member_ids, requested_minutes) =
                interval.ok_or(InvestigationExecutionError::IntervalMissing)?;
            let effect_members = member_ids
                .iter()
                .map(|id| adventuresim_core::identity::CharacterId::from(id.get()))
                .collect::<Vec<_>>();
            if effect_members != members
                || requested_minutes != u64::from(resolution.cost.minutes)
                || commit.is_some_and(|value| value != resolution)
            {
                return Err(InvestigationExecutionError::EffectsChanged);
            }
            (effect_members, requested_minutes, commit.is_some())
        } else {
            (members.clone(), u64::from(resolution.cost.minutes), true)
        };
    let mut interval_completed = true;
    for member_id in &effect_members {
        interval_completed &=
            advance_investigation_time(ctx, u64::from(*member_id), effect_minutes)?;
    }
    if !interval_completed {
        // Time, exposure, and death are already authoritative writes. Never
        // roll them back merely because the planned action interval clipped.
        let _ = crate::strategic::normalize_and_elect_party_leader(ctx, &party_id);
        let _ = crate::strategic::reconcile_party_objective_continuity(ctx, &party_id);
        let completed_at = effect_members
            .iter()
            .filter_map(|member_id| {
                ctx.db
                    .character_time()
                    .character_id()
                    .find(u64::from(*member_id))
                    .map(|time| time.minutes)
            })
            .max()
            .unwrap_or(started_at);
        ctx.db
            .investigation_action_attempt()
            .insert(InvestigationActionAttempt {
                id: attempt_id.clone(),
                capability_id: action_id.clone(),
                owner_character_id: actor_id,
                expected_version,
                method: method.clone(),
                started_at,
                completed_at,
                duration_minutes: completed_at
                    .elapsed_since(started_at)
                    .min(u64::from(u32::MAX)) as u32,
                success: false,
                resulting_uncertainty_bps: capability.uncertainty_bps,
                private_resolution_json: private_interrupted_action_resolution_json(
                    effect_minutes,
                )?,
            });
        capability.version = capability.version.saturating_add(1);
        capability.seed = ctx.random::<u64>();
        ctx.db
            .investigation_action_capability()
            .id()
            .update(capability);
        return Ok(());
    }
    if !permits_resolution {
        return Err(InvestigationExecutionError::ParticipantBoundary);
    }
    crate::strategic::normalize_and_elect_party_leader(ctx, &party_id)?;
    crate::strategic::reconcile_party_objective_continuity(ctx, &party_id)?;
    if resolution.success {
        commit_action_consequence(ctx, &capability, &party_id, &attempt_id)?;
        commit_generated_remediation(ctx, &capability, &party_id, &attempt_id)?;
    }
    persist_action_result_lead(ctx, &capability, &attempt_id, &resolution)?;
    let normalized_party = ctx
        .db
        .party_authority()
        .id()
        .find(&party_id)
        .ok_or(InvestigationExecutionError::PartyDisappeared)?;
    crate::character::require_living_character(ctx, (normalized_party.leader_id).into())?;
    let completed_at = ctx
        .db
        .character_time()
        .character_id()
        .find(normalized_party.leader_id)
        .ok_or(InvestigationExecutionError::LeaderClockMissing)?
        .minutes;
    ctx.db
        .investigation_action_attempt()
        .insert(InvestigationActionAttempt {
            id: attempt_id.clone(),
            capability_id: action_id.clone(),
            owner_character_id: actor_id,
            expected_version,
            method,
            started_at,
            completed_at,
            duration_minutes: resolution.cost.minutes,
            success: resolution.success,
            resulting_uncertainty_bps: resolution.resulting_uncertainty_bps,
            private_resolution_json: private_action_resolution_json(resolution, bounded_progress)?,
        });
    let outcome_case_id = capability.case_id.clone();
    let safe_result_on_success = capability.safe_result_on_success.clone();
    capability.version = capability.version.saturating_add(1);
    capability.seed = ctx.random::<u64>();
    capability.uncertainty_bps = resolution.resulting_uncertainty_bps;
    capability.active = !resolution.success;
    ctx.db
        .investigation_action_capability()
        .id()
        .update(capability);
    let alternate_available = activate_action_successors(
        ctx,
        &ctx.db
            .investigation_action_capability()
            .id()
            .find(&action_id)
            .ok_or(InvestigationExecutionError::ActionDisappeared)?,
        resolution.success,
    )?;
    ctx.db
        .investigation_action_outcome()
        .insert(InvestigationActionOutcome {
            id: generated_observer_id(ctx, &outcome_case_id, "outcome", &attempt_id)
                .unwrap_or_else(|| inv::compound_id(&["outcome", &attempt_id])),
            owner_character_id: actor_id,
            case_id: outcome_case_id,
            capability_id: action_id.clone(),
            attempt_id: attempt_id.clone(),
            safe_wording: if resolution.success {
                if resolution.risk_triggered {
                    format!(
                        "{} The party was exposed to danger during the attempt.",
                        safe_result_on_success
                    )
                } else {
                    safe_result_on_success
                }
            } else if let Some(progress) = bounded_progress {
                bounded_failure_wording(progress, alternate_available)
            } else {
                adventuresim_core::quest_generation::failed_action_outcome_wording(
                    alternate_available,
                )
                .into()
            },
            recorded_at: completed_at,
            official_recorded_at: official_minute(ctx),
        });
    Ok(())
}
