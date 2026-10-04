//! Strategic display time projected from the authoritative clocks.
use super::*;
use crate::spacetimedb::WorldClock;
use adventuresim_core::strategic_time::clock::{OfficialClockEpoch, UnixMicrosecondInstant};
use std::time::{SystemTime, UNIX_EPOCH};
#[derive(Serialize)]
struct CurrentTime {
    character_minutes: u64,
    official_minutes: u64,
}

pub(super) async fn current_time(State(state): State<AppState>, session: Session) -> Response {
    let Some(character_id) = session.character_id_u64() else {
        return Json(CurrentTime {
            character_minutes: 0,
            official_minutes: 0,
        })
        .into_response();
    };
    let character_time_sql =
        crate::spacetimedb::character_time_by_character_id(character_id.into());
    let world_clock_sql = crate::spacetimedb::world_clock_singleton();
    let (character_time, world_clock) = tokio::join!(
        state.db.query_sats::<CharacterTime>(character_time_sql),
        state.db.query_sats::<WorldClock>(world_clock_sql),
    );
    let _character_time = match character_time {
        Ok(value) => value,
        Err(error) => {
            tracing::error!(%error, "failed to load character time");
            return (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "Strategic time is unavailable",
            )
                .into_response();
        }
    };
    let world_clock = match world_clock {
        Ok(value) => value,
        Err(error) => {
            tracing::error!(%error, "failed to load world clock");
            return (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "Strategic time is unavailable",
            )
                .into_response();
        }
    };
    let official_minutes = world_clock.first().map_or(0, |clock| {
        let now_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();
        let now = i64::try_from(now_micros).unwrap_or(i64::MAX);
        OfficialClockEpoch::from(clock.epoch_micros)
            .at(UnixMicrosecondInstant::from(now))
            .get()
    });
    let active_character = state
        .db
        .query_sats_into::<adventuresim_stdb_client::Character, CharacterView>(
            crate::spacetimedb::character_by_id(character_id.into()),
        )
        .await
        .unwrap_or_default()
        .into_iter()
        .next();
    let display_minutes = if let Some(character) = active_character.as_ref() {
        if character.current_settlement_id.is_some() {
            official_minutes
        } else if let Some(party_id) = character.party_id.as_deref() {
            state
                .db
                .query_sats_into::<adventuresim_stdb_client::Party, PartyView>(
                    crate::spacetimedb::party_by_id(party_id),
                )
                .await
                .unwrap_or_default()
                .into_iter()
                .next()
                .and_then(|party| {
                    party.wilderness_canonical_anchor_minute.map(|anchor| {
                        anchor
                            .with_wrapped_time_of_day(
                                party.journey_start_minute_of_day,
                                party.wilderness_elapsed_minutes,
                            )
                            .get()
                    })
                })
                .unwrap_or(official_minutes)
        } else {
            official_minutes
        }
    } else {
        official_minutes
    };
    Json(CurrentTime {
        character_minutes: display_minutes,
        official_minutes,
    })
    .into_response()
}
