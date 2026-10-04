//! Point exposure preserves historical presence and live capability failures.

use crate::capability::CapabilityEvaluationError;
use adventuresim_core::disease::DiseaseIntervalError;

#[derive(Debug)]
pub(crate) enum DiseaseProtectionError {
    Presence(DiseaseIntervalError),
    Capability(CapabilityEvaluationError),
}

impl From<DiseaseIntervalError> for DiseaseProtectionError {
    fn from(source: DiseaseIntervalError) -> Self {
        Self::Presence(source)
    }
}

impl From<CapabilityEvaluationError> for DiseaseProtectionError {
    fn from(source: CapabilityEvaluationError) -> Self {
        Self::Capability(source)
    }
}

impl std::fmt::Display for DiseaseProtectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Presence(source) => source.fmt(f),
            Self::Capability(source) => source.fmt(f),
        }
    }
}

impl std::error::Error for DiseaseProtectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Presence(source) => Some(source),
            Self::Capability(source) => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disease::{EpisodeDecodeError, InfectionEpisodeRow};
    use adventuresim_core::{
        disease::{InfectionEpisode, ParseDiseaseIdError},
        identity::InfectionEpisodeId,
        physiology,
    };
    use adventuresim_world_schema::calendar::StrategicMinute;
    use std::error::Error;

    #[test]
    fn point_protection_retains_the_original_episode_and_parser_cause() {
        let row = InfectionEpisodeRow {
            id: 17,
            character_id: 23,
            disease_id: "Influenza".into(),
            contracted_at: StrategicMinute::new(41),
            ruleset_version: physiology::PHYSIOLOGY_RULESET_VERSION,
            phenotype_key_version: physiology::PHENOTYPE_KEY_VERSION,
        };
        let episode = InfectionEpisode::try_from(&row).unwrap_err();
        let protection = DiseaseProtectionError::from(CapabilityEvaluationError::from(episode));
        let capability = protection
            .source()
            .unwrap()
            .downcast_ref::<CapabilityEvaluationError>()
            .unwrap();
        let episode = capability
            .source()
            .unwrap()
            .downcast_ref::<EpisodeDecodeError>()
            .unwrap();
        assert!(matches!(episode, EpisodeDecodeError::DiseaseKey {
            episode, stored_key, ..
        } if *episode == InfectionEpisodeId::new(17) && stored_key == "Influenza"));
        assert_eq!(
            episode
                .source()
                .unwrap()
                .downcast_ref::<ParseDiseaseIdError>(),
            Some(&ParseDiseaseIdError)
        );
        assert_eq!(protection.to_string(), "Unknown disease");
    }
}
