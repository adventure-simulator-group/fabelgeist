// Apply already-validated sales, preserving the carried inventory identity.
fn apply_storefront_sale(
    ctx: &ReducerContext,
    row_id: CarriedInventoryRowId,
    quantity: u32,
) -> StrategicOperationResult<()> {
    let object = crate::inventory_container::object_for_row(ctx, row_id)?;
    if let Some(object) = object {
        crate::inventory_container::detach_if_nested(ctx, object.id)?;
        crate::inventory_container::delete_subtree(ctx, object.id)?;
        return Ok(());
    }
    match row_id {
        CarriedInventoryRowId::Party(row_id) => apply_party_storefront_sale(ctx, row_id, quantity),
        CarriedInventoryRowId::Personal(row_id) => {
            apply_personal_storefront_sale(ctx, row_id, quantity)
        }
    }
}

fn apply_party_storefront_sale(
    ctx: &ReducerContext,
    row_id: PartyInventoryRowId,
    quantity: u32,
) -> StrategicOperationResult<()> {
    let mut inventory = ctx
        .db
        .party_inventory_item()
        .id()
        .find(row_id.get())
        .ok_or("Party inventory item disappeared before trade application")?;
    if inventory.quantity == quantity {
        crate::food::delete_party_food_lot(ctx, row_id);
        ctx.db
            .party_item_amount()
            .party_inventory_item_id()
            .delete(row_id.get());
        ctx.db.party_inventory_item().id().delete(row_id.get());
        ctx.db
            .party_item_condition()
            .party_inventory_item_id()
            .delete(row_id.get());
    } else {
        if crate::food::party_lot(ctx, row_id).is_some() {
            crate::food::remove_party_lot_quantity(ctx, row_id, quantity, inventory.quantity)?;
        }
        inventory.quantity -= quantity;
        ctx.db.party_inventory_item().id().update(inventory);
    }
    Ok(())
}

fn apply_personal_storefront_sale(
    ctx: &ReducerContext,
    row_id: PersonalInventoryRowId,
    quantity: u32,
) -> StrategicOperationResult<()> {
    let mut inventory = ctx
        .db
        .inventory_item()
        .id()
        .find(row_id.get())
        .ok_or("Inventory item disappeared before trade application")?;
    if inventory.quantity == quantity {
        crate::food::delete_personal_food_lot(ctx, row_id);
        ctx.db
            .inventory_item_amount()
            .inventory_item_id()
            .delete(row_id.get());
        ctx.db.inventory_item().id().delete(row_id.get());
        ctx.db
            .item_condition()
            .inventory_item_id()
            .delete(row_id.get());
    } else {
        if crate::food::personal_lot(ctx, row_id).is_some() {
            crate::food::remove_lot_quantity(ctx, row_id, quantity, inventory.quantity)?;
        }
        inventory.quantity -= quantity;
        ctx.db.inventory_item().id().update(inventory);
    }
    Ok(())
}
