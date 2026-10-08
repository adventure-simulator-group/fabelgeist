use super::*;
use crate::spacetimedb::*;
use crate::templates::settlement::test_support::*;
use adventuresim_core::equipment::EncumbranceSummary;

#[test]
fn inferred_general_blacksmith_exposes_limited_weapon_and_armor_stock() {
    let industries = adventuresim_world_schema::InferredIndustryProfile::new(vec![
        adventuresim_world_schema::IndustryEvidence::Fallback(
            adventuresim_world_schema::FallbackIndustry::CommonAggregate,
        ),
    ])
    .unwrap();
    let economy =
        adventuresim_world_schema::infer_settlement_economy(2, 500, 1, false, &industries).unwrap();

    assert!(adventuresim_core::settlement_economy::storefront_stocks(
        &economy,
        adventuresim_core::settlement_economy::Storefront::Weapons,
        "club",
        adventuresim_core::settlement_economy::CatalogKind::Weapon,
    ));
    assert!(adventuresim_core::settlement_economy::storefront_stocks(
        &economy,
        adventuresim_core::settlement_economy::Storefront::Armor,
        "leather vest",
        adventuresim_core::settlement_economy::CatalogKind::Armor,
    ));
    for category in [
        adventuresim_world_schema::StockCategory::Weapons,
        adventuresim_world_schema::StockCategory::Armor,
    ] {
        assert_eq!(
            economy
                .stock
                .iter()
                .find(|stock| stock.category == category)
                .unwrap()
                .abundance,
            1
        );
    }
}

#[test]
fn authored_weapons_service_chapter_exposes_forge_without_economy_storefront() {
    let mut guildhall = settlement();
    guildhall.id = "viabundus-0".into();
    guildhall.economy = adventuresim_world_schema::SettlementEconomyProfile::stage_placeholder();
    assert!(Storefront::Weapons.available_at(&guildhall));

    guildhall.id = "settlement-without-weapons-chapter".into();
    assert!(!Storefront::Weapons.available_at(&guildhall));
}

