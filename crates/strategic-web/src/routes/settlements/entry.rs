//! Activation shared by both settlement entry pages.

use super::AppState;
use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock},
    time::Duration,
};

const ACTIVATE_SETTLEMENT: &str = "ensure_settlement_activity";
const ACTIVATION_RETRY_DELAY: Duration = Duration::from_secs(30);

pub(super) fn activate_settlement(state: &AppState, id: &str) {
    static SCHEDULED_SETTLEMENTS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let mut scheduled = match SCHEDULED_SETTLEMENTS
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
    {
        Ok(scheduled) => scheduled,
        Err(error) => {
            tracing::error!(%error, "settlement activity scheduler lock is poisoned");
            return;
        }
    };
    if !scheduled.insert(id.to_owned()) {
        return;
    }
    drop(scheduled);

    let db = state.db.clone();
    let settlement_id = id.to_owned();
    tokio::spawn(async move {
        if let Err(error) = db
            .call(ACTIVATE_SETTLEMENT, &[serde_json::json!(settlement_id)])
            .await
        {
            tracing::warn!(%error, settlement_id, "failed to activate settlement activity");
            tokio::time::sleep(ACTIVATION_RETRY_DELAY).await;
            if let Ok(mut scheduled) = SCHEDULED_SETTLEMENTS
                .get_or_init(|| Mutex::new(HashSet::new()))
                .lock()
            {
                scheduled.remove(&settlement_id);
            }
        }
    });
}
