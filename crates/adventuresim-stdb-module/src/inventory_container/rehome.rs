//! Change the owner of a row without changing its inventory table.

use super::*;
use adventuresim_core::physical_object::{CustodyCharacterId, CustodyPartyId};

pub(super) fn rehome_personal_row(
    ctx: &ReducerContext,
    row_id: PersonalInventoryRowId,
    character_id: CustodyCharacterId,
) -> Result<InventoryLocation, String> {
    let mut row = ctx
        .db
        .inventory_item()
        .id()
        .find(row_id.get())
        .ok_or("Inventory subtree row is missing")?;
    row.character_id = character_id.get();
    ctx.db.inventory_item().id().update(row);
    Ok(InventoryLocation::personal(character_id.get(), row_id))
}

pub(super) fn rehome_party_row(
    ctx: &ReducerContext,
    row_id: PartyInventoryRowId,
    party_id: &CustodyPartyId,
) -> Result<InventoryLocation, String> {
    let mut row = ctx
        .db
        .party_inventory_item()
        .id()
        .find(row_id.get())
        .ok_or("Party inventory subtree row is missing")?;
    row.party_id = party_id.as_str().into();
    ctx.db.party_inventory_item().id().update(row);
    Ok(InventoryLocation::party(party_id.as_str(), row_id))
}

pub(super) fn transfer_personal_row(
    ctx: &ReducerContext,
    row_id: PersonalInventoryRowId,
    party_id: &CustodyPartyId,
) -> Result<InventoryLocation, String> {
    let source = ctx
        .db
        .inventory_item()
        .id()
        .find(row_id.get())
        .ok_or("Inventory subtree row is missing")?;
    if source.quantity != 1 {
        return Err("Stable inventory objects must be quantity one".into());
    }
    let destination_row =
        ctx.db
            .party_inventory_item()
            .insert(crate::strategic::PartyInventoryItem {
                id: 0,
                party_id: party_id.as_str().into(),
                item_id: source.item_id.clone(),
                quantity: 1,
            });
    let destination_id = PartyInventoryRowId::from_raw(destination_row.id);
    if crate::food::personal_lot(ctx, row_id).is_some() {
        crate::food::move_or_split_to_party(ctx, row_id, destination_id, 1, 1)?;
    }
    if crate::inventory_amount::personal_fraction(ctx, source.id).is_some() {
        crate::inventory_amount::move_personal_to_party(ctx, source.id, destination_row.id)?;
    }
    if let Some(condition) = ctx.db.item_condition().inventory_item_id().find(source.id) {
        ctx.db
            .item_condition()
            .inventory_item_id()
            .delete(source.id);
        ctx.db
            .party_item_condition()
            .insert(crate::strategic::PartyItemCondition {
                party_inventory_item_id: destination_row.id,
                tier_1: condition.tier_1,
                tier_2: condition.tier_2,
                tier_3: condition.tier_3,
                tier_4: condition.tier_4,
                tier_5: condition.tier_5,
            });
    }
    ctx.db.inventory_item().id().delete(source.id);
    Ok(InventoryLocation::party(party_id.as_str(), destination_id))
}

pub(super) fn transfer_party_row(
    ctx: &ReducerContext,
    row_id: PartyInventoryRowId,
    character_id: CustodyCharacterId,
) -> Result<InventoryLocation, String> {
    let source = ctx
        .db
        .party_inventory_item()
        .id()
        .find(row_id.get())
        .ok_or("Party inventory subtree row is missing")?;
    if source.quantity != 1 {
        return Err("Stable inventory objects must be quantity one".into());
    }
    let destination_row = ctx.db.inventory_item().insert(crate::InventoryItem {
        id: 0,
        character_id: character_id.get(),
        item_id: source.item_id.clone(),
        quantity: 1,
    });
    let destination_id = PersonalInventoryRowId::from_raw(destination_row.id);
    if crate::food::party_lot(ctx, row_id).is_some() {
        crate::food::move_or_split_to_personal(ctx, row_id, destination_id, 1, 1)?;
    }
    if crate::inventory_amount::party_fraction(ctx, source.id).is_some() {
        crate::inventory_amount::move_party_to_personal(ctx, source.id, destination_row.id)?;
    }
    if let Some(condition) = ctx
        .db
        .party_item_condition()
        .party_inventory_item_id()
        .find(source.id)
    {
        ctx.db
            .party_item_condition()
            .party_inventory_item_id()
            .delete(source.id);
        ctx.db
            .item_condition()
            .insert(crate::repair::ItemCondition {
                inventory_item_id: destination_row.id,
                tier_1: condition.tier_1,
                tier_2: condition.tier_2,
                tier_3: condition.tier_3,
                tier_4: condition.tier_4,
                tier_5: condition.tier_5,
            });
    }
    ctx.db.party_inventory_item().id().delete(source.id);
    Ok(InventoryLocation::personal(
        character_id.get(),
        destination_id,
    ))
}
