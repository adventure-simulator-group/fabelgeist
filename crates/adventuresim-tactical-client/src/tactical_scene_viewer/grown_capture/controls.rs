//! Explicit batching and held-membership rejection controls.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum BatchControl {
    Ordinary,
    ForcedSingletons,
    HiddenLastRoot,
    MissingLastTag,
    ChangedLevel,
}

#[derive(Resource, Default, Clone, Copy)]
pub(super) enum RootVisibility {
    #[default]
    AllVisible,
    LastRootHidden,
}

impl RootVisibility {
    pub(super) fn expected(self, ordinal: layout::RootOrdinal) -> Visibility {
        if matches!(self, Self::LastRootHidden) && ordinal.is_last() {
            Visibility::Hidden
        } else {
            Visibility::Visible
        }
    }
}
