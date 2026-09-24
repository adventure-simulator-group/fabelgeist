//! Location and conversation HTTP route assembly.

use super::*;
use crate::location_urls::patterns as paths;
use axum::{
    Router,
    routing::{get, post},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/dialogue/start", post(start))
        .route("/api/dialogue/{session_id}", get(view))
        .route("/api/dialogue/topic", post(topic))
        .route("/api/dialogue/answer", post(answer))
        .route(
            "/api/dialogue/accept-order-errantry",
            post(accept_order_errantry),
        )
        .route("/api/dialogue/join", post(join))
        .route("/api/dialogue/claim-response", post(witness_approach))
        .route(paths::LOCATION_NPCS.pattern(), get(location_npcs))
        .route(
            paths::NPC_SOCIAL.pattern(),
            get(npc_social).post(chat_with_npc),
        )
        .route(
            paths::NPC_ROMANCE_ACTION.pattern(),
            post(npc_romance_action),
        )
        .layer(axum::middleware::from_fn(
            crate::location_urls::require_canonical_location_path,
        ))
}
