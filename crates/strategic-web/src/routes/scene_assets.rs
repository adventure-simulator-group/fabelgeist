//! Prepare the actual tactical scene document without requesting a server.
use super::{AppState, CharacterView, SettlementView};
use crate::{session::Session, spacetimedb};
use adventuresim_stdb_client as sdk;
use adventuresim_tactical_server_dispatcher::{
    scene_input::build_imported_scene,
    settlement_buildings::{SettlementBusinessOperatorProfile, SettlementSceneProfile},
    settlement_economy_adapter::building_use,
};
use adventuresim_world_schema::{
    coordinates::Wgs84CoordinateE7,
    person_names::RenderedPersonalName,
    settlement_buildings::{BusinessId, BusinessKey},
};
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
            let profile = SettlementSceneProfile {
                operators: operators(&state, settlement_id).await?,
                id: settlement.id.clone(),
                population_level: settlement.population_level,
                population_estimate: settlement.population_estimate,
                economy: settlement.economy,
            };
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

async fn operators(
    state: &AppState,
    settlement: &str,
) -> Result<Vec<SettlementBusinessOperatorProfile>, StatusCode> {
    let assignments = state
        .db
        .query_sats::<sdk::SettlementBusinessOperator>(&format!(
            "SELECT * FROM settlement_business_operator WHERE settlement_id = {}",
            spacetimedb::sql_string_literal(settlement),
        ))
        .await
        .map_err(unavailable)?;
    if assignments.is_empty() {
        return Ok(Vec::new());
    }
    let condition = assignments
        .iter()
        .map(|row| format!("id = {}", row.operator_character_id))
        .collect::<Vec<_>>()
        .join(" OR ");
    let people = state
        .db
        .query_sats_into::<sdk::Character, CharacterView>(&format!(
            "SELECT * FROM character WHERE {condition}"
        ))
        .await
        .map_err(unavailable)?;
    assignments
        .into_iter()
        .map(|row| {
            if row.business_id.settlement_id != settlement {
                return Err(StatusCode::SERVICE_UNAVAILABLE);
            }
            let person = people
                .iter()
                .find(|person| person.id == row.operator_character_id)
                .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
            Ok(SettlementBusinessOperatorProfile {
                business_id: BusinessId::new(
                    settlement,
                    BusinessKey {
                        usage: building_use(&row.business_id.key.usage),
                        ordinal: row.business_id.key.ordinal,
                    },
                ),
                operator_character_id: person.id,
                operator_name: RenderedPersonalName::try_from(person.name.clone())
                    .map_err(unavailable)?,
            })
        })
        .collect()
}

fn unavailable(error: impl std::fmt::Display) -> StatusCode {
    tracing::error!(%error, "tactical scene preparation failed");
    StatusCode::SERVICE_UNAVAILABLE
}

async fn scene_minute(
    state: &AppState,
) -> Result<adventuresim_world_schema::calendar::StrategicMinute, StatusCode> {
    let clock = state
        .db
        .query_sats::<sdk::WorldClock>(&spacetimedb::world_clock_singleton())
        .await
        .map_err(unavailable)?
        .into_iter()
        .next()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(unavailable)?
        .as_micros();
    let minute = adventuresim_core::strategic_time::official_minute(
        clock.epoch_micros,
        i64::try_from(now).map_err(unavailable)?,
    );
    Ok(minute)
}
