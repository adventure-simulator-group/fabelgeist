//! Merchant storefront policy, quoted stock, and trade presentation.

mod forge;

#[cfg(test)]
mod tests;

use super::super::service::service_page;
use super::*;
use super::{equipment::*, inventory::*, repairs::*};

/// Web presentation and inventory-panel policy for canonical storefronts.
pub trait StorefrontPresentation {
    fn available_at(self, settlement: &SettlementView) -> bool;
    fn stocks_at(
        self,
        settlement: &SettlementView,
        item: &crate::spacetimedb::CatalogItemView,
    ) -> bool;
    fn title(self) -> &'static str;
    fn stocks(self, item: &crate::spacetimedb::CatalogItemView) -> bool;
    fn shows_inventory(self, item: &crate::spacetimedb::CatalogItemView) -> bool;
}

impl StorefrontPresentation for Storefront {
    fn available_at(self, settlement: &SettlementView) -> bool {
        adventuresim_core::settlement_economy::storefront_available(&settlement.economy, self)
            || (matches!(self, Self::Weapons)
                && adventuresim_core::organization::organization_service_chapter(
                    &settlement.id,
                    self.service_id(),
                )
                .is_some())
    }

    fn stocks_at(
        self,
        settlement: &SettlementView,
        item: &crate::spacetimedb::CatalogItemView,
    ) -> bool {
        let stocked = adventuresim_core::settlement_economy::storefront_stocks(
            &settlement.economy,
            self,
            &item.id,
            item.kind.economy_kind(),
        );
        stocked
            && (!matches!(self, Self::Books)
                || adventuresim_core::item_catalog::definition(&item.id).is_some_and(
                    |definition| {
                        definition.capabilities.book.as_ref().is_some_and(|book| {
                            book.settlement_allowlist.is_empty()
                                || book.settlement_allowlist.contains(&settlement.id)
                        })
                    },
                ))
    }

    fn title(self) -> &'static str {
        match self {
            Self::General => "General Market",
            Self::Weapons => "Weaponsmith",
            Self::Armor => "Armourer",
            Self::Clothing => "Tailor",
            Self::Herbalist => "Herbalist",
            Self::Inn => "The Inn",
            Self::Books => "Bookstore",
        }
    }

    fn stocks(self, item: &crate::spacetimedb::CatalogItemView) -> bool {
        let kind = item.kind;
        match self {
            Self::General => !matches!(
                kind,
                crate::spacetimedb::CatalogItemKind::Currency
                    | crate::spacetimedb::CatalogItemKind::Ingredient
                    | crate::spacetimedb::CatalogItemKind::Medication
            ),
            Self::Weapons => matches!(
                kind,
                crate::spacetimedb::CatalogItemKind::Weapon
                    | crate::spacetimedb::CatalogItemKind::Shield
            ),
            Self::Armor => kind == crate::spacetimedb::CatalogItemKind::Armor,
            Self::Clothing => kind == crate::spacetimedb::CatalogItemKind::Clothing,
            Self::Herbalist => matches!(
                kind,
                crate::spacetimedb::CatalogItemKind::Ingredient
                    | crate::spacetimedb::CatalogItemKind::Medication
            ),
            Self::Inn => {
                adventuresim_core::food::definition(&item.id).is_some()
                    || matches!(
                        item.id.as_str(),
                        "cooking_pan" | "cooking_pot" | "portable_oven"
                    )
            }
            Self::Books => adventuresim_core::item_catalog::definition(&item.id)
                .is_some_and(|definition| definition.capabilities.book.is_some()),
        }
    }

    fn shows_inventory(self, item: &crate::spacetimedb::CatalogItemView) -> bool {
        item.kind == crate::spacetimedb::CatalogItemKind::Currency || self.stocks(item)
    }
}

