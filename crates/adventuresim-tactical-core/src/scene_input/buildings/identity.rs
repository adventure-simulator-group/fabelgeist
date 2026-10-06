//! Identity shared by a physical scene building and its operable openings.
use serde::{Deserialize, Serialize};

/// Stable scene building number, independent of an opening or geometry source.
/// Authoring placements retain their native number until scene installation.
///
/// ```compile_fail
/// use adventuresim_tactical_core::scene_input::SceneBuildingId;
/// use adventuresim_building_generator::OpeningAssemblyId;
/// fn wrong_identity(opening: OpeningAssemblyId) -> SceneBuildingId { opening }
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct SceneBuildingId(pub u64);

impl From<u64> for SceneBuildingId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for SceneBuildingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
