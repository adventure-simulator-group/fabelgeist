//! Prepare the actual tactical scene document without requesting a server.
use super::scene_preparation::{profile, scene_minute};
use super::{AppState, CharacterView, SettlementView};
use crate::{session::Session, spacetimedb};
use adventuresim_stdb_client as sdk;
use adventuresim_tactical_server_dispatcher::scene_input::build_imported_scene;
use adventuresim_world_schema::coordinates::Wgs84CoordinateE7;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::get,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
struct Request {
    settlement: Option<String>,
}

pub(super) fn routes() -> Router<AppState> {
    Router::new().route("/api/scene-assets", get(scene_assets))
}

async fn scene_assets(
    State(state): State<AppState>,
    session: Session,
    Query(request): Query<Request>,
) -> Result<Json<adventuresim_tactical_core::prelude::TacticalSceneInput>, StatusCode> {
    let id = session.character_id_u64().ok_or(StatusCode::UNAUTHORIZED)?;
    let mut actor = state
        .db
        .query_one_sats_into::<sdk::Character, CharacterView>(&spacetimedb::character_by_id(id))
        .await
        .map_err(unavailable)?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    if request
        .settlement
        .as_ref()
        .is_some_and(|requested| actor.current_settlement_id.as_ref() != Some(requested))
    {
        return Err(StatusCode::FORBIDDEN);
    }
    actor.current_case_site_id = super::character_case_site_id(&state, id)
        .await
        .map_err(unavailable)?;
    let (coordinates, scene_key, identity, profile) =
        if let Some(settlement_id) = actor.current_settlement_id.as_deref() {
            let settlement = state
                .db
                .query_one_sats_into::<sdk::Settlement, SettlementView>(
                    &spacetimedb::settlement_by_id(settlement_id),
                )
                .await
                .map_err(unavailable)?
                .ok_or(StatusCode::NOT_FOUND)?;
            let coordinates = Wgs84CoordinateE7::from_longitude_latitude_degrees(
                settlement.longitude,
                settlement.latitude,
            )
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
            let profile = profile(&state, &settlement).await?;
            (
                coordinates,
                settlement.scene_key,
                settlement.id,
                Some(profile),
            )
        } else {
            let location = super::vicinity::vicinity(&state, &actor)
                .await
                .map_err(unavailable)?;
            let coordinates = Wgs84CoordinateE7::from_longitude_latitude_degrees(
                location.longitude,
                location.latitude,
            )
            .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
            (coordinates, location.kind, location.id, None)
        };
    let minute = scene_minute(&state).await?;
    let terrain = state
        .terrain
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
        .pack
        .clone();
    // Scene generation is CPU-heavy; bound both concurrency and async-worker use.
    static PERMITS: std::sync::OnceLock<Arc<tokio::sync::Semaphore>> = std::sync::OnceLock::new();
    let permit = Arc::clone(PERMITS.get_or_init(|| Arc::new(tokio::sync::Semaphore::new(2))))
        .acquire_owned()
        .await
        .map_err(unavailable)?;
    let input = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        build_imported_scene(
            &terrain,
            &identity,
            &scene_key,
            coordinates.latitude().get(),
            coordinates.longitude().get(),
            minute,
            minute,
            profile.as_ref(),
        )
    })
    .await
    .map_err(unavailable)?
    .map_err(unavailable)?;
    if let Some(catalog) = &input.properties {
        super::settlement_properties::ensure_catalog(&state, catalog)
            .await
            .map_err(unavailable)?;
    }
    Ok(Json(input))
}

fn unavailable(error: impl std::fmt::Display) -> StatusCode {
    tracing::error!(%error, "tactical scene preparation failed");
    StatusCode::SERVICE_UNAVAILABLE
}
