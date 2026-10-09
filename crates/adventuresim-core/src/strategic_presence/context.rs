//! Contextual cast and interaction vocabulary shared by authoring and views.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "spacetimedb", runtime_catalog),
    derive(spacetimedb::SpacetimeType)
)]
#[serde(rename_all = "snake_case")]
pub enum CharacterContextRole {
    Counterparty,
    Patient,
    Bystander,
}

/// Authored answer retained by contextual authority. Live presence and privacy
/// may still make an otherwise allowed interaction unavailable to an observer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "spacetimedb", runtime_catalog),
    derive(spacetimedb::SpacetimeType)
)]
#[serde(rename_all = "snake_case")]
pub enum ContextualDecisionState {
    Allowed,
    Refused,
    Unavailable,
}

impl ContextualDecisionState {
    /// Ordinary presentation before observer-specific restrictions or emergency
    /// treatment policy. A permitted answer offers a request, not completion.
    pub const fn presentation(self) -> InteractionPresentationDecision {
        match self {
            Self::Allowed => InteractionPresentationDecision::Request,
            Self::Refused => InteractionPresentationDecision::Refused,
            Self::Unavailable => InteractionPresentationDecision::Unavailable,
        }
    }
}

/// Observer-specific interaction affordance, distinct from the authored answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "spacetimedb", runtime_catalog),
    derive(spacetimedb::SpacetimeType)
)]
#[serde(rename_all = "snake_case")]
pub enum InteractionPresentationDecision {
    Request,
    Refused,
    Unavailable,
    EmergencyTreatment,
}
