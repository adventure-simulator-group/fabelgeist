//! Admission checks precede replay; full read-set checks precede new effects.

use super::request::ExpectedActor;
use super::*;

impl InventoryCommand {
    pub(super) fn require_admissions(&self, ctx: &ReducerContext) -> CommandResult<()> {
        let source = self.actor.admission(ctx)?;
        if let Some(peer) = self.effect.peer() {
            let recipient = peer.admission(ctx)?;
            crate::inventory_effects::PersonalInventoryTransfer::require_participants(
                ctx,
                &source.character,
                &recipient.character,
            )?;
        }
        Ok(())
    }

    pub(super) fn require_current(&self, ctx: &ReducerContext) -> CommandResult<()> {
        for actor in self.actors() {
            let snapshot = InventorySnapshot::current(&ctx.as_read_only(), actor.id)
                .map_err(|error| error.to_string())?;
            if snapshot.token != actor.token {
                return Err("Inventory changed after this command was prepared".into());
            }
        }
        Ok(())
    }
}

impl ExpectedActor {
    fn admission(&self, ctx: &ReducerContext) -> CommandResult<InventoryAdmission> {
        let current = InventoryAdmission::for_actor(&ctx.as_read_only(), self.id)
            .map_err(|error| error.to_string())?;
        if !current.character.alive
            || current.role != TacticalActorRole::Player
            || current.scope != self.token.scope
        {
            return Err("Inventory command does not belong to this active player admission".into());
        }
        Ok(current)
    }

    pub(super) fn result(
        &self,
        ctx: &ReducerContext,
    ) -> CommandResult<TacticalInventoryActorState> {
        let snapshot = InventorySnapshot::current(&ctx.as_read_only(), self.id)
            .map_err(|error| error.to_string())?;
        Ok(TacticalInventoryActorState {
            character_id: self.id.get(),
            scope: snapshot.token.scope.bytes().to_vec(),
            revision: snapshot.token.revision.bytes().to_vec(),
        })
    }
}

pub(super) fn require_host(ctx: &ReducerContext) -> CommandResult<TacticalServer> {
    let server = ctx
        .db
        .tactical_server_authority()
        .identity()
        .find(ctx.sender())
        .ok_or("Inventory commands require the currently registered tactical host")?;
    if ctx.connection_id() != Some(server.hosting_connection) {
        return Err("Inventory commands require the registered serving connection".into());
    }
    Ok(server)
}
