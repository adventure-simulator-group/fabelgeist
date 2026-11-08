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
