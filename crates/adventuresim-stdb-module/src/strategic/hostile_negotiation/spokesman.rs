/// Exact live hostile spokesman selection for one observer minute.
fn exact_spokesman_for_view(
    ctx: &ViewContext,
    group: &HostileGroupAuthority,
    minute: adventuresim_world_schema::calendar::StrategicMinute,
) -> Option<crate::world_actor::CharacterContextMembership> {
    let rows = ctx
        .db
        .character_context_membership()
        .context_id()
        .filter(&group.id)
        .filter(|row| {
            row.context_kind == crate::world_actor::CharacterContextKind::HostileGroup
                && row.role == crate::world_actor::CharacterContextRole::Counterparty
                && row.location_id == group.case_site_id.as_str()
                && crate::world_actor::context_membership_valid_at(row, minute)
                && ctx
                    .db
                    .character()
                    .id()
                    .find(row.character_id)
                    .is_some_and(|character| character.alive)
                && crate::world_actor::character_alive_at_for_view(ctx, row.character_id, minute)
        })
        .collect();
    unique_lowest_ordinal_spokesman(rows)
}
