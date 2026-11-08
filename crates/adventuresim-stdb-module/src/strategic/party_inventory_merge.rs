// Move physical trees before fungible rows so descendants retain their backing.
fn merge_party_inventory(
    ctx: &ReducerContext,
    source_party_id: &str,
    destination_party_id: &str,
) -> Result<(), String> {
    let destination = OperationalCustody::party(destination_party_id.to_string())
        .map_err(|error| error.to_string())?;
    let rows = ctx
        .db
        .party_inventory_item()
        .party_id()
        .filter(source_party_id)
        .collect::<Vec<_>>();
    for entry in &rows {
        let row_id = PartyInventoryRowId::from_raw(entry.id);
        let Some(object) =
            crate::inventory_container::object_for_row(ctx, CarriedInventoryRowId::Party(row_id))?
        else {
            continue;
        };
        if !crate::inventory_container::object_is_nested(ctx, object.id) {
            crate::inventory_container::rehome_subtree(ctx, object.id, &destination)?;
        }
    }
    for entry in rows {
        let row = ctx
            .db
            .party_inventory_item()
            .id()
            .find(entry.id)
            .ok_or("Party inventory row disappeared during merge")?;
        if row.party_id == destination_party_id {
            continue;
        }
        if row.party_id != source_party_id {
            return Err("Party inventory row changed custody during merge".into());
        }
        let row_id = PartyInventoryRowId::from_raw(row.id);
        if crate::inventory_container::object_for_row(ctx, CarriedInventoryRowId::Party(row_id))?
            .is_some()
        {
            return Err("Nested party inventory object was not moved with its root".into());
        }
        let definition = ctx.db.item().id().find(&row.item_id);
        let food = crate::item::inventory_food_definition(
            definition.as_ref().map(|item| item.kind),
            &row.item_id,
        )?
        .is_some();
        let measured = crate::inventory_amount::is_measured_item(ctx, &row.item_id);
        if crate::item::requires_stable_object(definition.as_ref(), food, measured) {
            return Err("Stable party inventory row has no physical object".into());
        }
        add_to_party_inventory(ctx, destination_party_id, &row.item_id, row.quantity)?;
        ctx.db.party_inventory_item().id().delete(row.id);
    }
    Ok(())
}

pub(crate) fn attach_seeded_party_member(
    ctx: &ReducerContext,
    leader_id: u64,
    member_id: u64,
    role: &str,
) -> Result<(), String> {
    let leader = ctx
        .db
        .character()
        .id()
        .find(leader_id)
        .ok_or("Seed party leader not found")?;
    let party_id = leader
        .party_id
        .clone()
        .ok_or("Seed party leader has no party")?;
    let mut member = ctx
        .db
        .character()
        .id()
        .find(member_id)
        .ok_or("Seed party member not found")?;

    if member.party_id.as_deref() == Some(&party_id) {
        if let Some(mut membership) = ctx
            .db
            .party_member()
            .character_id()
            .filter(member_id)
            .find(|membership| membership.party_id == party_id)
        {
            membership.role = Some(role.into());
            ctx.db.party_member().id().update(membership);
        }
        return Ok(());
    }

    if let Some(source_party_id) = member.party_id.clone() {
        let source_members: Vec<_> = ctx
            .db
            .party_member()
            .party_id()
            .filter(&source_party_id)
            .collect();
        if source_members
            .iter()
            .any(|membership| membership.character_id != member_id)
        {
            return Err("Seed party member belongs to a non-solo party".into());
        }
        require_seed_party_inventory_empty(ctx, &source_party_id)?;
        for membership in source_members {
            ctx.db.party_member().id().delete(membership.id);
        }
        for vote in ctx
            .db
            .party_leader_vote()
            .party_id()
            .filter(&source_party_id)
            .collect::<Vec<_>>()
        {
            ctx.db.party_leader_vote().id().delete(&vote.id);
        }
        ctx.db.party_authority().id().delete(&source_party_id);
    }

    member.party_id = Some(party_id.clone());
    member.current_settlement_id = leader.current_settlement_id.clone();
    ctx.db.character().id().update(member);
    crate::investigation::set_character_case_site(
        ctx,
        member_id,
        crate::investigation::character_case_site_id(ctx, leader_id),
    )?;
    crate::social::reset_familiarity_after_join(ctx, member_id);
    ctx.db.party_member().insert(PartyMember {
        id: 0,
        party_id: party_id.clone(),
        character_id: member_id,
        role: Some(role.into()),
        recruitment_role_id: None,
    });
    put_leader_vote(ctx, &party_id, member_id, leader_id);
    if let Some(mut party) = ctx.db.party_authority().id().find(&party_id) {
        party.is_solo = false;
        ctx.db.party_authority().id().update(party);
    }
    normalize_and_elect_party_leader(ctx, &party_id)?;
    Ok(())
}

fn require_seed_party_inventory_empty(
    ctx: &ReducerContext,
    source_party_id: &str,
) -> Result<(), String> {
    if ctx
        .db
        .party_inventory_item()
        .party_id()
        .filter(source_party_id)
        .next()
        .is_some()
        || ctx
            .db
            .party_stake()
            .party_id()
            .filter(source_party_id)
            .next()
            .is_some()
        || ctx
            .db
            .party_inventory_state()
            .party_id()
            .find(source_party_id.to_owned())
            .is_some()
    {
        return Err("Seed party member has unsettled party inventory".into());
    }
    Ok(())
}
