//! Optional terrain plans and their distinct execution/approval encodings.
use super::super::{
    AppState, coordinates::wgs84_latitude_longitude_degrees, persisted_route_position,
};
use super::{
    PartyAction,
    error::{PartyTravelError, TravelQueryStage},
    location::character_case_site_id,
    payload::TravelRoutePayload,
    terrain_profile::{authoritative_party_departure_minute, party_terrain_profile},
};
use crate::spacetimedb::{
    self as db, BackendCaseSitePin, CharacterView, PartyActionRequestView, PartyJourney,
    PartyJourneyRouteView, SettlementView, SpacetimeClient, SpacetimeError, SqlQuery,
    sql_string_literal,
};
use adventuresim_core::identity::CharacterId;
use adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees;
use serde_json::json;

pub(super) struct PlannedPartyTravel<'a> {
    actor: CharacterId,
    action: &'a PartyAction,
    route: TravelRoutePayload,
}

impl PlannedPartyTravel<'_> {
    pub(super) async fn execute(
        self,
        db: &SpacetimeClient,
    ) -> std::result::Result<(), PartyTravelError> {
        let (reducer, destination) = match self.action {
            PartyAction::TravelToSettlement { settlement_id } => {
                ("travel_to_settlement_planned", json!(settlement_id))
            }
            PartyAction::TravelToCaseSite { case_site_id } => (
                "travel_to_case_site_planned",
                json!({ "value": case_site_id }),
            ),
            _ => unreachable!("only travel actions produce a terrain plan"),
        };
        db.call(
            reducer,
            &[json!(self.actor), destination, json!(self.route)],
        )
        .await
        .map_err(|source: SpacetimeError| -> PartyTravelError {
            PartyTravelError::query(TravelQueryStage::Execute, self.actor, source)
        })
    }

    pub(super) async fn approve(
        self,
        leader: CharacterId,
        request: &PartyActionRequestView,
        db: &SpacetimeClient,
    ) -> std::result::Result<(), PartyTravelError> {
        db.call(
            "approve_party_action_request_planned",
            &[json!(leader), json!(request.id), json!(self.route)],
        )
        .await
        .map_err(|source: SpacetimeError| -> PartyTravelError {
            PartyTravelError::query(TravelQueryStage::Approve, leader, source)
        })
    }
}

