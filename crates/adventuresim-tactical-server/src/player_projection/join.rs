//! Client admission and reconnect binding for mission and settlement scenes.

use super::*;
use crate::stdb::{SceneAdmissions, SceneReady};
use adventuresim_tactical_netcode::message::{ReconnectDisposition, SceneJoinRejected};

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects each connection resource and projection query as an independent observer parameter"
)]
pub(crate) fn on_join_request(
    join: On<FromClient<JoinRequest>>,
    mut commands: Commands,
    mut state: Option<ResMut<MissionState>>,
    loading_players: Query<(), With<LoadingPlayer>>,
    players: Query<(), With<Player>>,
    mut disconnected_players: Query<(Entity, &mut DisconnectedPlayer, &DisconnectedProjection)>,
    inventory_items: Query<(Entity, &ItemOf)>,
    mut pending_actions: ResMut<PendingEquipmentActions>,
    mut action_sequences: ResMut<LastEquipmentSequence>,
    conn: Res<SpacetimeDb>,
    ready: Res<SpacetimeDbReady>,
    scene_ready: Option<Res<SceneReady>>,
    mut admissions: ResMut<SceneAdmissions>,
    entities: Query<(Entity, Option<&ReconnectSession>)>,
    vista: Res<SceneVistaBundleResource>,
    combat_config: Res<TacticalCombatConfig>,
) -> Result {
    let Some(client) = join.client_id.entity() else {
        return Ok(());
    };
    if loading_players.contains(client) || players.contains(client) {
        return Ok(());
    }
    admissions
        .actors
        .retain(|_, entity| entities.contains(*entity));
    let reconnect =
        disconnected_players
            .iter_mut()
            .find_map(|(entity, mut session, projection)| {
                try_claim_reconnect(join.character_id, join.reconnect_token, &mut session)
                    .then(|| (entity, projection.clone()))
            });
    if let Some((disconnected, projection)) = reconnect {
        admissions.actors.insert(join.character_id, client);
        restore_projection(
            &mut commands,
            client,
            join.client_id,
            join.character_id,
            disconnected,
            projection,
            &inventory_items,
            &mut pending_actions,
            &mut action_sequences,
            &vista,
            &combat_config,
        );
        return Ok(());
    }
    let retained = disconnected_players
        .iter()
        .any(|(_, session, _)| session.character_id == join.character_id);
    if let Some(reason) = fresh_join_rejection(
        &join,
        state.as_deref(),
        &admissions,
        scene_ready.is_some(),
        retained,
        active_capability_valid(&join, &admissions, &entities),
    ) {
        reject(&mut commands, join.client_id, reason);
        return Ok(());
    }
    conn.request_join(join.character_id, client, ready.identity())?;
    admissions.actors.insert(join.character_id, client);
    if let Some(state) = state.as_mut() {
        state.begin_enrollment();
    }
    let token = fresh_reconnect_token();
    commands.entity(client).insert((
        LoadingPlayer {
            requested_character: join.character_id,
        },
        ReconnectSession {
            character_id: join.character_id,
            token,
        },
    ));
    send_reconnect_capability(&mut commands, join.client_id, join.character_id, token);
    send_scene_vista(&mut commands, join.client_id, &vista);
    send_combat_config(&mut commands, join.client_id, &combat_config);
    info!(
        "[startup] Scene entry requested for character {}; awaiting projection",
        join.character_id.0
    );
    Ok(())
}

/// Standalone-mode counterpart to [`on_join_request`]: no SpacetimeDB
/// authorization round-trip. [`bind_dumped_character_on_join`] is what turns
/// the resulting [`LoadingPlayer`] into a real character, by matching it
/// against an already-loaded world dump.
#[cfg(feature = "debug")]
pub(crate) fn on_join_request_standalone(
    join: On<FromClient<JoinRequest>>,
    mut commands: Commands,
    mut state: ResMut<MissionState>,
    loading_players: Query<(), With<LoadingPlayer>>,
    players: Query<(), With<Player>>,
) -> Result {
    let Some(client) = join.client_id.entity() else {
        return Ok(());
    };
    if loading_players.contains(client) || players.contains(client) {
        return Ok(());
    }
    if !state.allows_party_join(join.character_id) {
        warn!(
            character_id = join.character_id.0,
            "Rejected unseen Party join after enrollment sealed"
        );
        return Ok(());
    }
    state.begin_enrollment();
    commands.entity(client).insert(LoadingPlayer {
        requested_character: join.character_id,
    });
    info!(
        "[startup] Character {} connected, awaiting binding to a dumped character",
        join.character_id.0
    );
    Ok(())
}

