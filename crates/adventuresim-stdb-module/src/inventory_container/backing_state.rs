//! Reject fungible operations on rows with persistent per-instance state.
use super::*;

pub(crate) fn require_fungible_row(
    ctx: &ReducerContext,
    row_id: CarriedInventoryRowId,
) -> Result<(), String> {
    let has_object = object_for_row(ctx, row_id)?.is_some();
    let has_state = match row_id {
        CarriedInventoryRowId::Personal(id) => {
            ctx.db
                .inventory_item_amount()
                .inventory_item_id()
                .find(id.get())
                .is_some()
                || ctx
                    .db
                    .item_condition()
                    .inventory_item_id()
                    .find(id.get())
                    .is_some()
                || crate::food::personal_lot(ctx, id).is_some()
        }
        CarriedInventoryRowId::Party(id) => {
            ctx.db
                .party_item_amount()
                .party_inventory_item_id()
                .find(id.get())
                .is_some()
                || ctx
                    .db
                    .party_item_condition()
                    .party_inventory_item_id()
                    .find(id.get())
                    .is_some()
                || crate::food::party_lot(ctx, id).is_some()
        }
    };
    if has_object || has_state {
        return Err("Inventory row has non-fungible state".into());
    }
    Ok(())
}

/// Require a row without an object to be classified and stored as fungible.
pub(crate) fn require_fungible_inventory_row(
    ctx: &ReducerContext,
    row_id: CarriedInventoryRowId,
    item_id: &str,
) -> Result<(), String> {
    let definition = ctx.db.item().id().find(item_id.to_owned());
    let food =
        crate::item::inventory_food_definition(definition.as_ref().map(|item| item.kind), item_id)?
            .is_some();
    let measured = crate::inventory_amount::is_measured_item(ctx, item_id);
    if crate::item::requires_stable_object(definition.as_ref(), food, measured) {
        return Err("Stable inventory row has no physical object".into());
    }
    require_fungible_row(ctx, row_id)
}
