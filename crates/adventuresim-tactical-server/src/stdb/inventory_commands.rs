//! Owned reducer acknowledgements; callbacks never mutate tactical entities.

use adventuresim_core::inventory_command::{InventoryCommandId, InventoryScopeId};
use adventuresim_tactical_core::prelude::CharacterId;

use super::*;

pub(crate) struct InventorySubmissionResult {
    pub(crate) command: InventoryCommandId,
    pub(crate) outcome: InventorySubmissionOutcome,
}

pub(crate) enum InventorySubmissionOutcome {
    Committed,
    Rejected(String),
    AuthorityLost(String),
}

impl SpacetimeDb {
    /// This view shares the inventory subscription. Observing a receipt proves
    /// the successful transaction has reached the SDK cache, even if its reducer
    /// acknowledgement arrived before its view updates.
    pub(crate) fn has_inventory_receipt(
        &self,
        command: &InventoryCommandId,
        character: CharacterId,
        scope: InventoryScopeId,
    ) -> bool {
        self.conn
            .db
            .tactical_inventory_receipt()
            .iter()
            .any(|receipt| {
                receipt.command_id == command.as_str()
                    && receipt.actor_character_id == character.0
                    && receipt.participants.iter().any(|actor| {
                        actor.character_id == character.0 && actor.scope.as_slice() == scope.bytes()
                    })
            })
    }

    pub(crate) fn submit_inventory(
        &self,
        command: InventoryCommandId,
        actor: TacticalInventoryActorState,
        action: TacticalInventoryAction,
    ) -> spacetimedb_sdk::Result<()> {
        let mailbox = Arc::clone(&self.inventory_results);
        let connection = Arc::clone(&self.scene_operations);
        let request = TacticalInventoryCommand {
            command_id: command.as_str().to_owned(),
            actor,
            action,
        };
        self.reducers()
            .commit_tactical_inventory_then(request, move |_, result| {
                let outcome = match result {
                    Ok(Ok(())) => InventorySubmissionOutcome::Committed,
                    Ok(Err(reason)) => InventorySubmissionOutcome::Rejected(reason),
                    Err(error) => InventorySubmissionOutcome::AuthorityLost(format!(
                        "Inventory reducer acknowledgement failed: {error}"
                    )),
                };
                match mailbox.lock() {
                    Ok(mut mailbox) => mailbox.push(InventorySubmissionResult { command, outcome }),
                    Err(error) => scene_operations::connection_lost(
                        &connection,
                        format!("Inventory acknowledgement mailbox failed: {error}"),
                    ),
                }
            })
    }

    pub(crate) fn take_inventory_results(&self) -> Result<Vec<InventorySubmissionResult>, String> {
        self.inventory_results
            .lock()
            .map(|mut mailbox| std::mem::take(&mut *mailbox))
            .map_err(|error| format!("Inventory acknowledgement mailbox failed: {error}"))
    }
}
