//! Individually tracked inventory changes preserve equipment and custody.

use adventuresim_core::{
    inventory_instance::{InstanceQuantityChange, InstanceRemovalCandidate, select_removals},
    physical_object::CarriedInventoryScope,
};
use spacetimedb::ReducerContext;

use super::{add_inventory_item, inventory_item};

pub(super) fn change_instances(
    ctx: &ReducerContext,
    character_id: u64,
    item_id: &str,
    quantity: i32,
) -> crate::StrategicOperationResult<()> {
    match InstanceQuantityChange::try_from(quantity)? {
        InstanceQuantityChange::Add(count) => {
            add_inventory_item(ctx, character_id, item_id, count)?;
        }
        InstanceQuantityChange::Remove(count) => {
            let instances = ctx
                .db
                .inventory_item()
                .character_and_item_id()
                .filter((character_id, item_id))
                .collect::<Vec<_>>();
            if instances.len() < count as usize {
                return Err("not enough inventory instances to remove".into());
            }
            let candidates = instances
                .into_iter()
                .map(|item| InstanceRemovalCandidate {
                    row_id: item.id,
                    equipped: crate::character::wearable_is_equipped(ctx, item.id),
                })
                .collect();
            let mut equipment_changed = false;
            for candidate in select_removals(candidates, count) {
                if candidate.equipped {
                    crate::character::unequip_wearable(ctx, candidate.row_id);
                    equipment_changed = true;
                }
                if !crate::inventory_container::delete_carried_object_for_row(
                    ctx,
                    CarriedInventoryScope::Personal,
                    candidate.row_id,
                )? {
                    return Err("Inventory instance has no stable physical object".into());
                }
            }
            if equipment_changed {
                crate::capability::refresh_character_capability(ctx, character_id)?;
                crate::condition::refresh_character_strategic_condition(ctx, character_id)?;
            }
        }
    }
    Ok(())
}
