//! Dispatch once and acknowledge only the current connection and admission.

use adventuresim_core::inventory_command::{InventoryCommandId, InventoryScopeId};
use adventuresim_stdb_client::{
    ConnectedInventory, EquipmentChangeSetRequest, TacticalActorTime, TacticalInventoryAction,
};
use bevy::ecs::system::SystemParam;

use super::*;
use crate::{
    local_time::LocalVisits,
    player_projection::inventory_projection::AppliedInventories,
    stdb::{
        ConnectedActorInventory, InventorySubmissionOutcome, SceneAdmissions, SceneReady,
        SpacetimeDb,
    },
};

#[derive(Resource, Default)]
pub(super) struct EquipmentCommits(HashMap<InventoryCommandId, PendingCommit>);

struct PendingCommit {
    client: Entity,
    character: CharacterId,
    scope: InventoryScopeId,
    sequence: EquipmentSequence,
    outcome: Option<InventorySubmissionOutcome>,
}

#[derive(SystemParam)]
pub(super) struct EquipmentCommandHost<'w> {
    connection: Option<Res<'w, SpacetimeDb>>,
    ready: Option<Res<'w, SceneReady>>,
    applied: Option<Res<'w, AppliedInventories>>,
    visits: Option<Res<'w, LocalVisits>>,
    admissions: Option<ResMut<'w, SceneAdmissions>>,
    commits: ResMut<'w, EquipmentCommits>,
    exit: MessageWriter<'w, AppExit>,
}

impl EquipmentCommandHost<'_> {
    pub(super) fn is_busy(&self, client: Entity) -> bool {
        self.commits
            .0
            .values()
            .any(|pending| pending.client == client)
    }

    pub(super) fn submit(
        &mut self,
        client: Entity,
        character: CharacterId,
        sequence: EquipmentSequence,
        changes: EquipmentChangeSetRequest,
    ) -> Result<(), EquipmentRejection> {
        let connection = self
            .connection
            .as_ref()
            .ok_or(EquipmentRejection::Unavailable)?;
        let snapshots = connection.inventory_snapshots();
        let snapshot = self
            .current_snapshot(client, character, &snapshots)
            .ok_or(EquipmentRejection::Unavailable)?;
        let actor = self
            .applied
            .as_ref()
            .and_then(|applied| applied.command_actor(snapshot))
            .ok_or(EquipmentRejection::Unavailable)?;
        let scope = InventoryScopeId::try_from(actor.scope.clone())
            .map_err(|_| EquipmentRejection::Unavailable)?;
        let command =
            InventoryCommandId::try_new(format!("native-{:032x}", rand::random::<u128>()))
                .map_err(|_| EquipmentRejection::Unavailable)?;
        if let Err(error) = connection.submit_inventory(
            command.clone(),
            actor,
            TacticalInventoryAction::ChangeEquipment(changes),
        ) {
            self.lose_authority(format!("Could not dispatch inventory command: {error}"));
            return Err(EquipmentRejection::Unavailable);
        }
        self.commits.0.insert(
            command,
            PendingCommit {
                client,
                character,
                scope,
                sequence,
                outcome: None,
            },
        );
        Ok(())
    }

    fn current_snapshot<'a>(
        &self,
        client: Entity,
        character: CharacterId,
        snapshots: &'a [ConnectedActorInventory],
    ) -> Option<&'a ConnectedActorInventory> {
        let admissions = self.admissions.as_ref()?;
        if self.ready.is_none()
            || admissions.is_closing()
            || admissions.actors.get(&character) != Some(&client)
        {
            return None;
        }
        let snapshot = snapshots
            .iter()
            .find(|snapshot| snapshot.character == character)?;
        if let TacticalActorTime::LocalVisit(clock) = &snapshot.time {
            let visit = self.visits.as_ref()?.live_id(character)?;
            if visit.get() != clock.visit_id {
                return None;
            }
        }
        Some(snapshot)
    }

    fn collect_results(&mut self) {
        let Some(connection) = &self.connection else {
            return;
        };
        let results = match connection.take_inventory_results() {
            Ok(results) => results,
            Err(error) => {
                self.lose_authority(error);
                return;
            }
        };
        for result in results {
            if let InventorySubmissionOutcome::AuthorityLost(error) = result.outcome {
                self.lose_authority(error);
                return;
            }
            if let Some(pending) = self.commits.0.get_mut(&result.command) {
                pending.outcome = Some(result.outcome);
            }
        }
    }

    fn lose_authority(&mut self, error: String) {
        error!(%error, "Equipment commit lost database authority");
        if let Some(admissions) = self.admissions.as_mut() {
            admissions.close();
        }
        self.exit.write(AppExit::error());
    }
}

pub(super) fn finish_equipment_commits(
    mut host: EquipmentCommandHost,
    actors: Query<(&CharacterId, &EquipmentActionState), With<Player>>,
    mut commands: Commands,
) {
    host.collect_results();
    let Some(connection) = &host.connection else {
        return;
    };
    let snapshots = connection.inventory_snapshots();
    let mut finished = Vec::new();
    for (id, pending) in &host.commits.0 {
        let snapshot = host.current_snapshot(pending.client, pending.character, &snapshots);
        let Some((snapshot, (_, state))) =
            snapshot
                .zip(actors.get(pending.client).ok())
                .filter(|(snapshot, (character, _))| {
                    **character == pending.character && same_scope(snapshot, pending.scope)
                })
        else {
            finished.push(id.clone());
            continue;
        };
        let outcome = match &pending.outcome {
            None => continue,
            Some(InventorySubmissionOutcome::Committed) => {
                // A callback is not equipment state. Wait until the current SDK
                // snapshot has passed the ordinary native projection admission.
                if !connection.has_inventory_receipt(id, pending.character, pending.scope)
                    || host
                        .applied
                        .as_ref()
                        .and_then(|applied| applied.command_actor(snapshot))
                        .is_none()
                {
                    continue;
                }
                EquipmentActionOutcome::Committed {
                    revision: state.revision,
                }
            }
            Some(InventorySubmissionOutcome::Rejected(reason)) => {
                EquipmentActionOutcome::Rejected(EquipmentRejection::Durable(reason.clone()))
            }
            Some(InventorySubmissionOutcome::AuthorityLost(_)) => continue,
        };
        respond(&mut commands, pending.client, pending.sequence, outcome);
        finished.push(id.clone());
    }
    for id in finished {
        host.commits.0.remove(&id);
    }
}

fn same_scope(snapshot: &ConnectedActorInventory, scope: InventoryScopeId) -> bool {
    matches!(&snapshot.inventory, ConnectedInventory::Available(inventory)
        if inventory.scope.as_slice() == scope.bytes())
}
