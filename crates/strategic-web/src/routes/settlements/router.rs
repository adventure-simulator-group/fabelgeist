use crate::location_urls::LocationKind;
use crate::location_urls::patterns as paths;
use crate::spacetimedb as db;
use adventuresim_core::{
    equipment::{EncumbranceSummary, encumbrance_capacity_kg},
    item_references::{STANDARD_TRAVEL_RATION_ID, STANDARD_WATERSKIN_ID},
    physical_object::OperationalCustody,
    prelude::{PartyProvisioningInputs, STRATEGIC_TRAVEL_KCAL_PER_DAY, Skill},
    strategic_schedule::{CombatTrainingProfile, EquippedCombatItem},
    strategic_time::{is_walking_time, minutes_until_next_walking_start},
};
use adventuresim_stdb_client::{
    Character as DbCharacter, Item as DbItem, PartyJourneyRoute as DbPartyJourneyRoute,
    Settlement as DbSettlement,
};
use adventuresim_world_schema::OfficialReligion;
use axum::{
    Form, Json, Router,
    extract::{Path, Query, State, rejection::FormRejection},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use futures_util::{
    future::join_all,
    stream::{self, StreamExt},
};
use maud::Markup;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default, Deserialize)]
struct BuildingQuery {
    building: Option<String>,
    corpse: Option<String>,
    medical: Option<String>,
    forage: Option<bool>,
    forage_receipt: Option<String>,
    forage_error: Option<String>,
    social_feedback: Option<String>,
}

impl BuildingQuery {
    fn valid_for<'a>(&'a self, location: &LocationView) -> Option<&'a str> {
        self.building
            .as_deref()
            .and_then(|building| location.valid_building(building))
    }

    fn append_to_location(&self, location: &LocationView, path: String) -> String {
        self.valid_for(location).map_or_else(
            || path.clone(),
            |building| crate::location_urls::with_query(&path, "building", building),
        )
    }

    async fn append_to(&self, state: &AppState, kind: &str, id: &str, path: String) -> String {
        match resolve_location(state, kind, id).await {
            LocationLookup::Found(location) => self.append_to_location(&location, path),
            LocationLookup::NotFound | LocationLookup::Unavailable => path,
        }
    }
}

#[cfg(test)]
mod building_query_tests {
    use super::{BuildingQuery, SETTLEMENTS_SOURCE, merchant_service_location};

    #[test]
    fn building_query_is_closed_and_preserved_on_redirects() {
        let economy = adventuresim_world_schema::SettlementEconomyProfile::stage_placeholder();
        let (_organization, chapter) = adventuresim_core::organization::catalog()
            .organizations
            .iter()
            .find_map(|organization| {
                organization
                    .chapters
                    .iter()
                    .find(|chapter| {
                        adventuresim_core::organization::chapter_has_standalone_building(
                            organization,
                            chapter,
                            &economy,
                        )
                    })
                    .map(|chapter| (organization, chapter))
            })
            .expect("standalone catalog chapter");
        let location = crate::templates::settlement::LocationView {
            kind: crate::location_urls::LocationKind::Settlement,
            id: chapter.settlement_id.clone(),
            name: "Place".into(),
            religion_id: None,
            category: Some(crate::spacetimedb::SettlementCategory::Village),
            economy: Some(economy),
            active_building: None,
        };
        let valid = BuildingQuery {
            building: Some("inn".into()),
            ..Default::default()
        };
        assert_eq!(valid.valid_for(&location), Some("inn"));
        let unavailable = BuildingQuery {
            building: Some("books".into()),
            ..Default::default()
        };
        assert_eq!(unavailable.valid_for(&location), None);
        let organization_query = BuildingQuery {
            building: Some(chapter.location_id.clone()),
            ..Default::default()
        };
        assert_eq!(
            organization_query.valid_for(&location),
            Some(chapter.location_id.as_str())
        );
        if let Some(foreign) = adventuresim_core::organization::catalog()
            .organizations
            .iter()
            .flat_map(|organization| &organization.chapters)
            .find(|foreign| {
                foreign.settlement_id != location.id
                    && adventuresim_core::organization::organization_chapter_at(
                        &location.id,
                        &foreign.location_id,
                    )
                    .is_none()
            })
        {
            let foreign_query = BuildingQuery {
                building: Some(foreign.location_id.clone()),
                ..Default::default()
            };
            assert_eq!(foreign_query.valid_for(&location), None);
        }
        assert_eq!(
            valid.append_to_location(&location, "/locations/settlement/x/party/1".into()),
            "/locations/settlement/x/party/1?building=inn"
        );
        let non_service = BuildingQuery {
            building: Some("public-square".into()),
            ..Default::default()
        };
        assert_eq!(
            non_service.append_to_location(&location, "/locations/settlement/x/party/1".into()),
            "/locations/settlement/x/party/1?building=public-square"
        );
        let invalid = BuildingQuery {
            building: Some("../religion".into()),
            ..Default::default()
        };
        assert_eq!(invalid.valid_for(&location), None);
        assert_eq!(
            invalid.append_to_location(&location, "/locations/settlement/x/party/1".into()),
            "/locations/settlement/x/party/1"
        );
    }

