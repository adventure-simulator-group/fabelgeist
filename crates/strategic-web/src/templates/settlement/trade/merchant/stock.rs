//! Merchant stock and forge controls from an already authorized storefront projection.

use super::*;

pub(super) struct MerchantStockView<'a> {
    pub settlement: &'a SettlementView,
    pub items: &'a [crate::spacetimedb::CatalogItemView],
    pub personal_targets: &'a [InventoryQuantityTarget],
    pub shop: Storefront,
    pub trade_language: f32,
    pub problem_buy_bps: i32,
    pub problem_sell_penalty_bps: i32,
}

impl MerchantStockView<'_> {
    pub fn render(self) -> Markup {
        let Self {
            settlement,
            items,
            personal_targets,
            shop,
            trade_language,
            problem_buy_bps,
            problem_sell_penalty_bps,
        } = self;
        let stocked_items = items
            .iter()
            .filter(|item| shop.stocks_at(settlement, item))
            .collect::<Vec<_>>();

        html! {
            div class=(if matches!(shop, Storefront::Inn) { "service-left-stack" } else { "merchant-stock-stack" }) {
            div class=(if matches!(shop, Storefront::Inn) { "service-inventory-area" } else { "merchant-stock-area" }) {
            @if matches!(shop, Storefront::Weapons) {
                (forge::weapon_forge_controls(settlement))
            }
            @if !matches!(shop, Storefront::Weapons) {
            (sidebar_section(if matches!(shop, Storefront::Herbalist) { "Existing preparations and ingredients" } else if matches!(shop, Storefront::Inn) { "Cooking supplies" } else { "Merchant stock" }, html! {
                div class="smith-wares-scroll" {
                @if stocked_items.is_empty() {
                    (empty_state("No stock is available here.", None, None))
                } @else {
                (trade_inventory_table("merchant-left", if matches!(shop, Storefront::Weapons) { InventoryColumnSet::Weapons } else if matches!(shop, Storefront::Armor) { InventoryColumnSet::Armor } else { InventoryColumnSet::Basic }, false, false, false, html! {
                    @for item in stocked_items.iter().copied() {
                        @let is_currency = item.kind == crate::spacetimedb::CatalogItemKind::Currency;
                        @let intervention = adventuresim_core::physiology::intervention_profile(&item.id, 1);
                        @let buy_price = adventuresim_core::local_problem::adjust_price(adventuresim_core::strategic_economy::language_adjusted_buy_price(
                            adventuresim_core::strategic_economy::merchant_buy_price(item.base_value.unwrap_or(1)),
                            trade_language
                        ), problem_buy_bps);
                        @let sell_price = adventuresim_core::local_problem::adjust_price(adventuresim_core::strategic_economy::language_adjusted_sell_price((item.base_value.unwrap_or(1) as f32 / 1.25).floor().max(1.0) as u32, trade_language), -problem_sell_penalty_bps);
                        @let target = target_quantity(personal_targets, &item.id);
                        @let display_name = item_display_name(&item.id);
                        tr class="trade-inventory-row trade-row-merchant" data-merchant-item=(&item.id) data-merchant-sell-price=(sell_price) data-group-summary="catalog" data-intervention-profile-version=[intervention.map(|profile| profile.version)] { td class="inventory-item-type" { (item_type_icon(&item.id)) } td class="inventory-item-name" { (item_name_with_display(&item.id, &display_name, Some(item))) @if !is_currency { (merchant_buy_controls(&item.id, buy_price, target, 999)) } } td class="inventory-count" hidden { "999" } td class="inventory-weight" { (weight_display(item.weight)) } td class="inventory-gold" { (buy_price) } }
                    }
                }))
                (inventory_footer_controls("buy", "Buy to targets", "Buy everything"))
                @if matches!(shop, Storefront::Herbalist) {
                    p class="small-copy text-muted" { "Purchases go to your personal inventory." }
                }
                }
                }
            }))
            }
            }
            }

        }
    }
}
