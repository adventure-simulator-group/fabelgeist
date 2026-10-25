//! Registered location routes shared by handlers, templates, and HTTP tests.

use super::Route;

pub const TRAVEL_TO_CASE_SITE: Route<1> = Route::new("/locations/case-site/{id}/travel");
pub const TRACK_CASE_SITE: Route<1> = Route::new("/locations/case-site/{id}/track");
pub const CASE_SITE: Route<1> = Route::new("/locations/case-site/{id}");
pub const QUEST_LOCATION_ENEMY: Route<1> = Route::new("/locations/case-site/{id}/enemy");
pub const CONTACT_QUEST_COUNTERPARTY: Route<1> =
    Route::new("/locations/case-site/{id}/counterparty/contact");
pub const BANDAGE_QUEST_COUNTERPARTY: Route<1> =
    Route::new("/locations/case-site/{id}/counterparty/bandage");
pub const NEGOTIATE_HOSTILE_WITHDRAWAL: Route<1> =
    Route::new("/locations/case-site/{id}/hostile/withdrawal");
pub const DEMAND_HOSTILE_SURRENDER: Route<1> =
    Route::new("/locations/case-site/{id}/hostile/surrender/demand");
pub const ANSWER_HOSTILE_SURRENDER_OFFER: Route<1> =
    Route::new("/locations/case-site/{id}/hostile/surrender/offer");
pub const REST_AT_QUEST_LOCATION: Route<1> = Route::new("/locations/case-site/{id}/enemy/rest");
pub const REST_AT_QUEST_LOCATION_MAP: Route<1> = Route::new("/locations/case-site/{id}/rest");
pub const LOCATION_NPCS: Route<2> =
    Route::new("/api/locations/settlement/{id}/places/{place}/npcs");
pub const NPC_SOCIAL: Route<3> =
    Route::new("/api/locations/settlement/{id}/places/{place}/npcs/{resident_character_id}/social");
pub const NPC_ROMANCE_ACTION: Route<4> = Route::new(
    "/api/locations/settlement/{id}/places/{place}/npcs/{resident_character_id}/romance/{action}",
);
pub const CASE_SITE_EVIDENCE: Route<1> =
    Route::new("/api/locations/case-site/{case_site_id}/evidence");
pub const SETTLEMENT_PLACE: Route<2> = Route::new("/locations/settlement/{id}/places/{place}");
pub const CHANGE_RESIDENCE: Route<3> =
    Route::new("/locations/settlement/{id}/places/residences/{action}/{tier}");
pub const PUBLIC_SQUARE: Route<1> = Route::new("/locations/settlement/{id}/places/public-square");
pub const SETTLEMENT_FIREPLACE: Route<2> =
    Route::new("/locations/settlement/{id}/places/{place}/fireplace");
pub const SETTLEMENT_FIREPLACE_INGREDIENTS: Route<2> =
    Route::new("/locations/settlement/{id}/places/{place}/fireplace/ingredients");
pub const SETTLEMENT_FIREPLACE_RETRIEVE: Route<2> =
    Route::new("/locations/settlement/{id}/places/{place}/fireplace/retrieve");
pub const SETTLEMENT_FIREPLACE_CONTAINER_PLACE: Route<2> =
    Route::new("/locations/settlement/{id}/places/{place}/fireplace/container/place");
pub const SETTLEMENT_FIREPLACE_CONTAINER_START: Route<2> =
    Route::new("/locations/settlement/{id}/places/{place}/fireplace/container/start");
pub const SETTLEMENT_FIREPLACE_CONTAINER_REMOVE: Route<2> =
    Route::new("/locations/settlement/{id}/places/{place}/fireplace/container/remove");
pub const SETTLEMENT: Route<1> = Route::new("/locations/settlement/{id}");
pub const ALCHEMY: Route<1> = Route::new("/locations/settlement/{id}/alchemy");
pub const UPDATE_TRAVEL_CONFIGURATION: Route<1> =
    Route::new("/locations/settlement/{id}/travel-configuration");
pub const REST_AT_SETTLEMENT_MAP: Route<1> = Route::new("/locations/settlement/{id}/rest");
pub const UPDATE_CASE_SITE_TRAVEL_CONFIGURATION: Route<1> =
    Route::new("/locations/case-site/{id}/travel-configuration");
pub const CAMP: Route<0> = Route::new("/locations/camp");
pub const CAMP_FIREPLACE_PAGE: Route<0> = Route::new("/locations/camp/fireplace");
pub const CAMP_FIREPLACE_INGREDIENTS: Route<0> =
    Route::new("/locations/camp/fireplace/ingredients");
pub const CAMP_FIREPLACE_RETRIEVE: Route<0> = Route::new("/locations/camp/fireplace/retrieve");
pub const CAMP_FIREPLACE_CONTAINER_PLACE: Route<0> =
    Route::new("/locations/camp/fireplace/container/place");
pub const CAMP_FIREPLACE_CONTAINER_START: Route<0> =
    Route::new("/locations/camp/fireplace/container/start");
pub const CAMP_FIREPLACE_CONTAINER_REMOVE: Route<0> =
    Route::new("/locations/camp/fireplace/container/remove");
pub const REST_AT_CAMP: Route<0> = Route::new("/locations/camp/rest");
pub const RESOLVE_ERRANTRY_ROAD_CHALLENGE: Route<0> =
    Route::new("/locations/camp/errantry-road-challenge");
pub const UPDATE_CAMP_TRAVEL_CONFIGURATION: Route<0> =
    Route::new("/locations/camp/travel-configuration");
