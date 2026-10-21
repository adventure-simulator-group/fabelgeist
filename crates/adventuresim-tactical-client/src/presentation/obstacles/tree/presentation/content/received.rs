//! One bounded bank admission tied to the live tactical connection and scene.

use std::sync::Arc;

use adventuresim_environment::{
    prepared::{GrownMeshBudget, GrownMeshBytes, GrownPartVertexLimit, TreeBank},
    render::LevelSelection,
};
use adventuresim_tactical_core::prelude::{SceneEnvironment, SceneId, SceneTreeContent};
use adventuresim_tactical_netcode::{
    aeronet::{io::Session, transport::TransportConfig},
    prelude::{AdventureSimulatorClient, SceneTreeBankAsset},
    transport_policy::tactical_asset_transport_ready,
};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};
use fabelgeist_botany::identity::PlantId;

use super::super::grown::GrownSceneRequest;
use super::{TreeContentRequest, resolve_scene_tree_content};
use crate::presentation::ActiveTacticalScene;

// Bounds for one complete tactical catalogue, including all three geometry levels.
const TACTICAL_CATALOGUE_MESH_BYTES: u64 = 256 * 1024 * 1024;
const TACTICAL_PART_VERTICES: u32 = 100_000;

type LiveEndpoints<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static TransportConfig),
    (With<AdventureSimulatorClient>, With<Session>),
>;

#[derive(Resource, Default)]
struct ReceivedBank(Option<Admission>);

struct Admission {
    endpoint: Entity,
    asset: SceneTreeBankAsset,
    scene: Option<Entity>,
    state: AdmissionState,
}

enum AdmissionState {
    Decoding(Task<Result<GrownSceneRequest>>),
    Ready(GrownSceneRequest),
    Failed,
}

/// A retained scene cache cannot stand in for this connection's accepted bank.
#[cfg(all(target_family = "wasm", feature = "debug"))]
pub(super) fn admitted_for(
    world: &World,
    endpoint: Entity,
    scene: Entity,
    request: &GrownSceneRequest,
) -> bool {
    world.get_resource::<ReceivedBank>().and_then(|received|received.0.as_ref())
        .is_some_and(|admission| admission.endpoint == endpoint && admission.scene == Some(scene)
            && matches!(&admission.state, AdmissionState::Ready(admitted) if admitted == request)
            && world.get::<SceneId>(scene).is_some_and(|identity|identity == admission.asset.scene())
            && world.get::<SceneEnvironment>(scene).is_some_and(|environment|
                environment.scene_digest == admission.asset.scene_digest()))
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<ReceivedBank>()
        .add_observer(receive_bank)
        .add_systems(
            Update,
            resolve_bank
                .after(crate::presentation::refresh_active_tactical_scene)
                .before(resolve_scene_tree_content),
        );
}

impl Admission {
    fn new(endpoint: Entity, asset: SceneTreeBankAsset) -> Self {
        let input = asset.clone();
        let task = AsyncComputeTaskPool::get().spawn(async move {
            #[cfg(target_family = "wasm")]
            let bank = TreeBank::from_bytes(input.library_bytes()).await?;
            #[cfg(not(target_family = "wasm"))]
            let bank = TreeBank::from_bytes(input.library_bytes())?;
            let plants = input
                .source_words()
                .iter()
                .copied()
                .map(PlantId::from_word)
                .collect();
            Ok(GrownSceneRequest::from_sources(
                Arc::new(bank),
                plants,
                GrownMeshBudget {
                    part_vertices: GrownPartVertexLimit::from_vertices(TACTICAL_PART_VERTICES)?,
                    total_mesh_bytes: GrownMeshBytes::from_bytes(TACTICAL_CATALOGUE_MESH_BYTES),
                },
                LevelSelection::Adaptive,
            )?)
        });
        Self {
            endpoint,
            asset,
            scene: None,
            state: AdmissionState::Decoding(task),
        }
    }

    fn matches(&self, asset: &SceneTreeBankAsset) -> bool {
        self.asset.scene() == asset.scene()
            && self.asset.scene_digest() == asset.scene_digest()
            && self.asset.source_words() == asset.source_words()
            && self.asset.library_bytes() == asset.library_bytes()
    }

    fn poll(&mut self) -> Result {
        if let AdmissionState::Decoding(task) = &mut self.state
            && let Some(result) = block_on(poll_once(task))
        {
            self.state = AdmissionState::Failed;
            self.state = AdmissionState::Ready(result?);
        }
        Ok(())
    }
}

fn receive_bank(
    event: On<SceneTreeBankAsset>,
    endpoints: LiveEndpoints,
    mut received: ResMut<ReceivedBank>,
) -> Result {
    let (endpoint, config) = endpoints.single()?;
    if !tactical_asset_transport_ready(config) {
        return Err("tree bank arrived without an admitted transport budget".into());
    }
    if let Some(admission) = &received.0
        && admission.endpoint == endpoint
    {
        if !admission.matches(&event) {
            return Err("contradictory bank on one tactical connection".into());
        }
        return Ok(());
    }
    received.0 = Some(Admission::new(endpoint, (*event).clone()));
    Ok(())
}

fn resolve_bank(
    mut commands: Commands,
    endpoints: LiveEndpoints,
    active: Res<ActiveTacticalScene>,
    scenes: Query<(
        &SceneId,
        &SceneEnvironment,
        &SceneTreeContent,
        Option<&TreeContentRequest>,
    )>,
    mut received: ResMut<ReceivedBank>,
) -> Result {
    let Some(admission) = &mut received.0 else {
        return Ok(());
    };
    let Ok((endpoint, config)) = endpoints.single() else {
        received.0 = None;
        return Ok(());
    };
    if endpoint != admission.endpoint {
        received.0 = None;
        return Ok(());
    }
    if !tactical_asset_transport_ready(config) {
        return Err("tree bank connection lost its admitted transport budget".into());
    }
    if admission
        .scene
        .is_some_and(|scene| Some(scene) != active.entity)
    {
        received.0 = None;
        return Ok(());
    }
    admission.poll()?;
    let Some(scene) = active.entity else {
        return Ok(());
    };
    let Ok((identity, environment, content, installed)) = scenes.get(scene) else {
        return Ok(());
    };
    if *identity != *admission.asset.scene()
        || environment.scene_digest != admission.asset.scene_digest()
    {
        return Ok(());
    }
    if *content != SceneTreeContent::Grown {
        return Err("grown bank contradicts the active scene tree family".into());
    }
    content.validate_environment(environment)?;
    admission.scene = Some(scene);
    if let AdmissionState::Ready(request) = &admission.state {
        let requested = TreeContentRequest::Grown(request.clone());
        if let Some(installed) = installed {
            if *installed != requested {
                return Err("active scene tree bank changed after admission".into());
            }
        } else {
            commands.entity(scene).insert(requested);
        }
    }
    Ok(())
}