    #[test]
    fn merchant_offer_routes_accept_only_bound_storefront_services() {
        assert_eq!(
            crate::location_urls::place_service("market"),
            Some("merchants")
        );
        assert_eq!(
            crate::location_urls::place_service("forge"),
            Some("weapons")
        );
        assert_eq!(crate::location_urls::place_service("weapons"), None);
        assert_eq!(merchant_service_location("merchants"), Some("market"));
        assert_eq!(merchant_service_location("weapons"), Some("forge"));
        assert_eq!(merchant_service_location("armor"), Some("armoury"));
        assert_eq!(merchant_service_location("clothing"), Some("tailor"));
        assert_eq!(merchant_service_location("inn"), Some("inn"));
        assert_eq!(merchant_service_location("herbalist"), None);
        assert_eq!(merchant_service_location("../inn"), None);
    }

    #[test]
    fn settlement_entry_activates_activity_without_a_local_server_bypass() {
        for source in [include_str!("medical.rs"), include_str!("overview.rs")] {
            assert!(source.contains("entry::activate_settlement(&state, &id).await"));
        }
        let activation = include_str!("entry.rs");
        assert!(activation.contains("ensure_settlement_activity"));
        assert!(!activation.contains("is_local()"));

        let offers = SETTLEMENTS_SOURCE
            .split("async fn service_quest_offers")
            .nth(1)
            .and_then(|tail| tail.split("fn service_quest_greeting").next())
            .expect("service quest offers route");
        assert!(!offers.contains("ensure_settlement_activity"));
    }

    #[test]
    fn fireplace_pages_use_gateway_views_and_authoritative_locality_inputs() {
        let source = SETTLEMENTS_SOURCE;
        assert!(source.contains("SELECT * FROM backend_fireplace_stations"));
        assert!(source.contains("SELECT * FROM backend_fireplace_dishes"));
        let private_station = ["SELECT * FROM fireplace_", "station WHERE"].concat();
        let private_dish = ["SELECT * FROM fireplace_", "dish WHERE"].concat();
        assert!(!source.contains(&private_station));
        assert!(!source.contains(&private_dish));
        assert!(source.contains("party.camp_destination.as_ref() == Some(&journey.destination)"));
        assert!(
            source.contains("journey.completed_movement_minutes < journey.total_movement_minutes")
        );
        assert!(source.contains("reached_camp_movement_minutes"));
        assert!(source.contains("service_npc_location_available"));
        assert!(source.contains("chapter_has_standalone_building"));
        assert!(source.contains("StrategicFixtureId::fireplace"));
        assert!(source.contains("StrategicPlaceId::journey_camp"));
        assert!(!source.contains("format!(\"camp|"));
    }
}