pub(super) async fn planned_travel_call<'a>(
    state: &AppState,
    actor_id: CharacterId,
    action: &'a PartyAction,
) -> std::result::Result<Option<PlannedPartyTravel<'a>>, PartyTravelError> {
    let Some(terrain) = state.terrain.as_deref() else {
        return Ok(None);
    };
    let character = state
        .db
        .query_one_sats_into::<adventuresim_stdb_client::Character, CharacterView>(
            db::character_by_id(actor_id),
        )
        .await
        .map_err(|source: SpacetimeError| -> PartyTravelError {
            PartyTravelError::query(TravelQueryStage::Actor, actor_id, source)
        })?
        .ok_or(PartyTravelError::MissingActor(actor_id))?;
    let (terrain_profile, snow_check_millirank) = party_terrain_profile(state, &character).await?;
    let destination = match action {
        PartyAction::TravelToSettlement { settlement_id } => {
            let destination = state
                .db
                .query_one_sats_into::<adventuresim_stdb_client::Settlement, SettlementView>(
                    crate::spacetimedb::settlement_by_id(settlement_id),
                )
                .await
                .map_err(|source: SpacetimeError| -> PartyTravelError {
                    PartyTravelError::query(
                        TravelQueryStage::DestinationSettlement,
                        actor_id,
                        source,
                    )
                })?
                .ok_or(PartyTravelError::MissingDestinationSettlement)?;
            (destination.latitude, destination.longitude)
        }
        PartyAction::TravelToCaseSite { case_site_id } => {
            let destination = state
                .db
                .query_one_sats::<BackendCaseSitePin>(SqlQuery::from(format!(
                    "SELECT * FROM backend_case_site_pins WHERE owner_character_id = {actor_id} AND case_site_id = {}",
                    sql_string_literal(case_site_id)
                )))
                .await
                .map_err(|source: SpacetimeError| -> PartyTravelError { PartyTravelError::query(TravelQueryStage::DestinationCaseSite, actor_id, source) })?
                .ok_or(PartyTravelError::MissingDestinationCaseSite(actor_id))?;
            wgs84_latitude_longitude_degrees(destination.latitude_e_7, destination.longitude_e_7)?
        }
        _ => return Ok(None),
    };
    let origin = if let Some(id) = character.current_settlement_id.as_deref() {
        let settlement = state
            .db
            .query_one_sats_into::<adventuresim_stdb_client::Settlement, SettlementView>(
                crate::spacetimedb::settlement_by_id(id),
            )
            .await
            .map_err(|source: SpacetimeError| -> PartyTravelError {
                PartyTravelError::query(TravelQueryStage::OriginSettlement, actor_id, source)
            })?
            .ok_or(PartyTravelError::MissingOriginSettlement)?;
        (settlement.latitude, settlement.longitude)
    } else if let Some(id) = character_case_site_id(state, actor_id).await? {
        let site = state
            .db
            .query_one_sats::<BackendCaseSitePin>(SqlQuery::from(format!(
                "SELECT * FROM backend_case_site_pins WHERE owner_character_id = {actor_id} AND case_site_id = {}",
                sql_string_literal(&id)
            )))
            .await
            .map_err(|source: SpacetimeError| -> PartyTravelError { PartyTravelError::query(TravelQueryStage::OriginCaseSite, actor_id, source) })?
            .ok_or(PartyTravelError::MissingOriginCaseSite(actor_id))?;
        wgs84_latitude_longitude_degrees(site.latitude_e_7, site.longitude_e_7)?
    } else if let Some(party_id) = character.party_id.as_deref() {
        let journey = state
            .db
            .query_one_sats::<PartyJourney>(crate::spacetimedb::party_journey_by_party_id(party_id))
            .await
            .map_err(|source: SpacetimeError| -> PartyTravelError {
                PartyTravelError::query(TravelQueryStage::CampJourney, actor_id, source)
            })?
            .ok_or(PartyTravelError::MissingCampJourney)?;
        let route = state
            .db
            .query_one_sats_into::<adventuresim_stdb_client::PartyJourneyRoute, PartyJourneyRouteView>(crate::spacetimedb::party_journey_route_by_party_id(
                party_id,
            ))
            .await
            .map_err(|source: SpacetimeError| -> PartyTravelError { PartyTravelError::query(TravelQueryStage::CampRoute, actor_id, source) })?
            .ok_or(PartyTravelError::MissingCampRoute)?;
        persisted_route_position(&route, journey.completed_movement_minutes)
            .ok_or(PartyTravelError::UnavailableCampPosition)?
    } else {
        return Ok(None);
    };
    let departure_minute = authoritative_party_departure_minute(state, &character).await?;
    let elevation_m = terrain
        .forage_environment(origin.0, origin.1)
        .map(|(cell, _, _)| cell.elevation_m)
        .unwrap_or(0);
    let weather_coordinate =
        Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(origin.1, origin.0)
            .ok_or(PartyTravelError::InvalidOriginCoordinates)?;
    let weather = adventuresim_core::weather::weather_at(
        adventuresim_core::weather::WORLD_WEATHER_SEED,
        departure_minute,
        weather_coordinate.latitude().get(),
        weather_coordinate.longitude().get(),
        elevation_m,
    );
    let plan = match terrain
        .plan_with_profile_and_weather(
            origin,
            destination,
            terrain_profile,
            Some(weather),
            snow_check_millirank,
        )
        .await
    {
        Ok(plan) => plan,
        Err(error) => {
            tracing::warn!(%error, actor_id = u64::from(actor_id), "terrain route unavailable at execution; using unplanned travel reducer");
            return Ok(None);
        }
    };
    let return_plan = if matches!(action, PartyAction::TravelToCaseSite { .. }) {
        match terrain
            .plan_with_profile_and_weather(
                destination,
                origin,
                terrain_profile,
                Some(weather),
                snow_check_millirank,
            )
            .await
        {
            Ok(plan) => Some(plan),
            Err(error) => {
                tracing::warn!(%error, actor_id = u64::from(actor_id), "case-site return terrain route unavailable at execution; using unplanned travel reducer");
                return Ok(None);
            }
        }
    } else {
        None
    };
    let route =
        TravelRoutePayload::from_plan(terrain.digest(), &plan, return_plan.as_ref(), weather);
    Ok(Some(PlannedPartyTravel {
        actor: actor_id,
        action,
        route,
    }))
}
