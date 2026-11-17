//! Read-only carried ancestry: stored fixtures never burden their former owner.

use adventuresim_core::{
    inventory_containers::MAX_CONTAINER_DEPTH, physical_object::InventoryLocation,
};
use spacetimedb::ViewContext;

use super::InventoryProjectionError;
use crate::{
    inventory_container::{InventoryObject, inventory_containment__view, inventory_object__view},
    item::inventory_item__view,
};

pub(super) enum InventoryCarriage {
    Carried,
    StoredAtFireplace,
}

impl InventoryCarriage {
    pub(super) fn for_personal_object(
        ctx: &ViewContext,
        object: &InventoryObject,
        character_id: u64,
    ) -> Result<Self, InventoryProjectionError> {
        let mut cursor = object.clone();
        for _ in 0..=MAX_CONTAINER_DEPTH {
            match &cursor.location {
                InventoryLocation::Fireplace(_) => return Ok(Self::StoredAtFireplace),
                InventoryLocation::Personal(location) if location.character_id == character_id => {
                    let row = ctx
                        .db
                        .inventory_item()
                        .id()
                        .find(location.row_id)
                        .ok_or(InventoryProjectionError::InvalidContainment)?;
                    if row.character_id != character_id
                        || row.item_id != cursor.item_id
                        || row.quantity != 1
                    {
                        return Err(InventoryProjectionError::InvalidContainment);
                    }
                }
                _ => return Err(InventoryProjectionError::InvalidContainment),
            }
            let Some(edge) = ctx
                .db
                .inventory_containment()
                .child_object_id()
                .find(cursor.id)
            else {
                return Ok(Self::Carried);
            };
            cursor = ctx
                .db
                .inventory_object()
                .id()
                .find(edge.parent_object_id)
                .ok_or(InventoryProjectionError::InvalidContainment)?;
        }
        Err(InventoryProjectionError::InvalidContainment)
    }
}