pub const CONTINUE_CAMP_TRAVEL: Route<0> = Route::new("/locations/camp/continue");
pub const RESOLVE_CAMP_ENCOUNTER: Route<0> = Route::new("/locations/camp/encounter");
pub const CONTACT_CAMP_COUNTERPARTY: Route<0> = Route::new("/locations/camp/counterparty/contact");
pub const BANDAGE_CAMP_COUNTERPARTY: Route<0> = Route::new("/locations/camp/counterparty/bandage");
pub const CHANGE_CAMP_DESTINATION: Route<1> = Route::new("/locations/camp/destination/{id}");
pub const SERVICE_QUEST_OFFERS: Route<1> =
    Route::new("/api/locations/settlement/{id}/service-quests");
pub const BEGIN_SERVICE_APPRENTICESHIP: Route<2> =
    Route::new("/api/locations/settlement/{id}/places/{place}/apprenticeship");
pub const RELIGION_DIALOGUE: Route<1> =
    Route::new("/api/locations/settlement/{id}/places/church/religion");
pub const PARTY_PERSONAL: Route<3> = Route::new("/locations/{kind}/{id}/party/{character_id}");
pub const UPDATE_ORGANIZATION_PRESENTATION: Route<3> = Route::new(
    "/locations/settlement/{id}/party/{character_id}/organization-presentation/{organization_id}",
);
pub const CLEAR_PRESENTED_ORGANIZATION: Route<2> =
    Route::new("/locations/settlement/{id}/party/{character_id}/organization-presentation-none");
pub const STOP_PREPARATION: Route<4> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/physiology/{administration_id}/stop");
pub const PARTY_MEMBER: Route<3> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/inventory");
pub const TRANSFER_PARTY_ITEM: Route<3> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/inventory/transfer");
pub const REMOVE_PARTY_MEMBER: Route<3> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/remove");
pub const FINALIZE_PARTY_OFFER: Route<3> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/inventory/offer");
pub const DISCARD_INVENTORY_ITEMS: Route<3> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/inventory/discard");
pub const PARTY_POOL_INVENTORY: Route<2> = Route::new("/locations/{kind}/{id}/party-inventory");
pub const DEPOSIT_PARTY_INVENTORY: Route<2> =
    Route::new("/locations/{kind}/{id}/party-inventory/deposit");
pub const WITHDRAW_PARTY_INVENTORY: Route<2> =
    Route::new("/locations/{kind}/{id}/party-inventory/withdraw");
pub const LIQUIDATE_PARTY_ASSETS: Route<2> =
    Route::new("/locations/{kind}/{id}/party-inventory/liquidate");
pub const PARTY_STATS: Route<3> = Route::new("/locations/{kind}/{id}/party/{character_id}/stats");
pub const PARTY_SOCIAL: Route<3> = Route::new("/locations/{kind}/{id}/party/{character_id}/social");
pub const CHAT_WITH_PARTY_MEMBER: Route<3> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/social/chat");
pub const SET_AUTOMATIC_SOCIAL_CHAT: Route<3> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/social/automatic");
pub const SURGERY: Route<4> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/surgery/{limb}");
pub const PERFORM_SURGERY: Route<4> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/surgery/{limb}/procedure");
pub const PARTY_STATS_PLAYER: Route<3> =
    Route::new("/locations/{kind}/{id}/players/{character_id}");
pub const UPDATE_TRAINING_SCHEDULE: Route<3> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/schedule");
pub const PREVIEW_TRAINING_SCHEDULE: Route<3> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/schedule/preview");
pub const PERFORM_IMMEDIATE_ACTIVITY: Route<3> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/activity");
pub const RENOUNCE_RELIGION: Route<3> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/religion/renounce");
pub const RESOLVE_RELIGIOUS_DEMAND: Route<4> =
    Route::new("/locations/{kind}/{id}/party/{character_id}/religious-demand/{demand_id}");
pub const MERCHANTS: Route<1> = Route::new("/locations/settlement/{id}/places/market");
pub const FINALIZE_MERCHANT_OFFER: Route<2> =
    Route::new("/locations/settlement/{id}/places/{place}/offer");
pub const WEAPONS: Route<1> = Route::new("/locations/settlement/{id}/places/forge");
pub const FORGE_WEAPON: Route<1> = Route::new("/locations/settlement/{id}/places/forge/forge");
pub const ARMOR: Route<1> = Route::new("/locations/settlement/{id}/places/armoury");
pub const SUBMIT_REPAIR: Route<2> = Route::new("/locations/settlement/{id}/places/{place}/repair");
pub const SUBMIT_ALL_REPAIRS: Route<2> =
    Route::new("/locations/settlement/{id}/places/{place}/repair-all");
pub const RETRIEVE_REPAIR: Route<3> =
    Route::new("/locations/settlement/{id}/places/{place}/repairs/{order_id}/retrieve");
pub const RETRIEVE_REPAIRS: Route<2> =
    Route::new("/locations/settlement/{id}/places/{place}/repairs/retrieve");
pub const CLOTHING: Route<1> = Route::new("/locations/settlement/{id}/places/tailor");
pub const BOOKSTORE: Route<1> = Route::new("/locations/settlement/{id}/places/bookstore");
pub const HERBALIST: Route<1> = Route::new("/locations/settlement/{id}/places/herbalist");
pub const PURCHASE_FROM_HERBALIST: Route<1> =
    Route::new("/locations/settlement/{id}/places/herbalist/purchase");
pub const INN: Route<1> = Route::new("/locations/settlement/{id}/places/inn");
pub const RELIGION: Route<1> = Route::new("/locations/settlement/{id}/places/church");
pub const REST: Route<2> = Route::new("/locations/settlement/{id}/places/{place}/rest");
pub const TRAVEL: Route<1> = Route::new("/locations/settlement/{id}/travel");
