// Owns admission and measured consumption of planned soap.
fn consume_personal(
    ctx: &ReducerContext,
    stack_id: InventoryItemId,
    points: u32,
) -> Result<(), SoapConsumptionError> {
    let stack = ctx.db.inventory_item().id().find(stack_id.get()).ok_or(
        SoapConsumptionError::MissingStack {
            scope: CarriedInventoryScope::Personal,
            row: stack_id,
        },
    )?;
    if stack.item_id != SOFT_SOAP_ID {
        return Err(SoapConsumptionError::NotSoap {
            scope: CarriedInventoryScope::Personal,
            row: stack_id,
        });
    }
    let fraction = SOAP_FRACTION_PER_CLEANSING_POINT
        .checked_mul(points)
        .ok_or(SoapConsumptionError::PlannedAmountOverflow(
            CarriedInventoryScope::Personal,
        ))?;
    crate::inventory_amount::consume_personal(ctx, stack.id.into(), fraction)?;
    Ok(())
}
fn consume_party(
    ctx: &ReducerContext,
    stack_id: InventoryItemId,
    points: u32,
) -> Result<(), SoapConsumptionError> {
    let stack = ctx
        .db
        .party_inventory_item()
        .id()
        .find(stack_id.get())
        .ok_or(SoapConsumptionError::MissingStack {
            scope: CarriedInventoryScope::Party,
            row: stack_id,
        })?;
    if stack.item_id != SOFT_SOAP_ID {
        return Err(SoapConsumptionError::NotSoap {
            scope: CarriedInventoryScope::Party,
            row: stack_id,
        });
    }
    let fraction = SOAP_FRACTION_PER_CLEANSING_POINT
        .checked_mul(points)
        .ok_or(SoapConsumptionError::PlannedAmountOverflow(
            CarriedInventoryScope::Party,
        ))?;
    crate::inventory_amount::consume_party(ctx, stack.id.into(), fraction)?;
    Ok(())
}

pub fn consume_personal_soap_points(
    ctx: &ReducerContext,
    stack_id: InventoryItemId,
    points: u32,
) -> Result<(), SoapConsumptionError> {
    let required = SOAP_FRACTION_PER_CLEANSING_POINT
        .checked_mul(points)
        .ok_or(SoapConsumptionError::RequestedAmountOverflow)?;
    if crate::inventory_amount::personal_fraction(ctx, stack_id.get()).unwrap_or_default()
        < required
    {
        return Err(SoapConsumptionError::InsufficientSoap(stack_id));
    }
    consume_personal(ctx, stack_id, points)
}
