//! Validate live dialogue against the registered host and current visit owners.

use super::*;
use crate::{
    settlement_population::{ResolvedSettlementResident, settlement_resident_presence},
    strategic::TacticalDialogueAuthority,
};
use adventuresim_core::tactical_session::LocalSceneVisitId;
use local_time::{LocalSceneTimeOwner, LocalSceneVisit, local_scene_visit};

pub(crate) struct TacticalDialogueContact {
    pub(crate) actor: Character,
    pub(crate) npc: ResolvedSettlementResident,
    pub(crate) settlement_id: String,
    pub(crate) location_id: String,
}

pub(crate) fn require_contact(
    ctx: &ReducerContext,
    binding: &TacticalDialogueAuthority,
) -> Result<TacticalDialogueContact, String> {
    if ctx.sender() != binding.server {
        return Err("Only the owning tactical host can mutate live dialogue".into());
    }
    let server = ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(binding.server)
        .ok_or("Live dialogue host is no longer registered")?;
    let TacticalSession::Settlement(settlement) = &server.session else {
        return Err("Live dialogue requires an ordinary settlement scene".into());
    };
    let actor = require_visit(
        ctx,
        binding.actor_visit_id,
        &server,
        LocalSceneTimeOwner::Player,
    )?;
    let resident = require_visit(
        ctx,
        binding.resident_visit_id,
        &server,
        LocalSceneTimeOwner::Resident,
    )?;
    let official = crate::time::refresh_clock(ctx)?;
    residents::require_current_resident(ctx, resident.id, &server, official)?;
    let npc = crate::settlement_population::resolve_settlement_resident(ctx, resident.id)
        .ok_or("Live dialogue resident is no longer authoritative")?;
    if actor.id == resident.id
        || npc.home_settlement_id != settlement.id
        || actor.current_settlement_id.as_deref() != Some(settlement.id.as_str())
        || resident.current_settlement_id.as_deref() != Some(settlement.id.as_str())
    {
        return Err("Live dialogue participants do not share this settlement".into());
    }
    let presence = ctx
        .db
        .settlement_resident_presence()
        .character_id()
        .find(resident.id)
        .ok_or("Live dialogue resident has no current presence")?;
    Ok(TacticalDialogueContact {
        actor,
        npc,
        settlement_id: settlement.id.clone(),
        location_id: presence.location_id,
    })
}

fn require_visit(
    ctx: &ReducerContext,
    visit_id: u64,
    server: &TacticalServer,
    owner: LocalSceneTimeOwner,
) -> Result<Character, String> {
    let id = LocalSceneVisitId::try_from(visit_id).map_err(|error| error.to_string())?;
    let visit = ctx
        .db
        .local_scene_visit()
        .id()
        .find(id.get())
        .ok_or("Live dialogue visit is missing")?;
    let active =
        LocalSceneVisit::active(ctx, visit.character_id)?.ok_or("Live dialogue visit is closed")?;
    if active.id != visit.id || visit.server != server.identity || visit.owner != owner {
        return Err("Live dialogue visit has another host or participation role".into());
    }
    let character = crate::character::require_living_character(ctx, visit.character_id)?;
    if character.server != server.identity {
        return Err("Live dialogue character has left its host".into());
    }
    Ok(character)
}
