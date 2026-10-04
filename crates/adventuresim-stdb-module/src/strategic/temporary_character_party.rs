// Owns the party portion of temporary-character cascade deletion.
pub(crate) fn delete_temporary_character_party(
    ctx: &ReducerContext,
    character_id: u64,
    party_id: &str,
) -> Result<(), crate::character::CharacterDeletionError> {
    let party_key = party_id.to_string();
    let members: Vec<_> = ctx.db.party_member().party_id().filter(party_id).collect();
    if members
        .iter()
        .any(|member| member.character_id != character_id)
    {
        return Err(crate::character::CharacterDeletionError::UnexpectedPartyMember);
    }
    for member in members {
        ctx.db.party_member().id().delete(member.id);
    }
    for row in ctx
        .db
        .party_leader_vote()
        .party_id()
        .filter(party_id)
        .collect::<Vec<_>>()
    {
        ctx.db.party_leader_vote().id().delete(&row.id);
    }
    for row in ctx
        .db
        .party_stake()
        .party_id()
        .filter(party_id)
        .collect::<Vec<_>>()
    {
        ctx.db.party_stake().id().delete(row.id);
    }
    for row in ctx
        .db
        .party_inventory_item()
        .party_id()
        .filter(party_id)
        .collect::<Vec<_>>()
    {
        if crate::inventory_container::delete_carried_object_for_row(
            ctx,
            adventuresim_core::physical_object::CarriedInventoryScope::Party,
            row.id,
        )? {
            continue;
        }
        if let Some(condition) = ctx
            .db
            .party_item_condition()
            .party_inventory_item_id()
            .find(row.id)
        {
            ctx.db
                .party_item_condition()
                .party_inventory_item_id()
                .delete(condition.party_inventory_item_id);
        }
        ctx.db.party_inventory_item().id().delete(row.id);
    }
    if ctx
        .db
        .party_inventory_state()
        .party_id()
        .find(&party_key)
        .is_some()
    {
        ctx.db.party_inventory_state().party_id().delete(&party_key);
    }
    if ctx
        .db
        .party_journey_authority()
        .party_id()
        .find(&party_key)
        .is_some()
    {
        ctx.db
            .party_journey_authority()
            .party_id()
            .delete(&party_key);
    }
    if ctx
        .db
        .party_journey_route_authority()
        .party_id()
        .find(&party_key)
        .is_some()
    {
        ctx.db
            .party_journey_route_authority()
            .party_id()
            .delete(&party_key);
    }
    for row in ctx
        .db
        .party_action_request_authority()
        .party_id()
        .filter(party_id)
        .collect::<Vec<_>>()
    {
        ctx.db.party_action_request_authority().id().delete(row.id);
    }
    for row in ctx
        .db
        .party_join_request()
        .party_id()
        .filter(party_id)
        .collect::<Vec<_>>()
    {
        ctx.db.party_join_request().id().delete(row.id);
    }
    for row in ctx
        .db
        .party_recruitment_role()
        .party_id()
        .filter(party_id)
        .collect::<Vec<_>>()
    {
        ctx.db.party_recruitment_role().id().delete(row.id);
    }
    ctx.db.party_authority().id().delete(&party_key);
    Ok(())
}
