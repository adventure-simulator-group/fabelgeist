//! Guarded claim persistence, admission, and idempotent receipt checks.

use super::*;

fn witness_session(
    ctx: &ReducerContext,
    observer: u64,
) -> Result<crate::strategic::DialogueSession, String> {
    use crate::strategic::{
        DialogueEvent, DialogueParticipant, DialogueSession, dialogue_participant,
    };
    crate::character::create_named_character_with_id(ctx, observer, "Witness Fixture".into())?;
    let mut actor = ctx
        .db
        .character()
        .id()
        .find(observer)
        .ok_or("Actor missing")?;
    actor.current_settlement_id = Some("riverdale".into());
    let party_id = actor
        .party_id
        .clone()
        .ok_or("Witness actor has no solo party")?;
    ctx.db.character().id().update(actor);
    let resident = 732004;
    let profile = ctx
        .db
        .settlement_resident_profile()
        .character_id()
        .find(resident)
        .ok_or("Resident missing")?;
    let mut presence = ctx
        .db
        .settlement_resident_presence()
        .character_id()
        .find(resident)
        .ok_or("Presence missing")?;
    presence.start_minute = 0;
    presence.end_minute = 1440;
    presence.context_suppressed = false;
    presence.health_suppressed = false;
    ctx.db
        .settlement_resident_presence()
        .character_id()
        .update(presence.clone());
    let session = ctx.db.dialogue_session().insert(DialogueSession {
        id: format!("dialogue:{observer}:authority"),
        gateway_bucket: 0,
        conversation_id: profile.conversation_id,
        catalog_revision: "fixture".into(),
        settlement_id: "riverdale".into(),
        location_id: presence.location_id,
        owner_character_id: observer,
        owner_party_id: party_id,
        state: "active".into(),
        revision: 0,
        created_micros: 0,
    });
    for (role, character_id, actor_id) in [
        ("player", Some(observer), observer),
        ("resident", None, resident),
    ] {
        ctx.db.dialogue_participant().insert(DialogueParticipant {
            id: format!("{}:{actor_id}", session.id),
            gateway_bucket: 0,
            session_id: session.id.clone(),
            role: role.into(),
            character_id,
            actor_id: actor_id.to_string(),
            display_name: "Fixture".into(),
        });
    }
    ctx.db.dialogue_event().insert(DialogueEvent {
        id: format!("{}:0", session.id),
        gateway_bucket: 0,
        session_id: session.id.clone(),
        sequence: 0,
        response_id: "heard".into(),
        speaker_role: "resident".into(),
        fragments_json: "[]".into(),
        source_refs_json: "[]".into(),
        created_micros: 0,
    });
    ctx.db
        .dialogue_witness_capability()
        .insert(DialogueWitnessCapability {
            id: format!("{}:{observer}", session.id),
            session_id: session.id.clone(),
            observer_character_id: observer,
            resident_character_id: resident,
            has_bound_concern: false,
            bound_released: false,
        });
    Ok(session)
}

#[reducer]
pub fn authority_test_witness_resolutions(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    use adventuresim_core::social::{ClaimChallengeOutcome, WitnessClaimOutcome};
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let observer = 732041;
    let mut session = witness_session(ctx, observer)?;
    for (order, succeeded) in [(0, false), (1, true)] {
        let token = format!("authority-claim-{order}");
        ctx.db
            .dialogue_witness_claim()
            .insert(DialogueWitnessClaim {
                challenge_token: token.clone(),
                session_id: session.id.clone(),
                observer_character_id: observer,
                resident_character_id: 732004,
                event_sequence: 0,
                claim_order: order,
                proposition_id: "fixture".into(),
                displayed_text: "Heard statement".into(),
                charm_response: Some("Authored response".into()),
                command_response: None,
                bluff_response: None,
                claim_is_factually_accurate: !succeeded,
                demeanor_truth_signal: 0.0,
                assessment_direction: "uncertain".into(),
                assessment_strength: 0.0,
                resolution: None,
            });
        let action_id = format!("respond-{order}");
        let (_, _, mut claim) = require_witness_social_action(
            ctx,
            observer,
            &session.id,
            &action_id,
            session.revision,
            ClaimChallengeApproach::Charm,
            &token,
        )?;
        let challenge = ClaimChallengeOutcome {
            succeeded,
            morale_delta: 0.0,
            affinity_delta: -0.4,
        };
        let realized = apply_witness_relationship_outcome(
            ctx,
            observer,
            732004,
            SOCIAL_RESPONSE_MINUTES,
            challenge.morale_delta,
            challenge.affinity_delta,
            &action_id,
            adventuresim_core::morale::MoraleEventKind::WitnessCharm,
        )?;
        let resolution = WitnessClaimResolution::from_challenge(challenge, realized);
        claim.resolution = Some(resolution);
        ctx.db
            .dialogue_witness_claim()
            .challenge_token()
            .update(claim.clone());
        finish_witness_social_action(ctx, session.clone(), &claim, action_id.clone(), "charm");
        session = ctx
            .db
            .dialogue_session()
            .id()
            .find(&session.id)
            .ok_or("Session missing")?;
        let stored = ctx
            .db
            .dialogue_witness_claim()
            .challenge_token()
            .find(&token)
            .ok_or("Claim missing")?;
        if stored.resolution != Some(resolution)
            || resolution.outcome
                != if succeeded {
                    WitnessClaimOutcome::UsefulAnswer
                } else {
                    WitnessClaimOutcome::DidNotYield
                }
            || require_witness_social_action(
                ctx,
                observer,
                &session.id,
                "new-action",
                session.revision,
                ClaimChallengeApproach::Charm,
                &token,
            )
            .is_ok()
        {
            return Err("Resolution persistence or admission disagrees".into());
        }
        // The real reducer must exact-replay before reading mutable presence or revision.
        let affinity_before = current_affinity(ctx, 732004, observer);
        approach_dialogue_witness(
            ctx,
            observer,
            session.id.clone(),
            token.clone(),
            "charm".into(),
            action_id.clone(),
            0,
        )?;
        if ctx
            .db
            .dialogue_session()
            .id()
            .find(&session.id)
            .ok_or("Session missing")?
            .revision
            != session.revision
            || current_affinity(ctx, 732004, observer) != affinity_before
            || witness_social_action_replayed(
                ctx,
                &format!("{}:{action_id}", session.id),
                observer,
                "bluff",
                &token,
            )
            .is_ok()
        {
            return Err("Witness receipt retry reapplied or accepted conflicting action".into());
        }
    }
    Ok(())
}

