#[cfg(all(target_arch = "wasm32", feature = "server"))]
compile_error!("The `server` feature cannot be enabled when compiling for wasm32.");

#[cfg(feature = "client")]
pub mod client;
pub mod message;
pub mod replication;
#[cfg(feature = "server")]
pub mod server;

pub use aeronet;
pub use aeronet_replicon;
pub use aeronet_websocket;
pub use bevy_replicon;

pub mod prelude {
    pub use crate::AdventureSimulatorNetPlugins;
    #[cfg(feature = "client")]
    pub use crate::client::{AdventureSimulatorClient, PlayerInputOverride};
    pub use crate::message::{
        DebugDumpWorldRequest, DebugGameTimeScaleRequest, DefendRequest, EquipmentAction,
        EquipmentActionRequest, EquipmentHand, JoinRequest, JumpCommand, MeleeActionRequest,
        PlayerInputRequest, PostureActionRequest, PostureCommand, RangedActionRequest,
        ReconnectCapability, ReconnectToken, SceneVistaBundle, TacticalCombatConfigSnapshot,
        TacticalOutcome, TacticalOutcomeResponse,
    };
    #[cfg(feature = "server")]
    pub use crate::server::AdventureSimulatorServer;
    #[cfg(feature = "client")]
    pub use aeronet::io::connection::{DisconnectReason, Disconnected};
}

/// Fixed simulation tick shared by the tactical server and client. Matches the
/// 30 Hz ceiling of the netahoy transport the tactical layer is moving to.
/// Locomotion samples stay at their own rate (`locomotion.sample_hz`); the
/// server derives the sample tick from elapsed time, so a tick spans several
/// samples.
pub const TACTICAL_TICK_HZ: u32 = 30;
pub const DEFAULT_SERVER_ADDR: std::net::SocketAddr =
    std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 6000);
pub const DEFAULT_SERVER_URL: &str = "ws://127.0.0.1:6000";

bevy::app::plugin_group! {
    #[derive(Debug)]
    pub struct AdventureSimulatorNetPlugins {
        #[custom(cfg(feature = "server"))]
        crate::server:::AdventureSimulatorServerPlugin,
        #[custom(cfg(feature = "client"))]
        crate::client:::AdventureSimulatorClientPlugin,
        crate::replication:::AdventureSimulatorReplicationPlugin,
    }
}
