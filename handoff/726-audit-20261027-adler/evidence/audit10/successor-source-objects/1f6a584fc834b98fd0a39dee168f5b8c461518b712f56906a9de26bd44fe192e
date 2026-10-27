//! Authenticated carried construction mass, including food and fractional stock.
use super::*;
pub(super) fn dry_inventory_weight(ctx: &ReducerContext, character_id: u64) -> f32 {
    ctx.db
        .inventory_item()
        .character_id()
        .filter(character_id)
        .filter_map(|inventory: InventoryItem| {
            if crate::inventory_container::row_is_fireplace_rooted(
                ctx,
                CarriedInventoryScope::Personal,
                inventory.id,
            ) {
                return None;
            }
            if let Some(lot) = ctx
                .db
                .food_lot()
                .iter()
                .find(|lot| lot.inventory_item_id == Some(inventory.id))
            {
                return Some(lot.mass_kg.max(0.0));
            }
            ctx.db.item().id().find(&inventory.item_id).map(|item| {
                let effective_quantity =
                    crate::inventory_amount::personal_fraction(ctx, inventory.id)
                        .map_or(inventory.quantity as f32, |fraction| fraction.as_unit_f32());
                let unit_mass = if adventuresim_weapon_model::default_design(&item.id).is_some() {
                    crate::weapon_instance::combat_geometry(ctx, inventory.id, &item.id)
                        .expect("parametric inventory weapon has valid physical recipe")
                        .mass_kg
                } else {
                    crate::weapon_instance::fitted_holder_mass(ctx, inventory.id, &item.id)
                        .expect("fitted inventory holder has valid physical recipe")
                        .unwrap_or(item.weight)
                };
                unit_mass * effective_quantity
            })
        })
        .sum()
}
