fn ensure_standalone_host(
    ctx: &ReducerContext,
    scene_key: &str,
    character_id: u64,
) -> StrategicOperationResult<()> {
    if !ctx
        .db
        .settlement()
        .iter()
        .any(|settlement| settlement.scene_key == scene_key)
    {
        seed_tactical_host_world(ctx)?;
    }
    if ctx.db.character().id().find(character_id).is_none() {
        let default =
            adventuresim_core::starting_character::default_character_with_id(character_id);
        crate::character::insert_starting_character(ctx, &default)?;
    }
    Ok(())
}

fn refresh_expired_recruitment_offers(ctx: &ReducerContext, now: StrategicMinute) {
    for mut offer in ctx.db.recruitment_offer().iter().collect::<Vec<_>>() {
        if offer.status == RecruitmentOfferStatus::Open && now >= offer.expires_at_minute {
            if recruitment_offer_bindings_are_live(ctx, &offer, now) {
                // Reuse the canonical offer identity instead of allowing
                // historical rows to exhaust the finite NPC population.
                offer.created_at_minute = now;
                offer.expires_at_minute = renewed_recruitment_offer_expiry(now);
            } else {
                offer.status = RecruitmentOfferStatus::Closed;
            }
            ctx.db.recruitment_offer().id_key().update(offer);
        }
    }
}

fn standalone_server_exists(ctx: &ReducerContext, mission_id: &String) -> bool {
    ctx.db
        .tactical_server_request_authority()
        .mission_id()
        .find(mission_id)
        .is_some()
        || ctx
            .db
            .tactical_server_authority()
            .mission_id()
            .find(mission_id)
            .is_some()
}