pub fn merchants_page(
    settlement: &SettlementView,
    active_character: Option<&CharacterView>,
    inventory: &[InventoryItem],
    food_lots: &[FoodLot],
    party_members: &[CharacterView],
    logged_in_as: Option<&str>,
) -> Markup {
    service_page(
        settlement,
        "merchants",
        "Market Square",
        "Market Steward",
        "The market steward has no listed stock at present.",
        active_character,
        inventory,
        &[],
        food_lots,
        party_members,
        logged_in_as,
        None,
        None,
        SoapRestPreview::default(),
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "the merchant page boundary composes independently loaded stock, custody, and service projections"
)]
pub fn live_merchant_shop_page(
    settlement: &SettlementView,
    character: &CharacterView,
    inventory: &[InventoryItem],
    personal_amounts: &[InventoryItemAmount],
    items: &[crate::spacetimedb::CatalogItemView],
    food_lots: &[FoodLot],
    party_members: &[CharacterView],
    equip: Option<&CharacterEquipmentGraph>,
    personal_targets: &[InventoryQuantityTarget],
    party_targets: &[InventoryQuantityTarget],
    pooled: &[PartyInventoryItem],
    shop: Storefront,
    shared_language: f32,
    problem_buy_bps: i32,
    problem_sell_penalty_bps: i32,
    conditions: &[crate::spacetimedb::ItemCondition],
    smith: Option<&crate::spacetimedb::SettlementSmith>,
    repair_orders: &[crate::spacetimedb::RepairOrder],
    now_minutes: adventuresim_world_schema::calendar::StrategicMinute,
    personal_encumbrance: EncumbranceSummary,
    party_encumbrance: EncumbranceSummary,
    rest_default_minutes: Option<u64>,
    soap_preview: SoapRestPreview,
) -> Markup {
    let title = shop.title();
    let service_id = shop.service_id();
    // Herbalist purchases use a separate reducer and retain their specialized quote.
    let trade_language = if matches!(shop, Storefront::Herbalist) {
        1.0
    } else {
        shared_language
    };
    let smith_skill = smith
        .map(|smith| {
            if matches!(shop, Storefront::Armor) {
                smith.armourer_skill
            } else if matches!(shop, Storefront::Clothing) {
                smith.tailor_skill
            } else {
                smith.weaponsmith_skill
            }
        })
        .unwrap_or(0);
    let player_footer = if matches!(shop, Storefront::Herbalist | Storefront::Weapons) {
        html! {}
    } else {
        inventory_footer_controls_with_leading(
            matches!(
                shop,
                Storefront::Weapons | Storefront::Armor | Storefront::Clothing
            )
            .then(|| repair_all_control(settlement, service_id)),
            "sell",
            "Sell surplus",
            "Sell everything",
        )
    };
    let stocked_items = items
        .iter()
        .filter(|item| shop.stocks_at(settlement, item))
        .collect::<Vec<_>>();
    let content = html! {
        aside class=(if matches!(shop, Storefront::Inn) { "left-sidebar smith-wares-column service-left-sidebar" } else { "left-sidebar smith-wares-column" }) {
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
                p class="small-copy text-muted" { "Pre-existing preparations are sold into personal inventory for versioned administration. Physiology does not craft them; #214 owns preparation." }
            }
            }
            }
        }))
        }
        }
        @if matches!(shop, Storefront::Inn) {
            section class="inn-rest-panel" aria-label="Inn lodging and rest" {
                (rest_service_menu("Inn", &settlement.id, RestServiceKind::Inn, rest_default_minutes, None, soap_preview))
            }
        }
        }
        @if matches!(shop, Storefront::Weapons | Storefront::Armor | Storefront::Clothing) {
            (repair_custody_panel(settlement, shop, repair_orders, conditions, items, now_minutes, smith_skill))
        }
        }
        main class="center-content settlement-main" { (party_portrait_overlay(party_members, Some(character), &crate::location_urls::patterns::SETTLEMENT.url([&settlement.id]), None)) (npc_portrait_strip(&settlement.id, npc_location_id(service_id))) @if matches!(shop, Storefront::Weapons) { (forge_description_stage(title, "Forge preview loading")) } @else { (npc_description_stage(title, "Merchant counter and attending craftsperson")) } (settlement_resident_chat_area(title, Some(character), &settlement.id, npc_location_id(service_id), Some(service_id))) form # "merchant-offer" class="party-offer" action=(if matches!(shop, Storefront::Herbalist) { crate::location_urls::patterns::PURCHASE_FROM_HERBALIST.url([&settlement.id]) } else { crate::location_urls::patterns::FINALIZE_MERCHANT_OFFER.url([&settlement.id, &(crate::location_urls::service_place(service_id).id())]) }) method="post" hidden role="dialog" aria-modal="true" aria-label="Confirm merchant offer" tabindex="-1" { span class="party-offer-summary" { "Review and submit the staged trade." } input type="hidden" name="return_to" value=(crate::location_urls::service_path(&settlement.id, service_id)); input type="hidden" name="inventory_scope" value="player"; button type="button" class="party-offer-cancel" data-cancel-trade="merchant" { "Cancel" } button type="submit" disabled { "Offer" } } }
        aside class="right-sidebar inventory-owner-panel" data-inventory-tabs {
            nav class="inventory-owner-tabs" aria-label="Trading inventory" {
                button type="button" class="inventory-owner-tab active" data-inventory-tab="player" { "Player" }
                @if !matches!(shop, Storefront::Herbalist | Storefront::Weapons) {
                    button type="button" class="inventory-owner-tab" data-inventory-tab="party" { "Party" }
                }
            }
            div data-inventory-pane="player" {
            div class="sidebar-section" {
                (encumbrance_inventory_rail(html! {
                (trade_inventory_table("merchant-player-right", if matches!(shop, Storefront::Armor) { InventoryColumnSet::Armor } else { InventoryColumnSet::Basic }, true, !matches!(shop, Storefront::Weapons), matches!(shop, Storefront::Armor | Storefront::Clothing), html! {
                    @for item in inventory.iter().filter(|item| items.iter().find(|definition| definition.id == item.item_id).is_some_and(|definition| if matches!(shop, Storefront::Weapons) { matches!(definition.id.as_str(), "steel_stock" | "leather_stock" | "brass_stock" | "wood_stock") } else { shop.shows_inventory(definition) })) {
                        @let definition = items.iter().find(|definition| definition.id == item.item_id);
                        @let food_lot = food_lots.iter().find(|lot| lot.inventory_item_id == Some(item.id));
                        @let food_display_name = food_lot.map_or_else(|| item_display_name(&item.item_id), |lot| lot.display_name.clone());
                        @let is_currency = definition.is_some_and(|definition| definition.kind == crate::spacetimedb::CatalogItemKind::Currency);
                        @let is_equipped = equip.is_some_and(|equip| equip.contains(item.id));
                        @let sell_price = adventuresim_core::local_problem::adjust_price(adventuresim_core::strategic_economy::language_adjusted_sell_price(merchant_inventory_sell_price(definition, food_lot), trade_language), -problem_sell_penalty_bps);
                        @let target = target_quantity(personal_targets, &item.item_id);
                        @let measured_fraction = personal_amounts.iter().find(|amount| amount.inventory_item_id == item.id).map(|amount| adventuresim_core::inventory_measurement::ConsumableFractionMicros::try_new(amount.remaining_fraction_micros).expect("public consumable fraction must not exceed one whole"));
                        tr class="trade-inventory-row trade-row-player" data-merchant-item=(&item.item_id) data-personal-inventory-id=(item.id) data-merchant-equipped=(is_equipped) data-inventory-quantity=(item.quantity) data-target=(target) {
                        @let condition = conditions.iter().find(|condition| condition.inventory_item_id == item.id);
                        @let repair_skill = smith_skill;
                        @let durable_item = definition.is_some_and(|definition| definition.repairable);
                        @let service_matches = definition.is_some_and(|definition| if matches!(shop, Storefront::Armor) { definition.kind == crate::spacetimedb::CatalogItemKind::Armor } else if matches!(shop, Storefront::Clothing) { definition.kind == crate::spacetimedb::CatalogItemKind::Clothing } else { matches!(definition.kind, crate::spacetimedb::CatalogItemKind::Weapon | crate::spacetimedb::CatalogItemKind::Shield) });
                        @let can_sell = !is_currency && !is_equipped;
                        td class="inventory-item-type" { (item_type_icon(&item.item_id)) }
                        td class="inventory-item-name" { (item_name_with_food_lot(&item.item_id, &food_display_name, definition, food_lot)) @if !matches!(shop, Storefront::Herbalist | Storefront::Weapons) && (can_sell || service_matches) { (merchant_sell_action_controls(item.id, &item.item_id, sell_price, item.quantity, target, can_sell)) } }
                        td class="inventory-count" { @if matches!(shop, Storefront::Weapons) { (format!("{:.3} kg", measured_fraction.map_or(0.0, adventuresim_core::inventory_measurement::ConsumableFractionMicros::as_unit_f32) * definition.map_or(0.0, |definition| definition.weight))) } @else { (quantity_target_control(item.quantity, target, &item.item_id, false)) } } td class="inventory-equipped" { (equipment_control(item, definition, is_equipped, true, equip)) } td class="inventory-durability" { @if durable_item { (condition_bar(condition, service_matches.then_some(repair_skill))) } @else { "—" } } td class="inventory-weight" { (merchant_inventory_weight(definition, food_lot)) } td class="inventory-gold" { (sell_price) }
                        @if matches!(shop, Storefront::Armor | Storefront::Clothing) {
                            td class="inventory-repair-cell" { @if service_matches { (repair_submit_control(settlement, service_id, item.id, condition, repair_skill)) } }
                        }
                    }}
                    @for target in personal_targets.iter().filter(|target| target.quantity > 0 && !inventory.iter().any(|item| item.item_id == target.item_id) && items.iter().find(|definition| definition.id == target.item_id).is_some_and(|definition| shop.shows_inventory(definition))) {
                        @let definition = items.iter().find(|definition| definition.id == target.item_id);
                        tr class="trade-inventory-row trade-row-player" data-merchant-item=(&target.item_id) data-inventory-quantity="0" data-target=(target.quantity) {
                            td class="inventory-item-type" { (item_type_icon(&target.item_id)) }
                            td class="inventory-item-name" { (item_name_with_quality(&target.item_id, definition)) }
                            td class="inventory-count" { (quantity_target_control(0, target.quantity, &target.item_id, false)) }
                            td class="inventory-equipped" {
                                span class="equipment-unavailable" role="img" tabindex="0"
                                    aria-label="No equipment in this row"
                                    data-strategic-tooltip="No equipment is available in this row" {}
                            }
                            td class="inventory-durability" { "—" }
                            td class="inventory-weight" { (item_weight(definition)) }
                            td class="inventory-gold" { (item_value(definition)) }
                            @if matches!(shop, Storefront::Armor | Storefront::Clothing) { td class="inventory-repair-cell" {} }
                        }
                    }
                }))
                }, player_footer, personal_encumbrance))
            }
            }
            @if !matches!(shop, Storefront::Herbalist | Storefront::Weapons) { div data-inventory-pane="party" hidden {
            div class="sidebar-section" {
                (encumbrance_inventory_rail(html! {
                (trade_inventory_table("merchant-party-right", if matches!(shop, Storefront::Weapons) { InventoryColumnSet::Weapons } else if matches!(shop, Storefront::Armor) { InventoryColumnSet::Armor } else { InventoryColumnSet::Basic }, true, false, false, html! {
                    @for item in pooled.iter().filter(|item| items.iter().find(|definition| definition.id == item.item_id).is_some_and(|definition| shop.shows_inventory(definition))) {
                        @let definition = items.iter().find(|definition| definition.id == item.item_id);
                        @let food_lot = food_lots.iter().find(|lot| lot.party_inventory_item_id == Some(item.id));
                        @let food_display_name = food_lot.map_or_else(|| item_display_name(&item.item_id), |lot| lot.display_name.clone());
                        @let is_currency = definition.is_some_and(|definition| definition.kind == crate::spacetimedb::CatalogItemKind::Currency);
                        @let sell_price = adventuresim_core::local_problem::adjust_price(adventuresim_core::strategic_economy::language_adjusted_sell_price(merchant_inventory_sell_price(definition, food_lot), trade_language), -problem_sell_penalty_bps);
                        @let target = target_quantity(party_targets, &item.item_id);
                        tr class="trade-inventory-row trade-row-player" data-merchant-item=(&item.item_id) data-party-inventory-id=(item.id) data-inventory-quantity=(item.quantity) data-target=(target) {
                            td class="inventory-item-type" { (item_type_icon(&item.item_id)) }
                            td class="inventory-item-name" { (item_name_with_food_lot(&item.item_id, &food_display_name, definition, food_lot)) @if !is_currency { (merchant_sell_controls(item.id, &item.item_id, sell_price, item.quantity, target)) } }
                            td class="inventory-count" { (quantity_target_control(item.quantity, target, &item.item_id, true)) }
                            td class="inventory-weight" { (merchant_inventory_weight(definition, food_lot)) }
                            td class="inventory-gold" { (sell_price) }
                        }
                    }
                    // Party purchases may spend pooled coin first and the active
                    // character's coin second. Show both funding sources as the
                    // same collapsed Coin row in this scope.
                    @for item in inventory.iter().filter(|item| items.iter().find(|definition| definition.id == item.item_id).is_some_and(|definition| definition.kind == crate::spacetimedb::CatalogItemKind::Currency)) {
                        @let definition = items.iter().find(|definition| definition.id == item.item_id);
                        tr class="trade-inventory-row trade-row-player party-personal-currency" data-merchant-item=(&item.item_id) data-inventory-quantity=(item.quantity) data-target="0" title="Personal coin available for party purchases" {
                            td class="inventory-item-type" { (item_type_icon(&item.item_id)) }
                            td class="inventory-item-name" { (item_name_with_quality(&item.item_id, definition)) }
                            td class="inventory-count" { (item.quantity) }
                            td class="inventory-weight" { (item_weight(definition)) }
                            td class="inventory-gold" { (item_value(definition)) }
                        }
                    }
                    @for target in party_targets.iter().filter(|target| target.quantity > 0 && !pooled.iter().any(|item| item.item_id == target.item_id) && items.iter().find(|definition| definition.id == target.item_id).is_some_and(|definition| shop.shows_inventory(definition))) {
                        @let definition = items.iter().find(|definition| definition.id == target.item_id);
                        tr class="trade-inventory-row trade-row-player" data-merchant-item=(&target.item_id) data-inventory-quantity="0" data-target=(target.quantity) {
                            td class="inventory-item-type" { (item_type_icon(&target.item_id)) }
                            td class="inventory-item-name" { (item_name_with_quality(&target.item_id, definition)) }
                            td class="inventory-count" { (quantity_target_control(0, target.quantity, &target.item_id, true)) }
                            td class="inventory-weight" { (item_weight(definition)) }
                            td class="inventory-gold" { (item_value(definition)) }
                        }
                    }
                }))
                }, inventory_footer_controls("sell", "Sell surplus", "Sell everything"), party_encumbrance))
            }
            }
            }
        }
    };
    settlement_layout_with_session(
        title,
        &settlement.name,
        &settlement.id,
        &settlement.category,
        service_id,
        Some(&settlement.religion_id),
        Some(&settlement.economy),
        content,
        Some(&character.name),
    )
}