use super::inventory_forms::{
    DiscardInventoryForm, MerchantOfferForm, PartyOfferForm, PartyPoolTransferForm,
};
use super::redirect_to_local;
use super::travel::{
    CaseSiteKnowledgePresentation, ItineraryForecastSources, TravelDestination, TravelForm,
    TravelProvisionForecast, active_contract_tooltip, connected_destinations,
    populate_itinerary_forecasts,
};
use super::{
    AppState, PartyAction, PartyActionOutcome, SocialActionId, SocialDuration,
    execute_or_request_party_action,
};
use crate::session::Session;
use crate::spacetimedb::{
    AlcoholConsumption, AutomaticSocialChat, BackendCaseSitePin, BackendChallenge,
    BackendCharacterRelationshipStatus, BackendCharacterResidenceStatus, BackendContextCharacter,
    BackendContract, BackendCorpse, BackendFamilyChild, BackendFireplaceDish,
    BackendFireplaceStation, BackendIngredientPreparationPlan, BackendLocalProblemTradeEffect,
    BackendPhysiologyAdministration, BackendPhysiologyChart, BackendRoadChallenge,
    BackendTinctureStatus, BestiaryHoursExt, BodyRegion, CatalogItemKind, CatalogItemView,
    CharacterAffinity, CharacterAttributes, CharacterCapability, CharacterCondition,
    CharacterEquipmentGraph, CharacterFamiliarity, CharacterFilth, CharacterLimbs,
    CharacterMoraleSource, CharacterNeeds, CharacterSettlementReputation, CharacterSkills,
    CharacterStats, CharacterStrategicCondition, CharacterTime, CharacterTrainingSchedule,
    CharacterView, ContainerLiquid, ContractStatus, EquipmentAnchorKind, EquipmentAttachmentTarget,
    EquipmentOccupancy, EquippedItemView, FoodLot, IngredientPreparationAction,
    InventoryContainment, InventoryItem, InventoryItemAmount, InventoryLocation, InventoryObject,
    InventoryQuantityTarget, ItemCondition, ItemConditionExt, JourneyEndpointExt, LimbInjury,
    PartyInventoryItem, PartyItemAmount, PartyJourney, PartyJourneyRouteView, PartyMember,
    PartyStake, PartyView, Personality, RecruitmentOffer, RecruitmentOfferStatus,
    RecruitmentRoleView, ReligionHoursExt, ReligiousDemand, RepairOrder, RetainedProjectile,
    RoleRequirements, ScheduleAllocation, SettlementAlias, SettlementDescription,
    SettlementResidenceOffer, SettlementSmith, SettlementView, SocialAddress, SocialBelief,
    SocialChatOutcome, StrategicEncounter, StrategicEncounterStatus, TravelEdgeView,
};
use crate::spacetimedb::{party_by_id, settlement_by_id, sql_string_literal};
use crate::templates::settlement::{
    ActivityPreviewRates, CampTravelDestination, ChildPresentation, LocationView, MerchantShop,
    RelationshipPresentation, RestServiceKind, RestSummary, SoapRestPreview, SocialPresentation,
    WeddingPresentation, camp_page, live_merchant_shop_page, merchants_page, party_discard_page,
    party_inventory_page, party_personal_page, party_pool_page, party_social_dialog,
    party_stats_page, religion_page, rest_default_minutes, rest_result_page, settlement_map_page,
    settlement_overview_page, settlement_residence_page, settlement_resident_location_page,
    surgery_dialog,
};

fn contained_water_ml_for_custody(
    objects: &[InventoryObject],
    containment: &[InventoryContainment],
    liquids: &[ContainerLiquid],
    custody: &OperationalCustody,
) -> u64 {
    liquids.iter().fold(0_u64, |total, liquid| {
        let mut cursor = liquid.container_object_id;
        let mut visited = std::collections::BTreeSet::new();
        let root =
            (0..=adventuresim_core::inventory_containers::MAX_CONTAINER_DEPTH).find_map(|_| {
                if !visited.insert(cursor) {
                    return Some(None);
                }
                let object = objects.iter().find(|object| object.id == cursor)?;
                match containment
                    .iter()
                    .find(|edge| edge.child_object_id == cursor)
                {
                    Some(edge) => {
                        cursor = edge.parent_object_id;
                        None
                    }
                    None => Some(Some(object)),
                }
            });
        let held = root.flatten().is_some_and(|root| {
            matches!(
                (&root.location, custody),
                (
                    InventoryLocation::Personal(location),
                    OperationalCustody::Character(character_id)
                ) if location.character_id == character_id.get()
            ) || matches!(
                (&root.location, custody),
                (InventoryLocation::Party(location), OperationalCustody::Party(party_id))
                    if location.party_id == party_id.as_str()
            )
        });
        if held {
            total.saturating_add(liquid.water_ml)
        } else {
            total
        }
    })
}

pub fn routes() -> Router<AppState> {
    settlement_routes()
        .merge(camp_routes())
        .merge(party_routes())
        .merge(inventory_routes())
        .merge(commerce_routes())
        .layer(axum::middleware::from_fn(
            crate::location_urls::require_canonical_location_path,
        ))
}

