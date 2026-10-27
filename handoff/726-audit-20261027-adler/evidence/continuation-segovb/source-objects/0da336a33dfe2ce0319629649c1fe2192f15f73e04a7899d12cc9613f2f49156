//! Commit the exact measured ingredient when preparation starts.

use super::*;

pub(super) fn consume_tincture_ingredient(
    ctx: &ReducerContext,
    inventory_id: u64,
    ingredient_id: u64,
) {
    crate::food::delete_personal_food_lot(ctx, inventory_id);
    ctx.db
        .inventory_item_amount()
        .inventory_item_id()
        .delete(inventory_id);
    ctx.db
        .inventory_containment()
        .child_object_id()
        .delete(ingredient_id);
    ctx.db.inventory_object().id().delete(ingredient_id);
    ctx.db.inventory_item().id().delete(inventory_id);
}
