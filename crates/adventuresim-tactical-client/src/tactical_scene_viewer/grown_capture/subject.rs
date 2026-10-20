//! Explicit capture subjects, independent of private specimen installation.
use super::*;
use crate::presentation::GrownSource;
use adventuresim_environment::prepared::LibraryId;

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super) enum Subject {
    Specimens {
        role: Role,
        source: PlantId,
        level: Level,
    },
    Generated {
        bank: LibraryId,
        sources: [GrownSource; 2],
        selection: LevelSelection,
        trees: usize,
    },
}

impl Subject {
    pub(super) fn require_shared_sources(self, shared: &[GrownSource]) -> Result<()> {
        if let Self::Generated { sources, .. } = self {
            ensure!(
                sources.iter().all(|source| shared.contains(source)),
                "both generated sources require actual multi-root shared commands"
            );
        }
        Ok(())
    }
}

impl Serialize for Subject {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let value = match self {
            Self::Specimens {
                role,
                source,
                level,
            } => serde_json::json!({"mode":"specimens",
                "role":role,"source_id":source.word(),"level":level}),
            Self::Generated {
                bank,
                sources,
                selection,
                trees,
            } => serde_json::json!({"mode":"generated",
                "bank":bank,"sources":sources,"selection":selection,"trees":trees}),
        };
        value.serialize(serializer)
    }
}
