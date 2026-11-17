//! Dialogue actions travel through the host that owns physical contact.

use adventuresim_core::dialogue_boundary::{DialoguePromptRowId, DialogueSessionId};
use adventuresim_tactical_core::{
    control::{ControlGeneration, OrderSequence},
    player::CharacterId,
};
use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct DialogueRequest {
    pub generation: ControlGeneration,
    pub sequence: OrderSequence,
    pub session_id: DialogueSessionId,
    pub action: DialogueAction,
}

/// A start result belongs to the original approach, including failed starts.
#[derive(Event, Clone, Debug, Serialize, Deserialize, MapEntities)]
pub struct DialogueStartResponse {
    #[entities]
    pub controlled_actor: Entity,
    pub actor: CharacterId,
    pub generation: ControlGeneration,
    pub sequence: OrderSequence,
    pub resident: CharacterId,
    pub result: Result<DialogueSessionId, String>,
}

#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct DialogueActionResponse {
    pub session_id: DialogueSessionId,
    pub action_id: String,
    pub result: Result<(), String>,
}

#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct DialogueEnded {
    pub session_id: DialogueSessionId,
}

/// Catalog keys and action receipts are decoded at this transport boundary;
/// actor identity always comes from the sending connection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DialogueAction {
    Topic {
        topic_id: String,
        action_id: String,
        expected_revision: u64,
    },
    Answer {
        prompt_row_id: DialoguePromptRowId,
        choice_ids: Vec<String>,
        action_id: String,
        expected_revision: u64,
    },
    Close,
}

impl DialogueAction {
    pub fn action_id(&self) -> Option<&str> {
        match self {
            Self::Topic { action_id, .. } | Self::Answer { action_id, .. } => Some(action_id),
            Self::Close => None,
        }
    }
}
