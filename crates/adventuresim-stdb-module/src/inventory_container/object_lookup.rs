//! Lookup of the stable physical object behind a carried inventory row.

use adventuresim_core::physical_object::{CarriedInventoryScope, InventoryLocation};
use spacetimedb::DbContext;

use super::{InventoryObject, inventory_containment__view, inventory_object__view};
use crate::item::inventory_item__view;
use crate::strategic::party_inventory_item__view;

pub(crate) fn object_for_row(
    ctx: &impl DbContext,
    scope: CarriedInventoryScope,
    row_id: u64,
) -> Result<Option<InventoryObject>, String> {
    let db = ctx.db_read_only();
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
        CarriedInventoryScope::Personal => {
            db.inventory_item().id().find(row_id).map(|row| row.item_id)
        }
        CarriedInventoryScope::Party => db
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
            db.inventory_object()
                .item_id()
                .filter(item_id)
                .filter(is_row_object),
        ),
        None => Box::new(
            db.inventory_object()
                .item_id()
                .filter(""..)
                .filter(is_row_object),
        ),
    };
    let first = matches.next();
    if matches.next().is_some() {
        return Err("Inventory row has duplicate physical object identities".into());
    }
    Ok(first)
}

pub(crate) fn ancestry_reaches_fireplace(ctx: &impl DbContext, object_id: u64) -> bool {
    let db = ctx.db_read_only();
    let mut cursor = Some(object_id);
    for _ in 0..=adventuresim_core::inventory_containers::MAX_CONTAINER_DEPTH {
        let Some(id) = cursor else { return false };
        let Some(object) = db.inventory_object().id().find(id) else {
            return false;
        };
        if object.location.is_fireplace() {
            return true;
        }
        cursor = db
            .inventory_containment()
            .child_object_id()
            .find(id)
            .map(|edge| edge.parent_object_id);
    }
    true
}

pub(crate) fn row_is_fireplace_rooted(
    ctx: &impl DbContext,
    scope: CarriedInventoryScope,
    row_id: u64,
) -> bool {
    object_for_row(ctx, scope, row_id)
        .map(|object| object.is_some_and(|object| ancestry_reaches_fireplace(ctx, object.id)))
        .unwrap_or(true)
}
