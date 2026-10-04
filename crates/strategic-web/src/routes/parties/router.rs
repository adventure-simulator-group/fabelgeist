//! Party HTTP routes and their handler binding.

use super::*;
use axum::{
    Router,
    routing::{get, post},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/party-roles/{id}/join", post(join_party))
        .route("/parties/{id}/join-general", post(join_general_party))
        .route("/party-recruitment/panel", get(recruitment_panel_fragment))
        .route("/party-recruitment/roles", post(create_recruitment_role))
        .route(
            "/party-recruitment/roles/{id}",
            post(update_recruitment_role),
        )
        .route(
            "/party-recruitment/roles/{id}/delete",
            post(delete_recruitment_role),
        )
        .route("/party-recruitment/saved", post(save_recruitment_role))
        .route(
            "/party-recruitment/check-targets",
            post(update_party_check_targets),
        )
        .route(
            "/party-recruitment/saved/{id}/delete",
            post(delete_saved_role),
        )
        .route(
            "/party-recruitment/saved/{id}/rename",
            post(rename_saved_role),
        )
        .route(
            "/parties/{id}/requests/{request_id}/accept",
            post(accept_join_request),
        )
        .route(
            "/parties/{id}/requests/{request_id}/reject",
            post(reject_join_request),
        )
        .route("/party-notifications", get(party_notifications))
        .route(
            "/party-action-requests/{id}/approve",
            post(approve_action_request),
        )
        .route(
            "/party-action-requests/{id}/deny",
            post(deny_action_request),
        )
        .route("/party-leader-votes/{candidate_id}", post(vote_for_leader))
        .route("/parties/{id}/leave", post(leave_party))
        .route("/parties/{id}/disband", post(disband_party))
}
