//! Lookup of the stable physical object behind a carried inventory row.

use adventuresim_core::physical_object::{CarriedInventoryScope, InventoryLocation};
use spacetimedb::{ReducerContext, Table};

use super::{InventoryObject, inventory_object};
use crate::inventory_item;
use crate::strategic::party_inventory_item;

pub(crate) fn object_for_row(
    ctx: &ReducerContext,
    scope: CarriedInventoryScope,
    row_id: u64,
) -> Result<Option<InventoryObject>, String> {
    let is_row_object = |object: &InventoryObject| match (&object.location, scope) {
        (InventoryLocation::Personal(location), CarriedInventoryScope::Personal) => {
            location.row_id == row_id
        }
        (InventoryLocation::Party(location), CarriedInventoryScope::Party) => {
            location.row_id == row_id
        }
        _ => false,
    };
    let row_item_id = match scope {
        CarriedInventoryScope::Personal => ctx
            .db
            .inventory_item()
            .id()
            .find(row_id)
            .map(|row| row.item_id),
        CarriedInventoryScope::Party => ctx
            .db
            .party_inventory_item()
            .id()
            .find(row_id)
            .map(|row| row.item_id),
    };
    // A carried object always shares its backing row's item, so the item index
    // bounds the search. Only a row already consumed before reconciliation
    // requires scanning every object.
    let mut matches: Box<dyn Iterator<Item = InventoryObject>> = match &row_item_id {
        Some(item_id) => Box::new(
            ctx.db
                .inventory_object()
                .item_id()
                .filter(item_id)
                .filter(is_row_object),
        ),
        None => Box::new(ctx.db.inventory_object().iter().filter(is_row_object)),
    };
    let first = matches.next();
    if matches.next().is_some() {
        return Err("Inventory row has duplicate physical object identities".into());
    }
    Ok(first)
}