fn reject(
    commands: &mut Commands,
    client: adventuresim_tactical_netcode::bevy_replicon::prelude::ClientId,
    message: SceneJoinRejected,
) {
    commands.server_trigger(ToClients {
        targets: SendTargets::Single(client),
        message,
    });
}

#[expect(
    clippy::too_many_arguments,
    reason = "A reconnect transfers one owned snapshot and its equipment references to a new network entity"
)]
fn restore_projection(
    commands: &mut Commands,
    client: Entity,
    client_id: adventuresim_tactical_netcode::bevy_replicon::prelude::ClientId,
    character: CharacterId,
    disconnected: Entity,
    projection: DisconnectedProjection,
    inventory_items: &Query<(Entity, &ItemOf)>,
    pending_actions: &mut PendingEquipmentActions,
    action_sequences: &mut LastEquipmentSequence,
    vista: &SceneVistaBundleResource,
    combat_config: &TacticalCombatConfig,
) {
    let token = fresh_reconnect_token();
    let projected = projection.projected();
    if projected {
        queue_replication_rebind(commands, client);
    }
    projection.insert(commands, client);
    commands.entity(client).insert(ReconnectSession {
        character_id: character,
        token,
    });
    send_reconnect_capability(commands, client_id, character, token);
    send_scene_vista(commands, client_id, vista);
    send_combat_config(commands, client_id, combat_config);
    reconnect_equipment_lifecycle(disconnected, client, pending_actions, action_sequences);
    for (item, owner) in inventory_items {
        if owner.0 == disconnected {
            commands.entity(item).insert(ItemOf(client));
        }
    }
    if projected {
        commands.queue(move |world: &mut World| rebuild_inventory_holding_cache(world, client));
    }
    commands.entity(disconnected).despawn();
    info!(
        character_id = character.0,
        "Rebound reconnect to transient tactical state"
    );
}

fn fresh_join_rejection(
    join: &JoinRequest,
    mission: Option<&MissionState>,
    admissions: &SceneAdmissions,
    scene_ready: bool,
    retained: bool,
    capability_valid: bool,
) -> Option<SceneJoinRejected> {
    let rejection = |reason: &str, reconnect| {
        Some(SceneJoinRejected {
            character_id: join.character_id,
            reason: reason.into(),
            reconnect,
        })
    };
    if !scene_ready || admissions.is_closing() {
        return rejection(
            "The scene is still preparing. Try again shortly.",
            ReconnectDisposition::Retain,
        );
    }
    if join.reconnect_token.is_some() && !capability_valid {
        return rejection(
            "The reconnect capability is invalid or expired.",
            ReconnectDisposition::Discard,
        );
    }
    if admissions.actors.contains_key(&join.character_id) || retained {
        return rejection(
            "This character already has an active scene connection.",
            ReconnectDisposition::Retain,
        );
    }
    if mission.is_some_and(|mission| !mission.allows_party_join(join.character_id)) {
        return rejection(
            "Mission enrollment is closed.",
            ReconnectDisposition::Discard,
        );
    }
    None
}

fn active_capability_valid(
    join: &JoinRequest,
    admissions: &SceneAdmissions,
    entities: &Query<(Entity, Option<&ReconnectSession>)>,
) -> bool {
    let Some(token) = join.reconnect_token else {
        return false;
    };
    admissions
        .actors
        .get(&join.character_id)
        .and_then(|entity| entities.get(*entity).ok())
        .and_then(|(_, session)| session)
        .is_some_and(|session| session.character_id == join.character_id && session.token == token)
}
