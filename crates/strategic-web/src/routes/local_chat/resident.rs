//! Resident and presence evidence are read in order from the same subject key.

use super::{
    AppState,
    error::{ChatAuthorizationError, ChatReadStage},
};
use crate::spacetimedb::{BackendSettlementResident, SettlementResidentPresence, SpacetimeError};
use adventuresim_core::identity::CharacterId;

pub(super) struct ResidentChatEvidence {
    pub(super) npc: BackendSettlementResident,
    pub(super) presence: SettlementResidentPresence,
}

impl ResidentChatEvidence {
    pub(super) async fn load(
        state: &AppState,
        actor: CharacterId,
        resident: CharacterId,
    ) -> std::result::Result<Self, ChatAuthorizationError> {
        let npc = state
            .db
            .query_one_sats::<BackendSettlementResident>(
                crate::spacetimedb::settlement_resident_by_character_id(resident),
            )
            .await
            .map_err(|source: SpacetimeError| -> ChatAuthorizationError {
                ChatAuthorizationError::database(
                    ChatReadStage::Resident,
                    actor,
                    Some(resident),
                    source,
                )
            })?
            .ok_or(ChatAuthorizationError::NpcNotLocal)?;
        let presence = state
            .db
            .query_one_sats::<SettlementResidentPresence>(
                crate::spacetimedb::settlement_resident_presence_by_character_id(resident),
            )
            .await
            .map_err(|source: SpacetimeError| -> ChatAuthorizationError {
                ChatAuthorizationError::database(
                    ChatReadStage::Presence,
                    actor,
                    Some(resident),
                    source,
                )
            })?
            .ok_or(ChatAuthorizationError::NpcNotLocal)?;
        Ok(Self { npc, presence })
    }
}
