//! Reducer acknowledgements cross into Bevy through an owned mailbox.

use std::collections::HashMap;

use adventuresim_tactical_core::player::CharacterId;
use adventuresim_tactical_netcode::{
    bevy_replicon::prelude::{ClientId, SendTargets, ServerTriggerExt, ToClients},
    message::{ReconnectDisposition, SceneJoinRejected},
};

use super::*;
use crate::player_projection::{LoadingPlayer, ReconnectSession};

#[derive(Resource, Default)]
pub(crate) struct SceneAdmissions {
    pub(crate) actors: HashMap<CharacterId, Entity>,
    gate: AdmissionGate,
}

#[derive(Resource)]
pub(crate) struct SceneReady;

pub(crate) struct SceneOperationResult {
    operation: SceneOperation,
    result: Result<(), String>,
}

#[derive(Default)]
enum AdmissionGate {
    #[default]
    Open,
    Closing,
}

pub(crate) enum SceneOperation {
    Registration,
    Join {
        client: Entity,
        character: CharacterId,
    },
    Retirement,
    ConnectionLost,
}

impl SceneAdmissions {
    pub(crate) fn close(&mut self) {
        self.gate = AdmissionGate::Closing;
    }
    pub(crate) fn is_closing(&self) -> bool {
        matches!(self.gate, AdmissionGate::Closing)
    }
}

impl SpacetimeDb {
    pub(crate) fn register_scene(
        &self,
        instance_id: String,
        claim: String,
        addr: String,
        cert_digest: String,
    ) -> spacetimedb_sdk::Result<()> {
        let mailbox = self.scene_operations.clone();
        self.reducers().create_tactical_server_for_request_then(
            instance_id,
            claim,
            addr,
            cert_digest,
            move |_, result| record(&mailbox, SceneOperation::Registration, result),
        )
    }

    pub(crate) fn request_join(
        &self,
        character: CharacterId,
        client: Entity,
        server: Identity,
    ) -> spacetimedb_sdk::Result<()> {
        let mailbox = self.scene_operations.clone();
        self.reducers()
            .enter_tactical_scene_then(character.0, server, move |_, result| {
                record(&mailbox, SceneOperation::Join { client, character }, result);
            })
    }

    pub(crate) fn retire_settlement(&self) -> spacetimedb_sdk::Result<()> {
        let mailbox = self.scene_operations.clone();
        self.reducers()
            .retire_settlement_scene_then(move |_, result| {
                record(&mailbox, SceneOperation::Retirement, result);
            })
    }
}

pub(crate) fn process_scene_operations(
    conn: Res<SpacetimeDb>,
    mut commands: Commands,
    mut admissions: ResMut<SceneAdmissions>,
    mut exit: MessageWriter<AppExit>,
) {
    let results = match conn.scene_operations.lock() {
        Ok(mut mailbox) => std::mem::take(&mut *mailbox),
        Err(error) => {
            error!(%error, "Scene acknowledgement mailbox poisoned");
            exit.write(AppExit::error());
            return;
        }
    };
    for result in results {
        match (result.operation, result.result) {
            (SceneOperation::Registration, Ok(())) => {
                commands.insert_resource(SceneReady);
            }
            (SceneOperation::Registration, Err(error)) => {
                error!(%error, "Scene registration rejected");
                exit.write(AppExit::error());
            }
            (SceneOperation::ConnectionLost, result) => {
                admissions.close();
                error!(?result, "Scene lost database authority; shutting down");
                exit.write(AppExit::error());
            }
            (SceneOperation::Retirement, Ok(())) => {
                exit.write(AppExit::Success);
            }
            (SceneOperation::Retirement, Err(error)) => {
                error!(%error, "Scene retirement rejected; releasing server connection");
                exit.write(AppExit::error());
            }
            (SceneOperation::Join { client, character }, Err(error)) => {
                if admissions.actors.get(&character) == Some(&client) {
                    admissions.actors.remove(&character);
                    if let Ok(mut entity) = commands.get_entity(client) {
                        entity.remove::<(LoadingPlayer, ReconnectSession)>();
                        commands.server_trigger(ToClients {
                            targets: SendTargets::Single(ClientId::Client(client)),
                            message: SceneJoinRejected {
                                character_id: character,
                                reason: error,
                                reconnect: ReconnectDisposition::Discard,
                            },
                        });
                    }
                }
            }
            (SceneOperation::Join { .. }, Ok(())) => {}
        }
    }
}

pub(super) fn connection_lost(mailbox: &Mutex<Vec<SceneOperationResult>>, reason: String) {
    if let Ok(mut mailbox) = mailbox.lock() {
        mailbox.push(SceneOperationResult {
            operation: SceneOperation::ConnectionLost,
            result: Err(reason),
        });
    }
}

fn record(
    mailbox: &Mutex<Vec<SceneOperationResult>>,
    operation: SceneOperation,
    result: Result<Result<(), String>, spacetimedb_sdk::error::InternalError>,
) {
    let result = result.unwrap_or_else(|error| Err(format!("Reducer callback failed: {error}")));
    match mailbox.lock() {
        Ok(mut mailbox) => mailbox.push(SceneOperationResult { operation, result }),
        Err(error) => error!(%error, "Could not record scene acknowledgement"),
    }
}
