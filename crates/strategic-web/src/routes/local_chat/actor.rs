//! Selected-actor admission and the established missing-clock chat policy.

use super::{
    AppState,
    error::{ChatAuthorizationError, ChatReadStage},
};
use crate::spacetimedb::{CharacterTime, CharacterView, SpacetimeError};
use adventuresim_core::identity::CharacterId;
use adventuresim_world_schema::calendar::StrategicMinute;

const DEFAULT_CHAT_MINUTE: StrategicMinute = StrategicMinute::new(720);

pub(super) async fn selected_actor(
    state: &AppState,
    actor: CharacterId,
) -> std::result::Result<CharacterView, ChatAuthorizationError> {
    let character = state
        .db
        .query_sats_into::<adventuresim_stdb_client::Character, CharacterView>(
            crate::spacetimedb::character_by_id(actor),
        )
        .await
        .map_err(|source: SpacetimeError| -> ChatAuthorizationError {
            ChatAuthorizationError::database(ChatReadStage::Actor, actor, None, source)
        })?
        .into_iter()
        .next()
        .ok_or(ChatAuthorizationError::MissingActor(actor))?;
    character
        .party_id
        .as_deref()
        .ok_or(ChatAuthorizationError::ActorHasNoParty(actor))?;
    Ok(character)
}

pub(super) async fn actor_strategic_minute(
    state: &AppState,
    actor: CharacterId,
) -> std::result::Result<StrategicMinute, ChatAuthorizationError> {
    let time = state
        .db
        .query_one_sats::<CharacterTime>(crate::spacetimedb::character_time_by_character_id(actor))
        .await
        .map_err(|source: SpacetimeError| -> ChatAuthorizationError {
            ChatAuthorizationError::database(ChatReadStage::ActorClock, actor, None, source)
        })?;
    Ok(match time {
        Some(time) => StrategicMinute::new(time.minutes.minutes),
        None => DEFAULT_CHAT_MINUTE,
    })
}

pub(super) async fn require_player_frontier(
    state: &AppState,
    actor: CharacterId,
    subject: CharacterId,
) -> std::result::Result<(), ChatAuthorizationError> {
    let alignment = super::super::data::frontier_alignment(state, actor, subject)
        .await
        .map_err(|source: SpacetimeError| -> ChatAuthorizationError {
            ChatAuthorizationError::database(ChatReadStage::Frontier, actor, Some(subject), source)
        })?;
    if alignment == super::super::data::FrontierAlignment::Aligned {
        Ok(())
    } else {
        Err(ChatAuthorizationError::PlayerElsewhere(subject))
    }
}
