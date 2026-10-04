use clap::Parser;
use std::{net::IpAddr, path::PathBuf};

#[derive(Parser, Debug)]
#[command(name = "adventuresim-tactical-server-dispatcher")]
#[command(about = "Spawns adventuresim-tactical-server processes for pending missions")]
pub(super) struct Args {
    /// SpacetimeDB URL
    #[arg(long, default_value = "http://localhost:3000")]
    pub(super) spacetimedb_url: String,

    /// SpacetimeDB module name
    #[arg(long, default_value = "adventuresim-stdb-module")]
    pub(super) spacetimedb_module: String,

    /// Auth token for the registered strategic gateway identity.
    #[arg(long, env = "SPACETIMEDB_TOKEN")]
    pub(super) spacetimedb_token: String,

    /// Path to tactical-server binary
    #[arg(long, default_value = "adventuresim-tactical-server")]
    pub(super) tactical_server_bin: String,

    /// Base port for tactical servers (incremented for each new server)
    #[arg(long, default_value = "6000")]
    pub(super) base_port: u16,

    /// Final permitted mission listener port; allocation stops at this bound.
    #[arg(long, default_value_t = u16::MAX)]
    pub(super) last_port: u16,

    /// Bind address for spawned tactical listeners
    #[arg(long, default_value = "127.0.0.1")]
    pub(super) host: IpAddr,

    /// Final terrain manifest loaded once by this trusted dispatcher.
    #[arg(long, default_value = "target/strategic-map/terrain-routing-v3.json")]
    pub(super) terrain_manifest: PathBuf,

    /// Final compressed terrain payload paired with `terrain_manifest`.
    #[arg(long, default_value = "target/strategic-map/terrain-routing-v3.pack")]
    pub(super) terrain_pack: PathBuf,

    /// Worktree-local directory for immutable per-mission scene documents.
    #[arg(long, default_value = "target/tactical-scene-inputs")]
    pub(super) scene_input_dir: PathBuf,
}
