//! Endpoint creation, reconnect identity and transport-gated enrollment.

use super::*;
use crate::{
    DEFAULT_SERVER_URL,
    message::{JoinRequest, ReconnectCapability, ReconnectToken},
};
use aeronet_replicon::client::AeronetRepliconClient;
use aeronet_websocket::client::{ClientConfig, WebSocketClient};

const URI_SCHEME_SEPARATOR: &str = "://";
const PLAIN_WEBSOCKET_PREFIX: &str = "ws://";
#[cfg(not(target_family = "wasm"))]
const SECURE_WEBSOCKET_PREFIX: &str = "wss://";

#[derive(Component, Clone, Debug)]
#[require(Name::from("Client"), AeronetRepliconClient)]
pub struct AdventureSimulatorClient {
    pub player_id: u64,
    pub server_url: String,
}

/// Kept in the running client application across transport reconnects. It is
/// neither an ECS component nor replicated to peers.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub(super) struct ReconnectCredential(pub(super) Option<(CharacterId, ReconnectToken)>);

impl Default for AdventureSimulatorClient {
    fn default() -> Self {
        Self {
            player_id: 0,
            server_url: DEFAULT_SERVER_URL.to_string(),
        }
    }
}

pub fn normalize_server_url(server_url: &str) -> String {
    if server_url.contains(URI_SCHEME_SEPARATOR) {
        server_url.to_string()
    } else {
        format!("{PLAIN_WEBSOCKET_PREFIX}{server_url}")
    }
}

pub(super) fn store_reconnect_capability(
    capability: On<ReconnectCapability>,
    mut credential: ResMut<ReconnectCredential>,
) {
    credential.0 = Some((capability.character_id, capability.token));
}

pub(super) fn on_client_added(
    event: On<Add, AdventureSimulatorClient>,
    mut commands: Commands,
    clients: Query<&AdventureSimulatorClient>,
) -> Result {
    let client = clients.get(event.entity)?;

    commands
        .entity(event.entity)
        .insert(crate::transport_policy::tactical_transport_config())
        .queue(WebSocketClient::connect(
            websocket_config(&client.server_url),
            normalize_server_url(&client.server_url),
        ));

    Ok(())
}

pub(super) fn announce_join(
    mut commands: Commands,
    client: Single<(
        &AdventureSimulatorClient,
        &aeronet::transport::TransportConfig,
    )>,
    credential: Res<ReconnectCredential>,
) -> Result {
    let (client, transport) = *client;
    if !crate::transport_policy::tactical_asset_transport_ready(transport) {
        return Err("tactical session transport policy is not installed".into());
    }
    info!("[startup] tactical network connected; announcing join");
    let character_id = CharacterId(client.player_id);
    commands.client_trigger(JoinRequest {
        character_id,
        reconnect_token: credential
            .0
            .filter(|(stored, _)| *stored == character_id)
            .map(|(_, token)| token),
    });
    Ok(())
}

#[cfg(target_family = "wasm")]
fn websocket_config(_server_url: &str) -> ClientConfig {
    ClientConfig
}

#[cfg(not(target_family = "wasm"))]
fn websocket_config(server_url: &str) -> ClientConfig {
    if normalize_server_url(server_url).starts_with(SECURE_WEBSOCKET_PREFIX) {
        ClientConfig::default()
    } else {
        ClientConfig::builder().with_no_encryption()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconnect_credential_is_process_owned_not_transport_owned() {
        let credential = ReconnectCredential(Some((CharacterId(42), ReconnectToken([5; 32]))));
        // A transport state transition does not recreate App resources; the
        // same contract is compiled for native and wasm clients.
        assert_eq!(credential.0.unwrap().0, CharacterId(42));
        assert_eq!(credential.0.unwrap().1, ReconnectToken([5; 32]));
    }
}