fn settlement_routes() -> Router<AppState> {
    Router::new()
        .route(
            paths::SETTLEMENT_PLACE.pattern(),
            get(settlement_resident_place),
        )
        .route(paths::CHANGE_RESIDENCE.pattern(), post(change_residence))
        .route(
            paths::PUBLIC_SQUARE.pattern(),
            get(show_settlement_location),
        )
        .route(
            paths::SETTLEMENT_FIREPLACE.pattern(),
            get(settlement_fireplace),
        )
        .route(
            paths::SETTLEMENT_FIREPLACE_INGREDIENTS.pattern(),
            post(settlement_fireplace_ingredients),
        )
        .route(
            paths::SETTLEMENT_FIREPLACE_RETRIEVE.pattern(),
            post(settlement_fireplace_retrieve),
        )
        .route(
            paths::SETTLEMENT_FIREPLACE_CONTAINER_PLACE.pattern(),
            post(settlement_fireplace_container_place),
        )
        .route(
            paths::SETTLEMENT_FIREPLACE_CONTAINER_START.pattern(),
            post(settlement_fireplace_container_start),
        )
        .route(
            paths::SETTLEMENT_FIREPLACE_CONTAINER_REMOVE.pattern(),
            post(settlement_fireplace_container_remove),
        )
        .route(paths::SETTLEMENT.pattern(), get(settlement_map))
        .route(paths::ALCHEMY.pattern(), get(alchemy))
        .route(
            paths::UPDATE_TRAVEL_CONFIGURATION.pattern(),
            post(update_travel_configuration),
        )
        .route(
            paths::REST_AT_SETTLEMENT_MAP.pattern(),
            post(rest_at_settlement_map),
        )
        .route(
            paths::UPDATE_CASE_SITE_TRAVEL_CONFIGURATION.pattern(),
            post(update_travel_configuration),
        )
}

fn camp_routes() -> Router<AppState> {
    Router::new()
        .route(paths::CAMP.pattern(), get(camp))
        .route(
            paths::CAMP_FIREPLACE_PAGE.pattern(),
            get(camp_fireplace_page),
        )
        .route(
            paths::CAMP_FIREPLACE_INGREDIENTS.pattern(),
            post(camp_fireplace_ingredients),
        )
        .route(
            paths::CAMP_FIREPLACE_RETRIEVE.pattern(),
            post(camp_fireplace_retrieve),
        )
        .route(
            paths::CAMP_FIREPLACE_CONTAINER_PLACE.pattern(),
            post(camp_fireplace_container_place),
        )
        .route(
            paths::CAMP_FIREPLACE_CONTAINER_START.pattern(),
            post(camp_fireplace_container_start),
        )
        .route(
            paths::CAMP_FIREPLACE_CONTAINER_REMOVE.pattern(),
            post(camp_fireplace_container_remove),
        )
        .route(paths::REST_AT_CAMP.pattern(), post(rest_at_camp))
        .route(
            paths::RESOLVE_ERRANTRY_ROAD_CHALLENGE.pattern(),
            post(resolve_errantry_road_challenge),
        )
        .route(
            paths::UPDATE_CAMP_TRAVEL_CONFIGURATION.pattern(),
            post(update_camp_travel_configuration),
        )
        .route(
            paths::CONTINUE_CAMP_TRAVEL.pattern(),
            post(continue_camp_travel),
        )
        .route(
            paths::RESOLVE_CAMP_ENCOUNTER.pattern(),
            post(resolve_camp_encounter),
        )
        .route(
            paths::CONTACT_CAMP_COUNTERPARTY.pattern(),
            post(contact_camp_counterparty),
        )
        .route(
            paths::BANDAGE_CAMP_COUNTERPARTY.pattern(),
            post(bandage_camp_counterparty),
        )
        .route(
            paths::CHANGE_CAMP_DESTINATION.pattern(),
            post(change_camp_destination),
        )
}

