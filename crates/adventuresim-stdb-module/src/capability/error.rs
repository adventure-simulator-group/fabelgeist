//! Capability evaluation retains missing state and admitted disease causes.

use crate::disease::EpisodeDecodeError;
use adventuresim_core::identity::CharacterId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CapabilityComponent {
    Attributes,
    Skills,
    Limbs,
    Stats,
    Condition,
}

#[derive(Debug)]
pub(crate) enum CapabilityEvaluationError {
    Missing {
        character: CharacterId,
        component: CapabilityComponent,
    },
    DiseaseEpisode(EpisodeDecodeError),
}

impl From<EpisodeDecodeError> for CapabilityEvaluationError {
    fn from(source: EpisodeDecodeError) -> Self {
        Self::DiseaseEpisode(source)
    }
}

impl std::fmt::Display for CapabilityEvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DiseaseEpisode(source) => source.fmt(f),
            Self::Missing { component, .. } => f.write_str(match component {
                CapabilityComponent::Attributes => "Character attributes not found",
                CapabilityComponent::Skills => "Character skills not found",
                CapabilityComponent::Limbs => "Character limbs not found",
                CapabilityComponent::Stats => "Character stats not found",
                CapabilityComponent::Condition => "Character condition not found",
            }),
        }
    }
}

impl std::error::Error for CapabilityEvaluationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DiseaseEpisode(source) => Some(source),
            Self::Missing { .. } => None,
        }
    }
}
