//! Failures while admitting and projecting strategic condition state.

use crate::{capability::CapabilityEvaluationError, time::WorldClockError};
use adventuresim_core::identity::CharacterId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConditionComponent {
    Character,
    Attributes,
    Limbs,
    Stats,
    Skills,
    Condition,
    Needs,
    Exposure,
    PartyMember,
}

#[derive(Debug)]
pub(crate) enum StrategicConditionError {
    Missing {
        character: CharacterId,
        component: ConditionComponent,
    },
    UnknownReligion {
        character: CharacterId,
        stored_key: String,
    },
    UnknownThreat {
        character: CharacterId,
        stored_key: String,
        source: adventuresim_core::bestiary::UnknownThreatId,
    },
    NotPartyMember {
        character: CharacterId,
    },
    Capability(CapabilityEvaluationError),
    Clock(WorldClockError),
}

impl From<CapabilityEvaluationError> for StrategicConditionError {
    fn from(source: CapabilityEvaluationError) -> Self {
        Self::Capability(source)
    }
}

impl From<WorldClockError> for StrategicConditionError {
    fn from(source: WorldClockError) -> Self {
        Self::Clock(source)
    }
}

impl std::fmt::Display for StrategicConditionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { component, .. } => f.write_str(match component {
                ConditionComponent::Character => "Character not found",
                ConditionComponent::Attributes => "Character attributes not found",
                ConditionComponent::Limbs => "Character limbs not found",
                ConditionComponent::Stats => "Character stats not found",
                ConditionComponent::Skills => "Character skills not found",
                ConditionComponent::Condition => "Character condition not found",
                ConditionComponent::Needs => "Character needs not found",
                ConditionComponent::Exposure => "Character exposure not found",
                ConditionComponent::PartyMember => "Party member not found",
            }),
            Self::UnknownReligion { .. } => f.write_str("Character has an unknown religion"),
            Self::UnknownThreat { stored_key, .. } => {
                write!(f, "Unknown threat ID in quest: {stored_key}")
            }
            Self::NotPartyMember { .. } => f.write_str("Character is not a member of their party"),
            Self::Capability(source) => source.fmt(f),
            Self::Clock(source) => source.fmt(f),
        }
    }
}

impl std::error::Error for StrategicConditionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Capability(source) => Some(source),
            Self::Clock(source) => Some(source),
            Self::UnknownThreat { source, .. } => Some(source),
            _ => None,
        }
    }
}
