//! JSON-lines adapter for exercising reducers over the registered SDK connection.

use std::{
    io::{self, BufRead},
    sync::mpsc,
};

use adventuresim_stdb_client::{
    ConnectedInventory, ConnectedPlayersTableAccess, DbConnection, LocalSceneVisitExit,
    TacticalInventoryCommand, commit_tactical_inventory, enter_tactical_scene,
    finish_local_scene_visit, leave_tactical_scene,
    spacetimedb_sdk::{DbContext, Table},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use spacetimedb_sats::serde::SerdeWrapper;

#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub(super) enum PeerCommand {
    Inventory {
        request: SerdeWrapper<TacticalInventoryCommand>,
    },
    EquipmentArgument {
        request: SerdeWrapper<adventuresim_stdb_client::EquipmentChangeSetRequest>,
    },
    Enter {
        character_id: u64,
    },
    Leave {
        character_id: u64,
    },
    FinishVisit {
        visit_id: u64,
    },
    Snapshot {
        character_id: u64,
    },
    Stop,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum PeerStatus {
    Accepted,
    Rejected,
    TransportError,
    ProjectionError,
    DecodeError,
    Snapshot,
}

pub(super) enum PeerControl {
    Continue,
    Stop,
}

impl PeerCommand {
    pub(super) fn dispatch(self, conn: &DbConnection) -> PeerControl {
        match self {
            Self::EquipmentArgument {
                request: SerdeWrapper(request),
            } => match adventuresim_stdb_client::json_reducer_argument(&request) {
                Ok(argument) => println!(
                    "{}",
                    json!({"key": "equipment_argument", "status": PeerStatus::Accepted, "argument": argument})
                ),
                Err(error) => reply(
                    "equipment_argument",
                    PeerStatus::TransportError,
                    Some(error.to_string()),
                ),
            },
            Self::Inventory {
                request: SerdeWrapper(request),
            } => invoke_inventory(conn, request),
            Self::Enter { character_id } => enter(conn, character_id),
            Self::Leave { character_id } => leave(conn, character_id),
            Self::FinishVisit { visit_id } => finish_visit(conn, visit_id),
            Self::Snapshot { character_id } => snapshot(conn, character_id),
            Self::Stop => return PeerControl::Stop,
        }
        PeerControl::Continue
    }
}

pub(super) fn input() -> mpsc::Receiver<PeerCommand> {
    let (sender, received) = mpsc::channel();
    std::thread::spawn(move || {
        for line in io::stdin().lock().lines() {
            let command = line
                .map_err(|error| error.to_string())
                .and_then(|line| serde_json::from_str(&line).map_err(|error| error.to_string()));
            match command {
                Ok(command) => {
                    if sender.send(command).is_err() {
                        return;
                    }
                }
                Err(error) => reply("decode", PeerStatus::DecodeError, Some(error)),
            }
        }
        let _ = sender.send(PeerCommand::Stop);
    });
    received
}

fn invoke_inventory(conn: &DbConnection, request: TacticalInventoryCommand) {
    let key = request.command_id.clone();
    let callback_key = key.clone();
    if let Err(error) = conn
        .reducers
        .commit_tactical_inventory_then(request, move |_, result| {
            complete_callback(&callback_key, result);
        })
    {
        reply(&key, PeerStatus::TransportError, Some(error.to_string()));
    }
}

fn enter(conn: &DbConnection, character: u64) {
    let key = format!("enter:{character}");
    let Some(server) = conn.try_identity() else {
        reply(
            &key,
            PeerStatus::TransportError,
            Some("Peer has no registered identity".into()),
        );
        return;
    };
    let callback_key = key.clone();
    if let Err(error) =
        conn.reducers
            .enter_tactical_scene_then(character, server, move |_, result| {
                complete_callback(&callback_key, result);
            })
    {
        reply(&key, PeerStatus::TransportError, Some(error.to_string()));
    }
}

fn leave(conn: &DbConnection, character: u64) {
    let key = format!("leave:{character}");
    let callback_key = key.clone();
    if let Err(error) = conn
        .reducers
        .leave_tactical_scene_then(character, move |_, result| {
            complete_callback(&callback_key, result);
        })
    {
        reply(&key, PeerStatus::TransportError, Some(error.to_string()));
    }
}

fn finish_visit(conn: &DbConnection, visit: u64) {
    let key = format!("finish:{visit}");
    let callback_key = key.clone();
    if let Err(error) = conn.reducers.finish_local_scene_visit_then(
        visit,
        LocalSceneVisitExit::Unprojected,
        move |_, result| {
            complete_callback(&callback_key, result);
        },
    ) {
        reply(&key, PeerStatus::TransportError, Some(error.to_string()));
    }
}

fn snapshot(conn: &DbConnection, character: u64) {
    let key = format!("snapshot:{character}");
    let player = conn
        .db
        .connected_players()
        .iter()
        .find(|row| row.character.id == character);
    let Some(player) = player else {
        println!(
            "{}",
            json!({"key": key, "status": PeerStatus::Snapshot, "state": null, "items": []})
        );
        return;
    };
    match player.inventory {
        ConnectedInventory::Available(snapshot) => {
            let items: Result<Vec<_>, adventuresim_core::equipment_placement::AuthoredPlacementIdentityError> = snapshot.items.iter().map(|row| {
                let placement = row.selected_placement.clone().map(adventuresim_core::equipment_placement::EquippedPlacement::try_from).transpose()?;
                Ok(json!({
                "id": row.inventory_item_id, "catalog": row.item.id, "quantity": row.quantity,
                "object": row.physical_object_id, "mass_kg": row.carried_mass_kg,
                "placement": placement,
                "protection": row.protected_body_parts.iter().map(|part| format!("{part:?}")).collect::<Vec<_>>(),
            }))}).collect();
            let items = match items {
                Ok(items) => items,
                Err(error) => {
                    reply(&key, PeerStatus::ProjectionError, Some(error.to_string()));
                    return;
                }
            };
            println!(
                "{}",
                json!({"key": key, "status": PeerStatus::Snapshot, "state": {
                "character_id": character, "scope": snapshot.scope, "revision": snapshot.revision,
            }, "items": items})
            );
        }
        ConnectedInventory::Unavailable(error) => reply(
            &key,
            PeerStatus::ProjectionError,
            Some(format!("{error:?}")),
        ),
    }
}

fn complete_callback<E: std::fmt::Display>(key: &str, result: Result<Result<(), String>, E>) {
    match result {
        Ok(Ok(())) => reply(key, PeerStatus::Accepted, None),
        Ok(Err(error)) => reply(key, PeerStatus::Rejected, Some(error)),
        Err(error) => reply(key, PeerStatus::TransportError, Some(error.to_string())),
    }
}

fn reply(key: &str, status: PeerStatus, error: Option<String>) {
    println!("{}", json!({"key": key, "status": status, "error": error}));
}
