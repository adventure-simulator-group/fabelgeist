use crate::spacetimedb::SqlQuery;
use axum::{
    Form, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::data::{self, FrontierAlignment};
use super::{AppState, character_case_site_id};
mod actor;
mod error;
mod presence;
mod resident;
use crate::{
    session::Session,
    spacetimedb::{
        BackendLocalChatMessage, BackendSettlementResident, CharacterView, PartyMember,
        SettlementCategory, SettlementResidentPresence, SettlementView, SpacetimeError,
        sql_string_literal,
    },
};
use actor::{actor_strategic_minute, require_player_frontier, selected_actor};
use adventuresim_core::identity::CharacterId;
use error::{ChatAuthorizationError, ChatReadStage};
use presence::LocalPlayerPresence;
use resident::ResidentChatEvidence;

const MAX_CHAT_HISTORY: usize = 200;
const MAX_INCOMING_PLAYERS: usize = 50;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/local-chat/{kind}/{subject_id}",
            get(messages).post(send_message),
        )
        .route("/api/local-chat/incoming", get(incoming))
}

#[derive(Serialize)]
struct LocalChatResponse {
    messages: Vec<LocalChatMessageView>,
}

#[derive(Serialize)]
struct LocalChatMessageView {
    id: u64,
    sender_id: u64,
    sender_name: String,
    body: String,
    created_micros: i64,
}

impl From<BackendLocalChatMessage> for LocalChatMessageView {
    fn from(message: BackendLocalChatMessage) -> Self {
        let BackendLocalChatMessage {
            id,
            owner_character_id: _,
            conversation_kind: _,
            subject_party_id: _,
            subject_resident_character_id: _,
            sender_id,
            sender_name,
            body,
            created_micros,
        } = message;
        Self {
            id,
            sender_id,
            sender_name,
            body,
            created_micros,
        }
    }
}

#[derive(Deserialize)]
struct MessageForm {
    body: String,
    #[serde(default)]
    location_id: String,
}

#[derive(Default, Deserialize)]
struct LocationQuery {
    #[serde(default)]
    location_id: String,
}

fn npc_authority_matches(
    settlement_id: &str,
    npc: &BackendSettlementResident,
    presence: &SettlementResidentPresence,
    requested_location_id: &str,
    minute: adventuresim_world_schema::calendar::StrategicMinute,
) -> bool {
    npc.character_id == presence.character_id
        && npc.home_settlement_id == settlement_id
        && presence.settlement_id == settlement_id
        && presence.location_id == requested_location_id
        && !requested_location_id.is_empty()
        && minute.contains_daily_window(presence.start_minute, presence.end_minute)
}

fn npc_history_location_is_navigable(
    profile: &adventuresim_world_schema::SettlementEconomyProfile,
    category: &SettlementCategory,
    settlement_id: &str,
    location_id: &str,
) -> bool {
    let has_keep = matches!(
        category,
        SettlementCategory::Town | SettlementCategory::City | SettlementCategory::Capital
    );
    adventuresim_core::settlement_economy::npc_location_is_navigable(
        profile,
        has_keep,
        settlement_id,
        location_id,
    )
}

enum ConversationSelector {
    Npc(String),
    PlayerParty(String),
}

