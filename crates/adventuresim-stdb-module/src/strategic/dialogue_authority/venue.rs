//! Abstract venue authority excludes every character with a live time owner.

use super::*;

/// Venue authority proves abstract co-presence without a live scene owner.
pub(super) fn require_presence(
    ctx: &ReducerContext,
    session: &DialogueSession,
    character_id: u64,
) -> Result<crate::settlement_population::SettlementResidentProfile, String> {
    require_strategic_gateway(ctx)?;
    crate::tactical::local_time::require_unleased_time(ctx, character_id)?;
    let exact_place =
        require_navigable_npc_place(ctx, &session.settlement_id, &session.location_id)?;
    let character = ctx
        .db
        .character()
        .id()
        .find(character_id)
        .ok_or("Character not found")?;
    if session.owner_party_id.is_none() || character.party_id != session.owner_party_id {
        return Err("Dialogue joins are limited to the owning party".into());
    }
    if character.current_settlement_id.as_deref() != Some(session.settlement_id.as_str()) {
        return Err("Dialogue participant has left the settlement".into());
    }
    let npc_participants: Vec<_> = ctx
        .db
        .dialogue_participant()
        .session_id()
        .filter(&session.id)
        .filter(|participant| participant.character_id.is_none())
        .collect();
    if npc_participants.is_empty() {
        return Err("Dialogue has no persistent NPC participant".into());
    }
    let minute = ctx
        .db
        .character_time()
        .character_id()
        .find(character_id)
        .map(|time| time.minutes)
        .ok_or("Dialogue participant has no personal time authority")?;
    let actor_settlement_presence =
        adventuresim_core::strategic_presence::StrategicPresence::settlement_membership(
            character_id,
            session.settlement_id.clone(),
            adventuresim_core::strategic_presence::PresenceFrontier {
                observer_character_id: character_id,
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
    require_npc_participants(
        ctx,
        session,
        character_id,
        minute,
        &actor_presence,
        npc_participants,
    )
}

fn require_npc_participants(
    ctx: &ReducerContext,
    session: &DialogueSession,
    character_id: u64,
    minute: StrategicMinute,
    actor_presence: &adventuresim_core::strategic_presence::StrategicPresence,
    npc_participants: Vec<DialogueParticipant>,
) -> Result<crate::settlement_population::SettlementResidentProfile, String> {
    let mut primary = None;
    for participant in npc_participants {
        let npc_character_id = participant
            .actor_id
            .parse::<u64>()
            .map_err(|_| "Dialogue NPC identity is invalid")?;
        crate::tactical::local_time::require_unleased_time(ctx, npc_character_id)?;
        let npc = ctx
            .db
            .settlement_resident_profile()
            .character_id()
            .find(npc_character_id)
            .ok_or("Dialogue NPC is no longer authoritative")?;
        let presence = ctx
            .db
            .settlement_resident_presence()
            .character_id()
            .find(npc.character_id)
            .ok_or("Dialogue NPC has no authoritative presence")?;
        let npc_presence = crate::settlement_population::npc_strategic_presence_at(
            ctx,
            &presence,
            character_id,
            minute,
        )
        .ok_or_else(|| {
            adventuresim_core::reducer_error::coded_reducer_error(
                adventuresim_core::reducer_error::ReducerErrorCode::DialogueContactUnavailable,
                "Dialogue NPC is not present at the session location and time",
            )
        })?;
        if npc.home_settlement_id != session.settlement_id
            || !adventuresim_core::strategic_presence::are_co_present(
                actor_presence,
                npc_presence.presence(),
            )
        {
            return Err(adventuresim_core::reducer_error::coded_reducer_error(
                adventuresim_core::reducer_error::ReducerErrorCode::DialogueContactUnavailable,
                "Dialogue NPC is not present at the session location and time",
            ));
        }
        super::require_representative(ctx, &npc, &session.settlement_id, &session.location_id)?;
        primary.get_or_insert(npc);
    }
    primary.ok_or_else(|| "Dialogue requires an NPC participant".into())
}
