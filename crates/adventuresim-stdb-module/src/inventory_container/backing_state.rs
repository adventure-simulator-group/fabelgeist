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
