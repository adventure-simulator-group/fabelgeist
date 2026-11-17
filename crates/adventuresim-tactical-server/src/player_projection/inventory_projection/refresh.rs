//! Consume the current host-scoped cache; retired snapshots cannot update actors.

use std::collections::BTreeMap;

use adventuresim_stdb_client::TacticalActorTime;
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

use super::apply::{ActorInventory, AppliedInventories, apply_batch};
use super::{InventoryAdmissionError, InventoryAdmissionResult, PreparedInventory};
use crate::{
    local_time::LocalVisits,
    stdb::{ConnectedActorInventory, SpacetimeDb},
};

pub(in super::super::super) fn refresh_connected_inventory(world: &mut World) -> Result {
    let Some(connection) = world.get_resource::<SpacetimeDb>() else {
        return Ok(());
    };
    let snapshots = connection.inventory_snapshots();
    refresh_snapshots(world, snapshots)?;
    Ok(())
}

pub(super) fn refresh_snapshots(
    world: &mut World,
    snapshots: Vec<ConnectedActorInventory>,
) -> InventoryAdmissionResult<()> {
    let roots = actor_roots(world)?;
    if let Some(mut applied) = world.get_resource_mut::<AppliedInventories>() {
        applied
            .0
            .retain(|character, _| roots.contains_key(character));
    }
    let contact = world.resource::<TacticalCombatConfig>().resolution.contact;
    let mut changed = Vec::new();
    for snapshot in snapshots {
        let Some(owner) = roots.get(&snapshot.character).copied() else {
            continue;
        };
        if let TacticalActorTime::LocalVisit(clock) = &snapshot.time {
            let live_id = world
                .get_resource::<LocalVisits>()
                .and_then(|visits| visits.live_id(snapshot.character));
            if !live_id.is_some_and(|id| id.get() == clock.visit_id) {
                continue;
            }
        }
        if world
            .get_resource::<AppliedInventories>()
            .and_then(|applied| applied.0.get(&snapshot.character))
            .is_some_and(|applied| {
                applied.matches(&snapshot.time, &snapshot.inventory)
            })
        {
            continue;
        }
        let prepared = PreparedInventory::new(&snapshot.inventory, contact)?;
        changed.push(ActorInventory {
            owner,
            character: snapshot.character,
            time: snapshot.time,
            prepared,
        });
    }
    if !changed.is_empty() {
        apply_batch(world, changed)?;
    }
    Ok(())
}

fn actor_roots(world: &mut World) -> InventoryAdmissionResult<BTreeMap<CharacterId, Entity>> {
    let mut roots = BTreeMap::new();
    let mut live = world.query_filtered::<(Entity, &CharacterId), With<Player>>();
    for (entity, character) in live.iter(world) {
        if roots.insert(*character, entity).is_some() {
            return Err(InventoryAdmissionError::AmbiguousActor);
        }
    }
    let mut retained = world.query::<(Entity, &super::super::DisconnectedPlayer)>();
    for (entity, character) in retained
        .iter(world)
        .filter(|(_, character)| !character.claimed)
    {
        if roots.insert(character.character_id, entity).is_some() {
            return Err(InventoryAdmissionError::AmbiguousActor);
        }
    }
    Ok(roots)
}
