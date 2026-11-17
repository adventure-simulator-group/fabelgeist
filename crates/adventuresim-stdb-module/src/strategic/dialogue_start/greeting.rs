//! Render the authored greeting and apply its effects.

use super::*;

pub(super) fn emit(
    ctx: &ReducerContext,
    session: &DialogueSession,
    conversation: &adventuresim_dialogue::Conversation,
    character_id: u64,
) -> Result<(), String> {
    if !conversation.on_start.is_empty() {
        let facts = dialogue_fact_context(ctx, session, character_id)?;
        let response = adventuresim_dialogue::select_start_response(conversation, &facts)
            .map_err(|_| "No unambiguous eligible conversation greeting")?;
        dialogue_scene::require_response(session, response)?;
        for (turn_index, turn) in response.turns.iter().enumerate() {
            let source_refs: Vec<_> = turn
                .fragments
                .iter()
                .enumerate()
                .map(|(fragment_index, authored)| {
                    let field = match authored {
                        adventuresim_dialogue::Fragment::Text { .. } => "value",
                        adventuresim_dialogue::Fragment::Topic { .. } => "label",
                        adventuresim_dialogue::Fragment::PeriodClaim { .. } => "value",
                        adventuresim_dialogue::Fragment::AuthoritativeExplanation { .. } => "value",
                        adventuresim_dialogue::Fragment::Runtime { .. } => "slot",
                    };
                    adventuresim_dialogue::source_for_start_fragment(
                        &session.conversation_id,
                        &response.id,
                        turn_index,
                        fragment_index,
                        field,
                    )
                })
                .collect();
            ctx.db.dialogue_event().insert(DialogueEvent {
                id: format!("{}:event:{turn_index}", session.id),
                gateway_bucket: 0,
                session_id: session.id.clone(),
                sequence: turn_index as u32,
                response_id: response.id.clone(),
                speaker_role: turn.speaker.clone(),
                fragments_json: serde_json::to_string(&resolve_dialogue_fragments(
                    ctx,
                    session,
                    character_id,
                    &turn.speaker,
                    Some(&turn.addressee),
                    &turn.fragments,
                )?)
                .map_err(|_| "Could not encode dialogue greeting")?,
                source_refs_json: serde_json::to_string(&source_refs)
                    .map_err(|_| "Could not encode dialogue greeting sources")?,
                created_micros: ctx.timestamp.to_micros_since_unix_epoch(),
            });
        }
        for effect in &response.effects {
            let source_scope = format!("start:{}", response.id);
            apply_dialogue_effect(
                ctx,
                character_id,
                session,
                &source_scope,
                "start",
                session.revision,
                effect,
                None,
            )?;
        }
    }
    Ok(())
}
