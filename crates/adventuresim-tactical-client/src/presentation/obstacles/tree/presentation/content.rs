//! Scene intent is separate from the cache's installed asset ownership.

use super::*;
use adventuresim_tactical_core::prelude::{SceneId, SceneTreeContent};
use anyhow::{Context, Result, ensure};

#[cfg(feature = "grown-trees")]
mod received;

pub(in crate::presentation) struct TreeContentPlugin;

#[derive(Component, Clone, PartialEq, Eq)]
pub(crate) enum TreeContentRequest {
    Recipe,
    #[cfg(feature = "grown-trees")]
    Grown(super::grown::GrownSceneRequest),
}

impl Plugin for TreeContentPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "grown-trees")]
        received::install(app);
        app.add_systems(
            Update,
            resolve_scene_tree_content
                .after(crate::presentation::refresh_active_tactical_scene)
                .before(super::present_pending_trees)
                .before(crate::presentation::vista::pending::present_ready_vista),
        );
    }
}

#[cfg(all(feature = "foliage-review", not(target_family = "wasm")))]
impl TreeContentRequest {
    pub(super) fn require_recipe(&self) -> Result<()> {
        ensure!(
            matches!(self, Self::Recipe),
            "scene requires grown tree content"
        );
        Ok(())
    }

    pub(super) fn require_scene_recipe(world: &World, scene: Entity) -> Result<()> {
        world
            .get::<Self>(scene)
            .context("playable scene has no tree content request")?
            .require_recipe()
    }
}

impl TreeContentRequest {
    #[cfg(all(target_family = "wasm", feature = "debug", feature = "grown-trees"))]
    pub(in crate::presentation) fn has_current_admission(
        &self,
        world: &World,
        endpoint: Entity,
        scene: Entity,
    ) -> bool {
        matches!(self, Self::Grown(request) if received::admitted_for(world,endpoint,scene,request))
    }

    pub(crate) fn family(&self) -> SceneTreeContent {
        match self {
            Self::Recipe => SceneTreeContent::Recipe,
            #[cfg(feature = "grown-trees")]
            Self::Grown(_) => SceneTreeContent::Grown,
        }
    }
}

/// Replication and bank delivery may arrive in either order.
fn resolve_scene_tree_content(
    mut commands: Commands,
    scenes: Query<(Entity, &SceneTreeContent, Option<&TreeContentRequest>), With<SceneId>>,
) -> bevy::prelude::Result {
    for (scene, intent, request) in &scenes {
        if let Some(request) = request {
            if request.family() != *intent {
                return Err("tree content request contradicts scene intent".into());
            }
            continue;
        }
        match intent {
            SceneTreeContent::Recipe => {
                commands.entity(scene).insert(TreeContentRequest::Recipe);
            }
            SceneTreeContent::Grown => {
                #[cfg(not(feature = "grown-trees"))]
                return Err("scene requires the grown-trees runtime capability".into());
                // A matching admitted bank must resolve this request first.
            }
        }
    }
    Ok(())
}