fn authored_claim(session: &crate::strategic::DialogueSession, key: &str) -> DialogueWitnessClaim {
    DialogueWitnessClaim {
        challenge_token: format!("{}:{key}", session.id),
        session_id: session.id.clone(),
        observer_character_id: session.owner_character_id,
        resident_character_id: 732004,
        event_sequence: 0,
        claim_order: 0,
        proposition_id: "fixture".into(),
        displayed_text: "Heard statement".into(),
        charm_response: Some("Authored charm".into()),
        command_response: Some("Authored command".into()),
        bluff_response: Some("Authored bluff".into()),
        claim_is_factually_accurate: true,
        demeanor_truth_signal: 0.0,
        assessment_direction: "uncertain".into(),
        assessment_strength: 0.0,
        resolution: None,
    }
}

#[reducer]
pub fn authority_test_witness_request_keys(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let observer = 732960;
    let mut session = witness_session(ctx, observer)?;
    for key in ["charm", "command", "bluff"] {
        let claim = ctx
            .db
            .dialogue_witness_claim()
            .insert(authored_claim(&session, key));
        let action_id = format!("execute-{key}");
        let before = social_clock_or(ctx, observer, StrategicMinute::ZERO);
        approach_dialogue_witness(
            ctx,
            observer,
            session.id.clone(),
            claim.challenge_token.clone(),
            key.into(),
            action_id.clone(),
            session.revision,
        )?;
        let stored = ctx
            .db
            .dialogue_witness_claim()
            .challenge_token()
            .find(&claim.challenge_token)
            .ok_or("Claim missing")?;
        let receipt = ctx
            .db
            .witness_social_action_receipt()
            .id()
            .find(format!("{}:{action_id}", session.id))
            .ok_or("Receipt missing")?;
        session = ctx
            .db
            .dialogue_session()
            .id()
            .find(&session.id)
            .ok_or("Session missing")?;
        if stored.resolution.is_none_or(|resolution| {
            resolution.outcome != adventuresim_core::social::WitnessClaimOutcome::DidNotYield
        }) || receipt.action_kind != key
            || receipt.resulting_revision != session.revision
            || social_clock_or(ctx, observer, StrategicMinute::ZERO)
                != before.saturating_add_minutes(SOCIAL_RESPONSE_MINUTES)
        {
            return Err("Witness request did not execute its canonical action exactly once".into());
        }
        let after = social_clock_or(ctx, observer, StrategicMinute::ZERO);
        let affinity = current_affinity(ctx, 732004, observer);
        approach_dialogue_witness(
            ctx,
            observer,
            session.id.clone(),
            claim.challenge_token.clone(),
            key.into(),
            action_id.clone(),
            u64::MAX,
        )?;
        let conflicting = if key == "charm" { "command" } else { "charm" };
        if approach_dialogue_witness(
            ctx,
            observer,
            session.id.clone(),
            claim.challenge_token,
            conflicting.into(),
            action_id,
            session.revision,
        )
        .is_ok()
            || social_clock_or(ctx, observer, StrategicMinute::ZERO) != after
            || current_affinity(ctx, 732004, observer) != affinity
            || ctx
                .db
                .dialogue_session()
                .id()
                .find(&session.id)
                .unwrap()
                .revision
                != session.revision
        {
            return Err("Witness exact replay changed state or admitted a conflicting key".into());
        }
    }
    let mut claim = authored_claim(&session, "unauthored");
    claim.charm_response = None;
    claim.command_response = None;
    claim.bluff_response = None;
    let claim = ctx.db.dialogue_witness_claim().insert(claim);
    for (key, expected) in [
        ("charm", "That response is not authored for this claim"),
        ("command", "That response is not authored for this claim"),
        ("bluff", "That response is not authored for this claim"),
        ("Charm", "Unknown witness approach"),
        ("deception", "Unknown witness approach"),
        ("", "Unknown witness approach"),
    ] {
        let result = approach_dialogue_witness(
            ctx,
            observer,
            session.id.clone(),
            claim.challenge_token.clone(),
            key.into(),
            "rejected".into(),
            session.revision,
        );
        if result.as_ref().map_err(String::as_str) != Err(expected)
            || ctx
                .db
                .witness_social_action_receipt()
                .id()
                .find(format!("{}:rejected", session.id))
                .is_some()
            || ctx
                .db
                .dialogue_witness_claim()
                .challenge_token()
                .find(&claim.challenge_token)
                .unwrap()
                .resolution
                .is_some()
        {
            return Err(format!(
                "Witness request {key:?} did not reject without effects: {result:?}"
            ));
        }
    }
    Ok(())
}
