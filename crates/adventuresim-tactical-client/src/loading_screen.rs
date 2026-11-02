//! Full-screen gate between joining a tactical scene and playing it.
//!
//! A load begins whenever a tactical client is spawned, on native startup and
//! on browser `enter-tactical` alike. The scene builds behind the opaque
//! screen so that its render pipelines compile before the player can see or
//! control it. Losing the connection, a failed required asset and a stage that
//! stops advancing all end in a failure panel with retry and exit actions.
use crate::{
    equipment::{EquipmentHudSet, readiness::EquipmentReadiness},
    player::ClientPlayer,
    presentation::{
        ClientStartupTiming, is_required_scene_asset, pipeline_readiness::waiting_pipelines,
    },
};
use adventuresim_tactical_core::prelude::{ContextActivity, Player};
use adventuresim_tactical_netcode::prelude::{
    AdventureSimulatorClient, DisconnectReason, Disconnected,
};
use bevy::{asset::UntypedAssetLoadFailedEvent, prelude::*};
use bevy_egui::EguiPrimaryContextPass;
use progress::{LoadingFailure, LoadingRun, RunVerdict};
use sources::LoadingSources;

mod progress;
mod sources;
mod view;

const REVEAL_SECONDS: f32 = 0.6;

struct LoadingScreenPlugin;

/// Progress of the current tactical load, if any.
#[derive(Resource, Default)]
pub(crate) struct LoadingScreen {
    phase: LoadingPhase,
    /// A failed session despawns its client, so retry keeps its own copy.
    connection: Option<AdventureSimulatorClient>,
}

#[derive(Default)]
enum LoadingPhase {
    #[default]
    Idle,
    Loading(LoadingRun),
    Failed(LoadingFailure),
    Revealing(Timer),
}

impl LoadingScreen {
    /// Whether the player may see and control the scene.
    pub(crate) fn gameplay_unlocked(&self) -> bool {
        matches!(self.phase, LoadingPhase::Idle | LoadingPhase::Revealing(_))
    }

    /// Abandons the load when the browser leaves tactical play.
    #[cfg(target_family = "wasm")]
    pub(crate) fn dismiss(&mut self) {
        self.phase = LoadingPhase::Idle;
        self.connection = None;
    }

    fn fail(&mut self, failure: LoadingFailure) {
        if matches!(self.phase, LoadingPhase::Loading(_)) {
            warn!(%failure, "Tactical loading failed");
            self.phase = LoadingPhase::Failed(failure);
        }
    }
}

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingScreen>()
            .add_observer(begin_load)
            .add_observer(fail_on_disconnect)
            .add_systems(Startup, view::spawn)
            .add_systems(
                Update,
                (
                    fail_on_required_asset,
                    advance_load,
                    hold_new_players,
                    view::handle_actions,
                    view::sync_stage_bars,
                    view::sync_overlay,
                )
                    .chain(),
            );
    }
}

/// Adds the loading gate to a client. Headless automation has no one to show
/// it to and drives input itself, so it runs without one.
pub(crate) fn install(app: &mut App, headless: bool) {
    if !headless {
        app.add_plugins(LoadingScreenPlugin);
    }
    app.configure_sets(
        EguiPrimaryContextPass,
        EquipmentHudSet.run_if(gameplay_unlocked),
    );
}

/// Run condition for systems that must wait for the scene to be revealed.
/// Clients without the plugin, such as headless automation, are never held.
pub(crate) fn gameplay_unlocked(screen: Option<Res<LoadingScreen>>) -> bool {
    screen.is_none_or(|screen| screen.gameplay_unlocked())
}

/// Run condition for overlays of the local player's own state.
pub(crate) fn local_player_revealed(
    screen: Option<Res<LoadingScreen>>,
    players: Query<(), With<ClientPlayer>>,
) -> bool {
    !players.is_empty() && gameplay_unlocked(screen)
}

fn begin_load(
    client: On<Add, AdventureSimulatorClient>,
    clients: Query<&AdventureSimulatorClient>,
    mut screen: ResMut<LoadingScreen>,
    time: Res<Time>,
) -> Result {
    screen.connection = Some(clients.get(client.entity)?.clone());
    screen.phase = LoadingPhase::Loading(LoadingRun::new(time.elapsed()));
    Ok(())
}

fn fail_on_disconnect(
    disconnected: On<Disconnected>,
    clients: Query<(), With<AdventureSimulatorClient>>,
    mut screen: ResMut<LoadingScreen>,
) {
    if !clients.contains(disconnected.entity) {
        return;
    }
    let reason = match &disconnected.reason {
        DisconnectReason::ByUser(reason) | DisconnectReason::ByPeer(reason) => reason.clone(),
        DisconnectReason::ByError(error) => format!("{error:#}"),
    };
    screen.fail(LoadingFailure::ConnectionLost(reason));
}

fn fail_on_required_asset(
    mut events: MessageReader<UntypedAssetLoadFailedEvent>,
    mut screen: ResMut<LoadingScreen>,
) {
    for event in events.read() {
        let path = event.path.path();
        if is_required_scene_asset(path) {
            screen.fail(LoadingFailure::Asset(path.display().to_string()));
        }
    }
}

fn advance_load(
    mut commands: Commands,
    mut screen: ResMut<LoadingScreen>,
    sources: LoadingSources,
    equipment: EquipmentReadiness,
    players: Query<Entity, With<ClientPlayer>>,
    startup: Res<ClientStartupTiming>,
    time: Res<Time>,
) {
    let run = match &mut screen.phase {
        LoadingPhase::Loading(run) => run,
        LoadingPhase::Revealing(timer) => {
            if timer.tick(time.delta()).is_finished() {
                screen.phase = LoadingPhase::Idle;
            }
            return;
        }
        LoadingPhase::Idle | LoadingPhase::Failed(_) => return,
    };
    let progress = sources.progress(run, equipment.settled(), waiting_pipelines());
    match run.observe(progress, time.elapsed()) {
        RunVerdict::Stalled(stage) => screen.fail(LoadingFailure::Stalled(stage)),
        RunVerdict::Ready => {
            startup.mark("tactical scene ready; revealing");
            for player in &players {
                commands
                    .entity(player)
                    .insert(ContextActivity::<Player>::ACTIVE);
            }
            screen.phase =
                LoadingPhase::Revealing(Timer::from_seconds(REVEAL_SECONDS, TimerMode::Once));
        }
        RunVerdict::Waiting => {}
    }
}

/// Keeps a player that arrives mid-load from moving behind the screen.
fn hold_new_players(
    mut commands: Commands,
    screen: Res<LoadingScreen>,
    players: Query<Entity, Added<ClientPlayer>>,
) {
    if screen.gameplay_unlocked() {
        return;
    }
    for player in &players {
        commands
            .entity(player)
            .insert(ContextActivity::<Player>::INACTIVE);
    }
}
