/// Inn service and present dialogue-capable rumor contact at observer time.
fn inn_contact_available_at(
    ctx: &ReducerContext,
    settlement_id: &String,
    observer_minute: StrategicMinute,
) -> bool {
    let inn_service = ctx
        .db
        .settlement()
        .id()
        .find(settlement_id)
        .is_some_and(|s| {
            s.economy
                .has_service(adventuresim_world_schema::SettlementService::Inn)
        });
    inn_service
        && ctx
            .db
            .settlement_resident_presence()
            .settlement_id()
            .filter(settlement_id)
            .any(|presence| {
                let npc = ctx
                    .db
                    .settlement_resident_profile()
                    .character_id()
                    .find(presence.character_id);
                dialogue_capable_inn_contact(
                    &presence.settlement_id,
                    settlement_id,
                    &presence.location_id,
                    crate::settlement_population::npc_is_present(ctx, &presence, observer_minute),
                    npc.as_ref().is_some_and(|npc| {
                        npc.home_settlement_id == settlement_id.as_str()
                            && crate::settlement_population::resident_is_dialogue_capable(npc)
                    }),
                )
            })
}
