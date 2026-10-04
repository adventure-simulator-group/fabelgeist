//! Concrete failures when admitting persisted episode vocabulary and versions.

use adventuresim_core::{disease::ParseDiseaseIdError, identity::InfectionEpisodeId};

#[derive(Debug)]
pub enum EpisodeDecodeError {
    RulesetVersion {
        episode: InfectionEpisodeId,
        stored_version: u16,
    },
    PhenotypeKeyVersion {
        episode: InfectionEpisodeId,
        stored_version: u16,
    },
    DiseaseKey {
        episode: InfectionEpisodeId,
        stored_key: String,
        source: ParseDiseaseIdError,
    },
}

impl std::fmt::Display for EpisodeDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RulesetVersion { stored_version, .. } => {
                write!(f, "Unsupported physiology ruleset version {stored_version}")
            }
            Self::PhenotypeKeyVersion { stored_version, .. } => {
                write!(
                    f,
                    "Unsupported immutable physiology key version {stored_version}"
                )
            }
            Self::DiseaseKey { source, .. } => source.fmt(f),
        }
    }
}

impl std::error::Error for EpisodeDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DiseaseKey { source, .. } => Some(source),
            _ => None,
        }
    }
}
