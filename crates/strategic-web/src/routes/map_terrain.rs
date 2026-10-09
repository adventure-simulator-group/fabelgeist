//! Read-only regional terrain windows for the persistent environment renderer.
use super::AppState;
use adventuresim_tactical_core::regional_terrain::{
    RegionalTerrain, RegionalTerrainRequest, RegionalTerrainScale,
};
use adventuresim_tactical_server_dispatcher::regional_terrain::capture;
use adventuresim_world_schema::coordinates::Wgs84CoordinateMicrodegrees;
use adventuresim_world_schema::source_package::SourcePackageDigest;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header::CACHE_CONTROL},
    routing::get,
};
use serde::Deserialize;
use std::sync::{Arc, OnceLock};

const TERRAIN_WINDOW_PATH: &str = "/api/map/terrain/{source}";
const TERRAIN_CAPTURE_CONCURRENCY: usize = 2;
static CAPTURES: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();

/// Native integer query fields are admitted once at the HTTP boundary.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TerrainWindowQuery {
    latitude: i32,
    longitude: i32,
    scale: RegionalTerrainScale,
}

pub(super) fn routes() -> Router<AppState> {
    Router::new().route(TERRAIN_WINDOW_PATH, get(terrain_window))
}

async fn terrain_window(
    State(state): State<AppState>,
    Path(source): Path<SourcePackageDigest>,
    Query(query): Query<TerrainWindowQuery>,
) -> Result<
    (
        [(axum::http::HeaderName, &'static str); 1],
        Json<RegionalTerrain>,
    ),
    StatusCode,
> {
    let origin = Wgs84CoordinateMicrodegrees::new(query.latitude, query.longitude)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let pack = state
        .terrain
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?
        .pack
        .clone();
    if source.as_str() != pack.digest() {
        return Err(StatusCode::NOT_FOUND);
    }
    let permits =
        CAPTURES.get_or_init(|| Arc::new(tokio::sync::Semaphore::new(TERRAIN_CAPTURE_CONCURRENCY)));
    // Do not enqueue unbounded camera requests behind a busy terrain decoder.
    let permit = permits
        .clone()
        .try_acquire_owned()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let request = RegionalTerrainRequest {
        origin,
        scale: query.scale,
    };
    let window = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        capture(&pack, request)
    })
    .await
    .map_err(unavailable)?
    .map_err(unavailable)?;
    // HTTP persistence will need a sampling revision before immutable URLs can
    // include sampler policy as well as source identity. Renderer residency is
    // independent of HTTP caching and does not require another request on reopen.
    Ok(([(CACHE_CONTROL, "private, no-store")], Json(window)))
}

fn unavailable(cause: impl std::fmt::Display) -> StatusCode {
    tracing::error!(%cause, "regional terrain capture failed");
    StatusCode::SERVICE_UNAVAILABLE
}
