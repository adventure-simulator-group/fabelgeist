//! Headless transport acceptance peer: observes co-presence and reconnects.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use adventuresim_tactical_core::prelude::{CharacterId, Player};
use adventuresim_tactical_netcode::{
    message::{ReconnectCapability, SceneJoinRejected},
    prelude::{AdventureSimulatorClient, AdventureSimulatorNetPlugins},
};
use bevy::{app::ScheduleRunnerPlugin, input::InputPlugin, prelude::*};
use clap::Parser;

const ACCEPTANCE_TIMEOUT: Duration = Duration::from_secs(60);
const RECONNECT_DELAY: Duration = Duration::from_millis(250);
const STABLE_PRESENCE_HOLD: Duration = Duration::from_secs(8);

#[derive(Parser, Resource, Clone)]
struct Args {
    #[arg(long)]
    character: u64,
    #[arg(long)]
    server: String,
    #[arg(long, default_value_t = 1)]
    expected_players: usize,
    #[arg(long)]
    reconnect: bool,
    #[arg(long)]
    expect_rejection: bool,
    #[arg(long)]
    hold_open: bool,
}

#[derive(Resource)]
struct Acceptance {
    started: Instant,
    stable_since: Option<Instant>,
    capabilities: u32,
    reconnect_started: bool,
    reconnect_due: Option<Instant>,
    accepted: bool,
    finished: Arc<AtomicBool>,
}

fn main() {
    let args = Args::parse();
    let client = client(&args);
    let finished = Arc::new(AtomicBool::new(false));
    if args.hold_open {
        let signal = finished.clone();
        std::thread::spawn(move || {
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line);
            signal.store(true, Ordering::Release);
        });
    }
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(10))),
        bevy::log::LogPlugin::default(),
        InputPlugin,
        TransformPlugin,
        AdventureSimulatorNetPlugins,
    ))
    .insert_resource(args)
    .insert_resource(Acceptance {
        started: Instant::now(),
        stable_since: None,
        capabilities: 0,
        reconnect_started: false,
        reconnect_due: None,
        accepted: false,
        finished,
    })
    .add_observer(capability_received)
    .add_observer(join_rejected)
    .add_systems(Update, observe_presence);
    app.world_mut().spawn(client);
    if !matches!(app.run(), AppExit::Success) {
        std::process::exit(1);
    }
}

fn client(args: &Args) -> AdventureSimulatorClient {
    AdventureSimulatorClient {
        player_id: args.character,
        server_url: args.server.clone(),
    }
}

fn capability_received(_event: On<ReconnectCapability>, mut state: ResMut<Acceptance>) {
    state.capabilities += 1;
}

fn join_rejected(
    rejection: On<SceneJoinRejected>,
    args: Res<Args>,
    mut exit: MessageWriter<AppExit>,
) {
    if args.expect_rejection {
        println!("REJECTED character={}", rejection.character_id.0);
        exit.write(AppExit::Success);
    } else {
        eprintln!("Unexpected scene rejection: {}", rejection.reason);
        exit.write(AppExit::error());
    }
}

fn observe_presence(
    args: Res<Args>,
    mut state: ResMut<Acceptance>,
    mut commands: Commands,
    clients: Query<Entity, With<AdventureSimulatorClient>>,
    players: Query<&CharacterId, With<Player>>,
    mut exit: MessageWriter<AppExit>,
) {
    let now = Instant::now();
    if state.accepted {
        if !args.hold_open || state.finished.load(Ordering::Acquire) {
            exit.write(AppExit::Success);
        }
        return;
    }
    if now.duration_since(state.started) >= ACCEPTANCE_TIMEOUT {
        eprintln!(
            "Timed out waiting for scene acceptance: players={} own={} capabilities={}",
            players.iter().count(),
            players.iter().filter(|id| id.0 == args.character).count(),
            state.capabilities,
        );
        exit.write(AppExit::error());
        return;
    }
    if let Some(due) = state.reconnect_due {
        if now >= due {
            commands.spawn(client(&args));
            state.reconnect_due = None;
        }
        return;
    }
    let own = players.iter().filter(|id| id.0 == args.character).count();
    if own > 1 {
        eprintln!("The scene projected the same character twice");
        exit.write(AppExit::error());
        return;
    }
    if own != 1 || players.iter().count() < args.expected_players {
        state.stable_since = None;
        return;
    }
    if args.reconnect && !state.reconnect_started && state.capabilities > 0 {
        for entity in &clients {
            commands.entity(entity).despawn();
        }
        state.reconnect_started = true;
        state.reconnect_due = Some(now + RECONNECT_DELAY);
        return;
    }
    if args.expect_rejection || (args.reconnect && state.capabilities < 2) {
        return;
    }
    let since = *state.stable_since.get_or_insert(now);
    if now.duration_since(since) >= STABLE_PRESENCE_HOLD {
        println!(
            "ACCEPTED players={} reconnects={}",
            players.iter().count(),
            state.capabilities
        );
        state.accepted = true;
    }
}
