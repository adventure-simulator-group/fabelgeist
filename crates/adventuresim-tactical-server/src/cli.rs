use crate::{bot, mission::PARTY_RECONNECT_GRACE};
use bevy::prelude::Resource;
use clap::{ArgAction, Parser};
use std::{net::SocketAddr, path::PathBuf};

const MISSION_TIMEOUT_SECS: f32 = 300.0;

#[derive(Parser, Debug, Clone, Resource)]
#[command(name = "adventuresim-tactical-server")]
#[command(about = "Live tactical server for Fabelgeist")]
pub(crate) struct Args {
    #[arg(long, default_value = "127.0.0.1:6000")]
    pub(crate) addr: SocketAddr,
    #[arg(long)]
    pub(crate) instance_id: adventuresim_core::tactical_session::TacticalInstanceId,
    #[arg(long, env = "ADVENTURESIM_TACTICAL_CLAIM", hide_env_values = true)]
    pub(crate) tactical_claim: String,
    #[arg(long, default_value = "woodland")]
    pub(crate) scene_key: String,
    /// Exact versioned scene input. Defaults to the committed dense woodland
    /// fixture for standalone tactical development.
    #[arg(long, value_parser = bot::resolve_scene_fixture)]
    pub(crate) scene_input: Option<PathBuf>,
    /// Versioned tactical combat tuning loaded once for this server process.
    #[arg(long)]
    pub(crate) combat_config: Option<PathBuf>,
    /// Standalone enemy roster; selected independently from the scene input.
    #[arg(long, value_parser = bot::load_enemy_fixture)]
    pub(crate) enemy_fixture: Option<adventuresim_core::tactical_fixture::TacticalEnemyFixture>,
    #[arg(long)]
    pub(crate) required_enemy_kills: Option<u32>,
    #[arg(long, value_parser = clap::value_parser!(std::num::NonZeroU32))]
    pub(crate) expected_party_members: Option<std::num::NonZeroU32>,
    #[arg(long, default_value = "http://localhost:3000")]
    pub(crate) spacetimedb_url: String,
    #[arg(long, default_value = "adventuresim-stdb-module")]
    pub(crate) spacetimedb_module: String,
    #[arg(long, default_value_t = MISSION_TIMEOUT_SECS)]
    pub(crate) timeout: f32,
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "timeout")]
    pub(crate) no_timeout: bool,
    /// Seconds the mission tolerates an empty party (every member
    /// disconnected) before abandoning it as a failure and shutting down.
    /// Raise it so a client can reload/reconnect without killing the mission.
    #[arg(long, default_value_t = PARTY_RECONNECT_GRACE.as_secs_f32())]
    pub(crate) party_reconnect_grace: f32,
    /// Port to expose the Bevy Remote Protocol (BRP) HTTP JSON-RPC endpoint
    /// on for CLI-driven inspection/testing. Disabled unless set.
    #[cfg(feature = "debug")]
    #[arg(long)]
    pub(crate) brp_port: Option<u16>,
    /// Path to a `.scn.ron` world dump (see `DebugDumpWorldRequest`) to load
    /// at startup in place of generating fresh procedural terrain.
    #[cfg(feature = "debug")]
    #[arg(long)]
    pub(crate) world_dump: Option<std::path::PathBuf>,
}
