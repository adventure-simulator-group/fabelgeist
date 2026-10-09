//! Read-only canonical settlement capture for one focused regional-map city.
use super::{AppState, CharacterView, SettlementView, scene_preparation};
use crate::{session::Session, spacetimedb};
use adventuresim_core::strategic_place::StrategicPlaceId;
use adventuresim_stdb_client as sdk;
use adventuresim_tactical_core::regional_city::RegionalCityInput;
use adventuresim_tactical_server_dispatcher::scene_input::build_imported_scene;
use adventuresim_world_schema::{
    coordinates::Wgs84CoordinateE7, source_package::SourcePackageDigest,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header::CACHE_CONTROL},
    routing::get,
};
use serde::Deserialize;
use std::sync::{Arc, OnceLock};

const CITY_CAPTURE_PATH: &str = "/api/map/city/{source}";
const CITY_CAPTURE_CONCURRENCY: usize = 2;
static CAPTURES: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CityQuery {
    place: StrategicPlaceId,
}

pub(super) fn routes() -> Router<AppState> {
    Router::new().route(CITY_CAPTURE_PATH, get(city))
}

async fn city(
    State(state): State<AppState>,
    session: Session,
    Path(source): Path<SourcePackageDigest>,
    Query(query): Query<CityQuery>,
) -> Result<
    (
        [(axum::http::HeaderName, &'static str); 1],
        Json<RegionalCityInput>,
    ),
    StatusCode,
> {
    let character = session.character_id_u64().ok_or(StatusCode::UNAUTHORIZED)?;
    let StrategicPlaceId::Settlement { settlement_id } = &query.place else {
        return Err(StatusCode::BAD_REQUEST);
    };
    let pack = state
        .terrain
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
        .pack
        .clone();
    if source.as_str() != pack.digest() {
        return Err(StatusCode::NOT_FOUND);
    }
    state
        .db
        .query_one_sats_into::<sdk::Character, CharacterView>(&spacetimedb::character_by_id(
            character,
        ))
        .await
        .map_err(unavailable)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    // Admit before database profile capture as well as blocking generation.
    // Superseded camera requests cannot form an unbounded generation backlog.
    let permit = CAPTURES
        .get_or_init(|| Arc::new(tokio::sync::Semaphore::new(CITY_CAPTURE_CONCURRENCY)))
        .clone()
        .try_acquire_owned()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let settlement = state
        .db
        .query_one_sats_into::<sdk::Settlement, SettlementView>(&spacetimedb::settlement_by_id(
            settlement_id.as_str(),
        ))
        .await
        .map_err(unavailable)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let origin = Wgs84CoordinateE7::from_longitude_latitude_degrees(
        settlement.longitude,
        settlement.latitude,
    )
    .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let profile = scene_preparation::profile(&state, &settlement).await?;
    let minute = scene_preparation::scene_minute(&state).await?;
    let document = tokio::task::spawn_blocking(move || -> anyhow::Result<RegionalCityInput> {
        let _permit = permit;
        let input = build_imported_scene(
            &pack,
            &settlement.id,
            &settlement.scene_key,
            origin.latitude().get(),
            origin.longitude().get(),
            minute,
            minute,
            Some(&profile),
        )
        .map_err(anyhow::Error::msg)?;
        Ok(RegionalCityInput::from_scene(
            source,
            query.place,
            origin,
            input,
        )?)
    })
    .await
    .map_err(unavailable)?
    .map_err(unavailable)?;
    Ok(([(CACHE_CONTROL, "private, no-store")], Json(document)))
}

fn unavailable(cause: impl std::fmt::Display) -> StatusCode {
    tracing::error!(%cause, "focused regional city capture failed");
    StatusCode::SERVICE_UNAVAILABLE
}
