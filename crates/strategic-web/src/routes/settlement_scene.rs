//! Ordinary town admission uses selected-character presence, without a mission.

use adventuresim_core::{
    strategic_place::StrategicIdentityComponent, tactical_session::TacticalInstanceId,
};
use adventuresim_stdb_client::{TacticalServer, TacticalServerRequest, TacticalSession};
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use maud::{Markup, html};
use serde::Deserialize;

use std::net::SocketAddr;

use super::AppState;
use crate::{session::Session, spacetimedb::CharacterView, templates::scene_entry_layout};

const REQUEST_SETTLEMENT_SCENE_REDUCER: &str = "request_settlement_scene";
const SCENE_PATH: &str = "/scenes/settlement";

#[derive(Default, Deserialize)]
struct StatusQuery {
    #[serde(default)]
    fragment: bool,
}

enum SceneStatus {
    Ready(SocketAddr),
    Preparing,
    Unavailable,
}

enum ViewerFailure {
    SelectCharacter,
    Unavailable,
    Absent,
}

impl IntoResponse for ViewerFailure {
    fn into_response(self) -> Response {
        match self {
            Self::SelectCharacter => Redirect::to("/characters").into_response(),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Character data is unavailable",
            )
                .into_response(),
            Self::Absent => (
                StatusCode::CONFLICT,
                "Enter a settlement before joining its scene",
            )
                .into_response(),
        }
    }
}

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route(SCENE_PATH, get(status).post(enter))
}

async fn viewer(state: &AppState, session: &Session) -> Result<CharacterView, ViewerFailure> {
    let Some(id) = session.character_id_u64() else {
        return Err(ViewerFailure::SelectCharacter);
    };
    let character = crate::routes::data::character(state, id)
        .await
        .map_err(|error| {
            tracing::error!(%error, "Failed to load settlement scene viewer");
            ViewerFailure::Unavailable
        })?
        .ok_or(ViewerFailure::SelectCharacter)?;
    if !character.alive
        || character.current_settlement_id.is_none()
        || character.current_case_site_id.is_some()
    {
        return Err(ViewerFailure::Absent);
    }
    Ok(character)
}

async fn enter(State(state): State<AppState>, session: Session) -> Response {
    let character = match viewer(&state, &session).await {
        Ok(character) => character,
        Err(error) => return error.into_response(),
    };
    match state
        .db
        .call(
            REQUEST_SETTLEMENT_SCENE_REDUCER,
            &[serde_json::json!(character.id)],
        )
        .await
    {
        Ok(()) => Redirect::to(SCENE_PATH).into_response(),
        Err(error) => {
            tracing::warn!(%error, "Settlement scene request rejected");
            (
                StatusCode::CONFLICT,
                "Could not enter town. Return and try again.",
            )
                .into_response()
        }
    }
}

async fn status(
    State(state): State<AppState>,
    Query(query): Query<StatusQuery>,
    session: Session,
) -> Response {
    let character = match viewer(&state, &session).await {
        Ok(character) => character,
        Err(error) => return error.into_response(),
    };
    let scene = match scene_status(&state, &character).await {
        Ok(scene) => scene,
        Err(error) => {
            tracing::error!(%error, "Settlement scene status failed");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Town entry is unavailable. Try again shortly.",
            )
                .into_response();
        }
    };
    let content = status_fragment(&scene, character.id);
    if query.fragment {
        return Html(content.into_string()).into_response();
    }
    Html(status_page(&scene, character.id, &character.name).into_string()).into_response()
}

fn status_page(scene: &SceneStatus, character_id: u64, name: &str) -> Markup {
    scene_entry_layout(
        "Entering town",
        html! {
            aside class="left-sidebar notice-rail" aria-hidden="true" {}
            main class="center-content" {
                h2 class="page-title" { "Entering town" }
                (status_fragment(scene, character_id))
            }
            aside class="right-sidebar notice-rail" aria-hidden="true" {}
        },
        Some(name),
    )
}

async fn scene_status(
    state: &AppState,
    character: &CharacterView,
) -> crate::spacetimedb::Result<SceneStatus> {
    let settlement_id = character.current_settlement_id.as_deref().ok_or_else(|| {
        crate::spacetimedb::SpacetimeError::Spacetime("No current settlement".into())
    })?;
    let component = StrategicIdentityComponent::try_new(settlement_id)
        .map_err(|error| crate::spacetimedb::SpacetimeError::Spacetime(error.to_string()))?;
    let instance = TacticalInstanceId::for_settlement(&component).into_string();
    if let Some(server) = state
        .db
        .query_one_sats::<TacticalServer>(&crate::spacetimedb::tactical_server_by_instance_id(
            &instance,
        ))
        .await?
    {
        if matches!(&server.session, TacticalSession::Settlement(snapshot) if snapshot.id == settlement_id)
        {
            let address = server.addr.parse::<SocketAddr>().map_err(|error| {
                crate::spacetimedb::SpacetimeError::Spacetime(format!(
                    "Invalid scene address: {error}"
                ))
            })?;
            return Ok(SceneStatus::Ready(address));
        }
        return Err(crate::spacetimedb::SpacetimeError::Spacetime(
            "Scene purpose does not match its slot".into(),
        ));
    }
    let request = state
        .db
        .query_one_sats::<TacticalServerRequest>(
            &crate::spacetimedb::tactical_server_request_by_instance_id(&instance),
        )
        .await?;
    Ok(match request.map(|request| request.provisioning) {
        Some(adventuresim_stdb_client::TacticalProvisioningStatus::Pending) => {
            SceneStatus::Preparing
        }
        Some(adventuresim_stdb_client::TacticalProvisioningStatus::Failed) | None => {
            SceneStatus::Unavailable
        }
    })
}

fn status_fragment(scene: &SceneStatus, character_id: u64) -> Markup {
    html! {
        section # "settlement-scene-status" aria-live="polite" data-live-refresh
            "data-on:strategic-live-update"="@get('/scenes/settlement?fragment=true')" {
            @match scene {
                SceneStatus::Ready(address) => {
                    a class="btn btn-primary btn-large" data-persistent-tactical
                        data-server-addr=(address) data-character-id=(character_id)
                        href=(format!("/tactical/tactical.html?server={}&id={character_id}&autostart=1", address)) {
                        "Enter town"
                    }
                }
                SceneStatus::Preparing => { p { "Preparing your arrival…" } }
                SceneStatus::Unavailable => {
                    p { "Could not enter town." }
                    form method="post" action=(SCENE_PATH) {
                        button class="btn btn-primary" type="submit" { "Try again" }
                    }
                }
            }
            a class="btn btn-secondary mt-2" href="/" { "Return" }
        }
    }
}

#[cfg(test)]
mod interface_review {
    use super::*;

    #[test]
    fn export_scene_entry_review_fixtures() {
        let Ok(directory) = std::env::var("UX_CAPTURE_DIR") else {
            return;
        };
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).unwrap();
        for (name, scene) in [
            (
                "ready",
                SceneStatus::Ready("127.0.0.1:6000".parse().unwrap()),
            ),
            ("preparing", SceneStatus::Preparing),
            ("unavailable", SceneStatus::Unavailable),
        ] {
            std::fs::write(
                directory.join(format!("town-scene-{name}.html")),
                status_page(&scene, 41, "Town visitor").into_string(),
            )
            .unwrap();
        }
    }
}
