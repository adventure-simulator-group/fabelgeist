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
    ChangedSceneOwner,
    ParentedOriginalRoot,
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

impl BatchControl {
    /// Corrupt one real original root before the pre-isolation batch check.
    pub(super) fn apply_original_batch(self, world: &mut World) -> Result<()> {
        if !matches!(self, Self::ChangedSceneOwner | Self::ParentedOriginalRoot) {
            return Ok(());
        }
        use crate::presentation::SceneObstacleOf;
        use adventuresim_tactical_core::prelude::SceneObstacle;
        let root = world
            .query_filtered::<Entity, With<SceneObstacle>>()
            .iter(world)
            .next()
            .context("original obstacle control requires a real generated root")?;
        let scene = world
            .get::<SceneObstacleOf>(root)
            .context("original obstacle control requires scene ownership")?
            .0;
        ensure!(root != scene, "original root must differ from its scene");
        match self {
            Self::ChangedSceneOwner => {
                world.entity_mut(root).insert(SceneObstacleOf(root));
            }
            Self::ParentedOriginalRoot => {
                world.entity_mut(root).insert(ChildOf(scene));
            }
            _ => {}
        }
        Ok(())
    }
}
