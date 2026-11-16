//! Tactical Spawner - Watches SpacetimeDB and spawns tactical-server processes
//!
//! Subscribes to the tactical_server_request table and spawns a server process
//! whenever a new request appears. The spawned server will then call
//! create_tactical_server_for_request to register itself.

use adventuresim_tactical_server_dispatcher::settlement_economy_adapter;

use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use adventuresim_stdb_client::spacetimedb_sdk::{DbContext, Table};
use adventuresim_stdb_client::{
    DbConnection, TacticalServerRequest, TacticalServerRequestTableAccess,
    authorize_tactical_server_claim, record_tactical_scene_provisioning_failure,
    revoke_tactical_server_claim, tactical_server_requestQueryTableAccess,
};
use adventuresim_tactical_server_dispatcher::scene_input;
use adventuresim_tactical_server_dispatcher::settlement_buildings::{
    SettlementBusinessOperatorProfile, SettlementSceneProfile,
};
use adventuresim_terrain::{TerrainPack, TerrainPurpose};
use child_launch::{ChildExitOutcome, ExitedLaunch};
use clap::Parser;
use sha2::{Digest, Sha256};
use tracing::{error, info};

mod child_launch;
mod requested_scene;

const DISPATCH_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

#[derive(Parser, Debug)]
#[command(name = "adventuresim-tactical-server-dispatcher")]
#[command(about = "Spawns adventuresim-tactical-server processes for pending tactical scenes")]
struct Args {
    /// SpacetimeDB URL
    #[arg(long, default_value = "http://localhost:3000")]
    spacetimedb_url: String,

    /// SpacetimeDB module name
    #[arg(long, default_value = "adventuresim-stdb-module")]
    spacetimedb_module: String,

    /// Auth token for the registered strategic gateway identity.
    #[arg(long, env = "SPACETIMEDB_TOKEN")]
    spacetimedb_token: String,

    /// Path to tactical-server binary
    #[arg(long, default_value = "adventuresim-tactical-server")]
    tactical_server_bin: String,

    /// Base port for tactical servers (incremented for each new server)
    #[arg(long, default_value = "6000")]
    base_port: u16,

    /// Public host for clients to connect to
    #[arg(long, default_value = "0.0.0.0")]
    host: IpAddr,

    /// Final terrain manifest loaded once by this trusted dispatcher.
    #[arg(long, default_value = "target/strategic-map/terrain-routing-v3.json")]
    terrain_manifest: PathBuf,

    /// Final compressed terrain payload paired with `terrain_manifest`.
    #[arg(long, default_value = "target/strategic-map/terrain-routing-v3.pack")]
    terrain_pack: PathBuf,

    /// Worktree-local directory for immutable captured scene documents.
    #[arg(long, default_value = "target/tactical-scene-inputs")]
    scene_input_dir: PathBuf,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("tactical_spawner=info".parse().unwrap()),
        )
        .init();

    let args = Args::parse();
    info!("Starting tactical spawner");
    info!(
        "SpacetimeDB: {}/{}",
        args.spacetimedb_url, args.spacetimedb_module
    );
    info!("Tactical server binary: {}", args.tactical_server_bin);
    let terrain = load_final_terrain(&args).unwrap_or_else(|error| {
        error!(%error, "Invalid tactical terrain source");
        std::process::exit(2);
    });
    // Shared state for tracking spawned missions and port allocation
    let spawned: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    let mut next_port = args.base_port;

    // Connect to SpacetimeDB
    let conn = DbConnection::builder()
        .with_uri(&args.spacetimedb_url)
        .with_database_name(&args.spacetimedb_module)
        .with_token(Some(args.spacetimedb_token.clone()))
        .on_connect(|_ctx, _identity, _address| {
            info!("Connected to SpacetimeDB");
        })
        .on_connect_error(|_ctx, err| {
            error!("Connection error: {:?}", err);
        })
        .build()
        .expect("Failed to connect to SpacetimeDB");

    conn.subscription_builder()
        .on_applied(child_launch::reclaim_pending)
        .add_query(|query| query.from.tactical_server_request())
        .subscribe();
    let exited: Arc<Mutex<Vec<ExitedLaunch>>> = Arc::default();
    let bin = args.tactical_server_bin.clone();
    let stdb_url = args.spacetimedb_url.clone();
    let stdb_module = args.spacetimedb_module.clone();
    let host = args.host;
    let terrain_clone = terrain.clone();
    let scene_input_dir = args.scene_input_dir.clone();
    loop {
        if let Err(error) = conn.frame_tick() {
            error!(%error, "Dispatcher connection failed");
            break;
        }
        if let Err(error) = release_exited_launches(&conn, &spawned, &exited) {
            error!(%error, "Failed to release an exited scene launch");
            break;
        }
        for request in conn.db.tactical_server_request().iter() {
            let spawned_clone = spawned.clone();
            let admitted = match spawned_clone.lock() {
                Ok(mut spawned) => spawned.insert(request.instance_id.clone()),
                Err(error) => {
                    error!(%error, "Launch registry poisoned");
                    false
                }
            };
            if !admitted {
                continue;
            }
            let claim_bytes: [u8; 32] = rand::random();
            let claim = claim_bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            let claim_hash = Sha256::digest(claim.as_bytes()).to_vec();
            let instance_id = request.instance_id.clone();
            let scene_key = request.scene_key.clone();
            let Some(launch) = launch_arguments(&request) else {
                if let Ok(mut spawned) = spawned_clone.lock() {
                    spawned.remove(&instance_id);
                }
                continue;
            };
            let scene_input = match requested_scene::materialize_requested_scene(
                &terrain_clone,
                &scene_input_dir,
                &request,
            ) {
                Ok(path) => path.to_string_lossy().into_owned(),
                Err(error) => {
                    error!(%instance_id, %error, "Failed to provision tactical scene input");
                    if let Err(error) = conn
                        .reducers
                        .record_tactical_scene_provisioning_failure(instance_id.clone())
                    {
                        error!(%error, "Could not report scene provisioning failure");
                    }
                    if let Ok(mut spawned) = spawned_clone.lock() {
                        spawned.remove(&instance_id);
                    }
                    continue;
                }
            };
            let port = next_port;
            let Some(successor) = next_port.checked_add(1) else {
                error!("Tactical scene port allocation exhausted");
                break;
            };
            next_port = successor;

            let spawned = spawned_clone.clone();
            let child_launch = child_launch::ChildLaunch {
                instance_id: instance_id.clone(),
                scene_key,
                scene_input,
                addr: SocketAddr::new(host, port),
                bin: bin.clone(),
                stdb_url: stdb_url.clone(),
                stdb_module: stdb_module.clone(),
                claim,
                arguments: launch,
            };
            let exited_clone = exited.clone();
            let failed_id = instance_id.clone();
            if let Err(error) = conn.reducers.authorize_tactical_server_claim_then(
                instance_id.clone(),
                claim_hash,
                move |_ctx, result| match result {
                    Ok(Ok(())) => child_launch.start(_ctx, spawned.clone(), exited_clone),
                    Ok(Err(error)) => {
                        error!("Tactical claim rejected: {error}");
                        if let Ok(mut spawned) = spawned.lock() {
                            spawned.remove(&instance_id);
                        }
                    }
                    Err(error) => {
                        error!("Tactical claim reducer failed: {error}");
                        if let Ok(mut spawned) = spawned.lock() {
                            spawned.remove(&instance_id);
                        }
                    }
                },
            ) {
                error!("Failed to authorize tactical claim: {error}");
                if let Ok(mut spawned) = spawned_clone.lock() {
                    spawned.remove(&failed_id);
                }
            }
        }
        std::thread::sleep(DISPATCH_POLL_INTERVAL);
    }
}

