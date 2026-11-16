//! Read-only inventory inspection from the active character's authorized rows.

use maud::{Markup, html};

use adventuresim_core::inventory_measurement::ConsumableFractionMicros;

use crate::spacetimedb::{CatalogItemView, FoodLot, InventoryItem, InventoryItemAmount};
use crate::templates::inventory_browser::{InventoryBrowser, InventoryColumnSet};
use crate::templates::{item_display_name, item_type_icon};

use super::trade::item_name_with_food_lot;

pub(super) fn personal_inventory(
    inventory: &[InventoryItem],
    amounts: &[InventoryItemAmount],
    food_lots: &[FoodLot],
    definitions: &[CatalogItemView],
) -> Markup {
    InventoryBrowser {
        namespace: "inspection-personal",
        show_quantities: true,
        show_equipped: false,
        show_condition: false,
        optional_columns: InventoryColumnSet::All,
        rows: html! {
            @for item in inventory {
                @let definition = definitions.iter().find(|definition| definition.id == item.item_id);
                @let food = food_lots.iter().find(|lot| lot.inventory_item_id == Some(item.id));
                @let name = food.map_or_else(|| item_display_name(&item.item_id), |lot| lot.display_name.clone());
                @let fraction = amounts.iter().find(|amount| amount.inventory_item_id == item.id)
                    .map(|amount| ConsumableFractionMicros::try_new(amount.remaining_fraction_micros)).transpose();
                tr class="trade-inventory-row" {
                    td class="inventory-item-type" { (item_type_icon(&item.item_id)) }
                    td class="inventory-item-name" {
                        (item_name_with_food_lot(&item.item_id, &name, definition, food))
                    }
                    td class="inventory-count" {
                        @match fraction {
                            Ok(Some(amount)) => (amount.as_unit_f32()),
                            Ok(None) => (item.quantity),
                            Err(_) => "Unavailable",
                        }
                    }
                    td class="inventory-target" { "—" }
                    td class="inventory-weight" { @if let Some(food) = food { (food.mass_kg) } @else if let Some(definition) = definition { (definition.weight) } @else { "—" } }
                    td class="inventory-gold" { @if let Some(food) = food { (food.total_value) } @else if let Some(value) = definition.and_then(|item| item.base_value) { (value) } @else { "—" } }
                    td class="inventory-actions-cell" {}
                }
            }
        },
    }.render()
}
