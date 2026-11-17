//! Move personal stock or an entire physical subtree between party members.

use adventuresim_core::{
    inventory_measurement::{ConsumableFractionMicros, ItemQuantity},
    physical_object::{CarriedInventoryScope, InventoryLocation, OperationalCustody},
};
use spacetimedb::{ReducerContext, Table};

use crate::{Character, InventoryItem, inventory_amount::inventory_item_amount, inventory_item, item::item};

pub(crate) struct PersonalInventoryTransfer {
    pub(crate) source: InventoryItem,
    pub(crate) recipient: Character,
    pub(crate) quantity: ItemQuantity,
}

type Result<T> = std::result::Result<T, String>;

impl PersonalInventoryTransfer {
    pub(crate) fn apply(self, ctx: &ReducerContext) -> Result<()> {
        let from = crate::character::require_living_character(ctx, self.source.character_id)?;
        let to = crate::character::require_living_character(ctx, self.recipient.id)?;
        if from.id == to.id || from.party_id.is_none() || from.party_id != to.party_id {
            return Err("Inventory transfers require different members of the same party".into());
        }
        if self.source.quantity < self.quantity.get() {
            return Err("Source character does not have that quantity".into());
        }
        if crate::character::inventory_item_is_equipped(ctx, from.id, self.source.id) {
            return Err("Unequip an item before transferring it".into());
        }
        let definition = ctx.db.item().id().find(&self.source.item_id)
            .ok_or("Inventory item has no catalog definition")?;
        let food = crate::food::personal_lot(ctx, self.source.id).is_some()
            || adventuresim_core::food::definition(&definition.id).is_some();
        let amount = ctx.db.inventory_item_amount().inventory_item_id().find(self.source.id);
        if let Some(amount) = &amount {
            ConsumableFractionMicros::try_new(amount.remaining_fraction_micros)
                .map_err(|_| "Remaining inventory fraction exceeds one whole unit")?;
        }
        let measured = amount.is_some() || crate::inventory_amount::is_measured_definition(&definition);
        let object = crate::inventory_container::object_for_row(ctx, CarriedInventoryScope::Personal, self.source.id)?;
        if let Some(object) = object {
            if self.quantity != ItemQuantity::ONE || self.source.quantity != 1
                || object.location != InventoryLocation::personal(from.id, self.source.id)
                || object.item_id != self.source.item_id
            {
                return Err("A physical object must move as one exact owned inventory row".into());
            }
            crate::inventory_container::detach_if_nested(ctx, object.id)?;
            let destination = OperationalCustody::character(to.id).map_err(|error| error.to_string())?;
            crate::inventory_container::rehome_subtree(ctx, object.id, &destination)?;
        } else {
            if crate::item::requires_stable_object(Some(&definition), food, measured) {
                return Err("Inventory item is missing its required physical identity".into());
            }
            self.move_counted(ctx, to.id);
        }
        crate::capability::refresh_character_capability(ctx, from.id)?;
        crate::capability::refresh_character_capability(ctx, to.id)?;
        Ok(())
    }

    fn move_counted(self, ctx: &ReducerContext, recipient: u64) {
        let destination = ctx.db.inventory_item().character_and_item_id()
            .filter((recipient, &self.source.item_id)).next();
        let merged = destination.as_ref().and_then(|row| row.quantity.checked_add(self.quantity.get()));
        if self.source.quantity == self.quantity.get() {
            ctx.db.inventory_item().id().delete(self.source.id);
        } else {
            let mut source = self.source.clone();
            source.quantity -= self.quantity.get();
            ctx.db.inventory_item().id().update(source);
        }
        if let (Some(mut destination), Some(quantity)) = (destination, merged) {
            destination.quantity = quantity;
            ctx.db.inventory_item().id().update(destination);
        } else {
            // A full destination stack starts another counted row; no unit is
            // discarded or squeezed into an overflowing storage quantity.
            ctx.db.inventory_item().insert(InventoryItem {
                id: 0, character_id: recipient, item_id: self.source.item_id,
                quantity: self.quantity.get(),
            });
        }
    }
}
