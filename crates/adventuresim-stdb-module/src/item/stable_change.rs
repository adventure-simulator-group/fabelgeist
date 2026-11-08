//! Individually tracked inventory changes preserve equipment and custody.

use adventuresim_core::{
    inventory_instance::{InstanceQuantityChange, InstanceRemovalCandidate, select_removals},
    physical_object::CarriedInventoryScope,
};
use spacetimedb::ReducerContext;

use super::{add_inventory_item, inventory_item};

struct PlannedRemoval {
    candidate: InstanceRemovalCandidate,
    object_id: u64,
    subtree: Vec<u64>,
    selected_ancestors: usize,
}

fn plan_removals(
    ctx: &ReducerContext,
    candidates: Vec<InstanceRemovalCandidate>,
) -> crate::StrategicOperationResult<Vec<PlannedRemoval>> {
    let mut plans = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let object = crate::inventory_container::object_for_row(
            ctx,
            CarriedInventoryScope::Personal,
            candidate.row_id,
        )?
        .ok_or("Inventory instance has no stable physical object")?;
        plans.push(PlannedRemoval {
            candidate,
            object_id: object.id,
            subtree: crate::inventory_container::subtree_object_ids(ctx, object.id)?,
            selected_ancestors: 0,
        });
    }
    let ancestor_counts: Vec<_> = plans
        .iter()
        .map(|plan| {
            plans
                .iter()
                .filter(|other| {
                    other.object_id != plan.object_id && other.subtree.contains(&plan.object_id)
                })
                .count()
        })
        .collect();
    for (plan, count) in plans.iter_mut().zip(ancestor_counts) {
        plan.selected_ancestors = count;
    }
    plans.sort_by_key(|plan| std::cmp::Reverse(plan.selected_ancestors));
    Ok(plans)
}

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
            let plans = plan_removals(ctx, select_removals(candidates, count))?;
            for plan in plans {
                if plan.candidate.equipped {
                    crate::character::unequip_wearable(ctx, plan.candidate.row_id);
                    equipment_changed = true;
                }
                crate::inventory_container::delete_subtree(ctx, plan.object_id)?;
            }
            if equipment_changed {
                crate::capability::refresh_character_capability(ctx, character_id)?;
                crate::condition::refresh_character_strategic_condition(ctx, character_id)?;
            }
        }
    }
    Ok(())
}
