//! Read-only location resolution shared by terrain presentation and foraging.
use super::{AppState, persisted_route_position, wgs84_latitude_longitude_degrees};
mod error;
use crate::spacetimedb::{
    BackendCaseSitePin, CharacterView, PartyJourney, PartyJourneyRouteView, PartyView,
    SettlementView, SpacetimeError, SqlQuery, sql_string_literal,
};
use adventuresim_core::identity::CharacterId;
use adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees;
pub(super) use error::VicinityError;
use error::VicinityReadStage;
pub(super) struct Vicinity {
    pub(super) kind: String,
    pub(super) id: String,
    pub(super) latitude: f64,
    pub(super) longitude: f64,
    pub(super) settlement: bool,
}

pub(super) async fn vicinity(
    state: &AppState,
    character: &CharacterView,
) -> std::result::Result<Vicinity, VicinityError> {
    let actor = CharacterId::from(character.id);
    if let Some(id) = character.current_settlement_id.as_deref() {
        let row = state
            .db
            .query_one_sats_into::<adventuresim_stdb_client::Settlement, SettlementView>(
                crate::spacetimedb::settlement_by_id(id),
            )
            .await
            .map_err(|source: SpacetimeError| -> VicinityError {
                VicinityError::query(VicinityReadStage::Settlement, actor, source)
            })?
            .ok_or(VicinityError::MissingSettlement)?;
        let coordinate = Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(
            row.longitude,
            row.latitude,
        )
        .ok_or(VicinityError::InvalidSettlementCoordinate)?;
        let (longitude, latitude) = coordinate.longitude_latitude_degrees();
        return Ok(Vicinity {
            kind: "settlement".into(),
            id: id.into(),
            latitude,
            longitude,
            settlement: true,
        });
    }
    if let Some(id) = character.current_case_site_id.as_deref() {
        let row = state
            .db
            .query_one_sats::<BackendCaseSitePin>(SqlQuery::from(format!(
                "SELECT * FROM backend_case_site_pins WHERE owner_character_id = {} AND case_site_id = {}",
                character.id,
                sql_string_literal(id)
            )))
            .await
            .map_err(|source: SpacetimeError| -> VicinityError {
                VicinityError::query(VicinityReadStage::CaseSite, actor, source)
            })?
            .ok_or(VicinityError::InexactCaseSite)?;
        let (latitude, longitude) =
            wgs84_latitude_longitude_degrees(row.latitude_e_7, row.longitude_e_7)?;
        return Ok(Vicinity {
            kind: "case_site".into(),
            id: id.into(),
            latitude,
            longitude,
            settlement: false,
        });
    }
    let party_id = character
        .party_id
        .as_deref()
        .ok_or(VicinityError::NoStationaryVicinity)?;
    let party = state
        .db
        .query_one_sats_into::<adventuresim_stdb_client::Party, PartyView>(
            crate::spacetimedb::party_by_id(party_id),
        )
        .await
        .map_err(|source: SpacetimeError| -> VicinityError {
            VicinityError::query(VicinityReadStage::Party, actor, source)
        })?
        .ok_or(VicinityError::MissingParty)?;
    if party.camp_destination.is_none() {
        return Err(VicinityError::MovingOrUnknownLocation);
    }
    let journey = state
        .db
        .query_one_sats::<PartyJourney>(crate::spacetimedb::party_journey_by_party_id(party_id))
        .await
        .map_err(|source: SpacetimeError| -> VicinityError {
            VicinityError::query(VicinityReadStage::Journey, actor, source)
        })?
        .ok_or(VicinityError::MissingJourney)?;
    let route = state
        .db
        .query_one_sats_into::<adventuresim_stdb_client::PartyJourneyRoute, PartyJourneyRouteView>(
            crate::spacetimedb::party_journey_route_by_party_id(party_id),
        )
        .await
        .map_err(|source: SpacetimeError| -> VicinityError {
            VicinityError::query(VicinityReadStage::Route, actor, source)
        })?
        .ok_or(VicinityError::MissingRoute)?;
    let (latitude, longitude) =
        persisted_route_position(&route, journey.completed_movement_minutes)
            .ok_or(VicinityError::UnavailableCampPosition)?;
    Ok(Vicinity {
        kind: "camp".into(),
        id: party_id.into(),
        latitude,
        longitude,
        settlement: false,
    })
}
