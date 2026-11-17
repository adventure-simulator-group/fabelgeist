//! Resident admission owns a visit without assigning any player connection.

use super::*;
use crate::{
    tactical::local_time::{LocalSceneTimeOwner, LocalSceneVisit},
    time::character_time,
};
use adventuresim_world_schema::{calendar::StrategicMinute, settlement_buildings::BusinessId};

#[reducer]
pub fn enter_settlement_resident(ctx: &ReducerContext, character_id: u64) -> ResidentResult<()> {
    let server = ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(ctx.sender())
        .ok_or("Only a registered settlement host can admit residents")?;
    let TacticalSession::Settlement(settlement) = &server.session else {
        return Err("Ordinary residents cannot be admitted to a combat mission".into());
    };
    let official = crate::time::refresh_clock(ctx)?;
    require_current_resident(ctx, character_id, &server, official)?;
    let character = crate::character::require_living_character(ctx, character_id)?;
    super::super::settlement::require_settlement_admission(
        ctx,
        &character,
        settlement,
        server.identity,
    )?;
    if !crate::tactical::local_time::owns_character_time(ctx, character_id)? {
        crate::time::initialize_character_time(ctx, character_id)?;
        let minute = ctx
            .db
            .character_time()
            .character_id()
            .find(character_id)
            .ok_or("Resident has no personal clock")?
            .minutes;
        if minute < official {
            crate::time::advance_stationary_character_to(ctx, character_id, official)?;
        }
    }
    // Catch-up can settle age, death or transfer of NPC authority. Admit the
    // resulting character rather than overwriting it with the old snapshot.
    let mut character = crate::character::require_living_character(ctx, character_id)?;
    if !has_npc_authority(ctx, character_id) {
        return Err("Resident authority transferred to a player during catch-up".into());
    }
    require_current_resident(ctx, character_id, &server, official)?;
    super::super::settlement::require_settlement_admission(
        ctx,
        &character,
        settlement,
        server.identity,
    )?;
    LocalSceneVisit::begin(
        ctx,
        character_id,
        server.identity,
        LocalSceneTimeOwner::Resident,
    )?;
    character.server = server.identity;
    ctx.db.character().id().update(character);
    Ok(())
}

pub(in crate::tactical) fn require_current_resident(
    ctx: &ReducerContext,
    character_id: u64,
    server: &TacticalServer,
    official: StrategicMinute,
) -> ResidentResult<()> {
    let TacticalSession::Settlement(settlement) = &server.session else {
        return Err("Dialogue requires an ordinary settlement scene".into());
    };
    let roster = ctx
        .db
        .scene_resident_roster()
        .server()
        .find(server.identity)
        .ok_or("Configure the playable area's resident roster before admission")?;
    if !roster.character_ids.contains(&character_id) || !has_npc_authority(ctx, character_id) {
        return Err("Character is not a scene-owned resident in this area".into());
    }
    let business = ctx
        .db
        .settlement_business_operator()
        .operator_character_id()
        .find(character_id)
        .map(|assignment| assignment.business_id);
    if !resident_business_matches(settlement, &roster.area, character_id, business.as_ref()) {
        return Err("Resident business ownership changed outside this captured area".into());
    }
    require_present(
        ctx,
        character_id,
        &roster.area,
        settlement,
        business.as_ref(),
        official,
    )?;
    Ok(())
}

fn require_present(
    ctx: &ReducerContext,
    character_id: u64,
    area: &TacticalResidentArea,
    settlement: &TacticalSettlementSnapshot,
    business: Option<&BusinessId>,
    official: StrategicMinute,
) -> ResidentResult<()> {
    let presence = ctx
        .db
        .settlement_resident_presence()
        .character_id()
        .find(character_id)
        .ok_or("Resident has no current presence schedule")?;
    if presence.settlement_id != settlement.id
        || !area.includes_presence(&presence, business)
        || presence.context_suppressed
        || presence.health_suppressed
        || crate::settlement_population::npc_presence_remaining_minutes_at(ctx, &presence, official)
            .is_none()
    {
        return Err("Resident is absent or unavailable at the shared scene time".into());
    }
    Ok(())
}
