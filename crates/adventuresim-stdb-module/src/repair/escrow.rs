use super::*;
use adventuresim_core::physical_object::{
    CarriedInventoryRowId, InventoryLocation, PersonalInventoryRowId,
};

pub(super) fn stage_repair_object(
    ctx: &ReducerContext,
    row_id: PersonalInventoryRowId,
    settlement_id: &str,
) -> crate::StrategicOperationResult<()> {
    let carried = CarriedInventoryRowId::Personal(row_id);
    if let Some(mut object) = crate::inventory_container::object_for_row(ctx, carried)? {
        crate::inventory_container::detach_if_nested(ctx, object.id)?;
        object.location = InventoryLocation::repair(settlement_id, row_id);
        ctx.db.inventory_object().id().update(object);
    } else {
        crate::inventory_container::reconcile_consumed_row(ctx, carried, true)?;
    }
    Ok(())
}

pub(super) fn restore_repair_object(
    ctx: &ReducerContext,
    order: &RepairOrder,
    character_id: u64,
) -> crate::StrategicOperationResult<()> {
    let mut escrow_objects = ctx
        .db
        .inventory_object()
        .iter()
        .filter(|object| {
            matches!(
                &object.location,
                InventoryLocation::Repair(location)
                    if location.row_id.get() == order.inventory_item_id
            )
        })
        .collect::<Vec<_>>();
    if escrow_objects.len() > 1 {
        return Err("Repair escrow has duplicate physical object identities".into());
    }
    if let Some(mut object) = escrow_objects.pop() {
        let correct_escrow = matches!(
            &object.location,
            InventoryLocation::Repair(location)
                if location.settlement_id == order.settlement_id
        );
        if !correct_escrow || object.item_id != order.item_id {
            return Err("Repair escrow physical object does not match its order".into());
        }
        object.location = InventoryLocation::personal(
            character_id,
            PersonalInventoryRowId::from_raw(order.inventory_item_id),
        );
        ctx.db.inventory_object().id().update(object);
    }
    Ok(())
}
