use std::net::SocketAddr;

use aeronet_replicon::server::{AeronetRepliconServer, AeronetRepliconServerPlugin};
use aeronet_websocket::server::{ServerConfig, WebSocketServer, WebSocketServerPlugin};
use bevy::prelude::*;

use crate::DEFAULT_SERVER_ADDR;

#[derive(Default)]
pub struct AdventureSimulatorServerPlugin;

impl Plugin for AdventureSimulatorServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WebSocketServerPlugin, AeronetRepliconServerPlugin))
            .add_observer(on_server_added)
            .add_observer(configure_session);
    }
}

#[derive(Component, Debug, Clone)]
// Aeronet parents each connection entity to the server. Party players reuse their
// connection entity, so the server must be a transform root for their world-space
// transforms to remain valid.
#[require(Name::from("Server"), AeronetRepliconServer, Transform)]
pub struct AdventureSimulatorServer {
    pub addr: SocketAddr,
}

impl Default for AdventureSimulatorServer {
    fn default() -> Self {
        Self {
            addr: DEFAULT_SERVER_ADDR,
        }
    }
}

fn on_server_added(
    event: On<Add, AdventureSimulatorServer>,
    mut commands: Commands,
    servers: Query<&AdventureSimulatorServer>,
) -> Result {
    let server = servers.get(event.entity)?;

    commands
        .entity(event.entity)
        .queue(WebSocketServer::open(websocket_config(server.addr)));

    Ok(())
}

fn configure_session(
    event: On<Add, aeronet::io::Session>,
    sessions: Query<&ChildOf, With<aeronet::io::Session>>,
    servers: Query<(), With<AdventureSimulatorServer>>,
    mut commands: Commands,
) {
    let Ok(parent) = sessions.get(event.entity) else {
        return;
    };
    if servers.contains(parent.parent()) {
        commands
            .entity(event.entity)
            .insert(crate::transport_policy::tactical_transport_config());
    }
}

fn websocket_config(addr: SocketAddr) -> ServerConfig {
    ServerConfig::builder()
        .with_bind_address(addr)
        .with_no_encryption()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_is_an_identity_transform_root() {
        let mut world = World::new();
        let server = world.spawn(AdventureSimulatorServer::default()).id();

        assert_eq!(world.get::<Transform>(server), Some(&Transform::IDENTITY));
        assert_eq!(
            world.get::<GlobalTransform>(server),
            Some(&GlobalTransform::IDENTITY)
        );
    }
}
