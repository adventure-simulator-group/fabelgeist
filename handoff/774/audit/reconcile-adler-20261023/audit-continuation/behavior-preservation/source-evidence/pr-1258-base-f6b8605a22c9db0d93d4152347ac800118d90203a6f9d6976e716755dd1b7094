//! Authenticated connected-client appearance projections.
use super::*;
pub(crate) fn connected_appearance(
    ctx: &ViewContext,
    inventory_row_id: u64,
    item_id: &str,
) -> Option<ConnectedWeaponAppearance> {
    let mut objects = ctx
        .db
        .inventory_object()
        .item_id()
        .filter(""..)
        .filter(|object| {
            matches!(
                &object.location,
                InventoryLocation::Personal(location) if location.row_id == inventory_row_id
            )
        })
        .filter(|object| object.item_id == item_id);
    let object = objects.next()?;
    if objects.next().is_some() {
        return None;
    }
    let instance = ctx
        .db
        .weapon_instance()
        .physical_object_id()
        .find(object.id)?;
    evaluate_instance(&instance, item_id)?;
    Some(ConnectedWeaponAppearance {
        generator_version: instance.generator_version,
        design_hash: instance.design_hash,
        recipe: instance.recipe,
        mass_kg: instance.mass_grams as f32 / 1_000.0,
        length_m: instance.length_mm as f32 / 1_000.0,
        grip_to_tip_m: instance.grip_to_tip_mm as f32 / 1_000.0,
    })
}

pub(crate) fn connected_holder_appearance(
    ctx: &ViewContext,
    inventory_row_id: u64,
    item_id: &str,
) -> Option<ConnectedWeaponAppearance> {
    if !matches!(item_id, "scabbard" | "weapon_loop") {
        return None;
    }
    let mut objects = ctx
        .db
        .inventory_object()
        .item_id()
        .filter(""..)
        .filter(|object| {
            matches!(
                &object.location,
                InventoryLocation::Personal(location) if location.row_id == inventory_row_id
            )
        })
        .filter(|object| object.item_id == item_id);
    let object = objects.next()?;
    if objects.next().is_some() {
        return None;
    }
    let holder = ctx
        .db
        .weapon_holder_instance()
        .physical_object_id()
        .find(object.id)?;
    evaluate_holder_instance(&holder, item_id)?;
    Some(ConnectedWeaponAppearance {
        generator_version: holder.generator_version,
        design_hash: holder.design_hash,
        recipe: holder.recipe,
        mass_kg: holder.mass_grams as f32 / 1_000.0,
        length_m: holder.length_mm as f32 / 1_000.0,
        grip_to_tip_m: holder.grip_to_tip_mm as f32 / 1_000.0,
    })
}
