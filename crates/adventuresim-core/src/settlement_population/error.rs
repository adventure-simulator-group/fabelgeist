//! Population-generation admission and sampling failures.
use super::PopulationRelation;
use fabelgeist_determinism::SamplingError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PopulationError {
    DuplicateCandidate {
        relation: PopulationRelation,
    },
    Sampling {
        relation: PopulationRelation,
        context: String,
        source: SamplingError,
    },
}
impl std::fmt::Display for PopulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateCandidate { relation } => write!(
                f,
                "Duplicate candidate in relation {}",
                relation.stable_id()
            ),
            Self::Sampling {
                relation,
                context,
                source,
            } => write!(
                f,
                "Cannot select relation {} in {context}: {source}",
                relation.stable_id()
            ),
        }
    }
}
impl std::error::Error for PopulationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sampling { source, .. } => Some(source),
            _ => None,
        }
    }
}
