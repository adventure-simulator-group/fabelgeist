//! Retain disconnected actors until their authority handoff is acknowledged.

use super::*;

/// `aeronet_io`'s own `ConnectionPlugin` (part of `AdventureSimulatorNetPlugins`)
/// registers an observer on this exact same `Disconnected` trigger that
/// unconditionally despawns `entity` right afterward - see its doc comment:
/// "Immediately after this, the session will be despawned". Bevy documents
/// same-event observer ordering as unspecified. Snapshot every reconnect-owned
/// component while the source is still queryable, then queue only operations
/// whose inputs are owned values. Aeronet may therefore apply its source
/// despawn before or after these commands without invalidating them.
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects each observer resource and query as an independent system parameter"
)]
pub(crate) fn on_client_disconnected(
    disconnected: On<Disconnected>,
    query: Query<&ReconnectSession>,
    projected_core: ProjectedPlayerCoreQuery<'_, '_>,
    projected_motion: ProjectedPlayerMotionQuery<'_, '_>,
    projected_physics: Query<(
        &Collider,
        &CollisionMargin,
        &CharacterController,
        &AccumulatedInput,
    )>,
    loading_players: Query<&LoadingPlayer>,
    inventory_items: Query<(Entity, &ItemOf)>,
    mut commands: Commands,
) -> Result {
    let entity = disconnected.event_target();
    let Ok(session) = query.get(entity) else {
        return Ok(());
    };
    let projection = if let (Ok(core), Ok(motion), Ok(physics)) = (
        projected_core.get(entity),
        projected_motion.get(entity),
        projected_physics.get(entity),
    ) {
        DisconnectedProjection::Projected(Box::new(ProjectedPlayerSnapshot::new(
            core, motion, physics,
        )))
    } else if let Ok(loading) = loading_players.get(entity) {
        DisconnectedProjection::Loading(*loading)
    } else {
        warn!(
            ?entity,
            "Disconnected reconnect session had no player projection to retain"
        );
        return Ok(());
    };
    let projected = projection.projected();
    let orphan = commands
        .spawn((
            projection,
            DisconnectedPlayer {
                character_id: session.character_id,
                reconnect_token: session.token,
                remaining_secs: RECONNECT_GRACE_SECS,
                claimed: false,
            },
        ))
        .id();
    for (item, owner) in &inventory_items {
        if owner.0 == entity {
            commands.entity(item).insert(ItemOf(orphan));
        }
    }
    if projected {
        commands.queue(move |world: &mut World| rebuild_inventory_holding_cache(world, orphan));
    }
    let character_id = session.character_id.0;
    match &disconnected.reason {
        DisconnectReason::ByUser(reason) => {
            info!("Character {character_id} disconnected by server request: {reason}")
        }
        DisconnectReason::ByPeer(reason) => {
            info!("Character {character_id} disconnected by peer: {reason}")
        }
        DisconnectReason::ByError(error) => {
            warn!("Character {character_id} disconnected due to error: {error:#}")
        }
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects departure authority, clocks, and separate inventory lifecycle queries"
)]
pub(crate) fn expire_disconnected_players(
    time: Res<Time>,
    real_time: Res<Time<Real>>,
    mut commands: Commands,
    mut disconnected: Query<(Entity, &mut DisconnectedPlayer, &mut DisconnectedProjection)>,
    items: Query<(Entity, &ItemOf)>,
    mut pending_actions: ResMut<PendingEquipmentActions>,
    mut action_sequences: ResMut<LastEquipmentSequence>,
    conn: Res<SpacetimeDb>,
    args: Res<Args>,
    mut visits: ResMut<crate::local_time::LocalVisits>,
    mut exit: MessageWriter<AppExit>,
) {
    let local_scene = matches!(
        args.instance_id,
        adventuresim_core::tactical_session::TacticalInstanceId::Settlement(_)
    );
    for (entity, mut grace, mut projection) in &mut disconnected {
        // A successful reconnect synchronously owns this root. Its deferred
        // moves must complete before any expiry teardown can touch it.
        if grace.claimed {
            continue;
        }
        grace.remaining_secs -= if local_scene {
            real_time.delta_secs()
        } else {
            time.delta_secs()
        };
        if grace.remaining_secs > 0.0 {
            continue;
        }
        if local_scene {
            let fatigue = projection.combat_state_mut().map(|state| state.fatigue);
            match visits.depart(grace.character_id, fatigue, real_time.elapsed(), &conn) {
                Ok(true) => visits.release(grace.character_id),
                Ok(false) => continue,
                Err(error) => {
                    error!(%error, "Cannot finish local visit; releasing server authority");
                    exit.write(AppExit::error());
                    continue;
                }
            }
        } else if let Err(error) = conn.reducers().leave_tactical_scene(grace.character_id.0) {
            warn!(
                character_id = grace.character_id.0,
                ?error,
                "Failed to expire disconnected mission member"
            );
            continue;
        }
        for (item, owner) in &items {
            if owner.0 == entity {
                commands.entity(item).despawn();
            }
        }
        purge_equipment_lifecycle(entity, &mut pending_actions, &mut action_sequences);
        commands.entity(entity).despawn();
    }
}
