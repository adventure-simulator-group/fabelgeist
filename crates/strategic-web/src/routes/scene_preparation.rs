//! Capture the canonical settlement profile and official scene time.
use super::{AppState, CharacterView, SettlementView};
use crate::spacetimedb;
use adventuresim_stdb_client as sdk;
use adventuresim_tactical_server_dispatcher::{
    settlement_buildings::{SettlementBusinessOperatorProfile, SettlementSceneProfile},
    settlement_economy_adapter::building_use,
};
use adventuresim_world_schema::{
    person_names::RenderedPersonalName,
    settlement_buildings::{BusinessId, BusinessKey},
};
use axum::http::StatusCode;

pub(super) async fn profile(
    state: &AppState,
    settlement: &SettlementView,
) -> Result<SettlementSceneProfile, StatusCode> {
    Ok(SettlementSceneProfile {
        operators: operators(state, &settlement.id).await?,
        id: settlement.id.clone(),
        population_level: settlement.population_level,
        population_estimate: settlement.population_estimate,
        economy: settlement.economy.clone(),
    })
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
                operator_character_id: adventuresim_tactical_core::player::CharacterId(person.id),
                operator_name: RenderedPersonalName::try_from(person.name.clone())
                    .map_err(unavailable)?,
            })
        })
        .collect()
}

pub(super) async fn scene_minute(
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

fn unavailable(error: impl std::fmt::Display) -> StatusCode {
    tracing::error!(%error, "canonical scene profile capture failed");
    StatusCode::SERVICE_UNAVAILABLE
}
