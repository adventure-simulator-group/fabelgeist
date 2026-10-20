//! Stable root hierarchy, separate from the streamer's mutable selected level.

use super::*;
use adventuresim_environment::prepared::LibraryId;

/// Immutable ownership snapshot from the real physical-root stream.
#[derive(Clone, PartialEq)]
pub(crate) struct GrownRootAttachment {
    pub(crate) root: Entity,
    pub(crate) scene: Entity,
    pub(crate) visual: Entity,
    pub(crate) source: GrownSource,
    pub(crate) bank: LibraryId,
    pub(crate) hierarchy: GrownRootHierarchy,
    pub(crate) frame: Transform,
}

/// One retained group and the actual geometry level it owns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GrownLevelAttachment {
    pub(crate) entity: Entity,
    pub(crate) level: Level,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GrownRootHierarchy {
    Fixed {
        level: Level,
    },
    Adaptive {
        full: Entity,
        middle: Entity,
        coarse: Entity,
    },
}

impl GrownRootAttachment {
    /// All installed groups, including inactive adaptive levels.
    pub(crate) fn groups(&self) -> impl Iterator<Item = GrownLevelAttachment> {
        match self.hierarchy {
            GrownRootHierarchy::Fixed { level } => [
                Some(GrownLevelAttachment {
                    entity: self.visual,
                    level,
                }),
                None,
                None,
            ],
            GrownRootHierarchy::Adaptive {
                full,
                middle,
                coarse,
            } => [
                Some(GrownLevelAttachment {
                    entity: full,
                    level: Level::Full,
                }),
                Some(GrownLevelAttachment {
                    entity: middle,
                    level: Level::Middle,
                }),
                Some(GrownLevelAttachment {
                    entity: coarse,
                    level: Level::Coarse,
                }),
            ],
        }
        .into_iter()
        .flatten()
    }
}