pub(super) fn merchant_inventory_weight(
    definition: Option<&crate::spacetimedb::CatalogItemView>,
    food_lot: Option<&FoodLot>,
) -> String {
    food_lot.map_or_else(
        || item_weight(definition),
        |lot| weight_display(lot.mass_kg),
    )
}

pub(super) fn merchant_inventory_sell_price(
    definition: Option<&crate::spacetimedb::CatalogItemView>,
    food_lot: Option<&FoodLot>,
) -> u32 {
    food_lot.map_or_else(
        || {
            definition.map_or(0, |definition| {
                adventuresim_core::strategic_economy::merchant_sell_price(
                    definition.base_value.unwrap_or(1),
                )
            })
        },
        |lot| {
            adventuresim_core::strategic_economy::merchant_sell_food_lot_value(lot.total_value)
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(0)
        },
    )
}
pub(super) fn merchant_buy_controls(
    item_id: &str,
    price: u32,
    target: u32,
    available: u32,
) -> Markup {
    let item_name = item_display_name(item_id);
    html! { span class="inventory-row-actions" {
        button type="button" class="trade-transfer trade-transfer-right" data-dynamic-transfer data-default-transfer-mode="one" data-merchant-buy=(item_id) data-merchant-buy-price=(price) data-transfer-mode="one" data-target=(target) data-count=(available) data-label-one=(format!("Buy one {item_name}")) data-label-target=(format!("Buy {item_name} to target")) data-label-all=(format!("Buy all {item_name}")) aria-label=(format!("Buy one {item_name}")) title=(format!("Buy one {item_name}")) { (transfer_glyph(1)) }
    } }
}

pub(super) fn merchant_sell_controls(
    id: u64,
    item_id: &str,
    price: u32,
    quantity: u32,
    target: u32,
) -> Markup {
    let item_name = item_display_name(item_id);
    html! { span class="inventory-row-actions" {
        button type="button" class="trade-transfer trade-transfer-left" data-dynamic-transfer data-default-transfer-mode="one" data-merchant-sell=(id) data-item-name=(item_id) data-merchant-sell-price=(price) data-transfer-mode="one" data-count=(quantity) data-target=(target) data-label-one=(format!("Sell one {item_name}")) data-label-target=(format!("Sell surplus {item_name}")) data-label-all=(format!("Sell all {item_name}")) aria-label=(format!("Sell one {item_name}")) title=(format!("Sell one {item_name}")) { (transfer_glyph(1)) }
    } }
}
