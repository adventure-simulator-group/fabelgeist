//! Typed name catalog and semantic identity failures.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NameCatalogError {
    NoMatchingRepertoire,
    EmptyEligibleNames,
    InvalidRenderedName,
    MissingCatalogEntry(String),
    InconsistentNativeForm,
    Sampling(fabelgeist_determinism::SamplingError),
}

impl std::fmt::Display for NameCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoMatchingRepertoire => formatter.write_str("no matching name repertoire"),
            Self::EmptyEligibleNames => {
                formatter.write_str("name repertoire has no eligible entries")
            }
            Self::InvalidRenderedName => formatter.write_str(
                "rendered personal name must be nonempty, bounded, trimmed, and control-free",
            ),
            Self::MissingCatalogEntry(id) => {
                write!(formatter, "name catalog entry {id} is missing")
            }
            Self::InconsistentNativeForm => formatter.write_str(
                "native personal-name form disagrees with its family, culture, or register",
            ),
            Self::Sampling(error) => write!(formatter, "could not sample name catalog: {error}"),
        }
    }
}

impl std::error::Error for NameCatalogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sampling(error) => Some(error),
            _ => None,
        }
    }
}
