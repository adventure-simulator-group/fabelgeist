pub(crate) fn add_to_party_inventory(
    ctx: &ReducerContext,
    party_id: &str,
    item_id: &str,
    quantity: u32,
) -> Result<(), String> {
    if quantity == 0 {
        return Ok(());
    }
    let definition = ctx.db.item().id().find(item_id.to_string());
    let kind = definition.as_ref().map(|row| row.kind);
    let food = crate::item::inventory_food_definition(kind, item_id)?.is_some();
    let measured = crate::inventory_amount::is_measured_item(ctx, item_id);
    let durable = item_is_durable(ctx, item_id);
    let individual = crate::item::requires_stable_object(definition.as_ref(), food, measured);
    if individual {
        let minute = ctx
            .db
            .party_authority()
            .id()
            .find(party_id.to_string())
            .and_then(|party| ctx.db.character_time().character_id().find(party.leader_id))
            .map_or(StrategicMinute::ZERO, |time| time.minutes);
        for _ in 0..quantity {
            let row = ctx.db.party_inventory_item().insert(PartyInventoryItem {
                id: 0,
                party_id: party_id.into(),
                item_id: item_id.into(),
                quantity: 1,
            });
            crate::inventory_container::insert_party_object(ctx, &row)?;
            if measured {
                crate::inventory_amount::initialize_party(ctx, row.id);
            }
            if food {
                let food_row = PartyInventoryRowId::from_raw(row.id);
                crate::food::create_party_food_lot(ctx, food_row, item_id, 1, minute)
                    .ok_or_else(|| format!("Could not create party food lot for {item_id}"))?;
            }
            if durable {
                ctx.db.party_item_condition().insert(PartyItemCondition {
                    party_inventory_item_id: row.id,
                    tier_1: 0.0,
                    tier_2: 0.0,
                    tier_3: 0.0,
                    tier_4: 0.0,
                    tier_5: 0.0,
                });
            }
            crate::weapon_instance::initialize_party_weapon(ctx, &row)?;
        }
        return Ok(());
    }
    if let Some(mut stack) = ctx
        .db
        .party_inventory_item()
        .party_id()
        .filter(party_id)
        .find(|stack| stack.item_id == item_id)
    {
        crate::inventory_container::require_fungible_row(
            ctx,
            CarriedInventoryRowId::Party(PartyInventoryRowId::from_raw(stack.id)),
        )?;
        if let Some(merged) = stack.quantity.checked_add(quantity) {
            stack.quantity = merged;
            ctx.db.party_inventory_item().id().update(stack);
        } else {
            ctx.db.party_inventory_item().insert(PartyInventoryItem {
                id: 0,
                party_id: party_id.to_string(),
                item_id: item_id.to_string(),
                quantity,
            });
        }
    } else {
        ctx.db.party_inventory_item().insert(PartyInventoryItem {
            id: 0,
            party_id: party_id.to_string(),
            item_id: item_id.to_string(),
            quantity,
        });
    }
    Ok(())
}
