//! Capabilities belong to a character and server address. Browser tab storage
//! retains them across page reloads without sharing them with other tabs.

use super::*;
use crate::message::{JoinRequest, ReconnectDisposition, SceneJoinRejected};

#[derive(Resource, Default)]
pub(super) struct ReconnectCredential(Option<SavedReconnect>);

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedReconnect {
    server_url: String,
    character_id: CharacterId,
    token: ReconnectToken,
}

impl ReconnectCredential {
    pub(super) fn for_client(&self, client: &AdventureSimulatorClient) -> Option<ReconnectToken> {
        if let Some(saved) = self.0.as_ref().filter(|saved| {
            saved.character_id.0 == client.player_id && saved.server_url == client.server_url
        }) {
            return Some(saved.token);
        }
        #[cfg(target_family = "wasm")]
        {
            let value = browser_load(&client.server_url, &client.player_id.to_string());
            let saved: SavedReconnect = serde_json::from_str(&value).ok()?;
            if saved.character_id.0 == client.player_id && saved.server_url == client.server_url {
                return Some(saved.token);
            }
        }
        None
    }
}

pub(super) fn announce_join(
    mut commands: Commands,
    client: Single<&AdventureSimulatorClient>,
    credential: Res<ReconnectCredential>,
) {
    info!("[startup] tactical network connected; announcing join");
    let character_id = CharacterId(client.player_id);
    commands.client_trigger(JoinRequest {
        character_id,
        reconnect_token: credential.for_client(&client),
    });
}

pub(super) fn clear_disconnected_scene(
    mut commands: Commands,
    remote_entities: Query<Entity, With<Remote>>,
) {
    // Replicon resets its entity map on disconnect but leaves received entities
    // alive. A new connection must build one fresh projection from the server's
    // retained state instead of displaying both old and newly mapped roots.
    for entity in &remote_entities {
        commands.entity(entity).try_despawn();
    }
}

pub(super) fn store_reconnect_capability(
    capability: On<ReconnectCapability>,
    client: Single<&AdventureSimulatorClient>,
    mut credential: ResMut<ReconnectCredential>,
) {
    if capability.character_id.0 != client.player_id {
        return;
    }
    let saved = SavedReconnect {
        server_url: client.server_url.clone(),
        character_id: capability.character_id,
        token: capability.token,
    };
    #[cfg(target_family = "wasm")]
    if let Ok(value) = serde_json::to_string(&saved) {
        browser_store(&client.server_url, &client.player_id.to_string(), &value);
    }
    credential.0 = Some(saved);
}

pub(super) fn clear_rejected_reconnect(
    rejection: On<SceneJoinRejected>,
    client: Single<&AdventureSimulatorClient>,
    mut credential: ResMut<ReconnectCredential>,
) {
    if rejection.character_id.0 != client.player_id
        || rejection.reconnect == ReconnectDisposition::Retain
    {
        return;
    }
    credential.0 = None;
    #[cfg(target_family = "wasm")]
    browser_remove(&client.server_url, &client.player_id.to_string());
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(inline_js = "
const key = (server, character) => 'fabelgeist.reconnect:' + JSON.stringify([server, character]);
export function browser_load(server, character) {
    try { return sessionStorage.getItem(key(server, character)) || ''; } catch { return ''; }
}
export function browser_store(server, character, value) {
    try { sessionStorage.setItem(key(server, character), value); } catch {}
}
export function browser_remove(server, character) {
    try { sessionStorage.removeItem(key(server, character)); } catch {}
}
")]
extern "C" {
    fn browser_load(server: &str, character: &str) -> String;
    fn browser_store(server: &str, character: &str, value: &str);
    fn browser_remove(server: &str, character: &str);
}
