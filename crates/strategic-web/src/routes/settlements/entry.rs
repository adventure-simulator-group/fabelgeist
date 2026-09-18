//! Activation shared by both settlement entry pages.

use super::AppState;

const ACTIVATE_SETTLEMENT: &str = "ensure_settlement_activity";

pub(super) async fn activate_settlement(state: &AppState, id: &str) {
    if let Err(error) = state
        .db
        .call(ACTIVATE_SETTLEMENT, &[serde_json::json!(id)])
        .await
    {
        tracing::warn!(%error, settlement_id = id, "failed to activate settlement activity");
    }
}
