/// Characters entitled to see a residence holding at their own frontier.
fn view_character_ids_for_holding(ctx: &ViewContext, holding: &ResidenceHolding) -> Vec<u64> {
    let holding_id = holding.id.clone();
    let mut character_ids = ctx
        .db
        .residence_occupant()
        .property_id()
        .filter(&holding.property_id)
        .filter(|row| row.holding_id.as_deref() == Some(holding_id.as_str()))
        .filter(|row| {
            ctx.db
                .character_time()
                .character_id()
                .find(row.character_id)
                .is_some_and(|time| {
                    row.admitted_minute
                        <= time.minutes
                })
        })
        .map(|row| row.character_id)
        .collect::<Vec<_>>();
    // Legal tenure must remain visible to the holder even after a
    // different holding becomes primary and moves their occupancy.
    // Otherwise an unoccupied property continues billing privately
    // but cannot be inspected or managed through the gateway.
    character_ids.push(holding.holder_character_id);
    character_ids.extend(
        ctx.db
            .primary_residence()
            .holding_id()
            .filter(&holding_id)
            .filter(|row| {
                ctx.db
                    .character_time()
                    .character_id()
                    .find(row.character_id)
                    .is_some_and(|time| {
                        row.designated_minute
                            <= time.minutes
                    })
            })
            .map(|row| row.character_id),
    );
    character_ids.extend(
        ctx.db
            .residence_transition()
            .holding_id()
            .filter(&holding_id)
            .map(|transition| transition.affected_character_id),
    );
    character_ids.extend(ctx.db.property_occupancy_transition().property_id().filter(&holding.property_id)
        .filter(|event| event.holding_id.as_deref() == Some(holding_id.as_str()))
        .map(|event| event.character_id));
    character_ids.sort_unstable();
    character_ids.dedup();
    character_ids
}
