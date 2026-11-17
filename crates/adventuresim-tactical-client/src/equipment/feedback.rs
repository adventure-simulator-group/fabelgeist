//! Confirmed equipment state arrives through replication, never optimistic edits.

use adventuresim_tactical_netcode::{
    aeronet::io::connection::Disconnected,
    bevy_replicon::prelude::ClientTriggerExt,
    equipment::{EquipmentActionOutcome, EquipmentActionResponse},
    prelude::AdventureSimulatorClient,
};

use super::*;
use crate::hud::{self, HudText};

#[derive(Resource, Default)]
pub(super) struct EquipmentFeedback {
    pending: Option<PendingEquipmentIntent>,
    actor: Option<Entity>,
}

struct PendingEquipmentIntent {
    actor: Entity,
    sequence: EquipmentSequence,
    phase: ConfirmationPhase,
}

enum ConfirmationPhase {
    AwaitingHost,
    AwaitingProjection(EquipmentRevision),
}

impl EquipmentFeedback {
    pub(super) fn is_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub(super) fn submit(&mut self, commands: &mut Commands, request: EquipmentActionRequest) {
        if self.is_pending() {
            return;
        }
        self.actor = Some(request.actor);
        self.pending = Some(PendingEquipmentIntent {
            actor: request.actor,
            sequence: request.sequence,
            phase: ConfirmationPhase::AwaitingHost,
        });
        hud::text(HudText::Equipment, "Changing equipment…");
        commands.client_trigger(request);
    }

    fn clear(&mut self) {
        self.pending = None;
        self.actor = None;
        hud::text(HudText::Equipment, "");
    }
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<EquipmentFeedback>()
        .add_observer(on_equipment_response)
        .add_observer(on_disconnect)
        .add_systems(Update, observe_projection);
}

fn on_equipment_response(
    response: On<EquipmentActionResponse>,
    players: Query<Entity, With<ClientPlayer>>,
    mut feedback: ResMut<EquipmentFeedback>,
) {
    let Some(pending) = &mut feedback.pending else {
        return;
    };
    if pending.actor != response.actor
        || pending.sequence != response.sequence
        || !players.contains(pending.actor)
        || !matches!(pending.phase, ConfirmationPhase::AwaitingHost)
    {
        return;
    }
    match &response.outcome {
        EquipmentActionOutcome::Pending => {}
        EquipmentActionOutcome::Committed { revision } => {
            pending.phase = ConfirmationPhase::AwaitingProjection(*revision);
        }
        EquipmentActionOutcome::Rejected(reason) => {
            let message = reason.to_string();
            warn!(%message, "Equipment request rejected");
            hud::text(HudText::Equipment, &message);
            feedback.pending = None;
        }
    }
}

fn observe_projection(
    players: Query<&EquipmentActionState, With<ClientPlayer>>,
    mut feedback: ResMut<EquipmentFeedback>,
) {
    if feedback.actor.is_some_and(|actor| !players.contains(actor)) {
        feedback.clear();
        return;
    }
    let Some(pending) = &feedback.pending else {
        return;
    };
    let ConfirmationPhase::AwaitingProjection(revision) = pending.phase else {
        return;
    };
    if players
        .get(pending.actor)
        .is_ok_and(|state| state.revision.has_reached(revision))
    {
        feedback.pending = None;
        hud::text(HudText::Equipment, "Equipment changed.");
    }
}

fn on_disconnect(
    disconnected: On<Disconnected>,
    clients: Query<Entity, With<AdventureSimulatorClient>>,
    mut feedback: ResMut<EquipmentFeedback>,
    mut session: ResMut<GrabSession>,
) {
    if clients.contains(disconnected.entity) {
        feedback.clear();
        (session.active, session.selection, session.repeated_input) = (None, None, None);
        session.next_sequence = EquipmentSequence::FIRST;
    }
}

#[cfg(test)]
mod behavior;
