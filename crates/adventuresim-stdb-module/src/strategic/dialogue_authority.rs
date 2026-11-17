//! Session authority remains fixed when visits end or characters leave a scene.

use super::*;
use spacetimedb::Identity;

mod venue;

/// Storage adapters for immutable visit keys; never tactical tick state.
#[derive(Clone, Debug, Eq, PartialEq, SpacetimeType)]
pub struct TacticalDialogueAuthority {
    pub server: Identity,
    pub actor_visit_id: u64,
    pub resident_visit_id: u64,
}

/// Durable provenance only. Physical contact is checked by the native host.
#[derive(Clone, Debug, Eq, PartialEq, SpacetimeType)]
pub enum DialogueSessionAuthority {
    Venue,
    Tactical(TacticalDialogueAuthority),
}

pub(super) fn require_presence(
    ctx: &ReducerContext,
    session: &DialogueSession,
    character_id: u64,
) -> Result<crate::settlement_population::SettlementResidentProfile, String> {
    let DialogueSessionAuthority::Tactical(binding) = &session.authority else {
        return venue::require_presence(ctx, session, character_id);
    };
    let contact = crate::tactical::dialogue::require_contact(ctx, binding)?;
    if contact.actor.id != character_id
        || session.owner_character_id != character_id
        || contact.actor.party_id != session.owner_party_id
        || contact.settlement_id != session.settlement_id
        || contact.location_id != session.location_id
        || contact.npc.conversation_id != session.conversation_id
    {
        return Err("Live dialogue no longer belongs to these participants and visits".into());
    }
    let participants: Vec<_> = ctx
        .db
        .dialogue_participant()
        .session_id()
        .filter(&session.id)
        .collect();
    if participants.len() != 2
        || !participants
            .iter()
            .any(|participant| participant.character_id == Some(character_id))
        || !participants.iter().any(|participant| {
            participant.character_id.is_none()
                && participant.actor_id == contact.npc.character_id.to_string()
        })
    {
        return Err("Live dialogue requires its original player and resident".into());
    }
    require_representative(
        ctx,
        &contact.npc,
        &contact.settlement_id,
        &contact.location_id,
    )?;
    Ok(contact.npc.profile)
}

pub(super) fn require_representative(
    ctx: &ReducerContext,
    npc: &crate::settlement_population::SettlementResidentProfile,
    settlement_id: &str,
    location_id: &str,
) -> Result<(), String> {
    if (npc.organization_id.is_empty() && npc.conversation_id == "organization-representative")
        || (!npc.organization_id.is_empty()
            && exact_organization_representative(ctx, npc, settlement_id, location_id).is_none())
    {
        return Err("Dialogue NPC has no exact authority at this location".into());
    }
    Ok(())
}

/// Read live elapsed chronology without advancing or committing a visit.
pub(super) fn personal_minute(
    ctx: &ReducerContext,
    session: &DialogueSession,
    character_id: u64,
) -> Result<StrategicMinute, String> {
    match session.authority {
        DialogueSessionAuthority::Tactical(_) => {
            crate::tactical::local_time::current_personal_clock(ctx, character_id)
                .map(|clock| clock.minute())
        }
        DialogueSessionAuthority::Venue => ctx
            .db
            .character_time()
            .character_id()
            .find(character_id)
            .map(|time| time.minutes)
            .ok_or_else(|| "Dialogue participant has no personal clock".into()),
    }
}
