//! Read-only view of tactical scene presentation for loading gates.
use super::{
    PendingTerrainPresentation, VistaTerrain,
    ground_scatter::{GroundScatterPresented, InstancedGrassPresented},
    obstacles::tree::PendingTreePresentation,
};
use adventuresim_tactical_core::prelude::SceneTerrain;
use bevy::{ecs::system::SystemParam, prelude::*};

type ScenePresentationState = (
    Has<PendingTerrainPresentation>,
    Has<GroundScatterPresented>,
    Has<InstancedGrassPresented>,
);

/// Scene presentation has no single completion event, so each stage reads the
/// marker left behind by the system that owns it.
#[derive(SystemParam)]
pub(crate) struct SceneReadiness<'w, 's> {
    scenes: Query<'w, 's, ScenePresentationState, With<SceneTerrain>>,
    trees: Query<'w, 's, (), With<PendingTreePresentation>>,
    vista: Query<'w, 's, (), With<VistaTerrain>>,
}

/// Completed work against the scenes replicated so far.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SceneReadinessCounts {
    pub(crate) scenes: usize,
    pub(crate) terrain_presented: usize,
    pub(crate) ground_layers_presented: usize,
    pub(crate) pending_trees: usize,
    pub(crate) vista_presented: bool,
}

impl SceneReadiness<'_, '_> {
    pub(crate) fn counts(&self) -> SceneReadinessCounts {
        let mut counts = SceneReadinessCounts {
            pending_trees: self.trees.iter().count(),
            vista_presented: !self.vista.is_empty(),
            ..default()
        };
        for (terrain_pending, scatter, grass) in &self.scenes {
            counts.scenes += 1;
            counts.terrain_presented += usize::from(!terrain_pending);
            counts.ground_layers_presented += usize::from(scatter) + usize::from(grass);
        }
        counts
    }
}

/// Assets without which a scene cannot be presented at all.
pub(crate) fn is_required_scene_asset(path: &std::path::Path) -> bool {
    path.starts_with("textures/procedural")
        || path.ends_with("unarmed/base.glb")
        || path.ends_with("unarmed/idle_relaxed.glb")
}
