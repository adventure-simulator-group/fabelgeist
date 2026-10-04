//! Read-only location resolution shared by terrain presentation and foraging.
use super::{AppState, persisted_route_position, wgs84_latitude_longitude_degrees};
use crate::spacetimedb::{
    BackendCaseSitePin, CharacterView, PartyJourney, PartyJourneyRouteView, PartyView,
    SettlementView, sql_string_literal,
};
use adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees;
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
) -> Result<Vicinity, String> {
    if let Some(id) = character.current_settlement_id.as_deref() {
        let row = state
            .db
            .query_one_sats_into::<adventuresim_stdb_client::Settlement, SettlementView>(
                &crate::spacetimedb::settlement_by_id(id),
            )
            .await
            .map_err(|error| error.to_string())?
            .ok_or("Current settlement not found")?;
        let coordinate = Wgs84CoordinateMicrodegrees::from_longitude_latitude_degrees(
            row.longitude,
            row.latitude,
        )
        .ok_or("persisted settlement coordinate is outside WGS84 bounds")?;
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
            .query_one_sats::<BackendCaseSitePin>(&format!(
                "SELECT * FROM backend_case_site_pins WHERE owner_character_id = {} AND case_site_id = {}",
                character.id,
                sql_string_literal(id)
            ))
            .await
            .map_err(|error| error.to_string())?
            .ok_or("Current case site is not exact")?;
        let (latitude, longitude) =
            wgs84_latitude_longitude_degrees(row.latitude_e_7, row.longitude_e_7)
                .map_err(str::to_owned)?;
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
        .ok_or("Character has no stationary vicinity")?;
    let party = state
        .db
        .query_one_sats_into::<adventuresim_stdb_client::Party, PartyView>(
            &crate::spacetimedb::party_by_id(party_id),
        )
        .await
        .map_err(|error| error.to_string())?
        .ok_or("Party not found")?;
    if party.camp_destination.is_none() {
        return Err("Foraging is unavailable while moving or without a known location".into());
    }
    let journey = state
        .db
        .query_one_sats::<PartyJourney>(&crate::spacetimedb::party_journey_by_party_id(party_id))
        .await
        .map_err(|error| error.to_string())?
        .ok_or("Camp journey not found")?;
    let route = state
        .db
        .query_one_sats_into::<adventuresim_stdb_client::PartyJourneyRoute, PartyJourneyRouteView>(
            &crate::spacetimedb::party_journey_route_by_party_id(party_id),
        )
        .await
        .map_err(|error| error.to_string())?
        .ok_or("Camp terrain route not found")?;
    let (latitude, longitude) =
        persisted_route_position(&route, journey.completed_movement_minutes)
            .ok_or("Camp terrain position is unavailable")?;
    Ok(Vicinity {
        kind: "camp".into(),
        id: party_id.into(),
        latitude,
        longitude,
        settlement: false,
    })
}
