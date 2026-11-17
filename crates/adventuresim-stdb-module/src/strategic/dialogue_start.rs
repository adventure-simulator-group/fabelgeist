//! Dialogue construction after an encounter proves its authority.

use super::*;
use crate::{Character, settlement_population::ResolvedSettlementResident};
use adventuresim_core::dialogue_boundary::DialogueSessionId;

mod greeting;
mod referral;
mod scene;
mod venue;

pub(super) struct DialogueStartRequest {
    character_id: u64,
    session_id: DialogueSessionId,
    conversation_id: String,
    npc_character_id: u64,
    location_id: String,
    catalog_revision: String,
}

struct DialogueEncounter {
    character: Character,
    settlement_id: String,
    npc: ResolvedSettlementResident,
    available_npcs: Vec<ResolvedSettlementResident>,
    authority: DialogueSessionAuthority,
}

impl DialogueStartRequest {
    pub(super) fn decode(
        character_id: u64,
        session_id: String,
        conversation_id: String,
        npc_actor_id: String,
        location_id: String,
        catalog_revision: String,
    ) -> Result<Self, String> {
        require_dialogue_revision(&catalog_revision)?;
        let session_id =
            DialogueSessionId::try_from(session_id).map_err(|error| error.to_string())?;
        if session_id.owner_character_id() != character_id {
            return Err("Invalid dialogue session ID".into());
        }
        let npc_character_id = npc_actor_id
            .parse()
            .map_err(|_| "Dialogue NPC identity is invalid")?;
        Ok(Self {
            character_id,
            session_id,
            conversation_id,
            npc_character_id,
            location_id,
            catalog_revision,
        })
    }

    pub(super) fn start_at_venue(self, ctx: &ReducerContext) -> Result<(), String> {
        crate::tactical::local_time::require_unleased_time(ctx, self.character_id)?;
        crate::tactical::local_time::require_unleased_time(ctx, self.npc_character_id)?;
        let encounter = DialogueEncounter::at_venue(ctx, &self)?;
        encounter.start(ctx, self)
    }
}

impl DialogueEncounter {
    fn start(self, ctx: &ReducerContext, request: DialogueStartRequest) -> Result<(), String> {
        let conversation = self.conversation(&request)?;
        if let Some(existing) = ctx
            .db
            .dialogue_session()
            .id()
            .find(request.session_id.as_str().to_owned())
        {
            return if existing.conversation_id == request.conversation_id
                && existing.settlement_id == self.settlement_id
                && existing.location_id == request.location_id
                && existing.catalog_revision == request.catalog_revision
                && existing.authority == self.authority
                && DialogueSessionState::parse(&existing.state)? == DialogueSessionState::Active
            {
                require_live_dialogue_presence(ctx, &existing, request.character_id).map(|_| ())
            } else {
                Err("Dialogue session ID conflicts with another request".into())
            };
        }
        let session = self.insert_session(ctx, &request, conversation)?;
        validate_dialogue_cardinality(ctx, &session, conversation)?;
        require_live_dialogue_presence(ctx, &session, request.character_id)?;
        greeting::emit(ctx, &session, conversation, request.character_id)?;
        if matches!(self.authority, DialogueSessionAuthority::Venue) {
            referral::deliver(ctx, &session, request.character_id, self.npc.character_id)?;
        }
        refresh_dialogue_topic_options(ctx, &session, request.character_id)
    }

    fn conversation(
        &self,
        request: &DialogueStartRequest,
    ) -> Result<&'static adventuresim_dialogue::Conversation, String> {
        if request.conversation_id != self.npc.conversation_id {
            return Err("Dialogue conversation is not valid for this NPC".into());
        }
        let conversation = adventuresim_dialogue::find_conversation(&request.conversation_id)
            .ok_or("Unknown dialogue conversation")?;
        adventuresim_dialogue::validate(adventuresim_dialogue::catalog())
            .map_err(|_| "Dialogue catalog is invalid")?;
        if !conversation
            .roles
            .values()
            .any(|role| role.kind == adventuresim_dialogue::ParticipantKind::Npc)
        {
            return Err("Conversation has no NPC role".into());
        }
        Ok(conversation)
    }

    fn insert_session(
        &self,
        ctx: &ReducerContext,
        request: &DialogueStartRequest,
        conversation: &adventuresim_dialogue::Conversation,
    ) -> Result<DialogueSession, String> {
        let player_role = conversation
            .roles
            .iter()
            .find(|(_, role)| role.kind == adventuresim_dialogue::ParticipantKind::Player)
            .map(|(name, _)| name.clone())
            .ok_or("Conversation has no player role")?;
        let id = request.session_id.as_str().to_owned();
        let session = ctx.db.dialogue_session().insert(DialogueSession {
            id: id.clone(),
            gateway_bucket: 0,
            conversation_id: request.conversation_id.clone(),
            catalog_revision: request.catalog_revision.clone(),
            settlement_id: self.settlement_id.clone(),
            location_id: request.location_id.clone(),
            owner_character_id: request.character_id,
            owner_party_id: self.character.party_id.clone(),
            authority: self.authority.clone(),
            state: DialogueSessionState::Active.stable_id().into(),
            revision: 0,
            created_micros: ctx.timestamp.to_micros_since_unix_epoch(),
        });
        ctx.db.dialogue_participant().insert(DialogueParticipant {
            id: format!("{id}:character:{}", request.character_id),
            gateway_bucket: 0,
            session_id: id.clone(),
            role: player_role,
            character_id: Some(request.character_id),
            actor_id: format!("character:{}", request.character_id),
            display_name: self.character.name.clone(),
        });
        self.bind_npcs(ctx, &id, conversation)?;
        Ok(session)
    }

    fn bind_npcs(
        &self,
        ctx: &ReducerContext,
        id: &str,
        conversation: &adventuresim_dialogue::Conversation,
    ) -> Result<(), String> {
        let mut used_npcs = HashSet::new();
        for (index, (role_name, role)) in conversation
            .roles
            .iter()
            .filter(|(_, role)| role.kind == adventuresim_dialogue::ParticipantKind::Npc)
            .enumerate()
        {
            if role.min > 1 {
                return Err("Dialogue currently supports one persistent actor per NPC role".into());
            }
            if index > 0 && self.available_npcs.is_empty() && role.min == 0 {
                continue;
            }
            let bound = if index == 0 {
                self.npc.clone()
            } else {
                self.available_npcs
                    .iter()
                    .find(|candidate| {
                        candidate.character_id != self.npc.character_id
                            && !used_npcs.contains(&candidate.character_id)
                    })
                    .cloned()
                    .ok_or("Required NPC role has no persistent actor at this location")?
            };
            used_npcs.insert(bound.character_id);
            ctx.db.dialogue_participant().insert(DialogueParticipant {
                id: format!("{id}:npc:{role_name}"),
                gateway_bucket: 0,
                session_id: id.to_owned(),
                role: role_name.clone(),
                character_id: None,
                display_name: bound.name.clone(),
                actor_id: bound.character_id.to_string(),
            });
        }
        Ok(())
    }
}
