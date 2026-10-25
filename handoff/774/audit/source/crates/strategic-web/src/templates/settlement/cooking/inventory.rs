//! Available cooking ingredients and vessels, including the empty state.
use super::*;
#[expect(
    clippy::too_many_arguments,
    reason = "the cooking inventory renders independent custody projections"
)]
pub(super) fn available(
    action_base: &str,
    inventory_scope: &str,
    personal_inventory: &[InventoryItem],
    party_inventory: &[PartyInventoryItem],
    personal_amounts: &[InventoryItemAmount],
    party_amounts: &[PartyItemAmount],
    food_lots: &[FoodLot],
    definitions: &[CatalogItemView],
    instrument: Option<&str>,
) -> Markup {
    html! {
        p class="cooking-inventory-empty small-copy" { "No usable food or vessels here. Check the other inventory or buy ingredients at the market." }
        div data-inventory-browser="cooking-inventory-right" {
            table class="trade-inventory-table" { tbody {
                @if inventory_scope == "personal" {
                    @for item in personal_inventory.iter().filter(|row| row.quantity > 0) {
                        (fireplace_inventory_row(action_base, inventory_scope, item.id, &item.item_id, item.quantity, personal_amounts.iter().find(|a| a.inventory_item_id == item.id).map(|a| a.remaining_fraction_micros), food_lots.iter().find(|l| l.inventory_item_id == Some(item.id)), definitions, instrument))
                    }
                } @else {
                    @for item in party_inventory.iter().filter(|row| row.quantity > 0) {
                        (fireplace_inventory_row(action_base, inventory_scope, item.id, &item.item_id, item.quantity, party_amounts.iter().find(|a| a.party_inventory_item_id == item.id).map(|a| a.remaining_fraction_micros), food_lots.iter().find(|l| l.party_inventory_item_id == Some(item.id)), definitions, instrument))
                    }
                }
            } }
        }
    }
}
