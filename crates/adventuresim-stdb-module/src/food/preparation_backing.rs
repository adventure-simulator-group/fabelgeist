// Resolve the persisted food lot to an actor-owned carried inventory row.
fn view_lot_carried_backing(
    ctx: &ViewContext,
    actor: &crate::Character,
    lot: &FoodLot,
) -> Option<CarriedInventoryRowId> {
    if let Some(row_id) = lot.inventory_item_id
        && ctx
            .db
            .inventory_item()
            .id()
            .find(row_id)
            .is_some_and(|row| row.character_id == actor.id && row.quantity == 1)
    {
        Some(CarriedInventoryRowId::Personal(
            PersonalInventoryRowId::from_raw(row_id),
        ))
    } else if let Some(row_id) = lot.party_inventory_item_id
        && actor.party_id.as_deref().is_some_and(|party_id| {
            ctx.db
                .party_inventory_item()
                .id()
                .find(row_id)
                .is_some_and(|row| row.party_id == party_id && row.quantity == 1)
        })
    {
        Some(CarriedInventoryRowId::Party(PartyInventoryRowId::from_raw(
            row_id,
        )))
    } else {
        None
    }
}
