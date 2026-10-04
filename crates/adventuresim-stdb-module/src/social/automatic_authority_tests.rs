//! Guarded automatic-care preference lifecycle acceptance checks.
use super::*;

#[reducer]
pub fn authority_test_automatic_chat_preferences(
    ctx: &ReducerContext,
    bootstrap_token: String,
) -> Result<(), String> {
    crate::strategic::require_dev_bootstrap_token(&bootstrap_token)?;
    let actor = 732071;
    let target = 732072;
    for (id, name) in [(actor, "Care Actor"), (target, "Care Target")] {
        crate::character::create_named_character_with_id(ctx, id, name.into())?;
        let mut character = ctx
            .db
            .character()
            .id()
            .find(id)
            .ok_or("Character missing")?;
        character.current_settlement_id = Some("riverdale".into());
        ctx.db.character().id().update(character);
    }
    crate::strategic::attach_seeded_party_member(ctx, actor, target, "companion")?;
    crate::condition::record_morale_event(
        ctx,
        (target).into(),
        adventuresim_core::morale::MoraleEventKind::Defeat,
        -15.0,
        Some("automatic-care-fixture".into()),
    )?;
    let id = automatic_chat_id(actor, target);
    set_automatic_social_chat(ctx, actor, target, true)?;
    set_automatic_social_chat(ctx, actor, target, true)?;
    if ctx
        .db
        .automatic_social_chat()
        .actor_id()
        .filter(actor)
        .count()
        != 1
    {
        return Err("Repeated opt-in duplicated the preference".into());
    }
    set_automatic_social_chat(ctx, actor, target, false)?;
    apply_automatic_social_chats(ctx, (actor).into(), 10)?;
    if ctx.db.automatic_social_chat().id().find(&id).is_some()
        || ctx.db.social_interaction().actor_id().filter(actor).count() != 0
    {
        return Err("Opt-out retained a preference or performed care".into());
    }
    set_automatic_social_chat(ctx, actor, target, true)?;
    set_automatic_social_chat(ctx, actor, target, true)?;
    apply_automatic_social_chats(ctx, (actor).into(), 10)?;
    // Either random outcome is valid; enrollment must produce exactly one attempt.
    if ctx.db.social_interaction().actor_id().filter(actor).count() != 1 {
        return Err("Opt-in did not produce one bounded care attempt".into());
    }
    let mut character = ctx
        .db
        .character()
        .id()
        .find(target)
        .ok_or("Target missing")?;
    let party_id = character.party_id.take();
    ctx.db.character().id().update(character.clone());
    prune_invalid_automatic_social_chats(ctx);
    if ctx.db.automatic_social_chat().id().find(&id).is_some()
        || set_automatic_social_chat(ctx, actor, target, true).is_ok()
    {
        return Err("Party departure retained or accepted a preference".into());
    }
    character.party_id = party_id;
    ctx.db.character().id().update(character.clone());
    set_automatic_social_chat(ctx, actor, target, true)?;
    character.alive = false;
    ctx.db.character().id().update(character);
    prune_invalid_automatic_social_chats(ctx);
    if ctx.db.automatic_social_chat().id().find(&id).is_some()
        || set_automatic_social_chat(ctx, actor, target, true).is_ok()
        || set_automatic_social_chat(ctx, actor, actor, true).is_ok()
    {
        return Err("Death or self enrollment accepted a preference".into());
    }
    Ok(())
}
