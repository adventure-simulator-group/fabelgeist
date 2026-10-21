//! Current-destination CPU growth overlaps terrain; accepted join gates delivery.

use std::collections::HashSet;

use crate::player_projection::LoadingPlayer;
use adventuresim_environment::prepared::finite_growth::FiniteStand;
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_netcode::{
    aeronet::{io::Session, transport::TransportConfig},
    bevy_replicon::prelude::{ClientId, SendTargets, ServerTriggerExt, ToClients},
    prelude::SceneTreeBankAsset,
    transport_policy::tactical_asset_transport_ready,
};

use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};

type AssetSessions<'w, 's> =
    Query<'w, 's, (&'static TransportConfig, Has<LoadingPlayer>, Has<Player>), With<Session>>;

pub(crate) struct SceneTreeAssetsPlugin;

/// Internal server event; never registered as a remotely callable request.
#[derive(Event)]
pub(crate) struct RequestSceneTreeBank {
    pub(crate) client: ClientId,
}

#[derive(Component)]
pub(crate) struct SceneTreeAssets {
    scene: SceneId,
    digest: String,
    state: GrowthState,
    recipients: HashSet<Entity>,
}

enum GrowthState {
    Producing(Task<Result<SceneTreeBankAsset>>),
    Ready(SceneTreeBankAsset),
    Failed,
}

impl Plugin for SceneTreeAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(request_bank)
            .add_systems(Update, deliver_ready_banks);
    }
}

impl SceneTreeAssets {
    /// Begin one owned job for the selected destination before terrain generation.
    pub(crate) fn new(input: &TacticalSceneInput, environment: &SceneEnvironment) -> Result<Self> {
        SceneTreeContent::Grown.validate_environment(environment)?;
        let scene = SceneId(input.scene_key.clone());
        let digest = environment.scene_digest.clone();
        info!(scene_id = %scene.0, scene_digest = %digest,
            "Queued current-destination grown bank worker");
        let task = AsyncComputeTaskPool::get().spawn(grow_bank(
            scene.clone(),
            digest.clone(),
            FiniteStand::dense_deciduous(input.seed),
        ));
        Ok(Self {
            scene,
            digest,
            state: GrowthState::Producing(task),
            recipients: HashSet::new(),
        })
    }

    fn request(&mut self, client: Entity) -> Result {
        if matches!(self.state, GrowthState::Failed) {
            return Err("scene tree growth has failed".into());
        }
        self.recipients.insert(client);
        info!(scene_id = %self.scene.0, client = ?client,
            "Registered accepted scene tree recipient");
        Ok(())
    }

    fn poll(&mut self) -> Result {
        if let GrowthState::Producing(task) = &mut self.state
            && let Some(result) = block_on(poll_once(task))
        {
            self.state = GrowthState::Failed;
            self.state = GrowthState::Ready(result?);
        }
        Ok(())
    }

    pub(crate) fn verify_owner(&self, scene: &SceneId, environment: &SceneEnvironment) -> Result {
        if self.scene != *scene || self.digest != environment.scene_digest {
            return Err("grown bank owner changed during scene asset delivery".into());
        }
        SceneTreeContent::Grown.validate_environment(environment)?;
        Ok(())
    }
}

fn request_bank(
    event: On<RequestSceneTreeBank>,
    sessions: AssetSessions,
    mut scenes: Query<(&SceneId, &SceneEnvironment, &mut SceneTreeAssets)>,
) -> Result {
    // Recipe scenes have no growth owner and never start a task.
    if scenes.is_empty() {
        return Ok(());
    }
    let Some(client) = event.client.entity() else {
        return Ok(());
    };
    if !eligible(&sessions, client)? {
        return Ok(());
    }
    let (scene, environment, mut assets) = scenes.single_mut()?;
    assets.verify_owner(scene, environment)?;
    assets.request(client)
}

fn deliver_ready_banks(
    mut commands: Commands,
    sessions: AssetSessions,
    mut scenes: Query<(&SceneId, &SceneEnvironment, &mut SceneTreeAssets)>,
) -> Result {
    for (scene, environment, mut assets) in &mut scenes {
        assets.verify_owner(scene, environment)?;
        assets.poll()?;
        // Detached sessions cannot retain recipient entries while growth runs.
        let mut waiting = std::mem::take(&mut assets.recipients);
        for client in waiting.drain() {
            if !eligible(&sessions, client)? {
                continue;
            }
            if let GrowthState::Ready(bank) = &assets.state {
                info!(scene_id = %scene.0, client = ?client,
                    bytes = bank.library_bytes().len(),
                    "Queued foreground grown bank delivery");
                commands.server_trigger(ToClients {
                    targets: SendTargets::Single(client.into()),
                    message: bank.clone(),
                });
            } else {
                assets.recipients.insert(client);
            }
        }
    }
    Ok(())
}

fn eligible(sessions: &AssetSessions, client: Entity) -> Result<bool> {
    let Ok((config, loading, player)) = sessions.get(client) else {
        return Ok(false);
    };
    if !loading && !player {
        return Ok(false);
    }
    if !tactical_asset_transport_ready(config) {
        return Err("tactical session lost its admitted scene asset budget".into());
    }
    Ok(true)
}

async fn grow_bank(
    scene: SceneId,
    digest: String,
    stand: FiniteStand,
) -> Result<SceneTreeBankAsset> {
    let started = std::time::Instant::now();
    info!(scene_id = %scene.0, scene_digest = %digest,
        "Started foreground grown bank worker");
    let bank = stand.grow().await?;
    // Select actual surviving source identities, never model ordinals.
    let first = bank
        .sources
        .iter()
        .map(|source| source.word())
        .min()
        .ok_or("grown bank has no surviving source")?;
    let last = bank
        .sources
        .iter()
        .map(|source| source.word())
        .max()
        .ok_or("grown bank has no surviving source")?;
    if first == last {
        return Err("grown bank requires two distinct sources".into());
    }
    let asset = SceneTreeBankAsset::new(scene, digest, vec![first, last], bank.bytes)?;
    info!(scene_id = %asset.scene().0, bytes = asset.library_bytes().len(),
        elapsed_ms = started.elapsed().as_millis(), "Completed foreground grown bank worker");
    Ok(asset)
}
