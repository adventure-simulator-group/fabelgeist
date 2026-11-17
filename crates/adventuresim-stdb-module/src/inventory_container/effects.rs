//! Shared containment effects after a live or non-live entry point authenticates.

use adventuresim_core::{
    material::{Microliters, Milliliters},
    physical_object::{CustodyCharacterId, PhysicalObjectId},
};
use spacetimedb::{ReducerContext, Table};

use super::{
    InventoryContainment, InventoryObject, WATER_ITEM_ID, container_liquid, graph,
    inventory_containment, inventory_object, liquid_used, require_mutable,
};
use crate::item::item;

type Result<T> = std::result::Result<T, String>;

pub(crate) struct ContainerInsertion {
    pub(crate) actor: CustodyCharacterId,
    pub(crate) child: PhysicalObjectId,
    pub(crate) parent: PhysicalObjectId,
}

pub(crate) struct ContainedItemRemoval {
    pub(crate) actor: CustodyCharacterId,
    pub(crate) child: PhysicalObjectId,
}

pub(crate) struct ContainerWaterDisposal {
    pub(crate) actor: CustodyCharacterId,
    pub(crate) container: PhysicalObjectId,
    pub(crate) volume: Milliliters,
}

impl ContainerInsertion {
    pub(crate) fn apply(self, ctx: &ReducerContext) -> Result<()> {
        let parent = require_actor_object(ctx, self.actor, self.parent)?;
        let child = require_actor_object(ctx, self.actor, self.child)?;
        let definition = ctx
            .db
            .item()
            .id()
            .find(&parent.item_id)
            .ok_or("Container definition not found")?;
        if definition.container_capacity_ml == 0 {
            return Err("That item has no authored container capability".into());
        }
        if !parent.location.has_same_carried_custody(&child.location) {
            return Err("Move the item to the container owner's inventory first".into());
        }
        // The reducer transaction rolls this detachment back if graph or
        // capacity validation rejects the requested move.
        ctx.db
            .inventory_containment()
            .child_object_id()
            .delete(self.child.get());
        let mut model = graph(ctx)?;
        model
            .insert(self.child, self.parent)
            .map_err(str::to_owned)?;
        let used = model
            .used_volume(self.parent)
            .map_err(str::to_owned)?
            .checked_add(Milliliters::new(liquid_used(ctx, parent.id)))
            .ok_or("Container volume overflow")?;
        if used > Milliliters::new(u64::from(definition.container_capacity_ml)) {
            return Err(format!(
                "Container capacity exceeded: {} ml used of {} ml",
                used.get(),
                definition.container_capacity_ml,
            ));
        }
        ctx.db.inventory_containment().insert(InventoryContainment {
            child_object_id: self.child.get(),
            parent_object_id: self.parent.get(),
        });
        Ok(())
    }
}

impl ContainedItemRemoval {
    pub(crate) fn apply(self, ctx: &ReducerContext) -> Result<()> {
        require_actor_object(ctx, self.actor, self.child)?;
        if !ctx
            .db
            .inventory_containment()
            .child_object_id()
            .delete(self.child.get())
        {
            return Err("Object is not inside a container".into());
        }
        Ok(())
    }
}

impl ContainerWaterDisposal {
    pub(crate) fn apply(self, ctx: &ReducerContext) -> Result<()> {
        if self.volume.get() == 0 {
            return Err("Water amount must be positive".into());
        }
        require_actor_object(ctx, self.actor, self.container)?;
        let mut liquid = ctx
            .db
            .container_liquid()
            .container_object_id()
            .find(self.container.get())
            .ok_or("Container has no water")?;
        if liquid.liquid_item_id != WATER_ITEM_ID {
            return Err("Container does not hold water".into());
        }
        if liquid.water_ml < self.volume.get() {
            return Err("Container does not hold that much water".into());
        }
        crate::outbreak::take_container_water_contributions(
            ctx,
            self.container.get(),
            Microliters::try_from_milliliters(Milliliters::new(liquid.water_ml))
                .map_err(|_| "Container water volume is too large")?,
            Microliters::try_from_milliliters(self.volume)
                .map_err(|_| "Requested water volume is too large")?,
        )?;
        liquid.water_ml -= self.volume.get();
        if liquid.water_ml == 0 {
            ctx.db
                .container_liquid()
                .container_object_id()
                .delete(self.container.get());
        } else {
            ctx.db
                .container_liquid()
                .container_object_id()
                .update(liquid);
        }
        Ok(())
    }
}

fn require_actor_object(
    ctx: &ReducerContext,
    actor: CustodyCharacterId,
    id: PhysicalObjectId,
) -> Result<InventoryObject> {
    let character = crate::character::require_living_character(ctx, actor.get())?;
    let object = ctx
        .db
        .inventory_object()
        .id()
        .find(id.get())
        .ok_or("Inventory object not found")?;
    crate::object_custody::require_actor_carried_object(ctx, &character, &object)?;
    require_mutable(ctx, id.get())?;
    Ok(object)
}