#[test]
fn forge_client_restores_live_recipe_and_exposes_manual_preview_controls() {
    let renderer = include_str!("../../../../../static/strategic-renderer.js");
    assert!(renderer.contains("strategic-live-regions-refreshed"));
    assert!(renderer.contains("currentForgeDesign"));
    assert!(renderer.contains("wasm_weapon_editor_fields"));
    assert!(!renderer.contains("numericBounds"));
    assert!(renderer.contains(r#"type: "orbit-forge""#));
    assert!(renderer.contains(r#"type: "zoom-forge""#));

    let dialogue = include_str!("../../../../../static/dialogue-client.js");
    assert!(dialogue.contains("data-repair-custody-service"));
    assert!(dialogue.contains("npc.service_id !== panel.dataset.repairCustodyService"));

    let css = include_str!("../../../../../static/css/strategic.css");
    assert!(css.contains(".settlement-main:has(.forge-description-stage)"));
    assert!(css.contains("background: transparent"));
    assert!(css.contains(".repair-custody-panel[hidden] { display: none; }"));
}

#[test]
fn merchant_food_quote_and_weight_follow_remaining_lot() {
    let mut lot = FoodLot {
        id: 1,
        inventory_item_id: Some(9),
        party_inventory_item_id: None,
        material_revision: 1,
        display_name: "Roasted venison".into(),
        preparation: FoodPreparation::Stewed,
        ingredient_item_ids: vec!["raw_venison".into()],
        ingredient_quantities: vec![1.0],
        salty_kg: 0.0,
        spicy_kg: 0.0,
        sweet_kg: 0.0,
        sour_kg: 0.0,
        savory_kg: 0.36,
        quality: 3,
        mass_kg: 25.0,
        nutrition_kcal: 5_000.0,
        total_value: 10.0,
        created_at_minute: adventuresim_stdb_client::StrategicMinute { minutes: 1 },
    };
    assert_eq!(merchant_inventory_weight(None, Some(&lot)), "25");
    assert_eq!(merchant_inventory_sell_price(None, Some(&lot)), 8);
    let rendered =
        item_name_with_food_lot("cooked_meal", &lot.display_name, None, Some(&lot)).into_string();
    assert!(rendered.contains("Roasted venison"));
    assert!(rendered.contains("item-quality-3"));
    assert!(rendered.contains("title=\"Quality 3\""));
    assert!(!rendered.contains("munition grade"));
    lot.mass_kg = 6.25;
    lot.total_value = 2.5;
    assert_eq!(merchant_inventory_weight(None, Some(&lot)), "6.25");
    assert_eq!(merchant_inventory_sell_price(None, Some(&lot)), 2);
    lot.total_value = 0.5;
    let zero = merchant_inventory_sell_price(None, Some(&lot));
    assert_eq!(zero, 0);
    assert_eq!(
        adventuresim_core::strategic_economy::language_adjusted_sell_price(zero, 0.0),
        0
    );
}

#[test]
fn herbalist_stock_template_includes_every_prepared_course_and_ingredients() {
    let ingredient = crate::spacetimedb::CatalogItemView {
        kind: CatalogItemKind::Ingredient,
        ..Default::default()
    };
    let medication = crate::spacetimedb::CatalogItemView {
        kind: CatalogItemKind::Medication,
        ..Default::default()
    };
    assert!(Storefront::Herbalist.stocks(&ingredient));
    assert!(Storefront::Herbalist.stocks(&medication));
    let apple = crate::spacetimedb::CatalogItemView {
        id: "apple".into(),
        kind: CatalogItemKind::Food,
        ..Default::default()
    };
    let pan = crate::spacetimedb::CatalogItemView {
        id: "cooking_pan".into(),
        ..Default::default()
    };
    let honey = crate::spacetimedb::CatalogItemView {
        id: "honey".into(),
        kind: CatalogItemKind::Ingredient,
        ..Default::default()
    };
    assert!(Storefront::Inn.stocks(&apple));
    assert!(Storefront::Inn.stocks(&honey));
    assert!(Storefront::Inn.stocks(&pan));
    assert!(!Storefront::Inn.stocks(&medication));
    assert!(!adventuresim_core::physiology::INTERVENTION_PROFILES.is_empty());
    let definition = crate::spacetimedb::CatalogItemView {
        id: "black_death_tonic".into(),
        kind: CatalogItemKind::Medication,
        ..Default::default()
    };
    let rendered =
        item_name_with_display("black_death_tonic", "Black Death tonic", Some(&definition))
            .into_string();
    assert!(rendered.contains("data-item-name=\"black_death_tonic\""));
    assert!(rendered.contains("data-item-kind=\"medication\""));
    assert!(rendered.contains(">Black Death tonic</span>"));
}

#[test]
fn merchant_tabs_render_personal_and_party_encumbrance_as_applicable() {
    let character = CharacterView {
        id: 1,
        name: "Trader".into(),
        xp: 0,
        level: 1,
        current_settlement_id: Some("viabundus-1".into()),
        current_case_site_id: None,
        party_id: Some("party".into()),
        age_years: 20,
        alive: true,
        temporary: false,
        social_notification_count: 0,
        automatic_social_chat_enabled: false,
    };
    let render = |shop| {
        live_merchant_shop_page(
            &settlement(),
            &character,
            &[],
            &[],
            &[],
            &[],
            &[],
            None,
            &[],
            &[],
            &[],
            shop,
            1.0,
            0,
            0,
            &[],
            None,
            &[],
            adventuresim_world_schema::calendar::StrategicMinute::ZERO,
            EncumbranceSummary::new(10.0, 100.0),
            EncumbranceSummary::new(30.0, 200.0),
            None,
            SoapRestPreview::default(),
        )
        .into_string()
    };
    let merchant = render(Storefront::Weapons);
    assert!(merchant.contains("data-bevy-scene=\"forge\""));
    assert!(merchant.contains("data-forge-customization"));
    assert!(merchant.contains("action=\"/locations/settlement/viabundus-1/places/forge/forge\""));
    assert!(merchant.contains("name=\"recipe\""));
    assert!(merchant.contains("data-forge-editor"));
    assert!(merchant.contains("data-forge-eta"));
    assert!(merchant.contains("data-inventory-pane=\"player\""));
    assert!(!merchant.contains("data-inventory-pane=\"party\""));
    assert!(!merchant.contains("Merchant stock"));
    assert!(!merchant.contains("Sell surplus"));
    assert!(
        merchant
            .contains("data-strategic-tooltip=\"Weight 10.0 / 100.0 kilograms; Penalty -10.0%\"")
    );
    assert!(!merchant.contains(">10.0 / 100.0 kg<"));

    let armourer = render(Storefront::Armor);
    assert!(armourer.contains("data-inventory-pane=\"party\""));
    assert!(
        armourer
            .contains("data-strategic-tooltip=\"Weight 30.0 / 200.0 kilograms; Penalty -15.0%\"")
    );
    assert!(!armourer.contains(">30.0 / 200.0 kg<"));

    let herbalist = render(Storefront::Herbalist);
    assert!(herbalist.contains("aria-valuetext=\"Weight 10.0 / 100.0 kilograms; Penalty -10.0%\""));
    assert!(!herbalist.contains(">10.0 / 100.0 kg<"));
    assert!(!herbalist.contains("data-inventory-pane=\"party\""));
    assert!(!herbalist.contains("Weight 30.0 / 200.0 kilograms"));

    let inn = render(Storefront::Inn);
    assert!(inn.contains("Cooking supplies"));
    assert!(inn.contains("aria-label=\"Inn rest service\""));
    assert!(inn.contains("action=\"/locations/settlement/viabundus-1/places/inn/offer\""));
    assert!(inn.contains("class=\"inn-rest-panel\""));
    assert!(inn.contains("aria-label=\"Inn lodging and rest\""));
}

#[test]
fn inn_catalog_renders_an_authoritatively_quoted_travel_ration_purchase() {
    let mut town = settlement();
    town.economy.services = vec![adventuresim_world_schema::SettlementService::Inn];
    let character = CharacterView {
        id: 1,
        name: "Traveller".into(),
        xp: 0,
        level: 1,
        current_settlement_id: Some(town.id.clone()),
        current_case_site_id: None,
        party_id: Some("party".into()),
        age_years: 20,
        alive: true,
        temporary: false,
        social_notification_count: 0,
        automatic_social_chat_enabled: false,
    };
    let ration = CatalogItemView {
        id: "travel_ration".into(),
        weight: 0.65,
        base_value: Some(3),
        nutrition_kcal: 2_500.0,
        kind: CatalogItemKind::Food,
        ..Default::default()
    };

    let markup = live_merchant_shop_page(
        &town,
        &character,
        &[],
        &[],
        std::slice::from_ref(&ration),
        &[],
        &[],
        None,
        &[],
        &[],
        &[],
        Storefront::Inn,
        1.0,
        0,
        0,
        &[],
        None,
        &[],
        adventuresim_world_schema::calendar::StrategicMinute::ZERO,
        EncumbranceSummary::default(),
        EncumbranceSummary::default(),
        None,
        SoapRestPreview::default(),
    )
    .into_string();

    assert!(markup.contains("data-merchant-item=\"travel_ration\""));
    assert!(markup.contains("data-merchant-buy=\"travel_ration\""));
    assert!(markup.contains("data-merchant-buy-price=\"5\""));
    assert!(markup.contains(">0.65<"));
}

#[test]
fn smith_player_actions_keep_sell_and_repair_in_one_hover_area() {
    let repair = repair_submit_control(&settlement(), "weapons", 4, None, 3);
    let rendered =
        merchant_sell_repair_controls(4, "torch", 2, 3, 1, true, Some(repair)).into_string();

    assert!(rendered.starts_with("<div class=\"inventory-row-actions smith-player-actions\">"));
    assert_eq!(rendered.matches("data-merchant-sell=\"").count(), 1);
    assert!(rendered.contains("data-dynamic-transfer"));
    assert!(rendered.contains("data-default-transfer-mode=\"one\""));
    assert!(rendered.contains("data-label-target=\"Sell surplus Torch\""));
    assert!(rendered.contains("data-label-all=\"Sell all Torch\""));
    assert!(rendered.contains("row-repair-form"));
    assert_eq!(rendered.matches("smith-player-actions").count(), 1);
}

#[test]
fn non_smith_sell_controls_do_not_reserve_a_repair_slot() {
    let rendered = merchant_sell_repair_controls(4, "shirt", 3, 1, 0, true, None).into_string();

    assert!(rendered.starts_with("<div class=\"inventory-row-actions\">"));
    assert!(!rendered.contains("smith-player-actions"));
    assert!(rendered.contains("data-merchant-sell"));
}
