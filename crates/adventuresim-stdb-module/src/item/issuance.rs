//! Allocate personal inventory rows and initialize their material authorities.

use super::{
    CatalogItemKind, InventoryGrantError, InventoryItem, Item, inventory_food_definition,
    inventory_item, item,
};
use adventuresim_core::{
    identity::{CharacterId, InventoryItemId},
    inventory_measurement::{InventoryGrantQuantity, InventoryRowAllocation},
    item_catalog::ItemDefinitionId,
};
use spacetimedb::{ReducerContext, Table};

pub(crate) fn inventory_row_allocation(
    definition: Option<&Item>,
    food: bool,
    measured: bool,
) -> InventoryRowAllocation {
    if food
        || measured
        || definition.is_some_and(|definition| {
            definition.repairable
                || definition.kind == CatalogItemKind::Medication
                || (definition.kind == CatalogItemKind::Weapon && definition.melee)
                || definition.container_capacity_ml > 0
                || !definition.attachment_points.is_empty()
        })
    {
        InventoryRowAllocation::Individual
    } else {
        InventoryRowAllocation::Stacked
    }
}

pub(crate) fn add_inventory_item_checked(
    ctx: &ReducerContext,
    character_id: CharacterId,
    item_id: &ItemDefinitionId,
    quantity: InventoryGrantQuantity,
) -> Result<Option<InventoryItemId>, InventoryGrantError> {
    if quantity == InventoryGrantQuantity::Empty {
        return Ok(None);
    }

    let definition = ctx.db.item().id().find(item_id.to_string());
    let kind = definition.as_ref().map(|definition| definition.kind);
    let food_definition = inventory_food_definition(kind, item_id)?;
    let durable = definition
        .as_ref()
        .is_some_and(|definition| definition.repairable);
    let food = food_definition.is_some();
    let measured = crate::inventory_amount::is_measured_item(ctx, item_id);
    // Every food unit is its own non-fungible batch. A partly consumed unit
    // remains quantity one while its authoritative lot mass/value/provenance
    // shrink, so it can never be merged back into fresh stock.
    let allocation = inventory_row_allocation(definition.as_ref(), food, measured);
    let mut first = None;
    for row_quantity in quantity.rows(allocation) {
        let item = ctx.db.inventory_item().insert(InventoryItem {
            id: 0,
            character_id: character_id.into(),
            item_id: item_id.to_string(),
            quantity: row_quantity.get(),
        });
        if allocation == InventoryRowAllocation::Individual {
            crate::inventory_container::insert_personal_object(ctx, &item)?;
        }
        if durable {
            crate::repair::initialize_item_condition(ctx, &item);
        }
        crate::weapon_instance::initialize_personal_weapon(ctx, &item)?;
        if measured {
            crate::inventory_amount::initialize_personal(ctx, item.id);
        }
        if food {
            crate::food::create_personal_food_lot(
                ctx,
                character_id,
                item.id.into(),
                item_id,
                row_quantity,
            )?;
        }
        first.get_or_insert(InventoryItemId::new(item.id));
    }
    if food {
        let _ = crate::capability::refresh_character_capability(ctx, character_id.into());
    }
    Ok(first)
}

/// Foraging receipts bind every concrete harvested unit to an object and
/// material lot. Preserve the shared grant helper's stacking semantics for
/// every other caller while intentionally issuing one validated unit here.
pub(crate) fn add_foraged_inventory_item_checked_rows(
    ctx: &ReducerContext,
    character_id: CharacterId,
    item_id: &ItemDefinitionId,
    quantity: adventuresim_core::foraging::ForageYieldQuantity,
) -> Result<Vec<InventoryItemId>, InventoryGrantError> {
    let mut rows = Vec::with_capacity(quantity.units().len());
    for unit in quantity.units() {
        rows.push(
            add_inventory_item_checked(ctx, character_id, item_id, unit.into())?.ok_or_else(
                || InventoryGrantError::MissingForageRow {
                    character: character_id,
                    item: item_id.clone(),
                },
            )?,
        );
    }
    Ok(rows)
}

pub fn add_inventory_item(
    ctx: &ReducerContext,
    character_id: CharacterId,
    item_id: &ItemDefinitionId,
    quantity: InventoryGrantQuantity,
) -> Option<InventoryItemId> {
    add_inventory_item_checked(ctx, character_id, item_id, quantity)
        .ok()
        .flatten()
}

#[cfg(test)]
mod tests {
    #[test]
    fn kind_aware_insertion_keeps_ingredients_fungible_and_medication_individual() {
        let source = crate::production_source(include_str!("issuance.rs"));
        let checked = source
            .split("pub(crate) fn add_inventory_item_checked")
            .nth(1)
            .and_then(|tail| tail.split("pub fn add_inventory_item").next())
            .expect("checked inventory insertion");
        assert!(checked.contains("inventory_row_allocation(definition.as_ref(), food, measured)"));
        let stable_object_policy = source
            .split("pub(crate) fn inventory_row_allocation")
            .nth(1)
            .and_then(|tail| {
                tail.split("pub(crate) fn add_inventory_item_checked")
                    .next()
            })
            .expect("stable-object policy");
        assert!(stable_object_policy.contains("definition.kind == CatalogItemKind::Medication"));
        assert!(checked.contains("for row_quantity in quantity.rows(allocation)"));
        assert!(stable_object_policy.contains("InventoryRowAllocation::Individual"));
        assert!(stable_object_policy.contains("InventoryRowAllocation::Stacked"));
        assert!(checked.contains("quantity: row_quantity.get()"));
        assert_eq!(
            adventuresim_core::item_catalog::definition(&"tincture_spirit".into())
                .unwrap()
                .kind,
            adventuresim_core::item_catalog::ItemKind::Ingredient
        );
    }

    #[test]
    fn foraging_specific_insertion_issues_each_harvested_unit_separately() {
        let source = crate::production_source(include_str!("issuance.rs"));
        let helper = source
            .split("pub(crate) fn add_foraged_inventory_item_checked_rows")
            .nth(1)
            .and_then(|tail| tail.split("pub fn add_inventory_item").next())
            .expect("foraging-specific insertion helper");
        assert!(helper.contains("for unit in quantity.units()"));
        assert!(
            helper.contains("add_inventory_item_checked(ctx, character_id, item_id, unit.into())")
        );
    }
}
