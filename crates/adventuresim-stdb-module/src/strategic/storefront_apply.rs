// Validate objectless sales before quoting or changing inventory.
fn storefront_sale_definition(
    ctx: &ReducerContext,
    row_id: CarriedInventoryRowId,
    item_id: &str,
    object: Option<&crate::inventory_container::InventoryObject>,
) -> Result<crate::Item, String> {
    if object.is_none() {
        crate::inventory_container::require_fungible_inventory_row(ctx, row_id, item_id)?;
    }
    ctx.db
        .item()
        .id()
        .find(item_id.to_owned())
        .ok_or_else(|| "Item definition not found".into())
}

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
        ctx.db.party_inventory_item().id().delete(row_id.get());
    } else {
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
        ctx.db.inventory_item().id().delete(row_id.get());
    } else {
        inventory.quantity -= quantity;
        ctx.db.inventory_item().id().update(inventory);
    }
    Ok(())
}
