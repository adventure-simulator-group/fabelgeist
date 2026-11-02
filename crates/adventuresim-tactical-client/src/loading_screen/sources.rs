//! Reads each loading stage from the marker owned by the system doing the work.
use super::progress::{LoadingProgress, LoadingRun, LoadingStage, StageProgress};
use crate::presentation::{AtmosphereIblCache, TacticalGraphicsSettings};
use crate::{
    animation::pose_buffer::PoseBufferRig, player::ClientPlayer,
    presentation::scene_readiness::SceneReadiness,
};
use adventuresim_procedural_textures::{PROCEDURAL_TEXTURE_CATALOGUE, ProceduralTextureResidency};
use adventuresim_tactical_netcode::bevy_replicon::prelude::ClientState;
use bevy::{ecs::system::SystemParam, prelude::*};

/// Scatter and instanced grass each mark a scene when generated.
const GROUND_LAYERS_PER_SCENE: usize = 2;

#[derive(SystemParam)]
pub(super) struct LoadingSources<'w, 's> {
    client: Res<'w, State<ClientState>>,
    players: Query<'w, 's, Option<&'static PoseBufferRig>, With<ClientPlayer>>,
    textures: Res<'w, ProceduralTextureResidency>,
    scene: SceneReadiness<'w, 's>,
    sky: Res<'w, AtmosphereIblCache>,
    settings: Res<'w, TacticalGraphicsSettings>,
}

impl LoadingSources<'_, '_> {
    /// Equipment and pipelines are passed in because their sources are checked
    /// fallibly or live in the render world.
    pub(super) fn progress(
        &self,
        run: &mut LoadingRun,
        equipment_settled: bool,
        waiting_pipelines: usize,
    ) -> LoadingProgress {
        let scene = self.scene.counts();
        let scene_known = scene.scenes > 0;
        let local_player = self.players.iter().next();
        let trees = run.trees.observe(scene.pending_trees);
        let pipelines = run.pipelines.observe(waiting_pipelines);
        let mut progress = LoadingProgress::default();
        progress.set(
            LoadingStage::Connection,
            StageProgress::from_flag(*self.client.get() == ClientState::Connected),
        );
        progress.set(
            LoadingStage::Character,
            StageProgress::from_flag(local_player.is_some()),
        );
        progress.set(
            LoadingStage::Textures,
            StageProgress {
                completed: self
                    .textures
                    .installed_count(PROCEDURAL_TEXTURE_CATALOGUE.iter().map(|recipe| recipe.id)),
                total: PROCEDURAL_TEXTURE_CATALOGUE.len(),
            },
        );
        progress.set(
            LoadingStage::Terrain,
            scene_progress(scene_known, scene.terrain_presented, scene.scenes),
        );
        progress.set(
            LoadingStage::Vista,
            StageProgress::from_flag(scene.vista_presented),
        );
        progress.set(
            LoadingStage::Trees,
            if scene_known {
                trees
            } else {
                StageProgress::PENDING
            },
        );
        progress.set(
            LoadingStage::GroundCover,
            scene_progress(
                scene_known,
                scene.ground_layers_presented,
                scene.scenes * GROUND_LAYERS_PER_SCENE,
            ),
        );
        progress.set(
            LoadingStage::Sky,
            StageProgress::from_flag(self.sky.is_ready(&self.settings)),
        );
        progress.set(
            LoadingStage::Equipment,
            StageProgress::from_flag(local_player.is_some() && equipment_settled),
        );
        progress.set(
            LoadingStage::Pose,
            StageProgress::from_flag(
                local_player
                    .flatten()
                    .is_some_and(PoseBufferRig::has_presented_pose),
            ),
        );
        progress.set(LoadingStage::Shaders, pipelines);
        progress
    }
}

fn scene_progress(scene_known: bool, completed: usize, total: usize) -> StageProgress {
    if scene_known {
        StageProgress { completed, total }
    } else {
        StageProgress::PENDING
    }
}
