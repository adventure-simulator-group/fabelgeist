//! Complete landform products shared by actor and focused-city paving producers.
use adventuresim_tactical_core::prelude::SceneTerrainPatch;
use serde::{Deserialize, Serialize};

/// Required transport field: a product explicitly owns natural or patch ground.
#[derive(Clone, Serialize, Deserialize)]
pub(in crate::presentation) enum PreparedTerrainLandform {
    Natural,
    Patch(SceneTerrainPatch),
}

impl From<Option<SceneTerrainPatch>> for PreparedTerrainLandform {
    /// The canonical scene producer uses absence for ordinary natural terrain.
    fn from(patch: Option<SceneTerrainPatch>) -> Self {
        patch.map_or(Self::Natural, Self::Patch)
    }
}
