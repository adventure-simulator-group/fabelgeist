// Owns measured alcohol consumption and selected disinfectant admission.
fn consume_stack(
    ctx: &ReducerContext,
    stack: Stack,
    requested_fraction: ConsumableFractionMicros,
) -> Result<ConsumableFractionMicros, AlcoholConsumptionError> {
    let (scope, row_id, available) = match &stack {
        Stack::Party(row) => (
            CarriedInventoryScope::Party,
            row.id,
            crate::inventory_amount::party_fraction(ctx, row.id),
        ),
        Stack::Personal(row) => (
            CarriedInventoryScope::Personal,
            row.id,
            crate::inventory_amount::personal_fraction(ctx, row.id),
        ),
    };
    crate::inventory_container::reconcile_consumed_row(ctx, scope, row_id, false)?;
    let consumed = match stack {
        Stack::Party(row) => {
            crate::inventory_amount::consume_party(ctx, row.id.into(), requested_fraction)?
        }
        Stack::Personal(row) => {
            crate::inventory_amount::consume_personal(ctx, row.id.into(), requested_fraction)?
        }
    };
    if available.is_some_and(|amount| consumed >= amount) {
        crate::inventory_container::reconcile_consumed_row(ctx, scope, row_id, true)?;
    }
    Ok(consumed)
}

pub fn consume_inventory_row(
    ctx: &ReducerContext,
    id: adventuresim_core::identity::InventoryItemId,
) -> Result<(), AlcoholConsumptionError> {
    let row = ctx
        .db
        .inventory_item()
        .id()
        .find(id.get())
        .ok_or(AlcoholConsumptionError::MissingSelectedRow(id))?;
    let definition =
        ctx.db.item().id().find(&row.item_id).ok_or_else(|| {
            AlcoholConsumptionError::MissingDefinition(row.item_id.as_str().into())
        })?;
    let requested = ConsumableFractionMicros::try_from_ratio(
        SURGERY_DISINFECTANT_ML,
        u64::from(definition.alcohol_serving_ml),
    )
    .map_err(AlcoholConsumptionError::InvalidServing)?;
    consume_stack(ctx, Stack::Personal(row), requested)?;
    Ok(())
}
