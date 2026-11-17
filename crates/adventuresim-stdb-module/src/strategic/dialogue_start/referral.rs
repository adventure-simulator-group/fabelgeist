//! Venue discovery and witness authority after an ordinary greeting.

use super::*;

pub(super) fn deliver(
    ctx: &ReducerContext,
    session: &DialogueSession,
    character_id: u64,
    npc_character_id: u64,
) -> Result<(), String> {
    let npc_actor_id = npc_character_id.to_string();
    crate::local_problem::surface_problem(
        ctx,
        character_id,
        &session.id,
        npc_character_id,
        &session.location_id,
    )?;
    // Entering a tavern (or the overview fallback) is itself the reliable,
    // markerless discovery action. Persist the observer-safe journal lead
    // immediately; it does not accept a contract or disclose a hidden cause.
    if let Some(delivery) = ctx
        .db
        .local_problem_rumor_delivery()
        .session_id()
        .filter(&session.id)
        .find(|row| row.character_id == character_id)
    {
        let authority = referral_delivery_authority(ctx, &delivery)?;
        let speaker_role = ctx
            .db
            .dialogue_participant()
            .session_id()
            .filter(&session.id)
            .find(|participant| participant.actor_id == npc_actor_id)
            .map(|participant| participant.role)
            .ok_or("Rumor delivery speaker disappeared")?;
        let sequence = ctx
            .db
            .dialogue_event()
            .session_id()
            .filter(&session.id)
            .count() as u32;
        let (fragments_json, source_refs_json) = match &authority {
            ReferralDeliveryAuthority::PublicThreat => (
                delivery.fragments_json.clone(),
                serde_json::to_string(std::slice::from_ref(&delivery.receipt_id))
                    .map_err(|_| "Could not encode public referral source")?,
            ),
            ReferralDeliveryAuthority::LocalProblem(_) => render_quest_referral_variant(
                ctx,
                session,
                character_id,
                npc_character_id,
                &delivery,
            )?,
        };
        ctx.db.dialogue_event().insert(DialogueEvent {
            id: format!("{}:event:{sequence}", session.id),
            gateway_bucket: 0,
            session_id: session.id.clone(),
            sequence,
            response_id: "generated-referral".into(),
            speaker_role,
            fragments_json,
            source_refs_json,
            created_micros: ctx.timestamp.to_micros_since_unix_epoch(),
        });
        if let ReferralDeliveryAuthority::LocalProblem(receipt) = authority {
            crate::investigation::receive_local_problem_rumor(
                ctx,
                character_id,
                receipt.id.clone(),
                format!("receive-rumor:{character_id}:{}", receipt.id),
            )?;
        }
    }
    crate::social::ensure_dialogue_witness_capability(
        ctx,
        session,
        character_id,
        npc_character_id,
    )?;
    Ok(())
}