async fn actor_and_selector(
    state: &AppState,
    actor_id: CharacterId,
    kind: &str,
    subject_id: &str,
    location_id: &str,
) -> std::result::Result<(CharacterView, ConversationSelector), ChatAuthorizationError> {
    let actor = selected_actor(state, actor_id).await?;
    let selector = match kind {
        "npc" => {
            let settlement = actor
                .current_settlement_id
                .as_deref()
                .ok_or(ChatAuthorizationError::NpcNotLocal)?;
            let settlement_authority = state
                .db
                .query_one_sats_into::<adventuresim_stdb_client::Settlement, SettlementView>(
                    crate::spacetimedb::settlement_by_id(settlement),
                )
                .await
                .map_err(|source: SpacetimeError| -> ChatAuthorizationError {
                    ChatAuthorizationError::database(
                        ChatReadStage::Settlement,
                        actor_id,
                        None,
                        source,
                    )
                })?
                .ok_or(ChatAuthorizationError::NpcNotLocal)?;
            if !npc_history_location_is_navigable(
                &settlement_authority.economy,
                &settlement_authority.category,
                settlement,
                location_id,
            ) {
                return Err(ChatAuthorizationError::NpcNotLocal);
            }
            let resident_character_id = CharacterId::from(
                subject_id
                    .parse::<u64>()
                    .map_err(ChatAuthorizationError::InvalidNpcSubject)?,
            );
            let evidence =
                ResidentChatEvidence::load(state, actor_id, resident_character_id).await?;
            let minute = actor_strategic_minute(state, actor.id.into()).await?;
            if !npc_authority_matches(
                settlement,
                &evidence.npc,
                &evidence.presence,
                location_id,
                minute,
            ) {
                return Err(ChatAuthorizationError::NpcNotLocal);
            }
            ConversationSelector::Npc(subject_id.to_string())
        }
        "player" => {
            if !location_id.is_empty() {
                return Err(ChatAuthorizationError::UnexpectedPlayerLocation);
            }
            let id = CharacterId::from(
                subject_id
                    .parse::<u64>()
                    .map_err(ChatAuthorizationError::InvalidPlayerSubject)?,
            );
            let subject = super::data::character_as_observed(state, id, actor.id.into())
                .await
                .map_err(|source: SpacetimeError| -> ChatAuthorizationError {
                    ChatAuthorizationError::database(
                        ChatReadStage::ObservedPlayer,
                        actor_id,
                        Some(id),
                        source,
                    )
                })?
                .ok_or(ChatAuthorizationError::PlayerUnavailable(id))?;
            require_player_frontier(state, actor.id.into(), subject.id.into()).await?;
            if LocalPlayerPresence::observe(state, &actor, &subject).await?
                != LocalPlayerPresence::CoLocated
            {
                return Err(ChatAuthorizationError::PlayerElsewhere(id));
            }
            let other = subject
                .party_id
                .as_deref()
                .ok_or(ChatAuthorizationError::PlayerHasNoParty(id))?;
            ConversationSelector::PlayerParty(other.to_string())
        }
        _ => return Err(ChatAuthorizationError::UnknownSubject),
    };
    Ok((actor, selector))
}

async fn messages(
    State(state): State<AppState>,
    Path((kind, subject_id)): Path<(String, String)>,
    Query(query): Query<LocationQuery>,
    session: Session,
) -> std::result::Result<Json<LocalChatResponse>, (StatusCode, String)> {
    let actor_id = session
        .character_id_u64()
        .ok_or((StatusCode::UNAUTHORIZED, "Choose a character".into()))?;
    let (_, selector) = actor_and_selector(
        &state,
        actor_id.into(),
        &kind,
        &subject_id,
        &query.location_id,
    )
    .await
    .map_err(|error: ChatAuthorizationError| -> (StatusCode, String) {
        (StatusCode::FORBIDDEN, error.to_string())
    })?;
    let selector_filter = match &selector {
        ConversationSelector::Npc(resident_character_id) => format!(
            "conversation_kind = 'npc' AND subject_resident_character_id = {}",
            sql_string_literal(resident_character_id)
        ),
        ConversationSelector::PlayerParty(party_id) => format!(
            "conversation_kind = 'player' AND subject_party_id = {}",
            sql_string_literal(party_id)
        ),
    };
    let mut messages = state
        .db
        .query_sats::<BackendLocalChatMessage>(SqlQuery::from(format!(
            "SELECT * FROM backend_local_chat_messages WHERE owner_character_id = {actor_id} AND {selector_filter}"
        )))
        .await
        .map_err(|error: SpacetimeError| -> (StatusCode, String) {
            (StatusCode::SERVICE_UNAVAILABLE, error.to_string())
        })?;
    // Close the gap between the authority read and returning private message bodies.
    actor_and_selector(
        &state,
        actor_id.into(),
        &kind,
        &subject_id,
        &query.location_id,
    )
    .await
    .map_err(|error: ChatAuthorizationError| -> (StatusCode, String) {
        (StatusCode::FORBIDDEN, error.to_string())
    })?;
    messages.sort_by_key(|message| (message.created_micros, message.id));
    if messages.len() > MAX_CHAT_HISTORY {
        messages.drain(..messages.len() - MAX_CHAT_HISTORY);
    }
    Ok(Json(LocalChatResponse {
        messages: messages.into_iter().map(Into::into).collect(),
    }))
}

