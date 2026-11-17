//! Owning-client acknowledgements for equipment intents.

use adventuresim_tactical_core::inventory::{EquipmentRevision, EquipmentSequence};
use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Event, Clone, Debug, Serialize, Deserialize, MapEntities)]
pub struct EquipmentActionResponse {
    #[entities]
    pub actor: Entity,
    pub sequence: EquipmentSequence,
    pub outcome: EquipmentActionOutcome,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EquipmentActionOutcome {
    Pending,
    Committed { revision: EquipmentRevision },
    Rejected(EquipmentRejection),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum EquipmentRejection {
    #[error("You no longer control that character.")]
    Uncontrolled,
    #[error("Your equipment is not available yet.")]
    Unavailable,
    #[error("An equipment change is already pending.")]
    Busy,
    #[error("That equipment request has already been handled.")]
    Replayed,
    #[error("Your equipment changed. Choose the action again.")]
    Stale,
    #[error("Your character cannot use that hand right now.")]
    Unable,
    #[error("That item cannot be moved to the selected location.")]
    InvalidPlacement,
    #[error("Dropping and picking up items is currently unavailable.")]
    SceneCustodyUnavailable,
    #[error("{0}")]
    Durable(String),
}