fn party_routes() -> Router<AppState> {
    Router::new()
        .route(paths::PARTY_PERSONAL.pattern(), get(party_personal))
        .route(
            paths::UPDATE_ORGANIZATION_PRESENTATION.pattern(),
            post(update_organization_presentation),
        )
        .route(
            paths::CLEAR_PRESENTED_ORGANIZATION.pattern(),
            post(clear_presented_organization),
        )
        .route(paths::STOP_PREPARATION.pattern(), post(stop_preparation))
        .route(paths::PARTY_MEMBER.pattern(), get(party_member))
        .route(
            paths::TRANSFER_PARTY_ITEM.pattern(),
            post(transfer_party_item),
        )
        .route(
            paths::REMOVE_PARTY_MEMBER.pattern(),
            post(remove_party_member),
        )
        .route(
            paths::FINALIZE_PARTY_OFFER.pattern(),
            post(finalize_party_offer),
        )
        .route(
            paths::DISCARD_INVENTORY_ITEMS.pattern(),
            post(discard_inventory_items),
        )
        .route(
            paths::PARTY_POOL_INVENTORY.pattern(),
            get(party_pool_inventory),
        )
        .route(
            paths::DEPOSIT_PARTY_INVENTORY.pattern(),
            post(deposit_party_inventory),
        )
        .route(
            paths::WITHDRAW_PARTY_INVENTORY.pattern(),
            post(withdraw_party_inventory),
        )
        .route(
            paths::LIQUIDATE_PARTY_ASSETS.pattern(),
            post(liquidate_party_assets),
        )
        .route(paths::PARTY_STATS.pattern(), get(party_stats))
        .route(
            paths::PARTY_SOCIAL.pattern(),
            get(party_social).post(perform_social_action),
        )
        .route(
            paths::CHAT_WITH_PARTY_MEMBER.pattern(),
            post(chat_with_party_member),
        )
        .route(
            paths::SET_AUTOMATIC_SOCIAL_CHAT.pattern(),
            post(set_automatic_social_chat),
        )
        .route(paths::SURGERY.pattern(), get(surgery))
        .route(paths::PERFORM_SURGERY.pattern(), post(perform_surgery))
        .route(paths::PARTY_STATS_PLAYER.pattern(), get(party_stats))
        .route(
            paths::UPDATE_TRAINING_SCHEDULE.pattern(),
            post(update_training_schedule),
        )
        .route(
            paths::PERFORM_IMMEDIATE_ACTIVITY.pattern(),
            post(perform_immediate_activity),
        )
        .route(paths::RENOUNCE_RELIGION.pattern(), post(renounce_religion))
        .route(
            paths::RESOLVE_RELIGIOUS_DEMAND.pattern(),
            post(resolve_religious_demand),
        )
}

fn inventory_routes() -> Router<AppState> {
    Router::new()
        .route("/api/inventory/containers", get(inventory_containers))
        .route(
            "/api/inventory/containers/move",
            post(move_inventory_container_item),
        )
        .route(
            "/api/inventory/containers/remove",
            post(remove_inventory_container_item),
        )
        .route(
            "/api/inventory/containers/discard-water",
            post(discard_inventory_container_water),
        )
        .route(
            "/api/inventory/containers/tincture-spirit",
            post(pour_inventory_container_tincture_spirit),
        )
        .route(
            "/api/inventory/containers/tincture-start",
            post(start_inventory_container_tincture),
        )
        .route(
            "/api/inventory/containers/tincture-refresh",
            post(refresh_inventory_container_tincture),
        )
        .route(
            "/api/inventory/containers/tincture-dose",
            post(dose_inventory_container_tincture),
        )
        .route("/api/inventory/prepare", post(prepare_ingredient_lot))
        .route("/api/inventory-target", post(set_inventory_target))
        .route("/api/equipment", post(set_equipment))
}

fn commerce_routes() -> Router<AppState> {
    Router::new()
        .route(
            paths::SERVICE_QUEST_OFFERS.pattern(),
            get(service_quest_offers),
        )
        .route(
            paths::BEGIN_SERVICE_APPRENTICESHIP.pattern(),
            post(begin_service_apprenticeship),
        )
        .route(
            paths::RELIGION_DIALOGUE.pattern(),
            get(religion_dialogue).post(set_religion),
        )
        .route(paths::MERCHANTS.pattern(), get(merchants))
        .route(
            paths::FINALIZE_MERCHANT_OFFER.pattern(),
            post(finalize_merchant_offer),
        )
        .route(paths::WEAPONS.pattern(), get(weapons))
        .route(paths::FORGE_WEAPON.pattern(), post(forge_weapon))
        .route(paths::ARMOR.pattern(), get(armor))
        .route(paths::SUBMIT_REPAIR.pattern(), post(submit_repair))
        .route(
            paths::SUBMIT_ALL_REPAIRS.pattern(),
            post(submit_all_repairs),
        )
        .route(paths::RETRIEVE_REPAIR.pattern(), post(retrieve_repair))
        .route(paths::RETRIEVE_REPAIRS.pattern(), post(retrieve_repairs))
        .route(paths::CLOTHING.pattern(), get(clothing))
        .route(paths::BOOKSTORE.pattern(), get(bookstore))
        .route(paths::HERBALIST.pattern(), get(herbalist))
        .route(
            paths::PURCHASE_FROM_HERBALIST.pattern(),
            post(purchase_from_herbalist),
        )
        .route(paths::INN.pattern(), get(inn))
        .route(paths::RELIGION.pattern(), get(religion))
        .route(paths::REST.pattern(), post(rest))
        .route(paths::TRAVEL.pattern(), post(travel))
}
