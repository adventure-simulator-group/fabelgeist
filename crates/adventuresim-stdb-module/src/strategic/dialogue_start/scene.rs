//! A live encounter binds only the resident actually reached by the host.

use super::*;

impl DialogueStartRequest {
    pub(in crate::strategic) fn start_in_scene(
        ctx: &ReducerContext,
        binding: TacticalDialogueAuthority,
        session_id: String,
        catalog_revision: String,
    ) -> Result<(), String> {
        let contact = crate::tactical::dialogue::require_contact(ctx, &binding)?;
        dialogue_authority::require_representative(
            ctx,
            &contact.npc,
            &contact.settlement_id,
            &contact.location_id,
        )?;
        let request = Self::decode(
            contact.actor.id,
            session_id,
            contact.npc.conversation_id.clone(),
            contact.npc.character_id.to_string(),
            contact.location_id,
            catalog_revision,
        )?;
        let encounter = DialogueEncounter {
            character: contact.actor,
            settlement_id: contact.settlement_id,
            npc: contact.npc,
            available_npcs: Vec::new(),
            authority: DialogueSessionAuthority::Tactical(binding),
        };
        encounter.start(ctx, request)
    }
}
