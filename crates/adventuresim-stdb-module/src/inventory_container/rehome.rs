//! Change the owner of a row without changing its inventory table.

use super::{InventoryLocation, PartyInventoryRowId, PersonalInventoryRowId, ReducerContext};
use crate::item::inventory_item;
use crate::strategic::party_inventory_item;
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