fn release_exited_launches(
    conn: &DbConnection,
    spawned: &Mutex<HashSet<String>>,
    exited: &Mutex<Vec<ExitedLaunch>>,
) -> Result<(), String> {
    let retired = std::mem::take(&mut *exited.lock().map_err(|error| error.to_string())?);
    for retired_launch in retired {
        let instance_id = retired_launch.instance_id;
        spawned
            .lock()
            .map_err(|error| error.to_string())?
            .remove(&instance_id);
        // A failed child may leave an unused registration claim. A clean
        // retirement can be followed immediately by a new request for the
        // same shared slot; that request is not the old child's failure.
        if retired_launch.outcome == ChildExitOutcome::Failed
            && conn
                .db
                .tactical_server_request()
                .iter()
                .any(|row| row.instance_id == instance_id)
        {
            if let Err(error) = conn
                .reducers
                .record_tactical_scene_provisioning_failure(instance_id.clone())
            {
                error!(%error, "Failed to report interrupted scene launch");
            }
            if let Err(error) = conn.reducers.revoke_tactical_server_claim(instance_id) {
                error!(%error, "Failed to release interrupted launch claim");
            }
        }
    }
    Ok(())
}

fn load_final_terrain(args: &Args) -> Result<Arc<TerrainPack>, String> {
    let terrain = TerrainPack::load(&args.terrain_manifest, &args.terrain_pack)
        .map_err(|error| error.to_string())?;
    if terrain.purpose() != TerrainPurpose::Final {
        return Err("Dispatcher requires the final terrain pack".into());
    }
    info!(
        digest = terrain.digest(),
        "Loaded final tactical terrain source"
    );
    Ok(Arc::new(terrain))
}

/// Only missions project combat CLI arguments. Ordinary scenes have none.
fn launch_arguments(request: &TacticalServerRequest) -> Option<Vec<String>> {
    let adventuresim_stdb_client::TacticalSession::Mission(mission) = &request.session else {
        return Some(Vec::new());
    };
    match adventuresim_core::mission::TacticalPartyRoster::try_from(
        mission.authorized_party_member_ids.clone(),
    ) {
        Ok(roster) => Some(vec![
            "--expected-party-members".into(),
            roster.expected_members().to_string(),
            "--required-enemy-kills".into(),
            mission.required_enemy_kills.to_string(),
        ]),
        Err(error) => {
            error!(%error, "Invalid tactical party roster");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn child_spawn_waits_for_claim_and_drops_gateway_token() {
        let source = include_str!("main.rs");
        let launch = include_str!("child_launch.rs");
        let callback = source
            .split("authorize_tactical_server_claim_then")
            .nth(1)
            .expect("claim completion callback");
        assert!(callback.contains("Ok(Ok(()))"));
        assert!(launch.contains(".env_remove(\"SPACETIMEDB_TOKEN\")"));
        assert!(launch.contains(".env(\"ADVENTURESIM_TACTICAL_CLAIM\""));
    }
}
