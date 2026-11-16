//! Shared central-area admission without investigation or combat authority.

use adventuresim_world_schema::coordinates::Wgs84CoordinateE7;
use spacetimedb::{Identity, ReducerContext, Table, reducer};

use super::{
    TacticalServerRequest, TacticalSession, leave_tactical_scene_for_server,
    request::settlement_snapshot, tactical_server_authority, tactical_server_request_authority,
};
use crate::{character::character, strategic::settlement};

#[reducer]
pub fn request_settlement_scene(ctx: &ReducerContext, character_id: u64) -> Result<(), String> {
    crate::strategic::require_strategic_character_authority(ctx, character_id)?;
    let character = crate::character::require_living_character(ctx, character_id)?;
    let settlement_id = character
        .current_settlement_id
        .as_ref()
        .filter(|_| crate::investigation::character_case_site_id(ctx, character_id).is_none())
        .ok_or("Settlement admission requires presence in a settlement")?;
    if character.has_tactical_server_assignment()
        && let Some(active) = ctx
            .db
            .tactical_server_authority()
            .identity()
            .find(character.server)
    {
        return match active.session {
            TacticalSession::Settlement(snapshot) if snapshot.id == *settlement_id => Ok(()),
            _ => Err("Character is already assigned to another live scene".into()),
        };
    }
    let settlement = ctx
        .db
        .settlement()
        .id()
        .find(settlement_id)
        .ok_or("Settlement does not exist")?;
    let session = TacticalSession::Settlement(settlement_snapshot(ctx, &settlement)?);
    let instance_id = session.instance_id()?.into_string();
    if ctx
        .db
        .tactical_server_authority()
        .instance_id()
        .find(&instance_id)
        .is_some()
    {
        return Ok(());
    }
    if let Some(mut request) = ctx
        .db
        .tactical_server_request_authority()
        .instance_id()
        .find(&instance_id)
    {
        if request.provisioning != super::TacticalProvisioningStatus::Pending {
            request.provisioning = super::TacticalProvisioningStatus::Pending;
            ctx.db
                .tactical_server_request_authority()
                .instance_id()
                .update(request);
        }
        return Ok(());
    }
    let coordinate =
        Wgs84CoordinateE7::from_longitude_latitude_degrees(settlement.coord_x, settlement.coord_y)
            .ok_or("Settlement has invalid geographic coordinates")?;
    let minute = crate::time::refresh_clock(ctx)?;
    ctx.db
        .tactical_server_request_authority()
        .insert(TacticalServerRequest {
            provisioning: crate::tactical::TacticalProvisioningStatus::Pending,
            instance_id,
            gateway_bucket: 0,
            scene_key: settlement.scene_key,
            requested_by: character_id,
            longitude_e7: coordinate.longitude().get(),
            latitude_e7: coordinate.latitude().get(),
            absolute_minute: minute,
            lunar_phase_minute: minute,
            session,
        });
    Ok(())
}

/// Empty ordinary scenes retire without a combat result, defeat or reward.
/// The server's transient reconnect grace is responsible for deciding emptiness.
#[reducer]
pub fn retire_settlement_scene(ctx: &ReducerContext) -> Result<(), String> {
    let Some(server) = ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(ctx.sender())
    else {
        return Ok(());
    };
    if !matches!(server.session, TacticalSession::Settlement(_)) {
        return Err("Combat mission servers must submit a mission outcome".into());
    }
    let assigned: Vec<_> = ctx.db.character().server().filter(ctx.sender()).collect();
    for character in assigned {
        leave_tactical_scene_for_server(ctx, character, server.identity)?;
    }
    ctx.db
        .tactical_server_authority()
        .identity()
        .delete(server.identity);
    Ok(())
}

/// A lost server connection releases its slot and durable assignments. Tick
/// state is transient and a later admission creates a fresh ordinary scene.
#[reducer(client_disconnected)]
pub fn release_disconnected_settlement_scene(ctx: &ReducerContext) -> Result<(), String> {
    if ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(ctx.sender())
        .is_some_and(|server| {
            Some(server.hosting_connection) == ctx.connection_id()
                && matches!(server.session, TacticalSession::Settlement(_))
        })
    {
        retire_settlement_scene(ctx)?;
    }
    Ok(())
}

pub(super) fn require_settlement_admission(
    ctx: &ReducerContext,
    character: &crate::Character,
    settlement: &super::TacticalSettlementSnapshot,
    server: Identity,
) -> Result<(), String> {
    if character.temporary
        || character.current_settlement_id.as_deref() != Some(settlement.id.as_str())
        || crate::investigation::character_case_site_id(ctx, character.id).is_some()
    {
        return Err("Character is not present in this settlement area".into());
    }
    if character.has_tactical_server_assignment()
        && character.server != server
        && ctx
            .db
            .tactical_server_authority()
            .identity()
            .find(character.server)
            .is_some()
    {
        return Err("Character is already assigned to another live scene".into());
    }
    Ok(())
}
