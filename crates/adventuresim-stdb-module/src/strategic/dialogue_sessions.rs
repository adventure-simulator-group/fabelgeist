#[reducer]
pub fn start_dialogue(
    ctx: &ReducerContext,
    character_id: u64,
    session_id: String,
    conversation_id: String,
    npc_actor_id: String,
    location_id: String,
    catalog_revision: String,
) -> Result<(), String> {
    require_strategic_gateway(ctx)?;
    let request = dialogue_start::DialogueStartRequest::decode(
        character_id,
        session_id,
        conversation_id,
        npc_actor_id,
        location_id,
        catalog_revision,
    )?;
    request.start_at_venue(ctx)
}

/// Only a registered native host may attest physical contact between these visits.
#[reducer]
pub fn start_tactical_dialogue(
    ctx: &ReducerContext,
    actor_visit_id: u64,
    resident_visit_id: u64,
    session_id: String,
    catalog_revision: String,
) -> Result<(), String> {
    dialogue_start::DialogueStartRequest::start_in_scene(
        ctx,
        TacticalDialogueAuthority {
            server: ctx.sender(),
            actor_visit_id,
            resident_visit_id,
        },
        session_id,
        catalog_revision,
    )
}

/// Closing needs the original host, even after proximity or a visit has ended.
#[reducer]
pub fn close_tactical_dialogue(ctx: &ReducerContext, session_id: String) -> Result<(), String> {
    let mut session = ctx
        .db
        .dialogue_session()
        .id()
        .find(&session_id)
        .ok_or("Dialogue session not found")?;
    let DialogueSessionAuthority::Tactical(binding) = &session.authority else {
        return Err("Dialogue session does not belong to a tactical host".into());
    };
    if binding.server != ctx.sender() {
        return Err("Only the original tactical host can close this dialogue".into());
    }
    if DialogueSessionState::parse(&session.state)? == DialogueSessionState::Closed {
        return Ok(());
    }
    session.state = DialogueSessionState::Closed.stable_id().into();
    session.revision = session.revision.saturating_add(1);
    ctx.db.dialogue_session().id().update(session);
    Ok(())
}
