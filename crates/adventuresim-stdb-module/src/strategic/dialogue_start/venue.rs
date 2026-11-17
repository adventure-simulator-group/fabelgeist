//! Venue co-presence for characters outside live scenes.

use super::*;
use adventuresim_core::strategic_presence::StrategicPresence;

impl DialogueEncounter {
    pub(super) fn at_venue(
        ctx: &ReducerContext,
        request: &DialogueStartRequest,
    ) -> Result<Self, String> {
        crate::character::require_living_character(ctx, request.character_id)?;
        let character = ctx
            .db
            .character()
            .id()
            .find(request.character_id)
            .ok_or("Character not found")?;
        if character.party_id.is_none() {
            return Err("Venue dialogue requires a party".into());
        }
        let settlement_id = character
            .current_settlement_id
            .clone()
            .ok_or("Dialogue requires a settlement")?;
        // The route value selects a candidate only; server-side settlement
        // navigability must resolve it before it becomes exact actor context.
        let exact_place = require_navigable_npc_place(ctx, &settlement_id, &request.location_id)?;
        let npc_character_id = request.npc_character_id;
        let minute = crate::relationship::enforce_temporal_scope(
            ctx,
            request.character_id,
            Some(npc_character_id),
            crate::relationship::TemporalScope::PairwiseSoft,
        )?;
        let actor_settlement_presence =
            adventuresim_core::strategic_presence::StrategicPresence::settlement_membership(
                request.character_id,
                settlement_id.clone(),
                adventuresim_core::strategic_presence::PresenceFrontier {
                    observer_character_id: request.character_id,
                    personal_minute: minute,
                },
            )
            .map_err(|_| "Dialogue settlement identity is invalid")?;
        let actor_presence =
            adventuresim_core::strategic_presence::StrategicPresence::validated_venue_selection(
                &actor_settlement_presence,
                exact_place,
            )
            .map_err(|_| "Dialogue location is not in the actor's settlement")?;
        let npc = require_venue_npc(ctx, request, &settlement_id, minute, &actor_presence)?;
        let available_npcs = venue_audience(
            ctx,
            request.character_id,
            &settlement_id,
            minute,
            &actor_presence,
        );
        Ok(Self {
            character,
            settlement_id,
            npc,
            available_npcs,
            authority: DialogueSessionAuthority::Venue,
        })
    }
}

fn venue_audience(
    ctx: &ReducerContext,
    character_id: u64,
    settlement_id: &str,
    minute: StrategicMinute,
    actor_presence: &StrategicPresence,
) -> Vec<ResolvedSettlementResident> {
    let available_npcs: Vec<_> = ctx
        .db
        .settlement_resident_presence()
        .settlement_id()
        .filter(settlement_id)
        .filter(|candidate| {
            crate::settlement_population::npc_strategic_presence_at(
                ctx,
                candidate,
                character_id,
                minute,
            )
            .is_some_and(|candidate_presence| {
                adventuresim_core::strategic_presence::are_co_present(
                    actor_presence,
                    candidate_presence.presence(),
                )
            })
        })
        .filter_map(|presence| {
            crate::settlement_population::resolve_settlement_resident(ctx, presence.character_id)
        })
        .collect();
    available_npcs
}

fn require_venue_npc(
    ctx: &ReducerContext,
    request: &DialogueStartRequest,
    settlement_id: &str,
    minute: StrategicMinute,
    actor_presence: &StrategicPresence,
) -> Result<ResolvedSettlementResident, String> {
    let npc_character_id = request.npc_character_id;
    let npc = crate::settlement_population::resolve_settlement_resident(ctx, npc_character_id)
        .ok_or("Dialogue actor is not a persistent settlement NPC")?;
    if npc.home_settlement_id != settlement_id {
        return Err("Dialogue actor is not at this settlement".into());
    }
    dialogue_authority::require_representative(ctx, &npc, settlement_id, &request.location_id)?;
    let presence = ctx
        .db
        .settlement_resident_presence()
        .character_id()
        .find(npc_character_id)
        .ok_or("Dialogue actor has no authoritative presence")?;
    let npc_presence = crate::settlement_population::npc_strategic_presence_at(
        ctx,
        &presence,
        request.character_id,
        minute,
    )
    .ok_or_else(|| {
        adventuresim_core::reducer_error::coded_reducer_error(
            adventuresim_core::reducer_error::ReducerErrorCode::DialogueContactUnavailable,
            "Dialogue actor is not present at this time",
        )
    })?;
    if !adventuresim_core::strategic_presence::are_co_present(
        actor_presence,
        npc_presence.presence(),
    ) {
        return Err(adventuresim_core::reducer_error::coded_reducer_error(
            adventuresim_core::reducer_error::ReducerErrorCode::DialogueContactUnavailable,
            "Dialogue actor is not present at this time",
        ));
    }
    Ok(npc)
}