async fn send_message(
    State(state): State<AppState>,
    Path((kind, subject_id)): Path<(String, String)>,
    Query(query): Query<LocationQuery>,
    session: Session,
    Form(form): Form<MessageForm>,
) -> StatusCode {
    let Some(actor_id) = session.character_id_u64() else {
        return StatusCode::UNAUTHORIZED;
    };
    if query.location_id != form.location_id {
        return StatusCode::BAD_REQUEST;
    }
    match state
        .db
        .call(
            "send_local_chat_message",
            &[
                json!(actor_id),
                json!(kind),
                json!(subject_id),
                json!(form.location_id),
                json!(form.body),
            ],
        )
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

#[derive(Serialize)]
struct IncomingPlayer {
    id: String,
    name: String,
}

async fn incoming(State(state): State<AppState>, session: Session) -> Json<Vec<IncomingPlayer>> {
    let Some(actor_id) = session.character_id_u64() else {
        return Json(Vec::new());
    };
    let Some(actor) = state
        .db
        .query_sats_into::<adventuresim_stdb_client::Character, CharacterView>(
            crate::spacetimedb::character_by_id(actor_id.into()),
        )
        .await
        .ok()
        .and_then(|characters| characters.into_iter().next())
    else {
        return Json(Vec::new());
    };
    let Some(party_id) = actor.party_id.as_deref() else {
        return Json(Vec::new());
    };
    let memberships = state
        .db
        .query_sats::<PartyMember>(SqlQuery::from(format!(
            "SELECT * FROM party_member WHERE party_id = {}",
            sql_string_literal(party_id)
        )))
        .await
        .unwrap_or_default();
    let own: std::collections::HashSet<u64> =
        memberships.into_iter().map(|m| m.character_id).collect();
    let all_messages = state
        .db
        .query_sats::<BackendLocalChatMessage>(SqlQuery::from(format!(
            "SELECT * FROM backend_local_chat_messages WHERE owner_character_id = {actor_id} AND conversation_kind = 'player'"
        )))
        .await
        .unwrap_or_default();
    let mut ids = std::collections::BTreeSet::new();
    for message in &all_messages {
        if message.sender_id != 0 && !own.contains(&message.sender_id) {
            ids.insert(message.sender_id);
        }
    }
    let actor_site = character_case_site_id(&state, actor.id.into())
        .await
        .ok()
        .flatten();
    let mut candidate_sites = std::collections::HashMap::new();
    for id in ids.iter().copied().take(MAX_INCOMING_PLAYERS) {
        candidate_sites.insert(
            id,
            character_case_site_id(&state, id.into())
                .await
                .ok()
                .flatten(),
        );
    }
    let mut visible = Vec::new();
    for id in ids.into_iter().take(MAX_INCOMING_PLAYERS) {
        let alignment = data::frontier_alignment(&state, actor.id.into(), id.into())
            .await
            .unwrap_or(FrontierAlignment::Unknown);
        let candidate = if alignment == FrontierAlignment::Aligned {
            super::data::character_as_observed(&state, id.into(), actor.id.into())
                .await
                .ok()
                .flatten()
        } else {
            None
        };
        if let Some(candidate) = candidate
            && candidate.current_settlement_id == actor.current_settlement_id
            && candidate_sites.get(&candidate.id) == Some(&actor_site)
        {
            visible.push(IncomingPlayer {
                id: candidate.id.to_string(),
                name: candidate.name,
            });
        }
    }
    Json(visible)
}

#[cfg(test)]
mod tests {
    use super::{LocalChatMessageView, npc_authority_matches, npc_history_location_is_navigable};

    use adventuresim_world_schema::calendar::StrategicMinute;

    use crate::spacetimedb::{
        AgeBand, BackendLocalChatMessage, BackendSettlementResident, SettlementCategory,
        SettlementResidentPresence,
    };

    #[test]
    fn local_chat_row_projection_explicitly_omits_authority_columns() {
        let view = LocalChatMessageView::from(BackendLocalChatMessage {
            id: 17,
            owner_character_id: 7,
            conversation_kind: "npc".into(),
            subject_party_id: String::new(),
            subject_resident_character_id: "11".into(),
            sender_id: 11,
            sender_name: "Marta".into(),
            body: "Good morrow".into(),
            created_micros: 123_456,
        });
        assert_eq!((view.id, view.sender_id), (17, 11));
        assert_eq!(
            (view.sender_name.as_str(), view.body.as_str()),
            ("Marta", "Good morrow")
        );
        assert_eq!(view.created_micros, 123_456);
    }

    #[test]
    fn player_chat_co_location_requires_equal_personal_frontiers() {
        let source = include_str!("local_chat.rs");
        let selector = source
            .split("async fn actor_and_selector")
            .nth(1)
            .unwrap()
            .split("async fn messages")
            .next()
            .unwrap();
        assert!(selector.contains("character_as_observed(state, id, actor.id.into())"));
        assert!(
            selector.contains("require_player_frontier(state, actor.id.into(), subject.id.into())")
        );
        let authority = include_str!("local_chat/actor.rs");
        assert!(authority.contains("data::frontier_alignment(state, actor, subject)"));
        assert!(authority.contains("data::FrontierAlignment::Aligned"));

        let incoming = source
            .split("async fn incoming")
            .nth(1)
            .unwrap()
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        assert!(incoming.contains("frontier_alignment(&state, actor.id.into(), id.into())"));
        assert!(!incoming.contains(".query::<Character>(\"SELECT * FROM backend_characters\")"));
    }

    #[test]
    fn hidden_npc_locations_cannot_authorize_chat_history() {
        let mut profile = adventuresim_world_schema::SettlementEconomyProfile::stage_placeholder();
        assert!(npc_history_location_is_navigable(
            &profile,
            &SettlementCategory::Hamlet,
            "fixture-no-orgs",
            "inn"
        ));
        assert!(npc_history_location_is_navigable(
            &profile,
            &SettlementCategory::Hamlet,
            "fixture-no-orgs",
            "residences"
        ));
        assert!(!npc_history_location_is_navigable(
            &profile,
            &SettlementCategory::Hamlet,
            "fixture-no-orgs",
            "church"
        ));
        assert!(!npc_history_location_is_navigable(
            &profile,
            &SettlementCategory::Hamlet,
            "fixture-no-orgs",
            "armoury"
        ));
        assert!(!npc_history_location_is_navigable(
            &profile,
            &SettlementCategory::Hamlet,
            "fixture-no-orgs",
            "keep"
        ));
        profile
            .services
            .push(adventuresim_world_schema::SettlementService::Temple);
        assert!(npc_history_location_is_navigable(
            &profile,
            &SettlementCategory::Town,
            "fixture-no-orgs",
            "church"
        ));
        assert!(npc_history_location_is_navigable(
            &profile,
            &SettlementCategory::Town,
            "fixture-no-orgs",
            "keep"
        ));
    }

    #[test]
    fn riverdale_inn_npc_chain_uses_authority_not_encoded_id_shape() {
        let npc = BackendSettlementResident {
            character_id: 41,
            home_settlement_id: "riverdale".into(),
            name: "Innkeeper".into(),
            age_band: AgeBand::Adult,
            presentation: adventuresim_stdb_client::Presentation::Ambiguous,
            height: String::new(),
            build: String::new(),
            hair: String::new(),
            facial_hair: String::new(),
            complexion: String::new(),
            visible_features: String::new(),
            clothing: String::new(),
            profession: String::new(),
            household_kind: String::new(),
            local_role: String::new(),
            service_id: "inn".into(),
            organization_id: String::new(),
            conversation_id: "service-professions".into(),
        };
        let mut presence = SettlementResidentPresence {
            character_id: npc.character_id,
            settlement_id: "riverdale".into(),
            location_id: "inn".into(),
            start_minute: 0,
            end_minute: adventuresim_world_schema::calendar::MINUTES_PER_DAY as u16,
            is_default: true,
            context_suppressed: false,
            health_suppressed: false,
        };
        assert!(npc_authority_matches(
            "riverdale",
            &npc,
            &presence,
            "inn",
            adventuresim_world_schema::calendar::StrategicMinute::new(720)
        ));
        presence.start_minute = 1_200;
        presence.end_minute = 120;
        for minute in [1_380, 60] {
            assert!(npc_authority_matches(
                "riverdale",
                &npc,
                &presence,
                "inn",
                StrategicMinute::new(minute)
            ));
        }
        assert!(!npc_authority_matches(
            "riverdale",
            &npc,
            &presence,
            "inn",
            StrategicMinute::new(720)
        ));
        presence.start_minute = 0;
        presence.end_minute = adventuresim_world_schema::calendar::MINUTES_PER_DAY as u16;

        presence.settlement_id = "ironforge".into();
        assert!(!npc_authority_matches(
            "riverdale",
            &npc,
            &presence,
            "inn",
            adventuresim_world_schema::calendar::StrategicMinute::new(720)
        ));
        presence.settlement_id = "riverdale".into();
        presence.end_minute = 600;
        assert!(!npc_authority_matches(
            "riverdale",
            &npc,
            &presence,
            "inn",
            adventuresim_world_schema::calendar::StrategicMinute::new(720)
        ));
        presence.end_minute = adventuresim_world_schema::calendar::MINUTES_PER_DAY as u16;
        assert!(!npc_authority_matches(
            "riverdale",
            &npc,
            &presence,
            "market",
            adventuresim_world_schema::calendar::StrategicMinute::new(720)
        ));
    }

    #[test]
    fn npc_selection_waits_for_a_subject_and_preserves_reducer_diagnostics() {
        let javascript = include_str!("../../static/local-chat.js");
        assert!(javascript.contains("if (!kind || !subject) return null"));
        assert!(javascript.matches("if (!endpoint) return;").count() >= 2);
        assert!(!javascript.contains("/api/local-chat/${encodeURIComponent(node.dataset"));

        let local_route = include_str!("local_chat.rs")
            .split("async fn actor_and_selector")
            .nth(1)
            .and_then(|tail| tail.split("async fn messages").next())
            .expect("local chat authority handler");
        assert!(
            local_route
                .contains("ResidentChatEvidence::load(state, actor_id, resident_character_id)")
        );
        let resident_reads = include_str!("local_chat/resident.rs");
        assert!(resident_reads.contains("settlement_resident_by_character_id(resident)"));
        assert!(resident_reads.contains("settlement_resident_presence_by_character_id(resident)"));
        assert!(
            include_str!("local_chat.rs").contains("presence.location_id == requested_location_id")
        );
        assert!(!local_route.contains("subject_id.starts_with"));

        let dialogue_route = include_str!("dialogue.rs");
        assert!(dialogue_route.contains("start_dialogue reducer rejected an NPC encounter"));
        assert!(dialogue_route.contains("StatusCode::CONFLICT"));
    }
}
